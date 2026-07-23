# DeskPal 项目开发规范

> 🖥️ 桌面宠物应用 — Tauri v2 + Vue 3 + TypeScript

## 技术栈

| 层 | 技术 |
|---|---|
| 前端框架 | Vue 3 Composition API (`<script setup lang="ts">`) |
| 构建工具 | Vite 7 |
| 桌面壳 | Tauri v2 (Rust) |
| 包管理 | pnpm |
| 类型检查 | TypeScript 5.8 (strict mode) |

## 目录结构

```
src/
├── components/    # UI 组件
├── composables/   # 可复用逻辑 (use* 模式)
├── styles/        # 全局样式
├── types/         # TypeScript 类型定义
├── App.vue        # 根组件
└── main.ts        # 入口文件
src-tauri/
├── src/commands/  # Tauri 命令 (Rust)
└── src/lib.rs     # Tauri 应用配置
```

## 核心规范

### 文件大小

- **每个文件不得超过 300 行代码**
- 超出时拆分为子组件、composable 或工具函数

### 导入路径

- **禁止 `../../` 相对引用**，一律使用 `@` 别名
- `@` 映射到 `src/` 目录

```ts
// ✅ 正确
import type { SystemInfo } from '@/types/system'
import { useChat } from '@/composables/useChat'
import Pet from '@/components/Pet.vue'

// ❌ 禁止
import type { SystemInfo } from '../types/system'
import { useChat } from '../../composables/useChat'
```

### Vue 组件

- 必须使用 `<script setup lang="ts">` 语法
- Props 使用类型声明：`defineProps<{ title: string }>()`
- Emits 使用类型声明：`defineEmits<{ close: [] }>()`
- 样式使用 `<style scoped>`，BEM 命名（`.block__element--modifier`）
- 注释使用中文 JSDoc 风格：`/** 这是一条注释 */`

```vue
<script setup lang="ts">
/**
 * MyComponent.vue - 组件用途简述
 * 更详细的说明（如有必要）
 */
import { computed } from 'vue'
import type { SomeType } from '@/types/some'

const props = defineProps<{ data: SomeType }>()

/** 计算属性说明 */
const derived = computed(() => props.data.value * 2)
</script>
```

### Composables

- 文件命名：`use*.ts`（如 `usePetState.ts`）
- 函数命名：`use*()` 导出
- 返回值为具名对象 `{ state, action }` 模式
- 放置目录：`src/composables/`

```ts
import { ref } from 'vue'
import type { SomeType } from '@/types/some'

export function useFeature() {
  const state = ref<SomeType | null>(null)

  function doAction() {
    // ...
  }

  return { state, doAction }
}
```

### 类型定义

- 集中管理在 `src/types/` 目录
- 接口使用 PascalCase：`interface SystemInfo`
- 类型别名使用 PascalCase：`type PetMood`
- Rust 端的结构体需手动与 TS 类型保持同步

### CSS 样式

- 组件内样式使用 `<style scoped>`
- 使用 BEM 命名：`.chat-bubble__header`、`.pet--happy`
- 全局样式放在 `src/styles/main.css`
- 动画使用 `@keyframes`，命名简洁语义化

### Tauri (Rust)

- Tauri 命令放在 `src-tauri/src/commands/`
- 每个命令文件对应一个功能模块
- `mod.rs` 统一导出
- 命令使用 `#[tauri::command]` 注解
- 返回值需实现 `Serialize`

### TypeScript

- 严格模式已开启，不要绕过类型检查
- 禁止 `any`，使用具体类型或 `unknown`
- 变量和函数使用 camelCase
- 接口和类型使用 PascalCase
- 常量使用 UPPER_SNAKE_CASE

## Git 规范

- 提交信息使用中文或英文均可，但要简洁描述变更内容
- 每次提交聚焦单一变更
- 不提交 `node_modules/`、`dist/`、`src-tauri/target/`

## 注意事项

- 不引入额外的 UI 框架（如 Element Plus），保持轻量
- 不引入路由（vue-router），当前为单页面应用
- 状态管理使用 composables 模式，暂不引入 Pinia
- 所有用户可见文本使用中文
