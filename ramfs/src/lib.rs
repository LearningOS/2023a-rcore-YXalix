#![cfg_attr(not(test), no_std)]

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicUsize, Ordering};
pub use libvfs::VfsInode;
use spin::Mutex;

pub trait RamFsProvider: Send + Sync {
    fn current_time() -> TimeSpec;
}
pub struct TimeSpec {
    pub sec: usize,
    pub nsec: usize,
}

pub struct RamFs<T> {
    inode_index: AtomicUsize,
    inode_count: AtomicUsize,
    root: Mutex<Option<Arc<RamFsDirInode<T>>>>,
    _provider: PhantomData<T>,
}

impl<T: RamFsProvider> RamFs<T> {
    pub fn new() -> Arc<Self> {
        let fs = Arc::new(Self {
            inode_index: AtomicUsize::new(0),
            inode_count: AtomicUsize::new(0),
            root: Mutex::new(None),
            _provider: PhantomData,
        });
        fs
    }
    pub fn root(self: &Arc<Self>) -> Arc<RamFsDirInode<T>> {
        let mut root = self.root.lock();
        if root.is_none() {
            let inode = Arc::new(RamFsDirInode::new(0, String::from("/"), self.clone()));
            *root = Some(inode.clone());
            self.inode_count.fetch_add(1, Ordering::SeqCst);
            self.inode_index.fetch_add(1, Ordering::SeqCst);
        }
        root.clone().unwrap()
    }
    fn alloc_inode(&self) -> usize {
        self.inode_count.fetch_add(1, Ordering::SeqCst);
        self.inode_index.fetch_add(1, Ordering::SeqCst)
    }
}

pub struct RamFsDirInode<T> {
    id: usize,
    name: String,
    children: Mutex<Vec<Arc<RamFsFileInode>>>,
    fs: Arc<RamFs<T>>,
    ctime: TimeSpec,
}

impl<T: RamFsProvider> RamFsDirInode<T> {
    pub fn new(id: usize, name: String, fs: Arc<RamFs<T>>) -> Self {
        Self {
            id,
            name,
            children: Mutex::new(Vec::new()),
            fs,
            ctime: T::current_time(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn id(&self) -> usize {
        self.id
    }
}

impl<T: RamFsProvider> VfsInode for RamFsDirInode<T> {
    fn find(&self, name: &str) -> Option<Arc<dyn VfsInode>> {
        let children = self.children.lock();
        for child in children.iter() {
            if child.name() == name {
                return Some(child.clone());
            }
        }
        None
    }

    fn create(&self, name: &str) -> Option<Arc<dyn VfsInode>> {
        let mut children = self.children.lock();
        let inode = Arc::new(RamFsFileInode::new(
            self.fs.alloc_inode(),
            String::from(name),
            T::current_time(),
        ));
        children.push(inode.clone());
        Some(inode)
    }

    fn ls(&self) -> Vec<String> {
        let children = self.children.lock();
        let mut names = Vec::new();
        for child in children.iter() {
            names.push(child.name().to_string());
        }
        names
    }

    fn read_at(&self, _offset: usize, _buf: &mut [u8]) -> usize {
        0
    }

    fn write_at(&self, _offset: usize, _buf: &[u8]) -> usize {
        0
    }

    fn clear(&self) {
        let mut children = self.children.lock();
        children.clear();
    }
}

pub struct RamFsFileInode {
    id: usize,
    name: String,
    content: Mutex<Vec<u8>>,
    ctime: TimeSpec,
}

impl RamFsFileInode {
    pub fn new(id: usize, name: String, ctime: TimeSpec) -> Self {
        Self {
            id,
            name,
            content: Mutex::new(Vec::new()),
            ctime,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn id(&self) -> usize {
        self.id
    }
}

impl VfsInode for RamFsFileInode {
    fn find(&self, _name: &str) -> Option<Arc<dyn VfsInode>> {
        None
    }

    fn create(&self, _name: &str) -> Option<Arc<dyn VfsInode>> {
        None
    }

    fn ls(&self) -> Vec<String> {
        Vec::new()
    }

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let content = self.content.lock();
        let len = content.len();
        let buf_len = buf.len();
        let copy_start = offset.min(len);
        let copy_end = (offset + buf_len).min(len);
        buf[..(copy_end - copy_start)].copy_from_slice(&content[copy_start..copy_end]);
        copy_end - copy_start
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let mut content = self.content.lock();
        let len = content.len();
        let buf_len = buf.len();
        if len < offset + buf_len {
            content.resize(offset + buf_len, 0);
        }
        let copy_end = offset + buf_len;
        content[offset..copy_end].copy_from_slice(buf);
        buf_len
    }

    fn clear(&self) {
        let mut content = self.content.lock();
        content.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RamFsProvider;
    struct Provider;
    impl RamFsProvider for Provider {
        fn current_time() -> crate::TimeSpec {
            crate::TimeSpec { sec: 0, nsec: 0 }
        }
    }
    #[test]
    fn test_find() {
        let fs = RamFs::<Provider>::new();
        let root = fs.root();
        let etc = root.create("etc").unwrap();
        let passwd = root.create("passwd").unwrap();
        let _etc = root.find("etc").unwrap();
        let _passwd = root.find("passwd").unwrap();
        assert!(Arc::ptr_eq(&etc, &_etc));
        assert!(Arc::ptr_eq(&passwd, &_passwd));
        let no = root.find("no");
        assert!(no.is_none());
    }
    #[test]
    fn test_read() {
        let fs = RamFs::<Provider>::new();
        let root = fs.root();
        let r = root.read_at(0, &mut [0u8; 10]);
        assert_eq!(r, 0);
        let etc = root.create("etc").unwrap();
        etc.write_at(0, "hello world!".as_bytes());
        let mut buf = [0u8; 20];
        let r = etc.read_at(0, &mut buf);
        assert_eq!(r, 12);
        assert_eq!(core::str::from_utf8(&buf[..r]).unwrap(), "hello world!");
    }
}