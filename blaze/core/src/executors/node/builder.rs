use std::path::Path;

use blaze_common::error::Result;

use crate::executors::builder::ExecutorBuilder;

use super::package::NodeExecutorPackage;

pub struct NodeExecutorBuilder;

impl ExecutorBuilder for NodeExecutorBuilder {
    fn build(&self, root: &Path) -> Result<()> {
        let package = NodeExecutorPackage::from_root(root)?;

        
    }
}