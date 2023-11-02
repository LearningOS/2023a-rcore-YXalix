//! Mutex (spin-like and blocking(sleep))

use super::UPSafeCell;
use crate::task::{TaskControlBlock, current_process};
use crate::task::{block_current_and_run_next, suspend_current_and_run_next};
use crate::task::{current_task, wakeup_task};
use alloc::{collections::VecDeque, sync::Arc};

/// Mutex trait
pub trait Mutex: Sync + Send {
    /// Lock the mutex
    fn lock(&self);
    /// Unlock the mutex
    fn unlock(&self);
}

/// Spinlock Mutex struct
pub struct MutexSpin {
    locked: UPSafeCell<bool>,
}

impl MutexSpin {
    /// Create a new spinlock mutex
    pub fn new() -> Self {
        Self {
            locked: unsafe { UPSafeCell::new(false) },
        }
    }
}

impl Mutex for MutexSpin {
    /// Lock the spinlock mutex
    fn lock(&self) {
        trace!("kernel: MutexSpin::lock");
        loop {
            let mut locked = self.locked.exclusive_access();
            if *locked {
                drop(locked);
                suspend_current_and_run_next();
                continue;
            } else {
                *locked = true;
                return;
            }
        }
    }

    fn unlock(&self) {
        trace!("kernel: MutexSpin::unlock");
        let mut locked = self.locked.exclusive_access();
        *locked = false;
    }
}

/// Blocking Mutex struct
pub struct MutexBlocking {
    inner: UPSafeCell<MutexBlockingInner>,
}

pub struct MutexBlockingInner {
    locked: bool,
    wait_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl MutexBlocking {
    /// Create a new blocking mutex
    pub fn new() -> Self {
        trace!("kernel: MutexBlocking::new");
        Self {
            inner: unsafe {
                UPSafeCell::new(MutexBlockingInner {
                    locked: false,
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }
}

impl Mutex for MutexBlocking {
    /// lock the blocking mutex
    fn lock(&self) {
        trace!("kernel: MutexBlocking::lock");
        let mut mutex_inner = self.inner.exclusive_access();
        let task = current_task().unwrap();
        let mut task_inner = task.inner_exclusive_access();
        let mutex_id = task_inner.mutex_id;
        if mutex_inner.locked {
            mutex_inner.wait_queue.push_back(current_task().unwrap());
            task_inner.need_vec_mutex[mutex_id] += 1;
            drop(task_inner);
            drop(mutex_inner);
            block_current_and_run_next();
        } else {
            let process = current_process();
            let mut process_inner = process.inner_exclusive_access();
            process_inner.mutex_available_vec[mutex_id] -= 1;
            task_inner.alloc_vec_mutex[mutex_id] += 1;
            drop(process_inner);
            drop(task_inner);
            mutex_inner.locked = true;
        }
    }

    /// unlock the blocking mutex
    fn unlock(&self) {
        trace!("kernel: MutexBlocking::unlock");
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        let task = current_task().unwrap();
        let mut task_inner = task.inner_exclusive_access();
        let mutex_id = task_inner.unlock_mutex_id;
        task_inner.alloc_vec_mutex[mutex_id] -= 1;
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            let mut waking_task_inner = waking_task.inner_exclusive_access();
            waking_task_inner.need_vec_mutex[mutex_id] -= 1;
            waking_task_inner.alloc_vec_mutex[mutex_id] += 1;
            drop(waking_task_inner);
            drop(task_inner);
            wakeup_task(waking_task);
        } else {
            let process = current_process();
            let mut process_inner = process.inner_exclusive_access();
            process_inner.mutex_available_vec[mutex_id] += 1;
            drop(process_inner);
            drop(task_inner);
            mutex_inner.locked = false;
        }
    }
}
