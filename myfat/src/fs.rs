
use alloc::sync::Arc;
use libvfs::{VfsInode, BlockDevice};
use spin::Mutex;
use core::cell::UnsafeCell;

use fatfs::{Dir, File, LossyOemCpConverter, NullTimeProvider};

use crate::{block_dev::Disk, vfs::{FileWrapper, DirWrapper}};

pub struct FatFileSystem {
    inner: fatfs::FileSystem<Disk, NullTimeProvider, LossyOemCpConverter>,
    root_dir: UnsafeCell<Option<Arc<dyn VfsInode>>>,
}


unsafe impl Sync for FatFileSystem {}
unsafe impl Send for FatFileSystem {}



impl FatFileSystem {
    pub fn new(block_device: Arc<dyn BlockDevice>) -> Self {
        let disk = Disk::new(block_device);
        let inner = fatfs::FileSystem::new(disk, fatfs::FsOptions::new())
            .expect("failed to initialize FAT filesystem");
        Self {
            inner,
            root_dir: UnsafeCell::new(None),
        }
    }

    pub fn init(&'static self) {
        // must be called before later operations
        unsafe { *self.root_dir.get() = Some(Self::new_dir(self.inner.root_dir())) }
    }

    pub fn new_file(file: File<'_, Disk, NullTimeProvider, LossyOemCpConverter>) -> Arc<FileWrapper> {
        Arc::new(FileWrapper(Mutex::new(file)))
    }

    fn new_dir(dir: Dir<'_, Disk, NullTimeProvider, LossyOemCpConverter>) -> Arc<DirWrapper> {
        Arc::new(DirWrapper(dir))
    }

    pub fn root_dir(&self) -> Arc<dyn VfsInode> {
        let root_dir = unsafe { (*self.root_dir.get()).as_ref().unwrap() };
        root_dir.clone()
    }

}