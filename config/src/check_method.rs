use crate::Result;
use anyhow::anyhow;
use anyhow::Context;

pub enum CheckMethod {
    DiskCapacityRate { threshold: f64 },
}

impl CheckMethod {
    pub fn from(method: &str, threshold: &str) -> Result<Self> {
        let method = method.to_lowercase();
        let method = method.as_str();
        if method == "" || method.contains("capacity_rate") {
            let threshold = threshold
                .parse()
                .with_context(|| format!("threshold must be in float format: {}", threshold))?;
            return Ok(Self::DiskCapacityRate { threshold });
        }
        return Err(anyhow!("unexpected check method type: {}", method));
    }
}
