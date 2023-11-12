# Report

## 实现功能总结

hello程序需要能够读取argc和argv两个参数，在原本的实验框架下，argc和argv并没有被存储在sp指针往后的位置，所以按照c编译出来的汇编代码可以看到，在sp指针下先存放argc，然后+8得到argv数组指针，所以我们需要在exec中，将argc和argv存储在sp指针往后的位置，然后在hello程序中，读取sp指针往后的位置，就可以得到argc和argv了。

## 问答题
1. elf 文件和 bin 文件有什么区别？
    > elf文件是可执行文件，bin文件是二进制文件，elf文件包含了程序的各种信息，比如段表，符号表，bin文件只包含了程序的二进制代码，elf文件可以直接被加载到内存中，bin文件需要经过链接才能被加载到内存中。

    运行`file ch6_file0.elf`会得到以下信息，可以看到elf文件包含了很多信息，比如段表，符号表，动态链接表等等，而bin文件只包含了二进制代码。

    ```ch6_file0.elf: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), statically linked, stripped```

    运行`file ch6_file0.bin`会得到以下信息，可以看到bin文件只包含了二进制代码。

    ```ch6_file0.bin: data```

    运行`riscv64-linux-musl-objdump -ld ch6_file0.bin > debug.S`会得到以下信息，因为bin文件只包含了二进制代码，所以没办法被反汇编。
    
    ```riscv64-linux-musl-objdump: ch6_file0.bin: File format not recognized```

## 我的说明

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与以下各位就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    > 无  

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    > [rCore-Tutorial-Guide-2023A](https://learningos.cn/rCore-Tutorial-Guide-2023A)  
    > [rCore-Tutorial-V3](https://rcore-os.cn/rCore-Tutorial-Book-v3)  
    > [实验指导书](https://scpointer.github.io/rcore2oscomp/)
3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计