//! Types related to task management & Functions for completely changing TCB

use super::id::TaskUserRes;
use super::{kstack_alloc, KernelStack, ProcessControlBlock, TaskContext};
use crate::trap::TrapContext;
use crate::{mm::PhysPageNum, sync::UPSafeCell};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::cell::RefMut;
use alloc::vec;

/// Task control block structure
pub struct TaskControlBlock {
    /// immutable
    pub process: Weak<ProcessControlBlock>,
    /// Kernel stack corresponding to PID
    pub kstack: KernelStack,
    /// mutable
    inner: UPSafeCell<TaskControlBlockInner>,
}

impl TaskControlBlock {
    /// Get tid of current task
    pub fn get_tid(&self) -> usize {
        let inner = self.inner_exclusive_access();
        inner.res.as_ref().unwrap().tid
    }
    /// Get the mutable reference of the inner TCB
    pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }
    /// Get the address of app's page table
    pub fn get_user_token(&self) -> usize {
        let process = self.process.upgrade().unwrap();
        let inner = process.inner_exclusive_access();
        inner.memory_set.token()
    }
}

pub struct TaskControlBlockInner {
    pub res: Option<TaskUserRes>,
    /// The physical page number of the frame where the trap context is placed
    pub trap_cx_ppn: PhysPageNum,
    /// Save task context
    pub task_cx: TaskContext,

    /// Maintain the execution status of the current process
    pub task_status: TaskStatus,
    /// It is set when active exit or execution error occurs
    pub exit_code: Option<i32>,

    /// The allocation vec of mutex
    pub alloc_vec_mutex: Vec<usize>,
    /// The need vec of mutex
    pub need_vec_mutex: Vec<usize>,
    /// current mutex_id
    pub mutex_id: usize,
    /// current unlock mutex_id
    pub unlock_mutex_id: usize,

    /// The allocation vec of semaphore
    pub alloc_vec_semaphore: Vec<usize>,
    /// The need vec of semaphore
    pub need_vec_semaphore: Vec<usize>,
    /// current semaphore_id
    pub semaphore_id: usize,
    /// current unlock semaphore_id
    pub unlock_semaphore_id: usize,
}

impl TaskControlBlockInner {

    /// set the mutex_id
    pub fn set_mutex_id(&mut self, mutex_id: usize) {
        self.mutex_id = mutex_id;
    }

    /// set the unlock_mutex_id
    pub fn set_unlock_mutex_id(&mut self, unlock_mutex_id: usize) {
        self.unlock_mutex_id = unlock_mutex_id;
    }

    /// set the semaphore_id
    pub fn set_semaphore_id(&mut self, semaphore_id: usize) {
        self.semaphore_id = semaphore_id;
    }

    /// set the unlock_semaphore_id
    pub fn set_up_semaphore_id(&mut self, unlock_semaphore_id: usize) {
        self.unlock_semaphore_id = unlock_semaphore_id;
    }

    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    #[allow(unused)]
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
}

impl TaskControlBlock {
    /// Create a new task
    pub fn new(
        process: Arc<ProcessControlBlock>,
        ustack_base: usize,
        alloc_user_res: bool,
    ) -> Self {
        let res = TaskUserRes::new(Arc::clone(&process), ustack_base, alloc_user_res);
        let trap_cx_ppn = res.trap_cx_ppn();
        let kstack = kstack_alloc();
        let kstack_top = kstack.get_top();
        Self {
            process: Arc::downgrade(&process),
            kstack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    res: Some(res),
                    trap_cx_ppn,
                    task_cx: TaskContext::goto_trap_return(kstack_top),
                    task_status: TaskStatus::Ready,
                    exit_code: None,
                    alloc_vec_mutex: vec![0;20],
                    need_vec_mutex: vec![0;20],
                    mutex_id: 0,
                    unlock_mutex_id: 0,
                    alloc_vec_semaphore: vec![0;20],
                    need_vec_semaphore: vec![0;20],
                    semaphore_id: 0,
                    unlock_semaphore_id: 0,
                })
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
/// The execution status of the current process
pub enum TaskStatus {
    /// ready to run
    Ready,
    /// running
    Running,
    /// blocked
    Blocked,
}
