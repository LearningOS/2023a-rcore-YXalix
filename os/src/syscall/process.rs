//! Process management syscalls
use alloc::sync::Arc;

use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE, BIG_STRIDE},
    loader::get_app_data_by_name,
    mm::{translated_refmut, translated_str, translated_buffer, VirtAddr, MapPermission, MemorySet},
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TaskStatus,
    }, timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel:pid[{}] sys_yield", current_task().unwrap().pid.0);
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    trace!("kernel::pid[{}] sys_waitpid [{}]", current_task().unwrap().pid.0, pid);
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    let sec =  us / 1_000_000;
    let usec =  us % 1_000_000;
    let token = current_task().unwrap().inner_exclusive_access().memory_set.token();
    translated_refmut(token, _ts).sec = sec;
    translated_refmut(token, _ts).usec = usec;
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    let token = current_task().unwrap().inner_exclusive_access().memory_set.token();
    unsafe{
        // set status to running into ti
        let status = &(*_ti).status as *const TaskStatus as *mut TaskStatus;
        *translated_refmut(token, status) = TaskStatus::Running;
        // set syscall_times into ti
        let syscall_times = &(*_ti).syscall_times as *const u32 as *mut u32;
        let mut arr = translated_buffer(token, syscall_times, MAX_SYSCALL_NUM);
        let arr1 = current_task().unwrap().inner_exclusive_access().syscall_times;
        let mut index = 0;
        for i in 0..arr.len() {
            for j in 0..arr[i].len() {
                arr[i][j] = arr1[index];
                index += 1;
            }
        }

        // set time into ti
        let time = &(*_ti).time as *const usize as *mut usize;
        *translated_refmut(token, time) = (get_time_us() - current_task().unwrap().inner_exclusive_access().time) /1000;
    }
    0
}

/// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    if VirtAddr::from(_start).page_offset() != 0  || _port & !0x7 != 0 || _port & 0x7 == 0{
        return -1;
    }
    let permission = MapPermission::from_bits((_port << 1) as u8).unwrap() | MapPermission::U;
    let memory_set = current_task().unwrap().inner_exclusive_access().get_memory_set() as *mut MemorySet;
    let memory_set = unsafe {&mut *memory_set };
    let mut start = _start;
    let end = start + _len;
    while start < end {
        let start_va = VirtAddr::from(start);
        let end_va = VirtAddr::from(start + PAGE_SIZE);
        let vpn = start_va.floor();

        match memory_set.translate(vpn) {
            Some(pte) => {
                if pte.is_valid() {
                    return -1;
                } else {
                    if memory_set.append_to(start_va, end_va) {
                        start = start + PAGE_SIZE;
                    } else {
                        memory_set.insert_framed_area(start_va, end_va, permission);
                        memory_set.append_to(start_va, end_va);
                        start = start + PAGE_SIZE;
                    }
                }
            },
            None => {
                memory_set.insert_framed_area(start_va, end_va, permission);
                memory_set.append_to(start_va, end_va);
                start = start + PAGE_SIZE;
            }
        };
    }

    0
}

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    // _start 没有按页大小对齐
    if _start & (PAGE_SIZE - 1) != 0 {
        return -1;
    }
    let memory_set = current_task().unwrap().inner_exclusive_access().get_memory_set() as *mut MemorySet;
    let memory_set = unsafe { &mut *memory_set };
    let mut start = _start;
    let end = start + _len;
    while start < end {
        let start_va = VirtAddr::from(start);
        match memory_set.translate(start_va.floor()) {
            Some(pte) => {
                if pte.is_valid() {
                    memory_set.shrink_to(start_va, start_va);
                    start = start + PAGE_SIZE;
                } else {
                    return -1;
                }
            },
            None => {
                return -1;
            }
        };
    }
    0
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// YOUR JOB: Implement spawn.
/// HINT: fork + exec =/= spawn
pub fn sys_spawn(_path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_spawn", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, _path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        let new_task = task.spawn(data);
        let new_pid = new_task.pid.0;
        let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
        trap_cx.x[10] = 0;
        add_task(new_task);
        new_pid as isize
    } else {
        -1
    }
}

// YOUR JOB: Set task priority.
pub fn sys_set_priority(_prio: isize) -> isize {
    if _prio >= 2{
        current_task().unwrap().inner_exclusive_access().stride = BIG_STRIDE / _prio;
        return _prio;
    }
    -1
}
