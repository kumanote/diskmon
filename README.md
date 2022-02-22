# diskmon

> diskmon is a simple disk usage monitoring tool.

# Features

- alert if disk capacity is more than threshold(e.g. 0.80 i.e. 80%)
- check multiple mounted paths
- alert to [Airbrake](https://airbrake.io/) (or [Errbit](https://github.com/errbit/errbit))
  - you can customize [logger](https://github.com/kumanote/logger-rs) to change how and where to report the alerting log to.

# How to install

## Prerequisite

- [Rust with Cargo](http://rust-lang.org) `1.57.0` or later.

## Install

```bash
# download
$ git clone git@github.com:kumanote/diskmon.git
# install
$ cd diskmon
$ cargo build --release
# then you will find an executable in the following path
$ ls -ls ./target/release/diskmon
```

# Docker build (optional)

```bash
# download
$ git clone git@github.com:kumanote/diskmon.git
# build
$ docker build -t diskmon:latest .
```

# Run

Please set up config files before running the tool.
See [config.toml.example](config.toml.example) for configuration file example.

```bash
$ diskmon -c /path/to/config.toml
```
