use std::path::{Path, PathBuf};

use anyhow::anyhow;
use blaze_common::{
    error::{Error, Result},
    executor::{GitCheckout, GitOptions},
    value::{to_value, Value},
};
use git2::{build::CheckoutBuilder, FetchOptions, RemoteCallbacks};
use serde::{Deserialize, Serialize};
use url::Url;

use super::{
    kinds::infer_local_executor_type, loader::LoadMetadata, resolver::ExecutorResolution,
    ExecutorResolver, ExecutorUpdate, ResolverContext,
};

const REPOSITORIES_PATH: &str = ".blaze/repositories";

fn get_repository_path(package_id: u64) -> PathBuf {
    Path::new(REPOSITORIES_PATH).join(package_id.to_string())
}

#[derive(Serialize, Deserialize)]
struct State {
    repository_path: PathBuf,
}

pub struct GitHeadlessResolver {
    git_options: GitOptions,
    remote_callbacks_customizer: Box<dyn Fn(&mut RemoteCallbacks<'_>)>,
    fetch_options_customizer: Box<dyn Fn(&mut FetchOptions<'_>)>,
}

impl GitHeadlessResolver {
    pub fn new(
        git_options: GitOptions,
        remote_callbacks_customizer: impl Fn(&mut RemoteCallbacks<'_>) + 'static,
        fetch_options_customizer: impl Fn(&mut FetchOptions<'_>) + 'static,
    ) -> Self {
        Self {
            remote_callbacks_customizer: Box::new(remote_callbacks_customizer),
            fetch_options_customizer: Box::new(fetch_options_customizer),
            git_options,
        }
    }

    fn default_remote_callbacks(&self) -> RemoteCallbacks {
        let remote_callbacks = RemoteCallbacks::new();
        remote_callbacks
    }

    fn default_fetch_options<'a>(&self, remote_callbacks: RemoteCallbacks<'a>) -> FetchOptions<'a> {
        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(remote_callbacks);
        fetch_options.download_tags(git2::AutotagOption::All);
        fetch_options
    }
}

impl ExecutorResolver for GitHeadlessResolver {
    fn resolve(&self, url: &Url, context: ResolverContext<'_>) -> Result<ExecutorResolution> {
        let repository_path = context
            .workspace
            .root()
            .join(get_repository_path(context.package_id));

        if repository_path.try_exists()? {
            std::fs::remove_dir_all(&repository_path)?;
        }

        std::fs::create_dir_all(&repository_path)?;

        let mut repo_builder = git2::build::RepoBuilder::new();
        let mut remote_callbacks = self.default_remote_callbacks();
        (self.remote_callbacks_customizer)(&mut remote_callbacks);

        let mut fetch_options = self.default_fetch_options(remote_callbacks);
        (self.fetch_options_customizer)(&mut fetch_options);

        repo_builder.fetch_options(fetch_options);

        let repository = repo_builder.clone(url.as_ref(), &repository_path)?;

        context
            .logger
            .debug(format!("cloned {} to {}", url, repository_path.display()));

        if let Some(checkout) = &self.git_options.checkout() {
            match checkout {
                GitCheckout::Branch {
                    branch: branch_name,
                } => {
                    let branch = repository
                        .find_branch(&format!("origin/{branch_name}"), git2::BranchType::Remote)?;
                    repository.set_head(
                        branch
                            .into_reference()
                            .name()
                            .ok_or_else(|| anyhow!("could not get refname for {branch_name}"))?,
                    )?;
                }
                GitCheckout::Tag { tag } => {
                    let tag = repository.find_reference(&format!("refs/tags/{tag}"))?;
                    repository.set_head_detached(tag.peel_to_commit()?.id())?;
                }
                GitCheckout::Revision { rev } => {
                    let revision = repository.revparse_single(rev)?;
                    repository.set_head_detached(revision.peel_to_commit()?.id())?;
                }
            }
            repository.checkout_head(Some(&mut CheckoutBuilder::default().force()))?;
        }

        let state = State {
            repository_path: repository_path.to_owned(),
        };

        Ok(ExecutorResolution {
            load_metadata: LoadMetadata {
                kind: self
                    .git_options
                    .kind()
                    .map(Ok::<_, Error>)
                    .unwrap_or_else(|| infer_local_executor_type(&repository_path))?,
                src: if let Some(path) = &self.git_options.path() {
                    repository_path.join(path)
                } else {
                    repository_path
                },
            },
            state: to_value(state)?,
        })
    }

    fn update(
        &self,
        url: &Url,
        context: ResolverContext<'_>,
        state: &Value,
    ) -> Result<ExecutorUpdate> {
        let state = State::deserialize(state)?;
        let repository = git2::Repository::open(&state.repository_path)?;

        if !self.git_options.pull() {
            return Ok(ExecutorUpdate::Keep);
        }

        let refspecs = match &self.git_options.checkout() {
            Some(checkout) => match checkout {
                GitCheckout::Branch { branch } => {
                    vec![format!("refs/heads/{branch}")]
                }
                GitCheckout::Tag { tag } => {
                    vec![format!("refs/tags/{tag}")]
                }
                GitCheckout::Revision { rev } => {
                    vec![rev.to_owned()]
                }
            },
            None => vec!["HEAD".to_owned()],
        };

        let remote_callbacks = self.default_remote_callbacks();
        let mut fetch_options = self.default_fetch_options(remote_callbacks);
        let mut remote = repository.find_remote("origin")?;

        remote.fetch(&refspecs, Some(&mut fetch_options), None)?;

        context
            .logger
            .debug(format!("fetched refspecs {:?} for {}", refspecs, url));

        let fetch_head = repository.find_reference("FETCH_HEAD")?;
        let mut head = repository.head()?;

        let fetch_head_commit = fetch_head
            .resolve()?
            .target()
            .ok_or_else(|| anyhow!("could not resolve commit id for FETCH_HEAD"))?;
        let head_commit = head
            .resolve()?
            .target()
            .ok_or_else(|| anyhow!("could not resolve commit id for HEAD"))?;

        if fetch_head_commit == head_commit {
            context
                .logger
                .debug(format!("{url} is up to date ({fetch_head_commit})"));
            return Ok(ExecutorUpdate::Keep);
        }

        head.set_target(
            fetch_head_commit,
            &format!(
                "Blaze repository update: {:?} to {:?}",
                head.name(),
                fetch_head_commit
            ),
        )?;
        repository.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;

        context.logger.debug(format!(
            "executor files were updated for {url} (now at {fetch_head_commit})"
        ));

        Ok(ExecutorUpdate::Update {
            new_state: None,
            reload_with_metadata: LoadMetadata {
                kind: infer_local_executor_type(&state.repository_path)?,
                src: state.repository_path,
            },
        })
    }
}
