//! Semaphore

use crate::sync::UPSafeCell;
use crate::task::{block_current_and_run_next, current_task, wakeup_task, TaskControlBlock, current_process};
use alloc::{collections::VecDeque, sync::Arc};

/// semaphore structure
pub struct Semaphore {
    /// semaphore inner
    pub inner: UPSafeCell<SemaphoreInner>,
}

pub struct SemaphoreInner {
    pub count: isize,
    pub wait_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl Semaphore {
    /// Create a new semaphore
    pub fn new(res_count: usize) -> Self {
        trace!("kernel: Semaphore::new");
        Self {
            inner: unsafe {
                UPSafeCell::new(SemaphoreInner {
                    count: res_count as isize,
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }

    /// up operation of semaphore
    pub fn up(&self) {
        trace!("kernel: Semaphore::up");
        let mut inner = self.inner.exclusive_access();
        inner.count += 1;
        let current_task = current_task().unwrap();
        let mut task_inner = current_task.inner_exclusive_access();
        let semaphore_id = task_inner.unlock_semaphore_id;
        task_inner.alloc_vec_semaphore[semaphore_id] -= 1;
        if inner.count <= 0 {
            if let Some(task) = inner.wait_queue.pop_front() {
                let mut waking_task_inner = task.inner_exclusive_access();
                waking_task_inner.need_vec_semaphore[semaphore_id] -= 1;
                waking_task_inner.alloc_vec_semaphore[semaphore_id] += 1;
                drop(waking_task_inner);
                drop(task_inner);
                wakeup_task(task);
            }
        } else {
            let process = current_process();
            let mut process_inner = process.inner_exclusive_access();
            process_inner.semaphore_available_vec[semaphore_id] += 1;
            drop(process_inner);
            drop(task_inner);
        }
    }

    /// down operation of semaphore
    pub fn down(&self) {
        trace!("kernel: Semaphore::down");
        let mut inner = self.inner.exclusive_access();
        inner.count -= 1;
        let current_task = current_task().unwrap();
        let mut task_inner = current_task.inner_exclusive_access();
        let semaphore_id = task_inner.semaphore_id;
        if inner.count < 0 {
            inner.wait_queue.push_back(current_task.clone());
            task_inner.need_vec_semaphore[semaphore_id] += 1;
            drop(task_inner);
            drop(inner);
            block_current_and_run_next();
        } else {
            let process = current_process();
            let mut process_inner = process.inner_exclusive_access();
            process_inner.semaphore_available_vec[semaphore_id] -= 1;
            task_inner.alloc_vec_semaphore[semaphore_id] += 1;
            drop(process_inner);
            drop(task_inner);
        }
    }
}
