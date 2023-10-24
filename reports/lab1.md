# Report

## 实现功能总结
要求我们实现一个sys_task_info的系统调用，用于获取进程的信息。这个系统调用，我分为两步来完成：
- 第一步，能够返回task运行总时间，即第一次执行时间到当前时间的时间差，单位为ms，要实现这个功能，我在TaskControlBlock结构体中添加了我需要的开始时间戳，然后在第一次run该task的时候，记录开始时间戳，然后在sys_task_info中，通过current_task，找到对应的TaskControlBlock，然后计算时间差，返回即可。
- 第二步，能够统计task的sys_call的调用次数，同上，我在TaskControlBlock结构体中添加了我需要的sys_call调用次数，然后在每次调用sys_call的时候，增加该系统调用的调用次数即trap_handler中抓取到系统调用的中断处理时处理即可，然后在sys_task_info中，通过current_task，找到对应的TaskControlBlock，然后返回即可。

## 问答题
1. 

