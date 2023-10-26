//! Implementation of [`PageTableEntry`] and [`PageTable`].

use crate::config::MAX_SYSCALL_NUM;
use crate::task::{get_current_task_syscall_times, get_current_task_time};

use super::{frame_alloc, FrameTracker, PhysPageNum, StepByOne, VirtAddr, VirtPageNum};
use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;

bitflags! {
    /// page table entry flags
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
/// page table entry structure
pub struct PageTableEntry {
    /// bits of page table entry
    pub bits: usize,
}

impl PageTableEntry {
    /// Create a new page table entry
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }
    /// Create an empty page table entry
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
    /// Get the physical page number from the page table entry
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    /// Get the flags from the page table entry
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    /// The page pointered by page table entry is valid?
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    /// The page pointered by page table entry is readable?
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    /// The page pointered by page table entry is writable?
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    /// The page pointered by page table entry is executable?
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

/// page table structure
pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

/// Assume that it won't oom when creating/mapping.
impl PageTable {
    /// Create a new page table
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }
    /// Temporarily used to get arguments from user space.
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }
    /// Find PageTableEntry by VirtPageNum, create a frame for a 4KB page table if not exist
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }
    /// Find PageTableEntry by VirtPageNum
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }
    /// set the map between virtual page number and physical page number
    #[allow(unused)]
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    /// remove the map between virtual page number and physical page number
    #[allow(unused)]
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }
    /// get the page table entry from the virtual page number
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }
    /// get the token from the page table
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}

/// Translate&Copy a ptr[u8] array with LENGTH len to a mutable u8 Vec through page table
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token);
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn.step();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        if end_va.page_offset() == 0 {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into();
    }
    v
}


/// put timeinfo into the a ptr[usize] array with LENGTH= 2
pub fn translated_timeinfo(token: usize, ptr: *const usize, us: usize) {
    let page_table = PageTable::from_token(token);
    let start = ptr as usize;

    let sec =  us / 1_000_000;
    let usec =  us % 1_000_000;

    // set second first
    let start_va = VirtAddr::from(start);
    let mut vpn = start_va.floor();
    let ppn = page_table.translate(vpn).unwrap().ppn();
    vpn.step();
    let mut end_va: VirtAddr = vpn.into();
    end_va = end_va.min(VirtAddr::from(start + 8));
    let secref: &mut [usize] = &mut ppn.get_array()[start_va.page_offset()/8..end_va.page_offset()/8];
    secref[0] = sec;

    // set usecond
    let start_va = VirtAddr::from(start + 8);
    let mut vpn = start_va.floor();
    let ppn = page_table.translate(vpn).unwrap().ppn();
    vpn.step();
    let mut end_va: VirtAddr = vpn.into();
    end_va = end_va.min(VirtAddr::from(start + 16));
    let usecref: &mut [usize] = &mut ppn.get_array()[start_va.page_offset()/8..end_va.page_offset()/8];
    usecref[0] = usec;
}

/// put taskinfo into the a ptr[u32] array
pub fn translated_taskinfo(token: usize, ptr: *const u32) {
    let page_table = PageTable::from_token(token);

    // set syscall times
    let task_sys_calls = get_current_task_syscall_times();
    let mut start = ptr as usize;
    let end = start + MAX_SYSCALL_NUM * 4;
    let mut index = 0;
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn.step();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        if end_va.page_offset() == 0 {
            // v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
            let syscallref: &mut [u32] = &mut ppn.get_array()[start_va.page_offset()/4..];
            for i in 0..syscallref.len() {
                syscallref[i] = task_sys_calls[index];
                index += 1;
                if index >= MAX_SYSCALL_NUM{
                    break;
                }
            }
        } else {
            let syscallref: &mut [u32] = &mut ppn.get_array()[start_va.page_offset()/4..end_va.page_offset()/4];
            for i in 0..task_sys_calls.len() {
                syscallref[i] = task_sys_calls[i];
            }
        }
        start = end_va.into();
    }

    let start = ptr as usize + MAX_SYSCALL_NUM * 4 + 8;

    // set task status
    let start_va = VirtAddr::from(start);
    let mut vpn = start_va.floor();
    let ppn = page_table.translate(vpn).unwrap().ppn();
    vpn.step();
    let mut end_va: VirtAddr = vpn.into();
    end_va = end_va.min(VirtAddr::from(start + 8));
    let statusref: &mut [usize] = &mut ppn.get_array()[start_va.page_offset()/8..end_va.page_offset()/8];
    statusref[0] = 2;

    // set time
    let start_va = VirtAddr::from(ptr as usize + MAX_SYSCALL_NUM * 4);
    let mut vpn = start_va.floor();
    let ppn = page_table.translate(vpn).unwrap().ppn();
    vpn.step();
    let mut end_va: VirtAddr = vpn.into();
    end_va = end_va.min(VirtAddr::from(ptr as usize + 16 + MAX_SYSCALL_NUM * 4));
    let timeref: &mut [usize] = &mut ppn.get_array()[start_va.page_offset()/8..end_va.page_offset()/8];
    timeref[0] = get_current_task_time();
}
