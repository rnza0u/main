use std::{
    collections::BTreeSet,
    io,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};
use blaze_common::{
    cache::{FileChangesMatcher, MatchingBehavior},
    error::Result,
    executor::{FileSystemOptions, RebuildStrategy},
    value::{to_value, Value},
    workspace::Workspace,
};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::system::file_changes::{MatchedFiles, MatchedFilesState};

use super::{
    kinds::infer_local_executor_type, loader::LoadMetadata, resolver::ExecutorResolution,
    ExecutorResolver, ExecutorUpdate, ResolverContext,
};

#[derive(Serialize, Deserialize)]
pub struct State {
    files: MatchedFilesState,
}

fn default_file_changes_matchers(root: &Path) -> BTreeSet<FileChangesMatcher> {
    [FileChangesMatcher::new("**")
        .with_exclude(["node_modules/**", "target/**"])
        .with_root(root)
        .with_behavior(MatchingBehavior::Mixed)]
    .into()
}

/// Resolves an executor based on a file URL.
pub struct FileSystemResolver {
    options: FileSystemOptions,
}

impl FileSystemResolver {
    pub fn new(options: FileSystemOptions) -> Self {
        Self { options }
    }

    fn get_matched_files(&self, root: &Path) -> Result<MatchedFiles> {
        let default = default_file_changes_matchers(root);
        MatchedFiles::try_new(root, self.options.watch().unwrap_or(&default))
    }
}

impl ExecutorResolver for FileSystemResolver {
    fn resolve(&self, url: &Url, context: ResolverContext<'_>) -> Result<ExecutorResolution> {
        let root = get_canonical_root_path(url, context.workspace)
            .with_context(|| format!("could not get canonical executor path from {url}"))?;

        let is_dir = match std::fs::metadata(&root) {
            Ok(metadata) => metadata.is_dir(),
            Err(err) if err.kind() == io::ErrorKind::NotFound => false,
            Err(err) => return Err(err.into()),
        };

        if !is_dir {
            bail!("{url} is not a directory. file:// URLs must point to the source files root directory of your executor.")
        }
        Ok(ExecutorResolution {
            state: to_value(State {
                files: MatchedFilesState::from_files(self.get_matched_files(&root)?)?,
            })?,
            load_metadata: LoadMetadata {
                kind: if let Some(kind) = self.options.kind() {
                    kind
                } else {
                    infer_local_executor_type(&root)?
                },
                src: root,
            },
        })
    }

    fn update(
        &self,
        url: &Url,
        context: ResolverContext<'_>,
        state: &Value,
    ) -> Result<ExecutorUpdate> {
        let root = get_canonical_root_path(url, context.workspace)
            .with_context(|| format!("could not get canonical executor path from {url}"))?;
        let state = State::deserialize(state)?;

        let matched_files = self.get_matched_files(&root)?;
        let merged_state = state.files.merge(matched_files)?;

        let new_state = State {
            files: merged_state.state,
        };

        let update = || {
            Ok(ExecutorUpdate::Update {
                new_state: Some(to_value(new_state)?),
                reload_with_metadata: LoadMetadata {
                    kind: infer_local_executor_type(&root)?,
                    src: root,
                },
            })
        };

        match self.options.rebuild() {
            RebuildStrategy::Always => update(),
            RebuildStrategy::OnChanges if !merged_state.changes.is_empty() => update(),
            _ => Ok(ExecutorUpdate::Keep),
        }
    }
}

fn get_canonical_root_path(url: &Url, workspace: &Workspace) -> Result<PathBuf> {
    let url_path = Path::new(url.path());
    let absolute = if url_path.is_absolute() {
        url_path.to_path_buf()
    } else {
        workspace.root().join(url_path)
    };
    Ok(dunce::canonicalize(absolute)?)
}
