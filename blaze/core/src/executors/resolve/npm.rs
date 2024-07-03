use blaze_common::{error::Result, executor::NpmOptions, value::Value};
use url::Url;

use super::resolver::{ExecutorResolver, ExecutorSource};

#[allow(unused)]
struct NpmResolver {
    options: NpmOptions,
}

impl ExecutorResolver for NpmResolver {
    fn resolve(&self, _url: &Url) -> Result<ExecutorSource> {
        todo!()
    }

    fn update(&self, _url: &Url, _state: &Value) -> Result<Option<ExecutorSource>> {
        todo!()
    }
}
 