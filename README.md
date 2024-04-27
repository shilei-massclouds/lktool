# lktool
A tool to manage lk_model components.

### 准备：允许cargo通过ssh下载crate

```sh
vi ~/.cargo/config
```

在config文件中加如下内容：

```rust
[net]
git-fetch-with-cli = true
```

### 下载lktool和编译该工具

```sh
git clone git@github.com:shilei-massclouds/lktool.git
cd lktool
cargo build
```

把lktool加入到环境变量，采取临时方式

```sh
export PATH=$PATH:/home/cloud/gitWork/lktool/target/debug
```

> 注意：需要把/home/cloud/gitWork/lktool替换为实际路径

### 查看可以作为根的组件

```sh
lktool list -c top
```

目前只有一个top组件top_early_console。

### 创建新的构造工程

选一个路径，例如/tmp

```sh
lktool new test1 --root top_early_console
cd test1
ls
```

这样会在当前目录(/tmp)产生一个名为test1的工程目录，入口组件是top_early_console。**注意**：目前只有top组件可以作为root的参数。

进入/tmp/test1目录，后面的命令都是在该目录下执行。可以先用ls查看一下，已经生成了一系列基础文件。

### 配置目标内核

目前仅能选择体系结构

```sh
lktool config [riscv64|x86_64]
```

正在试验加上uml，即内核作为Host Linux的一个进程，方便测试。

另外，下步需要能够支持lktool menuconfig，以精细的控制配置选项，以替代features控制的方式。

### 构建目标内核

```sh
lktool build
```

正常会产生一个最小的内核，它的功能是打印一句Hello，然后退出。所以该内核的作用仅是对early_console组件进行测试，并无其它的实际用途。

### 运行目标内核

```rust
lktool run
```

正常会打印Hello，确认内核的构建和模块的测试成功。

### 查看仓库中的普通组件

```sh
lktool list
```

显示：

```sh
cloud@server:/tmp/test_earlycon$ lktool list
arch_boot
axconfig
early_console
early_mmu
kernel_guard_base
spinbase
```

后面的get/put命令可以对对现有组件进行本地修改。

### 从云端取出组件在本地修改

以需要修改arch_boot为例：

```sh
lktool get arch_boot
```

> 组件arch_boot被clone到本地，进入目录正常修改、commit和push
>
> 抄的杨金博的作业

### 完成组件在本地修改后放回

仍以arch_boot为例：

```sh
lktool put arch_boot
```

> 查看本地发现，组件在本地的目录已被清除
>
> 抄的杨金博的作业



### 依赖关系

在工程目录下执行命令，产生依赖关系图

```sh
lktool dep-graph
```

这个工具需要改进，能够适应体系结构的配置，目前展示不全。
