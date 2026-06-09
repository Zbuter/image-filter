<template>
  <footer class="status-bar">
    <div class="status-left">
      <span class="status-item">
        {{ store.totalImages }} 张图片
      </span>
      <span v-if="store.selectedCount > 0" class="status-item status-selected">
        已选 {{ store.selectedCount }}
      </span>
    </div>
    <div class="status-right">
      <span class="shortcut-hint">Ctrl+点击多选 · Ctrl+A全选 · Ctrl+I反选 · Ctrl+F搜索</span>
    </div>
  </footer>
</template>

<script setup lang="ts">
import { useAppStore } from '../stores/app'

const store = useAppStore()
</script>

<style scoped>
.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--statusbar-height);
  padding: 0 12px;
  background: var(--bg-surface);
  border-top: 1px solid var(--border-subtle);
  font-size: 11px;
  flex-shrink: 0;
}

.status-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-item {
  color: var(--text-tertiary);
}

.status-selected {
  color: var(--accent);
  font-weight: 500;
}

.shortcut-hint {
  color: var(--text-disabled);
  font-size: 10px;
}
</style>
