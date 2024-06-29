use blaze_common::{error::Result, executor::GitOptions, value::Value};
use url::Url;

use super::{
    git_common::GitHeadlessResolver,
    resolver::{ExecutorResolution, ExecutorResolver, ExecutorUpdate, ResolverContext},
};

pub struct GitResolver {
    delegate: GitHeadlessResolver,
}

impl GitResolver {
    pub fn new(options: GitOptions) -> Self {
        Self {
            delegate: GitHeadlessResolver::new(options, |_| {}, |_| {}),
        }
    }
}

impl ExecutorResolver for GitResolver {
    fn resolve(&self, url: &Url, context: ResolverContext<'_>) -> Result<ExecutorResolution> {
        self.delegate.resolve(url, context)
    }

    fn update(
        &self,
        url: &Url,
        context: ResolverContext<'_>,
        state: &Value,
    ) -> Result<ExecutorUpdate> {
        self.delegate.update(url, context, state)
    }
}
