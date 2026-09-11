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
| 🧩 **角色导入** | 支持导入社区角色资源包，兼容 [codex-pets.net](https://codex-pets.net/) 下载的 ZIP |
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
│  │ Rust：窗口定位、层级守护、原生菜单、sysinfo │ │
│  └────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

全部原生窗口共用同一个 Vue + Vite 入口，通过窗口 URL 参数渲染对应根组件；无需引入前端路由。
除桌宠主窗口外，聊天、信息、右键菜单、提醒、系统反馈、设置均为独立透明置顶窗口，
由 Rust 侧统一处理定位、屏幕边界与浮窗互斥仲裁。

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

## 🧩 导入角色资源包

DeskPal 支持导入社区制作的桌宠角色。推荐使用 **<https://codex-pets.net/>** —— 这是
一个 Codex Pet 社区图库，收录了大量社区分享的角色。

### 三步导入

1. 打开 <https://codex-pets.net/>，挑选喜欢的角色
2. 进入角色详情页，点击 **Download** 下载 ZIP 文件（**无需解压**）
3. 右键 DeskPal 托盘图标 → **设置** → **角色** → **导入角色资源包**，选择刚下载的 ZIP

导入后角色会立刻出现在列表中，选中即可同步到桌面。导入的角色带「已导入」标记，
可随时在设置中删除。

> 💡 站点提供 `npx codex-pets add` 等命令行安装方式，那是给 Codex 用的，DeskPal
> 不适用。请使用上面的 **Download** 按钮获取 ZIP。

### 资源包格式

DeskPal 直接读取站点原始 ZIP，**无需修改或重新打包**。ZIP 需恰好包含两个文件：

```
你的角色.zip
├── pet.json           # 角色元数据
└── spritesheet.webp   # 精灵图（也可为 spritesheet.png）
```

`pet.json` 字段：

```json
{
  "id": "fox",
  "displayName": "Fox",
  "description": "角色说明文字",
  "spritesheetPath": "spritesheet.webp",
  "kind": "animal"
}
```

### 兼容性说明

| 项目 | 要求 |
|:---|:---|
| 归档格式 | `.zip`（或 `.deskpal-role.zip`），**体积 ≤ 12 MB** |
| 文件数量 | **恰好 2 个** —— 元数据 + 精灵图，不含子目录或其他文件 |
| 元数据 | `pet.json`（社区格式）或 `manifest.json`（完整格式，`schemaVersion: 1`） |
| 精灵图 | `spritesheet.png` 或 `spritesheet.webp`，**单张 ≤ 10 MB** |
| 图片尺寸 | 长宽均 ≤ 4096 px，且总像素 ≤ 8,388,608（约 8.4 MP） |
| 角色 ID | 小写 ASCII slug，不能与内置角色（`guga` / `monthly-salary-cat` / `broom-witch`）重名 |

**使用 `pet.json`（社区常见格式）时**，精灵图必须是 **1536 × 1872** 的固定布局：
单帧 192 × 208，共 8 列 × 9 行，动画行顺序固定为：

| 行 | 动画 | 帧数 | FPS |
|:---:|:---|:---:|:---:|
| 0 | `Idle` | 6 | 4 |
| 1 | `RunRight` | 8 | 6 |
| 2 | `RunLeft` | 8 | 6 |
| 3 | `Waving` | 4 | 5 |
| 4 | `Jumping` | 5 | 5 |
| 5 | `Failed` | 8 | 5 |
| 6 | `Waiting` | 6 | 3 |
| 7 | `Running` | 6 | 6 |
| 8 | `Review` | 6 | 4 |

codex-pets.net 上的角色均遵循此布局，可直接导入。

**若你的精灵图尺寸或行数不同**，请改用 `manifest.json` 显式声明帧尺寸与动画行，
此时不受 1536 × 1872 限制：

```json
{
  "schemaVersion": 1,
  "role": {
    "id": "my-pet",
    "displayName": "我的角色",
    "description": "角色说明",
    "kind": "animal",
    "spritesheet": {
      "width": 1024,
      "height": 1024,
      "frameWidth": 128,
      "frameHeight": 128,
      "rowGap": 0,
      "animations": [
        { "name": "Idle", "row": 0, "frames": 4, "fps": 8 }
      ]
    }
  }
}
```

> ⚠️ `manifest.json` 必须包含名为 `Idle` 的动画，且动画名只能由英文字母组成。
> `role` 对象使用 `deny_unknown_fields`，不接受额外字段。

### 常见导入失败原因

| 提示 | 原因 |
|:---|:---|
| 角色资源包只能包含…… | ZIP 内文件数不是 2（例如解压后重新打包带进了文件夹或 `preview.webp`） |
| pet.json 兼容格式仅支持 1536 × 1872 | 精灵图尺寸不是 1536 × 1872，需改用 `manifest.json` |
| 角色资源包超过 12 MB 限制 | ZIP 体积超限 |
| 角色 ID 必须是非内置的小写 ASCII slug | ID 含大写、空格或与内置角色重名 |
| 角色包必须包含 Idle 动画 | `manifest.json` 中缺少 `Idle` |

---

## 📁 项目结构

```
deskpal/
├── src/                      # 前端源码
│   ├── windows/              # 原生窗口对应的 Vue 根组件
│   │   ├── PetWindow.vue     # 🐧 桌宠主窗口
│   │   ├── ChatWindow.vue    # 💬 独立聊天窗口
│   │   ├── InfoWindow.vue    # 📊 独立信息窗口
│   │   ├── ContextMenuWindow.vue   # 🖱️ 右键菜单
│   │   ├── ReminderWindow.vue      # ⏰ 提醒浮窗
│   │   ├── SystemFeedbackWindow.vue # 🔔 系统反馈浮窗
│   │   └── SettingsWindow.vue      # ⚙️ 设置窗口
│   ├── components/           # 精灵、聊天、状态与设置 UI
│   ├── composables/          # 动画、命中检测、窗口通信等逻辑
│   ├── config/               # 角色定义与性格配置
│   ├── types/                # 类型定义
│   ├── styles/               # 全局样式
│   ├── App.vue               # 窗口角色分发器
│   └── main.ts               # 共用入口
├── src-tauri/                # Rust 后端
│   ├── src/windowing/        # 多窗口定位、层级守护与互斥仲裁
│   ├── src/role_packs/       # 角色资源包解析、校验与安装
│   ├── src/settings/         # 设置持久化与导入导出
│   ├── src/commands/         # Tauri 命令
│   └── src/tray.rs           # 系统托盘菜单
├── scripts/                  # 性能基准脚本（pnpm bench）
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
