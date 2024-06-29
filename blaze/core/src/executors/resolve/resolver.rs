use std::hash::{Hash, Hasher};

use blaze_common::{
    error::Result,
    executor::{ExecutorReference, Location},
    logger::Logger,
    value::Value,
    workspace::Workspace,
};
use url::Url;

use crate::system::hash::hasher;

use super::{
    file_system::FileSystemResolver, git::GitResolver, http_git::GitOverHttpResolver,
    loader::LoadMetadata, ssh_git::GitOverSshResolver,
};

#[derive(Clone, Copy)]
pub struct ResolverContext<'a> {
    pub workspace: &'a Workspace,
    pub logger: &'a Logger<'a>,
    pub package_id: u64,
}

pub struct ExecutorResolution {
    pub load_metadata: LoadMetadata,
    pub state: Value,
}

pub enum ExecutorUpdate {
    Keep,
    Update {
        new_state: Option<Value>,
        reload_with_metadata: LoadMetadata,
    },
}

pub trait ExecutorResolver {
    fn resolve(&self, url: &Url, context: ResolverContext<'_>) -> Result<ExecutorResolution>;

    fn update(
        &self,
        url: &Url,
        context: ResolverContext<'_>,
        state: &Value,
    ) -> Result<ExecutorUpdate>;
}

pub fn get_executor_package_id(reference: &ExecutorReference) -> u64 {
    let mut hasher = hasher();
    match reference {
        ExecutorReference::Standard { url } => {
            url.hash(&mut hasher);
        }
        ExecutorReference::Custom { url, location } => {
            url.hash(&mut hasher);
            match location {
                Location::GitOverHttp {
                    transport,
                    git_options,
                    authentication,
                } => {
                    transport.headers().hash(&mut hasher);
                    git_options.checkout().hash(&mut hasher);
                    authentication.hash(&mut hasher);
                }
                Location::GitOverSsh {
                    git_options,
                    authentication,
                    ..
                } => {
                    git_options.checkout().hash(&mut hasher);
                    authentication.hash(&mut hasher);
                }
                Location::TarballOverHttp {
                    transport,
                    authentication,
                    ..
                } => {
                    transport.headers().hash(&mut hasher);
                    authentication.hash(&mut hasher);
                }
                Location::LocalFileSystem { .. } => {}
                Location::Npm { options } => {
                    options.version().hash(&mut hasher);
                    options.token().hash(&mut hasher);
                }
                Location::Cargo { options } => {
                    options.version().hash(&mut hasher);
                    options.token().hash(&mut hasher);
                }
                Location::Git { options } => {
                    options.checkout().hash(&mut hasher);
                }
            }
        }
    }
    hasher.finish()
}

pub fn resolver_for_location(location: Location) -> Box<dyn ExecutorResolver> {
    match location {
        Location::LocalFileSystem { options } => Box::new(FileSystemResolver::new(options)),
        Location::Git { options } => Box::new(GitResolver::new(options)),
        Location::GitOverHttp {
            transport,
            git_options,
            authentication,
        } => Box::new(GitOverHttpResolver::new(
            git_options,
            transport,
            authentication,
        )),
        Location::GitOverSsh {
            transport,
            git_options,
            authentication,
        } => Box::new(GitOverSshResolver::new(
            git_options,
            transport,
            authentication,
        )),
        _ => todo!(),
    }
}
