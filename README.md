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

一只透明窗口、无边框、始终置顶的小宠物，它会根据你的电脑状态变换心情，还能陪你聊天 ✨

</div>

---

## ✨ 功能特性

| 功能 | 说明 |
|:---|:---|
| 🐱 **心情系统** | 根据 CPU / 内存使用率和时间段自动切换 `happy` · `normal` · `sleepy` · `warning` 四种心情 |
| 📊 **系统监控** | 实时显示 CPU、内存、磁盘使用率和运行时间 |
| 💬 **趣味对话** | 点击桌宠聊天，支持关键词问答和心情感知闲聊 |
| 🖱️ **窗口拖拽** | 左键拖拽移动位置，双击打开信息面板，右键打开菜单 |
| 🎨 **纯 CSS 渲染** | 角色完全由 CSS 绘制，支持呼吸、摇摆、弹跳等动画 |
| 🪶 **极轻量** | Tauri 原生壳，内存占用极低 |

---

## 📦 技术架构

```
┌─────────────────────────────────────────────┐
│              Tauri v2 (Rust)                │
│  ┌───────────────────────────────────────┐  │
│  │         Frontend (Vue 3 + Vite)       │  │
│  │  ┌──────────┐  ┌──────────────────┐   │  │
│  │  │Components│  │   Composables    │   │  │
│  │  │ Pet.vue  │  │ useSystemInfo.ts │   │  │
│  │  │ InfoPanel│  │ usePetState.ts   │   │  │
│  │  │ ChatBubble│ │ useChat.ts       │   │  │
│  │  └──────────┘  └──────────────────┘   │  │
│  └───────────────┬───────────────────────┘  │
│                  │ Tauri IPC (invoke)       │
│  ┌───────────────▼───────────────────────┐  │
│  │         Rust Backend                  │  │
│  │  commands/system_info.rs (sysinfo)    │  │
│  └───────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

---

## 🚀 快速开始

### 环境要求

- [Node.js](https://nodejs.org) ≥ 18
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
│   ├── components/           # UI 组件
│   │   ├── Pet.vue           # 🐱 桌宠角色 (CSS 渲染)
│   │   ├── InfoPanel.vue     # 📊 系统信息面板
│   │   ├── ChatBubble.vue    # 💬 对话气泡
│   │   └── ContextMenu.vue   # 📋 右键菜单
│   ├── composables/          # 可复用逻辑
│   │   ├── useSystemInfo.ts  # 系统信息轮询
│   │   ├── usePetState.ts    # 心情状态管理
│   │   └── useChat.ts        # 对话引擎
│   ├── types/                # 类型定义
│   ├── styles/               # 全局样式
│   ├── App.vue               # 根组件
│   └── main.ts               # 入口文件
├── src-tauri/                # Rust 后端
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
