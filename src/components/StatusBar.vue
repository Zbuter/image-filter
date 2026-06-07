<template>
  <div class="status-bar">
    <div class="status-left">
      <span class="status-item">共 {{ store.totalImages }} 张图片</span>
      <span class="status-item">
        已选择: <span class="highlight">{{ store.selectedCount }} 张</span>
      </span>
    </div>
    <div class="status-right">
      <span class="shortcuts">Ctrl+点击多选 | 滚轮缩放 | ←→ 切换</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useAppStore } from '../stores/app'

const store = useAppStore()
</script>

<style scoped>
.status-bar {
  background: #2a2a2a;
  padding: 6px 16px;
  border-top: 1px solid #3a3a3a;
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  height: 28px;
}

.status-left {
  display: flex;
  align-items: center;
  gap: 20px;
}


.status-item {
  color: #888;
}

.highlight {
  color: #007acc;
  font-weight: 500;
}

.status-right {
  color: #666;
}

.shortcuts {
  font-size: 11px;
}
</style>
