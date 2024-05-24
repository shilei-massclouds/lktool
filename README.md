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
git checkout split
cargo build
```

把lktool加入到环境变量，采取临时方式

```sh
export PATH=$PATH:/home/cloud/gitWork/lktool/target/debug
```

> 注意：需要把/home/cloud/gitWork/lktool替换为实际路径

### 查看可以作为根的组件

```sh
lktool list -c root
```

选择一个root组件rt_macrokernel为示例，基于它可以构建宏内核。

### 创建新的构造工程

选一个路径作为当前工作目录，执行

```sh
lktool new proj_mk --root rt_macrokernel
cd proj_mk
ls
```

这样会在当前工作目录下产生一个名为proj_mk的工程目录，入口组件是rt_macrokernel。**注意**：目前只有root组件可以作为root的参数。

进入./proj_mk目录，后面的命令都是在该目录下执行。可以先用ls查看一下，已经生成了一系列基础文件。

### 配置目标内核

目前仅能选择体系结构

```sh
lktool config [riscv64|x86_64|]
```

另外，下步需要能够支持lktool menuconfig，以精细的控制配置选项，以替代features控制的方式。

### 为构建和运行内核作准备

以当前要构建的宏内核为例，需要为它创建根文件系统磁盘，格式化和安装部分linux应用以进行测试验证。

```sh
lktool prepare
```

当前目录下建立了disk.img磁盘文件，其中包含必要的linux应用程序和应用库。

### 构建并运行目标内核

```rust
lktool run
```

正常会切换用户态之后，启动第一个用户态应用init，该应用目前只是打印Hello，确认内核的构建和模块的测试成功。

注：可以随时按照“配置目标内核”的方式切换当前体系结构，重新构建或运行目标内核。

### 查看仓库中的普通组件

```sh
lktool list
```

显示：

```sh
cloud@server:/tmp/test_earlycon$ lktool list
boot
config
earlycon
```

后面的get/put命令可以对对现有组件进行本地修改。

### 从云端取出组件在本地修改

以需要修改boot为例：

```sh
lktool get boot
```

> 组件boot被clone到本地，进入目录正常修改、commit和push
>
> 抄的杨金博的作业

### 完成组件在本地修改后放回

仍以boot为例：

```sh
lktool put boot
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
