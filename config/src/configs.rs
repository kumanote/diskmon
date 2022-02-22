use crate::toml::*;
use crate::{CheckMethod, Result};
use anyhow::{anyhow, Context};
use std::env;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

const DEFAULT_INTERVAL: &'static str = "10s";

pub trait FromEnv: Sized {
    fn from_env() -> Result<Self>;
}

pub trait SelfValidation: Sized {
    fn validate(&self) -> Result<()>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct ApplicationConfig {
    pub interval: String,
    pub targets: Vec<TargetConfig>,
    pub logger: LoggerConfig,
}

impl FromEnv for ApplicationConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            interval: DEFAULT_INTERVAL.to_owned(),
            targets: Vec::new(),
            logger: LoggerConfig::from_env()?,
        })
    }
}

impl SelfValidation for ApplicationConfig {
    fn validate(&self) -> Result<()> {
        let _interval = duration_str::parse(self.interval.as_str())
            .with_context(|| format!("illegal interval: {}", self.interval.as_str()))?;
        for t in &self.targets {
            let _ok = t.validate()?;
        }
        let _ok = self.logger.validate()?;
        Ok(())
    }
}

impl ApplicationConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let app_toml = ApplicationToml::load_from_file(path)?;
        let interval = if let Some(interval) = app_toml.interval {
            interval
        } else {
            DEFAULT_INTERVAL.to_owned()
        };
        let mut targets = Vec::new();
        for target in app_toml.targets {
            let target = target.try_into()?;
            targets.push(target);
        }
        let logger = match app_toml.logger {
            Some(logger) => logger.try_into()?,
            None => LoggerConfig::from_env()?,
        };
        Ok(Self {
            interval,
            targets,
            logger,
        })
    }

    pub fn add_target(&mut self, target: TargetConfig) {
        self.targets.push(target)
    }

    pub fn get_interval(&self) -> Duration {
        duration_str::parse(self.interval.as_str()).expect("illegal interval config value...")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TargetConfig {
    pub mount_point: String,
    pub check_method: String,
    pub threshold: String,
}

impl TargetConfig {
    pub fn get_check_method(&self) -> Result<CheckMethod> {
        CheckMethod::from(self.check_method.as_str(), self.threshold.as_str())
    }
}

impl SelfValidation for TargetConfig {
    fn validate(&self) -> Result<()> {
        let _ = self.get_check_method()?;
        Ok(())
    }
}

impl FromEnv for TargetConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            mount_point: "".to_owned(),
            check_method: "".to_owned(),
            threshold: "".to_owned(),
        })
    }
}

impl TryFrom<TargetToml> for TargetConfig {
    type Error = anyhow::Error;

    fn try_from(toml: TargetToml) -> Result<Self> {
        let mut result = Self::from_env()?;
        if let Some(mount_point) = toml.mount_point {
            result.mount_point = mount_point;
        }
        if let Some(check_method) = toml.check_method {
            result.check_method = check_method;
        }
        if let Some(threshold) = toml.threshold {
            result.threshold = threshold;
        }
        Ok(result)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoggerConfig {
    pub chan_size: Option<usize>,
    pub is_async: bool,
    pub level: Option<String>,
    pub airbrake_host: Option<String>,
    pub airbrake_project_id: Option<String>,
    pub airbrake_project_key: Option<String>,
    pub airbrake_environment: Option<String>,
}

impl FromEnv for LoggerConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            chan_size: None,
            is_async: true,
            level: None,
            airbrake_host: try_get_env_var("AIRBRAKE_HOST")?,
            airbrake_project_id: try_get_env_var("AIRBRAKE_PROJECT_ID")?,
            airbrake_project_key: try_get_env_var("AIRBRAKE_PROJECT_KEY")?,
            airbrake_environment: try_get_env_var("AIRBRAKE_ENVIRONMENT")?,
        })
    }
}

impl SelfValidation for LoggerConfig {
    fn validate(&self) -> Result<()> {
        if let Some(level) = self.level.as_deref() {
            let valid_values = vec!["CRASH", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"];
            if valid_values.iter().find(|&&x| x == level).is_none() {
                return Err(anyhow!("illegal logger level: {}", level));
            }
        }
        Ok(())
    }
}

impl TryFrom<LoggerToml> for LoggerConfig {
    type Error = anyhow::Error;

    fn try_from(toml: LoggerToml) -> Result<Self> {
        let mut result = Self::from_env()?;
        if let Some(chan_size) = toml.chan_size {
            result.chan_size = Some(chan_size);
        }
        if let Some(is_async) = toml.is_async {
            result.is_async = is_async;
        }
        if let Some(level) = toml.level {
            result.level = Some(level);
        }
        if let Some(airbrake_host) = toml.airbrake_host {
            result.airbrake_host = Some(airbrake_host);
        }
        if let Some(airbrake_project_id) = toml.airbrake_project_id {
            result.airbrake_project_id = Some(airbrake_project_id);
        }
        if let Some(airbrake_project_key) = toml.airbrake_project_key {
            result.airbrake_project_key = Some(airbrake_project_key);
        }
        if let Some(airbrake_environment) = toml.airbrake_environment {
            result.airbrake_environment = Some(airbrake_environment);
        }
        Ok(result)
    }
}

#[allow(dead_code)]
fn get_env_var<T: FromStr>(var_name: &str, default_value: T) -> Result<T> {
    match env::var(var_name) {
        Ok(val) => val
            .parse::<T>()
            .map_err(|_| anyhow!("illegal env var: {}", var_name)),
        Err(_) => Ok(default_value),
    }
}

#[allow(dead_code)]
fn try_get_env_var<T: FromStr>(var_name: &str) -> Result<Option<T>> {
    match env::var(var_name) {
        Ok(val) => val
            .parse::<T>()
            .map(|v| Some(v))
            .map_err(|_| anyhow!("illegal env var: {}", var_name)),
        Err(_) => Ok(None),
    }
}
