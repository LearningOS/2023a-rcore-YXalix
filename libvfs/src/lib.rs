#![cfg_attr(not(test), no_std)]

extern crate alloc;
use core::any::Any;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

pub trait BlockDevice: Send + Sync + Any {
    /// Read a block from the block device.
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    /// Write a block to the block device.
    fn write_block(&self, block_id: usize, buf: &[u8]);
}

/// The VfsInode trait
pub trait VfsInode:Send+Sync {
    /// find the disk inode of the file with 'name'
    fn find(&self, name: &str) -> Option<Arc<dyn VfsInode>>;
    /// create a file with 'name' in the directory
    fn create(&self, name: &str) -> Option<Arc<dyn VfsInode>>;
    /// list the file names in the root directory
    fn ls(&self) -> Vec<String>;
    /// Read the content in offset position of the file into 'buf'
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize;
    /// Write the content in 'buf' into offset position of the file
    fn write_at(&self, offset: usize, buf: &[u8]) -> usize;
    /// Set the file(disk inode) length to zero, delloc all data blocks of the file.
    fn clear(&self);
}