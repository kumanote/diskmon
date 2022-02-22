pub type Result<T> = anyhow::Result<T>;

mod stats;

pub use stats::Stats;
