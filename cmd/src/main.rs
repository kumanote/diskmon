use config::configs::{SelfValidation, TargetConfig};
use logger::default::DefaultLoggerBuilder;
use logger::prelude::*;
use std::panic::{self, PanicInfo};
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "diskmon running options")]
struct Opts {
    #[structopt(short = "c", long, help = "Path to Config")]
    config: Option<PathBuf>,
    #[structopt(short = "p", long, help = "Mount path to check")]
    mount_point: Option<String>,
    #[structopt(short = "m", long, help = "How to check the mount point")]
    check_method: Option<String>,
    #[structopt(short = "t", long, help = "Check threshold")]
    threshold: Option<String>,
}

fn main() {
    panic::set_hook(Box::new(move |panic_info: &PanicInfo<'_>| {
        let details = format!("{}", panic_info);
        crash!("{}", details);
        logger::flush();
        // Kill the process
        process::exit(12);
    }));

    // load configs
    let options: Opts = Opts::from_args();
    let mut config =
        config::load_app_config(options.config.as_ref()).expect("Failed to load config file...");
    let config = if let Some(mount_point) = options.mount_point.as_deref() {
        let check_method = options.check_method.as_deref().unwrap_or("");
        let threshold = options.threshold.as_deref().unwrap_or("");
        config.add_target(TargetConfig {
            mount_point: mount_point.to_owned(),
            check_method: check_method.to_owned(),
            threshold: threshold.to_owned(),
        });
        config::set_app_config(Arc::new(config));
        config::app_config()
    } else {
        &config
    };

    if let Err(err) = config.validate() {
        eprintln!("{}", err);
        return;
    }

    // set up logger
    let mut logger_builder = DefaultLoggerBuilder::new();
    logger_builder.is_async(config.logger.is_async);
    if let Some(chan_size) = config.logger.chan_size {
        logger_builder.channel_size(chan_size);
    }
    if let Some(level) = config.logger.level.as_deref() {
        let level = level.parse().expect("log level must be valid");
        logger_builder.level(level);
    }
    if let Some(airbrake_host) = config.logger.airbrake_host.as_deref() {
        logger_builder.airbrake_host(airbrake_host.to_owned());
    }
    if let Some(airbrake_project_id) = config.logger.airbrake_project_id.as_deref() {
        logger_builder.airbrake_project_id(airbrake_project_id.to_owned());
    }
    if let Some(airbrake_project_key) = config.logger.airbrake_project_key.as_deref() {
        logger_builder.airbrake_project_key(airbrake_project_key.to_owned());
    }
    if let Some(airbrake_environment) = config.logger.airbrake_environment.as_deref() {
        logger_builder.airbrake_environment(airbrake_environment.to_owned());
    }
    let _logger = logger_builder.build();

    // Let's now log some important information, since the logger is set up
    debug!(
        "Loaded disk usage daemon monitoring tool config, config: {:?}",
        config
    );

    if let Err(err) = diskmon::start() {
        error!("{}", err);
    }

    logger::flush();
}
