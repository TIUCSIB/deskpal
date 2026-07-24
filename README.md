<div align="center">

# 🐾 DeskPal

**你的桌面小伙伴 — 一只住在桌面上的小宠物**

[![Tauri v2](https://img.shields.io/badge/Tauri-v2-blue?logo=tauri)](https://tauri.app)
[![Vue 3](https://img.shields.io/badge/Vue-3-brightgreen?logo=vuedotjs)](https://vuejs.org)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.8-blue?logo=typescript)](https://www.typescriptlang.org)
[![Vite](https://img.shields.io/badge/Vite-7-purple?logo=vite)](https://vitejs.dev)
[![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-yellow)](#license)

<br/>

一只住在透明置顶窗口中的精灵桌宠，会根据电脑状态变换心情，还能通过独立悬浮窗口陪你聊天 ✨

</div>

---

## ✨ 功能特性

| 功能 | 说明 |
|:---|:---|
| 🐱 **心情系统** | 根据 CPU / 内存使用率和时间段自动切换 `happy` · `normal` · `sleepy` · `warning` 四种心情 |
| 📊 **系统监控** | 实时显示 CPU、内存、磁盘使用率和运行时间 |
| 💬 **趣味对话** | 点击桌宠聊天，支持关键词问答和心情感知闲聊 |
| 🖱️ **桌面交互** | 左键拖拽、滚轮缩放，右键打开系统原生菜单 |
| 🎨 **精灵动画** | WebP 精灵表随机播放多组动画，支持像素级命中检测 |
| 🪟 **独立浮窗** | 聊天和系统状态使用透明置顶窗口，不受桌宠窗口边界裁剪 |
| 🪶 **轻量架构** | Vue 负责界面，Tauri/Rust 负责窗口、定位和原生菜单 |

---

## 📦 技术架构

```
┌──────────────────────────────────────────────────┐
│                 Tauri v2 (Rust)                  │
│                                                  │
│  main 窗口       chat 窗口       info 窗口       │
│  ┌────────┐     ┌──────────┐    ┌──────────┐    │
│  │ 精灵宠物│     │ 聊天输入框│    │ 系统状态 │    │
│  └────────┘     └──────────┘    └──────────┘    │
│       └──────── Tauri Event / Command ─────┘     │
│                         │                        │
│  ┌──────────────────────▼─────────────────────┐ │
│  │ Rust：窗口定位、屏幕边界、原生菜单、sysinfo │ │
│  └────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

三个原生窗口共用同一个 Vue + Vite 入口，通过窗口 URL 参数渲染对应根组件；无需引入前端路由。

---

## 🚀 快速开始

### 环境要求

- [Node.js](https://nodejs.org) ≥ 20.19（Vite 7 要求）
- [pnpm](https://pnpm.io) ≥ 9
- [Rust](https://www.rust-lang.org/tools/install) ≥ 1.77
- [Tauri 系统依赖](https://v2.tauri.app/start/prerequisites/)

### 安装与运行

```bash
# 克隆仓库
git clone https://github.com/TIUCSIB/deskpal.git
cd deskpal

# 安装依赖
pnpm install

# 启动开发模式 🚀
pnpm tauri dev

# 构建生产包 📦
pnpm tauri build
```

---

## 📁 项目结构

```
deskpal/
├── src/                      # 前端源码
│   ├── windows/              # 原生窗口对应的 Vue 根组件
│   │   ├── PetWindow.vue     # 🐧 桌宠主窗口
│   │   ├── ChatWindow.vue    # 💬 独立聊天窗口
│   │   └── InfoWindow.vue    # 📊 独立信息窗口
│   ├── components/           # 精灵、聊天和状态 UI
│   ├── composables/          # 动画、命中、窗口通信等逻辑
│   ├── types/                # 类型定义
│   ├── styles/               # 全局样式
│   ├── App.vue               # 窗口角色分发器
│   └── main.ts               # 共用入口
├── src-tauri/                # Rust 后端
│   ├── src/menu.rs           # 原生上下文菜单
│   ├── src/windowing.rs      # 多窗口定位与生命周期
│   └── src/commands/         # Tauri 命令
├── AGENTS.md                 # 项目开发规范
└── package.json
```

---

## 🎭 心情系统

```
         CPU < 30% 且 内存 < 50%
              ┌──────────┐
              │  happy   │  ^ ^
              └────┬─────┘
                   │
        CPU / 内存 正常范围
              ┌────▼─────┐
              │  normal  │  ● ●
              └────┬─────┘
                   │
          0:00 ~ 6:00 深夜
              ┌────▼─────┐
              │  sleepy  │  - -
              └──────────┘

      CPU > 80% 或 内存 > 85%
              ┌──────────┐
              │ warning  │  > <
              └──────────┘
```

---

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交变更 (`git commit -m '添加了某个功能'`)
4. 推送到远程 (`git push origin feature/amazing-feature`)
5. 发起 Pull Request

请先阅读 [AGENTS.md](./AGENTS.md) 了解项目编码规范。

---

## 📄 License

MIT © [TIUCSIB](https://github.com/TIUCSIB)

---

<div align="center">

**如果觉得有趣，请给个 ⭐ Star 支持一下！**

Made with ❤️ and ☕

</div>
