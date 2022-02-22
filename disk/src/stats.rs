use crate::Result;
use anyhow::anyhow;
use std::{ffi, mem, os::unix::ffi::OsStrExt, path};

#[derive(Debug, Clone)]
pub struct Stats {
    pub bsize: u64,
    pub blocks: u64,
    pub bfree: u64,
    pub bavail: u64,
    pub files: u64,
    pub ffree: u64,
    pub favail: u64,
}

impl Stats {
    pub fn from<P: AsRef<path::Path>>(mount_point: P) -> Result<Option<Self>> {
        let mount_point = mount_point.as_ref();
        let c_mount_point = ffi::CString::new(mount_point.as_os_str().as_bytes()).unwrap();
        unsafe {
            let mut statvfs = mem::MaybeUninit::<libc::statvfs>::uninit();
            let code = libc::statvfs(c_mount_point.as_ptr(), statvfs.as_mut_ptr());
            match code {
                0 => {
                    // good
                    let statvfs = statvfs.assume_init();
                    Ok(Some(Stats {
                        bsize: statvfs.f_bsize as u64,
                        blocks: statvfs.f_blocks as u64,
                        bfree: statvfs.f_bfree as u64,
                        bavail: statvfs.f_bavail as u64,
                        files: statvfs.f_files as u64,
                        ffree: statvfs.f_ffree as u64,
                        favail: statvfs.f_favail as u64,
                    }))
                }
                -1 => Ok(None),
                _ => Err(anyhow!(
                    "libc.statvfs({:?}) returned {}",
                    mount_point.to_path_buf(),
                    code
                )),
            }
        }
    }
    pub fn size(&self) -> u64 {
        self.bsize * self.blocks
    }
    pub fn available(&self) -> u64 {
        self.bsize * self.bavail
    }
    pub fn used(&self) -> u64 {
        self.size() - self.available()
    }
    pub fn inodes_used(&self) -> u64 {
        self.files - self.favail // this will panic on unconsistent data
    }
    pub fn inodes_use_share(&self) -> f64 {
        if self.files == 0 {
            0.0
        } else {
            self.inodes_used() as f64 / self.files as f64
        }
    }
    pub fn use_share(&self) -> f64 {
        if self.size() == 0 {
            0.0
        } else {
            self.used() as f64 / (self.size() as f64)
        }
    }
}
