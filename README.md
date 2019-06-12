# TwitchDL

[![Build Status](https://travis-ci.org/sylingd/TD.svg?branch=master)](https://travis-ci.org/sylingd/TD)
[![Build Status](https://ci.appveyor.com/api/projects/status/66lpn5xt5yqjqcdw?svg=true)](https://ci.appveyor.com/project/sylingd/td)

一个抓取Twitch直播流的工具

## 下载

### 方式一

运行`twitchdl`或`twitchdl.exe`，按提示进行，例如：（“>”开头的行表示用户输入）

```shell
> twitchdl.exe
Input output directory, end without '/':
> Z:/test
Input OAuth Token (optional):
> YOUR_OAUTH_TOKEN
Modes:
 * 1 : default
 * 2 : All Access Pass mode
 * 3 : Auto All Access Pass mode
Choose mode (default is 1):
> 1
Input channel name(s), separated by ',':
> overwatchcontenders
██████████████████████████████████████████████████░░ 19/20
```

### 方式二

运行`twitchdl（或twitchdl.exe） --dir={保存路径} --token={OAuth Token} --mode={模式} --channel={频道名}`。

注意：路径的结尾**不要**有“/”，在Windows下不要有反斜线“\\”。

如果使用自动All Access Pass模式，可以加上这些参数，控制录制频道：

* `--three-screen`：录制三屏，默认
* `--pov`：录制POV
* `--team={队伍名}`：录制指定队伍，会同时包含POV和三屏，如`--team=Chengdu`
* `--player={选手名}`：录制指定选手，会同时包含POV和三屏，如`--player=ameng`

队伍和选手名称均区分大小写。大小写请参见[官网](https://www.overwatchleague.cn/zh-cn/players)。

例如：

```shell
twitchdl --dir=/web/wwwroot --token=123456 --mode=3
twitchdl --dir=/web/wwwroot --mode=1 --channel=esl_csgo
twitchdl --dir=/web/wwwroot --token=123456 --mode=1 --channel=esl_csgo

twitchdl --dir=/web/wwwroot --token=123456 --mode=3 --pov
twitchdl --dir=/web/wwwroot --token=123456 --mode=3 --team=Chengdu

twitchdl.exe --dir=D:/download --mode=1 --channel=esl_csgo
twitchdl.exe --dir=D:/download/twitch --token=123456 --mode=1 --channel=esl_csgo
```

### 获取OAuth Token

部分频道需要使用OAuth Token访问。使用浏览器打开Twitch并登录，按F12打开控制台，找到`Application - Cookies - www.twitch.tv`，找到auth-token，将“Value”栏的内容复制。

![token](http://wx4.sinaimg.cn/large/0060lm7Tly1g10zqdg6p1j30pk08gdgj.jpg)

## 生成播放列表

### 方式一

运行`m3u8`或`m3u8.exe`，按提示进行，例如：（“>”开头的行表示用户输入）

```
> m3u8.exe
Input directory, end without '/':
> Z:/test
 * 0: 0313_10_26_54_overwatchcontenders
Choose dir(s), separated by ',', or input all/new:
> all
Write to 0313_10_26_54_overwatchcontenders success
```

### 方式二

运行`m3u8（或m3u8.exe） --dir={路径} --mode={模式}`。

注意：路径的结尾**不要**有“/”，在Windows下不要有反斜线“\\”。

例如：

```shell
m3u8 --dir=/web/wwwroot/0313_10_26_54_overwatchcontenders --mode=1
m3u8 --dir=/web/wwwroot --mode=2
m3u8 --dir=/web/wwwroot --mode=3

m3u8.exe --dir=D:/download/0313_10_26_54_overwatchcontenders --mode=1
m3u8.exe --dir=D:/download --mode=2
```

**模式说明：**

* 1 直接模式：直接生成此路径的播放列表
* 2 增量模式：（对应方式一的“new”）生成此路径下所有目录的播放列表，但跳过已有播放列表的目录
* 3 全部模式：（对应方式一的“all”）生成此路径下所有目录的播放列表

## 合并

**注意：合并前，需先生成播放列表**

### 安装ffmpeg

* Windows
  * 下载：[https://ffmpeg.zeranoe.com/builds/](https://ffmpeg.zeranoe.com/builds/)

* Mac
  * 命令行安装：`brew install ffmpeg`
  * 下载压缩包安装：[https://evermeet.cx/ffmpeg/](https://evermeet.cx/ffmpeg/)

* CentOS

```shell
# CentOS 7
sudo rpm --import http://li.nux.ro/download/nux/RPM-GPG-KEY-nux.ro
sudo rpm -Uvh http://li.nux.ro/download/nux/dextop/el7/x86_64/nux-dextop-release-0-5.el7.nux.noarch.rpm

# CentOS 6
sudo rpm --import http://li.nux.ro/download/nux/RPM-GPG-KEY-nux.ro
sudo rpm -Uvh http://li.nux.ro/download/nux/dextop/el6/x86_64/nux-dextop-release-0-2.el6.nux.noarch.rpm

# Install
sudo yum install ffmpeg ffmpeg-devel -y
```

* Ubuntu

```shell
sudo add-apt-repository ppa:mc3man/trusty-media
sudo apt-get update
sudo apt-get install ffmpeg
```

### 运行

```shell
ffmpeg -hide_banner -i {路径}/playlist.m3u8 -vcodec copy -acodec copy -absf aac_adtstoasc {输出文件}
```

例如：

```shell
ffmpeg -hide_banner -i /web/wwwroot/0313_10_26_54_overwatchcontenders/playlist.m3u8 -vcodec copy -acodec copy -absf aac_adtstoasc /web/wwwroot/overwatchcontenders.mp4
ffmpeg.exe -hide_banner -i D:/download/0313_10_26_54_overwatchcontenders/playlist.m3u8 -vcodec copy -acodec copy -absf aac_adtstoasc D:/download/overwatchcontenders.mp4
D:/download/ffmpeg/bin/ffmpeg.exe -hide_banner -i D:/download/0313_10_26_54_overwatchcontenders/playlist.m3u8 -vcodec copy -acodec copy -absf aac_adtstoasc D:/download/overwatchcontenders.mp4
```