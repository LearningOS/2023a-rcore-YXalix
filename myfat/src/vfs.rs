use alloc::{sync::Arc, vec::Vec, string};
use fatfs::{Dir, File, NullTimeProvider, LossyOemCpConverter, SeekFrom, Seek, Read, Write};
use libvfs::VfsInode;

use spin::Mutex;
use crate::{block_dev::Disk, fs::FatFileSystem};


pub struct FileWrapper<'a>(pub Mutex<File<'a, Disk, NullTimeProvider, LossyOemCpConverter>>);
pub struct DirWrapper<'a>(pub Dir<'a, Disk, NullTimeProvider, LossyOemCpConverter>);

unsafe impl<'a> Send for FileWrapper<'a> {}
unsafe impl<'a> Sync for FileWrapper<'a> {}
unsafe impl<'a> Send for DirWrapper<'a> {}
unsafe impl<'a> Sync for DirWrapper<'a> {}


impl VfsInode for FileWrapper<'static> {
    fn find(&self, _name: &str) -> Option<Arc<dyn VfsInode>> {
        None
    }

    fn create(&self, _name: &str) -> Option<Arc<dyn VfsInode>> {
        None
    }

    fn ls(&self) -> alloc::vec::Vec<alloc::string::String> {
        Vec::new()
    }

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        // let mut file = self.0.lock();
        // file.seek(SeekFrom::Start(offset)).map_err(as_vfs_err)?; // TODO: more efficient
        // file.read(buf).map_err(as_vfs_err)
        let mut file = self.0.lock();
        let _ = file.seek(SeekFrom::Start(offset as u64)); // TODO: more efficient
        // file.read(buf).unwrap();                                                    // file.read(buf).map_err(as_vfs_err)
        let buf_len = buf.len();
        let mut now_offset = 0;
        let mut probe = buf.to_vec();
        while now_offset < buf_len {
            let ans = file.read(&mut probe);
            let read_len = ans.unwrap();

            if read_len == 0 {
                break;
            }
            buf[now_offset..now_offset + read_len].copy_from_slice(&probe[..read_len]);
            now_offset += read_len;
            probe = probe[read_len..].to_vec();
        }
        now_offset
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        // let mut file = self.0.lock();
        // file.seek(SeekFrom::Start(offset)).map_err(as_vfs_err)?; // TODO: more efficient
        // file.write(buf).map_err(as_vfs_err)
        let mut file = self.0.lock();
        let _ = file.seek(SeekFrom::Start(offset as u64)); // TODO: more efficient
                                                                 // file.write(buf).map_err(as_vfs_err)
        let buf_len = buf.len();
        let mut now_offset = 0;
        let mut probe = buf.to_vec();
        while now_offset < buf_len {
            let ans = file.write(&probe);
            let write_len = ans.unwrap();
            if write_len == 0 {
                break;
            }
            now_offset += write_len;
            probe = probe[write_len..].to_vec();
        }
        now_offset
    }

    fn clear(&self) {
        todo!()
    }
}

impl VfsInode for DirWrapper<'static> {
    fn find(&self, _name: &str) -> Option<Arc<dyn VfsInode>> {
        let iter = self.0.iter();
        let mut name_elf = string::String::from(_name);
        name_elf.push_str(".elf");
        for (_, outentry) in iter.enumerate() {
            let x = outentry;
            if let Ok(entry) = x {
                if entry.file_name() == _name || entry.file_name() == name_elf {
                    let item = entry.to_file();
                    return Some(FatFileSystem::new_file(item));
                }
            }
        }
        None
    }

    fn create(&self, _name: &str) -> Option<Arc<dyn VfsInode>> {
        None
    }

    fn ls(&self) -> alloc::vec::Vec<alloc::string::String> {
        let iter = self.0.iter();
        let mut names = Vec::new();
        for (_, outentry) in iter.enumerate() {
            let x = outentry;
            if let Ok(entry) = x {
                names.push(entry.file_name());
            }
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
        ()
    }
    
}