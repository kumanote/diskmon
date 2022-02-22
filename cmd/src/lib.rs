use anyhow::anyhow;
use config::CheckMethod;
use disk::Stats;
use logger::prelude::*;
use std::{path, thread};
use tokio::signal::unix::{signal, SignalKind};

pub type Result<T> = anyhow::Result<T>;

pub fn start() -> Result<()> {
    let app_config = config::app_config();
    let mut managers = Vec::new();
    for target in &app_config.targets {
        let manager = CheckManager {
            mount_point: target.mount_point.clone(),
            check_method: target.get_check_method()?,
        };
        managers.push(manager);
    }
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("tick")
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime!");
    let interval = app_config.get_interval();

    runtime.block_on(async move {
        let mut sigint =
            signal(SignalKind::interrupt()).expect("signal interrupt must be captured...");
        let mut sigterm =
            signal(SignalKind::terminate()).expect("signal termination must be captured...");
        let mut sleep = false;
        loop {
            let tick = async {
                if sleep {
                    thread::sleep(interval);
                    debug!("next tick");
                }
            };
            tokio::select! {
                _ = sigint.recv() => {
                    info!("sigint detected");
                    break
                }
                _ = sigterm.recv() => {
                    info!("sigterm detected");
                    break
                }
                _ = tick => {
                    for manager in &mut managers {
                        if let Err(err) = manager.check() {
                            error!("{}", err);
                        }
                    }
                    sleep = true;
                }
            }
        }
    });
    Ok(())
}

pub struct CheckManager {
    mount_point: String,
    check_method: CheckMethod,
}

impl CheckManager {
    fn check(&self) -> Result<()> {
        match self.check_method {
            CheckMethod::DiskCapacityRate { threshold } => {
                let mount_point = path::PathBuf::from(self.mount_point.as_str());
                let stats = Stats::from(mount_point)?;
                match stats {
                    Some(stats) => {
                        let current = stats.use_share();
                        if threshold < current {
                            Err(anyhow!(
                                "the mount_point: {} capacity over its threshold {} > {}",
                                self.mount_point.as_str(),
                                current,
                                threshold
                            ))
                        } else {
                            info!(
                                "the mount_point: {} capacity ok {} <= {}",
                                self.mount_point.as_str(),
                                current,
                                threshold
                            );
                            Ok(())
                        }
                    }
                    None => Err(anyhow!(
                        "fs stats not found...mount_point: {}",
                        self.mount_point.as_str()
                    )),
                }
            }
        }
    }
}
