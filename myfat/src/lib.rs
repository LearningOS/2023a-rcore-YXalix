#![no_std]
extern crate log;
extern crate alloc;

pub mod block_dev;
pub mod driver_common;
pub mod vfs;
pub mod fs;
pub mod root;
pub mod lazy_init;

use alloc::sync::Arc;
pub use libvfs::{VfsInode, BlockDevice};
use log::info;

/// Initializes filesystems by block devices.
pub fn init_filesystems(blk_devs: Arc<dyn BlockDevice>) -> Arc<dyn VfsInode>{
    info!("Initialize filesystems...");
    self::root::init_rootfs(blk_devs)
}
