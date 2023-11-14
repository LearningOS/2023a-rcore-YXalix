# Report

## 实现功能总结

为了能够运行带有标准库的hellostd.c，在跟着实验指导书的步骤进行实验的过程中，需要将在生成memery_set后，通过新导入的loader库，初始化用户栈。然后就是添加新的所需系统调用，在本实验中需要新加的系统调用有：
1. `SYSCALL_SET_TID_ADDRESS`：直接返回pid就行；
2. `SYSCALL_IOCTL`：直接返回0；
3. `SYSCALL_WRITEV`：需要构建iovec数据结构，包括两个字段，字符串初始地址和字符串长度，然后分别将传入的所有vec的字符串一个一个的write到file buffer中，然后返回总长度就行；
4. `SYSCALL_EXIT_GROUP`：直接返回0；

## 问答题
1. 查询标志位定义。

    > 标准的 waitpid 调用的结构是 pid_t waitpid(pid_t pid, int *_Nullable wstatus, int options);。其中的 options 参数分别有哪些可能（只要列出不需要解释），用 int 的 32 个 bit 如何表示？
    1. options可能有: WNOHANG、WUNTRACED、WCONTINUED、WSTOPPED、WEXITED、WEXITSTATUS、WIFEXITED、WIFSIGNALED、WIFSTOPPED、WIFCONTINUED
    2. 每个option按照位OR的方式组合在一起，比如WNOHANG | WUNTRACED | WCONTINUED


## 我的说明

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与以下各位就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    > 无  

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    > [rCore-Tutorial-Guide-2023A](https://learningos.cn/rCore-Tutorial-Guide-2023A)  
    > [rCore-Tutorial-V3](https://rcore-os.cn/rCore-Tutorial-Book-v3)  
    > [实验指导书](https://scpointer.github.io/rcore2oscomp/)
3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计