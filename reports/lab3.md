# Report

## 实现功能总结
要求在新增进程功能下，不仅让代码能通过之前的测试，还要新实现sys_spawn,以及进程调度算法，我在让代码能通过之前的测试时，对之前的功能改进了一下结构，由于对于Arc的理解加深了，所以很多代码看着就明白了很多。要实现sys_spawn，就是参考fork以及exec的实现步骤，实现一个spawn，然后在sys_spawn中，调用spawn。然后实现stride调度算法，我在TaskControlBlockInner中添加了stride以及pass字段，并在config中设置BIG_STRIDE为1<<20， 然后在调度算法中，就是找到pass最小的task，然后fetch，然后更新pass，然后返回即可。

## 问答题
1. 

## 我的说明

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与以下各位就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    > 无  

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    > [rCore-Tutorial-Guide-2023A](https://learningos.cn/rCore-Tutorial-Guide-2023A)  
    > [rCore-Tutorial-V3](https://rcore-os.cn/rCore-Tutorial-Book-v3)  
    > [stride原论文](https://people.cs.umass.edu/~mcorner/courses/691J/papers/PS/waldspurger_stride/waldspurger95stride.pdf)
3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计