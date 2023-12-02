use alloc::sync::Arc;
use libvfs::{BlockDevice, VfsInode};

use crate::{fs::FatFileSystem, lazy_init::LazyInit};




pub(crate) fn init_rootfs(blk_devs: Arc<dyn BlockDevice>) -> Arc<dyn VfsInode>{
    static FAT_FS: LazyInit<Arc<FatFileSystem>> = LazyInit::new();
    FAT_FS.init_by(Arc::new(FatFileSystem::new(blk_devs)));
    FAT_FS.init();
    FAT_FS.root_dir()
}