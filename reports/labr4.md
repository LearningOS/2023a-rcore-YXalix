参考[`Starry` 的 `axfs`](https://github.com/Azure-stars/Starry/tree/main/modules/axfs)来在`rCore-Tutorial`下通过虚拟文件系统统一接口，使得其既能支持`eazy-fs`和`fatfs`。
### 具体实现细节
1. `libvfs`定义接口
	- `trait VfsInode` 由文件系统实现，交给内核OS来使用
	- `trait BlockDevice` 由内核实现，交给文件系统使用
2. `os/src/fs/inode.rs`
	- 修改该文件使得其能够完全支持`VfsInode`
3. `rust-fatfs`
	- `git clone https://github.com/rafalh/rust-fatfs.git` 到根目录下作为一个crate
4. `myfs`
	作为一个crate存在，参考[`Starry` 的 `axfs`](https://github.com/Azure-stars/Starry/tree/main/modules/axfs)基于rust-fatfs来实现，整体组织形式类似于`eazy-fs`，每个文件具体含义与功能如下
	- `block_dev.rs`：在基于`trait BlockDevice`往上抽象一层`Disk`，同时为了能够适配`rust-fatfs`，还实现了`trait Read, Write, Seek`
	- `driver_common.rs`：基本没什么用，主要是为了能够更快地适配`Starry`的代码
	- `fs.rs`：基于`Disk`来构建`FatFileSystem`
	- `lazy_init.rs`：来自`Starry`，为了能够实现全局的初始化
	- `lib.rs`：crate主体，提供`init_filesystems`函数用于初始化文件系统，并返回`Arc<dyn VfsInode> 形式的 root_inode`
	- `root.rs`：提供被`init_filesystems`函数调用的`init_rootfs`函数
	- `vfs.rs`：分别为`FileWrapper<'static>`以及`DirWrapper<'static>`实现 `trait VfsInode`, 值得注意的是，在实现`find`的时候需要注意文件扩展名，是否有`.elf`
5. 生成`.img`文件
在`os/makefile`里面写对应的脚本，将原先生成的空的`fs.img`, 然后用本机的文件系统将测例全部装进`fs.img`中，并放到`os/target/`下。

很多功能由于在实际测试中并不会被用到，所以在实现的时候进行了大量的删减，只保留了所需的部分。
最终实现效果如下：

![](https://blognashzhou.oss-cn-shanghai.aliyuncs.com/img/rcore-3-lab4-result.png)  
可以看到在fat文件系统下通过了前两章的测例