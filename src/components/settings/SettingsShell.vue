<script setup lang="ts">
/**
 * SettingsShell.vue - 设置窗口基础外壳
 * 负责标题栏、关闭按钮、Toast 容器和加载态容器。
 */
import 'vue-sonner/style.css'
import { XIcon } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Toaster } from '@/components/ui/sonner'

const props = defineProps<{
  ready: boolean
  title: string
}>()

const emit = defineEmits<{
  close: []
}>()
</script>

<template>
  <main class="size-full bg-transparent p-3">
    <Card
      v-if="props.ready"
      class="flex h-full flex-col gap-4 rounded-[20px] border-border/70 bg-card/90 p-4 text-foreground shadow-none ring-border/70 backdrop-blur-sm"
      aria-label="设置"
    >
      <header class="flex items-center justify-between gap-3 px-0.5 pt-0.5" data-tauri-drag-region>
        <h1 class="text-[22px] leading-none font-semibold tracking-tight">
          {{ props.title }}
        </h1>
        <Button
          variant="outline"
          size="icon-lg"
          class="rounded-full bg-background/70 text-muted-foreground hover:bg-muted"
          type="button"
          title="关闭"
          data-tauri-drag-region="false"
          @click="emit('close')"
        >
          <XIcon class="size-4" />
        </Button>
      </header>

      <slot />
    </Card>

    <Card
      v-else
      class="rounded-2xl border-border/70 bg-card/90 px-5 py-4 text-sm text-muted-foreground shadow-none"
    >
      <slot name="loading">
        正在载入设置…
      </slot>
    </Card>

    <Toaster
      position="top-center"
      rich-colors
      close-button
      :toast-options="{ duration: 1800 }"
    />
  </main>
</template>
