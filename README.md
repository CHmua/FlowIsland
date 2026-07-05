# FlowIsland

FlowIsland 是一个基于 Tauri 2、Rust 和 Vue 3 的 Windows 桌面灵动岛组件，聚焦音乐播放、歌词、消息提醒和轻量桌面状态展示。

## 主要功能

- 音乐灵动岛：读取 Windows 媒体会话，显示歌曲、歌手、封面、播放状态和播放控制。
- 封面取色：根据专辑封面生成灵动岛背景与流光颜色。
- 动态歌词：支持在线歌词匹配与 LRC 时间轴同步，快进后会重新对齐播放进度。
- 消息岛：接收系统通知后弹出简洁消息提示，平时保持隐藏。
- 右键菜单：在灵动岛上直接打开设置、重置位置、关闭流光或退出。
- 低打扰模式：浏览器视频媒体默认不接管，避免观影时常驻遮挡。
- 设置面板：提供灵动岛颜色、透明度、音乐识别、消息模式、置于任务栏等配置。

## 技术栈

- Tauri 2
- Rust
- Vue 3
- TypeScript
- Vite
- Lucide Vue Next

## 开发

```bash
npm install
npm run tauri dev
```

## 构建

```bash
npm run build
npm run tauri build
```

构建产物位于：

```text
src-tauri/target/release/bundle/
```

## GitHub 更新方案

发布新版本时，将安装包上传到 GitHub Releases。应用内更新检测可以读取仓库的最新 Release，并引导用户下载新版安装包。

仓库地址确定后，需要把更新检测地址改成你的 GitHub 仓库 Releases API，例如：

```text
https://api.github.com/repos/<owner>/<repo>/releases/latest
```

## 目录结构

```text
FlowIsland/
├─ src/                 # Vue 前端
│  ├─ views/            # 设置面板与灵动岛界面
│  ├─ router/           # 路由
│  └─ assets/           # 图标与静态资源
├─ src-tauri/           # Tauri / Rust 后端
│  ├─ src/              # Rust 入口与系统能力
│  └─ tauri.conf.json   # 应用配置
├─ package.json
└─ README.md
```

## 说明

当前版本以音乐灵动岛为核心，默认不显示旧版网速岛和游戏硬件监控岛。
