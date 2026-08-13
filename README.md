# Cyreneの罗盘

Cyreneの罗盘是面向 Windows 10/11 与触屏一体机的 Fluent 风格快捷启动器。它使用 Vue 3、Tauri 2 与 GSAP 构建，默认主题色为桃粉色，并提供自定义单色主题与跟随 Windows 强调色的 Fluent 主题。

当前稳定版本：**26.0.0（初次发布）**。安装包可从 [GitHub Releases](https://github.com/Cyrene2008/CyreneCompass/releases/tag/v26.0.0) 下载。

发布页提供一个适用于普通用户的安装包：`CyreneCompass_26.0.0_x64-setup.exe`。安装到当前用户目录，不需要安装证书，也不要求用户拥有管理员凭据。

## 当前功能

- 常驻桌面的可拖动悬浮球，左键或触摸展开罗盘；右键与触屏长按不会触发操作。
- 触屏拖动使用物理坐标手动定位与串行事件合并，在高 DPI 和多屏环境下避免抖动、跳动和移动距离缩短。
- 3×3 罗盘布局：中央按钮负责关闭或返回上一级，其余 8 个位置可配置动作或子菜单。
- 每次展开都从首页开始；动作执行成功后自动收起。
- 无操作自动收起默认开启，默认 10 秒，可在 3–30 秒间调节。
- 可调悬浮球大小、罗盘窗口大小、UI 缩放、图标大小、字体大小和背景透明度。
- 悬浮球支持最多 4 个字的圆形文字模式、Fluent 图标模式以及图片/GIF 模式；4 个字自动按两行显示，图片可选是否裁剪为圆形。
- UI 缩放最低 50%；无论缩放值如何，每个罗盘元素的物理点击区域均保底为 32 px。
- 使用 HarmonyOS Sans SC，支持简体中文和英文界面。
- 桃粉、Fluent、自定义主题色与深色模式；主题色通过单一强调色派生整套界面颜色。
- 离线 Fluent Iconify 图标选择器。系统文件图标默认优先，也可主动覆盖为自定义图片或 Iconify 图标。
- 原生文件拖放：文件、文件夹、快捷方式、媒体、脚本和可执行文件均由 Windows Shell 解析。
- `.lnk` 会额外解析真实目标用于识别，但执行时保留快捷方式本身，从而保留参数、工作目录和快捷方式行为。
- 支持文件路径、URI、URL、PowerShell 脚本，以及通过 `ShellExecuteW("runas")` 进行的管理员执行。
- 托盘常驻；右键菜单仅提供“打开设置”，退出只能在设置页经过三次确认完成。
- 本机回环 IPC 单实例限制，可检测不同完整性级别启动的已有实例。
- `TOPMOST + WS_EX_NOACTIVATE`：悬浮球和罗盘可显示在普通 UWP 应用及 PPT 放映上方，且不主动抢夺焦点；进入设置时临时恢复焦点以便输入。系统安全桌面、锁屏、登录界面和 `Win+Tab` 不在覆盖范围内。
- 高优先级登录计划任务与普通用户注册表启动项。开机启动直接显示悬浮球，不启动到托盘。
- Per-Monitor V2 DPI 清单，用于多屏和运行时缩放变化的基础适配；从屏幕边角展开时会约束在悬浮球所在显示器的工作区内。启动恢复位置时会完整校验屏幕工作区，越界、压到任务栏或异常坐标会回到主屏中央并自动修正记录。
- 内置更新检查、安装包下载、文件名/大小/Windows PE 文件头校验与安装器启动；关于页也提供仓库与 GPLv3 链接。

## 开发

要求：Node.js 20+、Rust stable、Windows 10/11、WebView2。

坐标换算与移动队列回归测试：

```powershell
bun test
```

```powershell
bun install
bun run tauri:dev
```

只预览前端：

```powershell
bun run dev
```

## 构建

构建 Windows x64 安装包：

```powershell
bun run tauri:build
```

安装包输出为：

- `src-tauri/target/release/bundle/nsis/CyreneCompass_26.0.0_x64-setup.exe`

应用清单固定使用 `uiAccess=false`。项目不安装自签名证书，不复制系统进程令牌，也不要求安装到受保护目录。

## 权限与启动设计

- 主进程默认以当前用户身份运行，避免整个常驻程序持续持有管理员权限。
- 单个动作勾选“管理员”后才调用 UAC。
- 创建 `/RL HIGHEST` 登录计划任务时，会启动一次独立的提权 helper；主实例继续正常运行。
- 普通启动项写入 `HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run`，可作为无需管理员权限的兜底。
- 计划任务名：`CyreneCompassAutoStart`；注册表值名：`CyreneCompass`。

## 项目结构

```text
src/                         Vue 界面、主题、i18n、离线 Fluent 图标
src-tauri/src/lib.rs         Windows 窗口、托盘、IPC、动作与启动任务
src-tauri/windows-app-manifest.xml  Windows 应用清单
public/cyrene.png            应用 Logo
```

## 说明

项目当前版本为 `26.0.0`。配置保存在 WebView 的本地存储中。

- 作者：Cyrene2008 (星海昔涟)
- 版权：Copyright (C) Cyrene2008 2026. All Rights Reserved.
- 开源协议：[GNU General Public License v3.0](./LICENSE)
- 项目仓库：[Cyrene2008/CyreneCompass](https://github.com/Cyrene2008/CyreneCompass)

## 版本历史

### 26.0.0 - 初次发布（2026-08-01）

- 发布适用于 Windows 10/11 与触屏一体机的首个公开版本。
- 提供可拖动悬浮球、3×3 多级罗盘、Fluent 设置界面和中英双语支持。
- 支持文件、快捷方式、URI、URL、PowerShell 脚本、管理员执行及 Windows Shell 图标解析。
- 支持自定义主题、文字球、图片/GIF 球、自定义动作图标、透明度、尺寸、字体和自动收起。
- 提供多屏/DPI 边界处理、手动物理坐标拖动、单实例 IPC、托盘、置顶与不抢焦点能力。
- 使用单一普通用户安装包，不包含 UIAccess 证书或令牌借用逻辑。
