//! File and filesystem-related syscalls
use alloc::sync::Arc;

use crate::fs::{open_file, OpenFlags, Stat,get_root_inode, OSInode, StatMode};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer, translated_refmut};
use crate::task::{current_task, current_user_token};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        println!("fd:{} fd_table.len():{} no fd",fd,inner.fd_table.len());
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        // if !file.writable() {
        //     return -1;
        // }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // if !file.readable() {
        //     return -1;
        // }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(_fd: usize, _st: *mut Stat) -> isize {
    trace!("kernel:pid[{}] sys_fstat",current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    let rootnode = get_root_inode();
    println!("fd_table.len():{} _fd:{}",inner.fd_table.len(), _fd);
    if _fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[_fd] {
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_fstat .. file.read");
        unsafe{
            let file = Arc::into_raw(file);
            let file = Arc::from_raw(file as *const OSInode);
            let inode_id = file.get_inode_id();
            let nlink = rootnode.countlink_id(inode_id);
            let mode = (file).get_inode_type();
            let nlink_ref = &(*_st).nlink as *const u32 as *mut u32;
            *translated_refmut(token, nlink_ref) = nlink as u32;
            let mode_ref = &(*_st).mode as *const StatMode as *mut StatMode;
            *translated_refmut(token, mode_ref) = mode;
            let dev_ref = &(*_st).dev as *const u64 as *mut u64;
            *translated_refmut(token, dev_ref) = 0;
            let ino_ref = &(*_st).ino as *const u64 as *mut u64;
            *translated_refmut(token, ino_ref) = inode_id as u64;

            println!("nlink:{} mode:{:x} dev:{} ino:{}",nlink,mode.bits(),0,inode_id);
        }
        return 0;
    } else {
        -1
    }
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(_old_name: *const u8, _new_name: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_linkat",current_task().unwrap().pid.0);
    let token = current_user_token();
    let old_name = translated_str(token, _old_name);
    let new_name = translated_str(token, _new_name);
    let inode = get_root_inode();
    if let Some(_) = inode.find(old_name.as_str()) {
        inode.link(new_name.as_str(), old_name.as_str())
    } else {
        -1
    }
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(_name: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_unlinkat",current_task().unwrap().pid.0);
    let token = current_user_token();
    let name = translated_str(token, _name);
    let root_node = get_root_inode();
    if let Some(inode) = root_node.find(name.as_str()) {
        root_node.unlink(name.as_str(),inode.inode_id())
    } else {
        -1
    }
}
