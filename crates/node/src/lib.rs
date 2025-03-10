pub mod depman;
mod detect;
pub mod download;
mod execute;
mod install;
mod platform;
mod resolve;
mod shim;
mod verify;

pub use depman::*;

use once_cell::sync::OnceCell;
use proto_core::{impl_tool, Describable, Manifest, Proto, ProtoError, Tool};
use std::{
    any::Any,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct NodeLanguage {
    pub base_dir: PathBuf,
    pub bin_path: Option<PathBuf>,
    pub shim_path: Option<PathBuf>,
    pub temp_dir: PathBuf,
    pub version: Option<String>,

    manifest: OnceCell<Manifest>,
}

impl NodeLanguage {
    pub fn new<P: AsRef<Proto>>(proto: P) -> Self {
        let proto = proto.as_ref();

        NodeLanguage {
            base_dir: proto.tools_dir.join("node"),
            bin_path: None,
            manifest: OnceCell::new(),
            shim_path: None,
            temp_dir: proto.temp_dir.join("node"),
            version: None,
        }
    }
}

impl Describable<'_> for NodeLanguage {
    fn get_id(&self) -> &str {
        "node"
    }

    fn get_name(&self) -> String {
        "Node.js".into()
    }
}

impl_tool!(NodeLanguage);
