# EyeCareAlarm 护眼提醒

跨平台（Windows / macOS）护眼提醒小工具。常驻系统托盘（Windows）/ 菜单栏（macOS），每 20 分钟弹出一次全屏遮罩，提醒你看向至少 6 米（20 英尺）外的物体 20 秒，期间播放宁静音效；倒计时结束自动退出，程序回到托盘。

基于 [20-20-20 法则](https://en.wikipedia.org/wiki/Computer_vision_syndrome)：每 20 分钟，看 20 英尺外，20 秒。

## 功能

- **托盘常驻**：无任务栏 / Dock 图标；左键或右键点击托盘图标打开控制面板
- **单实例**：重复启动只显示「护眼提醒已启动」提示（3 秒自动消失）后退出
- **27 秒两阶段遮罩**（覆盖所有显示器）：
  - `0–5s` 预热：遮罩从全透明渐变到设定透明度，仍可正常操作；文字「眼睛需要休息啦~」
  - `4–6s` 两段文字交叉淡化衔接
  - `8–27s` 屏蔽全部输入：播放休息音效（结尾 2 秒渐隐），右下角倒计时按钮
  - `Esc` 或点击「退出」随时提前结束（1 秒渐隐退出）
- **控制面板**：倒计时 + 今日放松次数、提醒间隔（20–120 分钟）、休息时长（5–60 秒）、遮罩透明度、音效选择与导入、音量、中/英双语、开机自启动
- **音效**：内置音效（`public/sound/`）+ 「+我的音效」导入（wav/mp3/ogg/flac），选择或调音量后自动试听 5 秒；播放带响度归一化，安静素材自动提升增益
- **全屏勿扰**：检测到他应用全屏（演示、会议、游戏）时跳过本次提醒并重置倒计时
- **统计**：完整完成一次休息计 1 次，每日清零，持久化保存

## 技术栈

- **壳**：Tauri 2（Rust 后端持有全部权威状态：定时器、设置、统计、音频、窗口编排）
- **前端**：Vite + React 18 + TypeScript（三个 webview 窗口：控制面板 / 遮罩 / 启动提示）
- **音频**：rodio（MP3 走 minimp3 解码），Rust 端统一播放避免多屏重复发声
- **打包**：NSIS（Windows，默认安装到 Program Files）/ dmg（macOS，universal2 配置）

## 开发

### 环境要求

- Node.js 20+ 与 npm
- Rust 工具链（1.77+）
- Windows：WebView2 Runtime（Win11 自带；Win10 安装包会引导安装）
- macOS：Xcode Command Line Tools（构建 dmg 用）

### 常用命令

```bash
npm install                 # 安装依赖
npm run tauri dev           # 开发模式（托盘启动；面板「立即提醒」即冒烟入口）
npm run build               # 前端类型检查 + 构建（tsc -b && vite build）
cargo check --manifest-path src-tauri/Cargo.toml   # Rust 检查

npm run sync:sounds         # 同步 public/sound -> src-tauri/sound（打包前自动执行）
npm run gen:assets          # 由 design/app-icon.svg 重新生成全部图标

npm run tauri build         # 产出安装包
#   Windows: src-tauri/target/release/bundle/nsis/EyeCareAlarm_<ver>_x64-setup.exe
#   macOS:   src-tauri/target/release/bundle/dmg/
```

Windows 打包后可校验 exe 子系统（应为 GUI 无控制台）：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check-subsystem.ps1 -Path "src-tauri/target/release/eye-care-alarm.exe"
```

### 调试技巧

dev 模式下可用 WebView2 远程调试端口驱动任意窗口（托盘面板、遮罩）：

```bash
set WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222
npm run tauri dev
# 然后访问 http://127.0.0.1:9222 查看各 webview target
```

Rust 侧音频链路有日志：`[audio] play <文件> vol=X% rms=R gain=G play=Ns fade=Ns`。

## 项目结构

```
├── panel.html / overlay.html / toast.html   # 三个窗口的 Vite 入口
├── src/
│   ├── panel/        # 控制面板（开关/步进/滑块/下拉组件）
│   ├── overlay/      # 全屏遮罩（27s 时间线，rAF 驱动）
│   ├── toast/        # 「护眼提醒已启动」提示条
│   └── shared/       # IPC 契约、i18n（中英文案单文件字典）、类型与时间线常量
├── src-tauri/
│   ├── src/
│   │   ├── main.rs        # 命令注册、单实例(fs2 锁)、启动流程
│   │   ├── timer.rs       # 休息定时器（后端持有，窗口开关不影响）
│   │   ├── overlay.rs     # 多屏遮罩编排：预创建窗口 + 27s 状态机
│   │   ├── audio.rs       # rodio 播放、RMS 归一化、音量渐隐
│   │   ├── fullscreen.rs  # 全屏勿扰检测（Win: QUNS+前台窗口; macOS: CGWindow）
│   │   ├── settings.rs    # settings.json / stats.json 持久化、音效目录解析
│   │   ├── sound.rs       # 内置+用户双目录音效扫描合并
│   │   └── tray.rs        # 托盘图标、面板定位、启动提示
│   └── capabilities/      # webview 权限声明
├── public/sound/     # 内置音效（用户维护的源，打包前同步进 src-tauri/sound）
├── design/           # 原始设计稿与交互原型（HTML）
└── scripts/          # 图标生成、音效同步、PE 子系统校验
```

## 数据位置

- 设置与统计：`%APPDATA%/com.eyecarealarm.app/`（Windows）——`settings.json`、`stats.json`
- 用户导入音效：`%APPDATA%/com.eyecarealarm.app/sound/`
- macOS：`~/Library/Application Support/com.eyecarealarm.app/`

## 已知限制

- 运行中新插入的显示器不会被遮罩覆盖（窗口按启动时的显示器创建，重启应用后适配）
- macOS 平台分支（勿扰检测、菜单栏行为、dmg 打包）尚未在真机验证
- 「更多护眼知识」暂未配置目标页面
