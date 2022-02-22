use crate::Result;
use anyhow::Context;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Deserialize, Eq, PartialEq, Clone)]
pub struct ApplicationToml {
    pub interval: Option<String>,
    pub targets: Vec<TargetToml>,
    pub logger: Option<LoggerToml>,
}

impl ApplicationToml {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(&path).with_context(|| {
            format!(
                "could not open toml file of {:?}",
                path.as_ref().as_os_str()
            )
        })?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).with_context(|| {
            format!(
                "could not read toml file of {:?}",
                path.as_ref().as_os_str()
            )
        })?;
        toml::from_str(contents.as_str()).with_context(|| {
            format!(
                "could not parse toml file of {:?}",
                path.as_ref().as_os_str()
            )
        })
    }
}

#[derive(Deserialize, Eq, PartialEq, Clone)]
pub struct TargetToml {
    pub mount_point: Option<String>,
    pub check_method: Option<String>,
    pub threshold: Option<String>,
}

#[derive(Deserialize, Eq, PartialEq, Clone)]
pub struct LoggerToml {
    pub chan_size: Option<usize>,
    pub is_async: Option<bool>,
    pub level: Option<String>,
    pub airbrake_host: Option<String>,
    pub airbrake_project_id: Option<String>,
    pub airbrake_project_key: Option<String>,
    pub airbrake_environment: Option<String>,
}
