# Lab1 Report

**谢新格 PB19081644**

## Git & GitHub仓库

学习git的使用并创建了私有仓库 X-XG/OSH-2021-Labs，并且把助教账户 OSH-2021-TA 加为 collaborator。

在本地windows系统和虚拟机linux系统上多次commit，进行了版本控制

## Linux 内核

##### 直接编译未裁剪的内核

![https://github.com/X-XG/OSH-2021-Labs/lab1/docs/pics/1.png](C:\Users\93416\AppData\Roaming\Typora\typora-user-images\image-20210414094925795.png)

屏幕上有输出，说明成功编译。

##### 裁剪Linux内核

![https://github.com/X-XG/OSH-2021-Labs/lab1/docs/pics/2.png](C:\Users\93416\AppData\Roaming\Typora\typora-user-images\image-20210414095247604.png)

得到内核大小5,478,912B，小于6MB，并能够完成「初始内存盘」中的全部任务。

![https://github.com/X-XG/OSH-2021-Labs/lab1/docs/pics/3.png](C:\Users\93416\AppData\Roaming\Typora\typora-user-images\image-20210414095655142.png)

## 初始内存盘

用C语言写了init.c程序，并编译，与程序1、2、3一起构建了initrd.cpio.gz 文件，能够在QEMU中以此运行程序1、2、3。在init.c中的末尾，添加了`while(1)`代码防止程序退出，从而在程序 3 执行完成后不出现Kernel Panic。

## 初始 Boot

学习了与理解了Make命令的概念与意义，能看懂简单的Makefile文件，用make命令构建了能启动的bootloader.img文件。

![https://github.com/X-XG/OSH-2021-Labs/lab1/docs/pics/4.png](C:\Users\93416\AppData\Roaming\Typora\typora-user-images\image-20210414101353229.png)

##### 第一组问题

- `xor ax, ax` 使用了异或操作 `xor`，这是在干什么？这么做有什么好处呢？

  ax 是一个通用寄存器，ax 异或上 ax 的值一定为0，xor ax, ax是将通用寄存器ax清零。好处：使用xor指令通常会更快，因为指令更小，获取速度更快，花费的时钟周期少。

- `boot.asm` 文件前侧的 `org 0x7c00` 有什么用？

  org 是origin的缩写：起始地址，源。在汇编语言源程序的开始通常都用一条org伪指令来实现规定程序的起始地址。 org 0x7c00 规定了在链接时由boot.asm生成的机器码的起始地址是内存0x7c00。

##### 第二组问题

- 尝试修改代码，在目前已有的输出中增加一行输出“I am OK!”，样式不限，位置不限，但不能覆盖其他的输出；

  在loader.asm中的第190行增加代码`NewLine: db 'I am OK!'`在第167行增加代码`log_info NewLine, 8, 4`重新make后运行如下。

![https://github.com/X-XG/OSH-2021-Labs/lab1/docs/pics/5.png](C:\Users\93416\AppData\Roaming\Typora\typora-user-images\image-20210414134633192.png)

成功在最后一行增加了输出，且不覆盖其他输出。并将重新make得到的img文件上传至github。

## 思考题

- 请简要解释 `Linux` 与 `Ubuntu`、`Debian`、`ArchLinux`、`Fedora` 等之间的关系和区别。

  1. Ubuntu、Debian、ArchLinux、Fedora是Linux的不同发行版。
  2. Debian 是一个完全由自由软件构成的类 UNIX 操作系统，最早的发行版之一。其以坚持自由软件精神和生态环境优良而出名，拥有庞大的用户群体，甚至自己也成为了一个主流的子框架 ，称为“Debian GNU/Linux”。
  3. Ubuntu是Debian GNU/Linux 派生的发行版，是一个主打桌面应用的操作系统。其为一般用户提供了一个时新且稳定的由自由软件构成的操作系统，且拥有庞大的社群力量和资源，十分适合普通用户使用。
  4. 在 Red Hat Linux 在停止官方更新后，由社群启动的 Fedora 项目接管了其源代码并构筑了自己的更新，演变成了如今的 Fedora 发行版。Fedora 是一套功能完备且更新迅速的系统。
  5. Arch Linux 是一个基于 x86-64 架构的 Linux 发行版，不过因为其内核默认就包含了部分非自由的模块，所以其未受到 GNU 计划的官方支持。Arch Linux 因其“简单、现代、实在、人本、万能”的宗旨赢得了 Linux 中坚用户的广泛青睐。

- 简述树莓派启动的各个阶段。

  1. 第一阶段, 从系统芯片中加载第一阶段的启动程序，这个启动程序负责挂载在SD卡中的FAT32的文件系统，从而让他可以启动第二阶段的boot（bootcode.bin），这部分程序是写死在在芯片中的，所以不能修改。

  2. 第二阶段bootcode.bin则用来从SD卡上检索GPU固件（start.elf），然后运行它，从而启动GPU。

  3. 第三阶段，start.elf启动后，读取存放系统配置的文件config.txt。当config.txt文件被加载解析之后, start.elf会读取cmdline.txt和kernel.img. cmdline.txt包含内核运行的参数。

  4. 最后阶段，GPU从目录下寻找kernel.img（Linux内核），将其加载到处理器分配的共享内存中，当内核加载成功，处理器将结束复位状态，内核开始正式运行，系统启动正式开始。

     注：start.elf除了上面的，也会传递一些额外的参数给内核，比如DMA通道，GPU参数，MAC地址，eMMC时钟速度、内核寻址范围等等。

- 查阅 `PXE` 的资料，用自己的话描述它启动的过程。
  1. 客户端电脑开机后，若BIOS设置从网络启动，那么网卡中的PXE Boot ROM获得控制权之前先做自我测试，然后发送一个动态获得IP地址的广播包（请求FIND帧）到网络上。 
  2. DHCP服务器在收到该广播包后，发送给客户端分配IP地址的DHCP回应包。内容包括客户端的IP地址，TFTP服务器的IP地址，预设通讯通道，及开机启动文件（该文件应该是一种由PXE启动规范规定的固定格式的可执行文件） 。
  3. 客户面收到DHCP回应后，则会响应一个FRAME，以请求传送启动文件。之后，服务端将和客户机再进行一系列应答，以决定启动的一些参数。
  4. 客户端通过TFTP通讯协议从服务器下载开机启动文件。启动文件接收完成后，将控制权转交给启动块，完成PXE启动。

- 说明 `UEFI Boot` 的流程，截图指出你的某一个系统的 `EFI` 分区中包含哪些文件。

  UEFI启动流程：

  1. 系统开机 - 上电自检（Power On Self Test 或 POST）。
  2. UEFI 固件被加载，并由它初始化启动要用的硬件。
  3. 固件读取其引导管理器以确定从何处（比如，从哪个硬盘及分区）加载哪个 UEFI 应用。
  4. 固件按照引导管理器中的启动项目，加载UEFI 应用。
  5. 已启动的 UEFI 应用还可以启动其他应用（对应于 UEFI shell 或 rEFInd 之类的引导管理器的情况）或者启动内核及initramfs（对应于GRUB之类引导器的情况），这取决于 UEFI 应用的配置。

  Windows下的EFI目录：

  <img src="C:\Users\93416\AppData\Roaming\Typora\typora-user-images\image-20210414222904718.png" alt="https://github.com/X-XG/OSH-2021-Labs/lab1/docs/pics/6.png" style="zoom:67%;" />

  包含了bootmgfw.efi、bootmgr.efi、memtest.efi等文件与文件夹。

  <img src="C:\Users\93416\AppData\Roaming\Typora\typora-user-images\image-20210414223034643.png" alt="https://github.com/X-XG/OSH-2021-Labs/lab1/docs/pics/7.png" style="zoom:67%;" />

  总共包含了98个文件，36个文件夹。