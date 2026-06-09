<template>
  <div class="directory-tree">
    <div class="tree-header">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
      <span>目录</span>
    </div>
    <n-tree
      :data="treeData"
      :selected-keys="selectedKeys"
      :on-load="handleLoad"
      block-line
      selectable
      @update:selected-keys="handleSelect"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { NTree } from 'naive-ui'
import type { TreeOption } from 'naive-ui'
import { useAppStore } from '../stores/app'
import { invoke } from '@tauri-apps/api/core'

const store = useAppStore()

const treeData = ref<TreeOption[]>([])
const selectedKeys = ref<string[]>([])

watch(() => store.currentDirectory, (newDir) => {
  if (newDir && selectedKeys.value[0] !== newDir) {
    selectedKeys.value = [newDir]
  }
}, { immediate: true })

function convertToTreeOption(entry: any): TreeOption {
  return {
    key: entry.path,
    label: entry.name,
    isLeaf: false,
    children: undefined
  }
}

async function handleLoad(node: TreeOption): Promise<void> {
  const path = node.key as string
  
  try {
    const content = await invoke<any>('list_directory', { path })
    const children = content.directories.map(convertToTreeOption)
    node.children = children
  } catch (e) {
    console.error('Failed to load directory:', e)
    node.children = []
  }
}

function handleSelect(keys: string[]) {
  if (keys.length > 0) {
    selectedKeys.value = keys
    store.loadDirectory(keys[0])
  }
}

onMounted(async () => {
  try {
    const drives = await invoke<any[]>('list_drives')
    treeData.value = drives.map(d => ({
      key: d.path,
      label: d.name,
      isLeaf: false,
      children: undefined
    }))
  } catch (e) {
    console.error('Failed to load drives:', e)
  }
})
</script>

<style scoped>
.directory-tree {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  padding: 8px;
}

.tree-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  margin-bottom: 4px;
  color: var(--text-secondary);
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

:deep(.n-tree) {
  --n-node-color-hover: var(--bg-hover) !important;
  --n-node-color-active: var(--accent-muted) !important;
  --n-node-text-color: var(--text-primary) !important;
  --n-arrow-color: var(--text-tertiary) !important;
  --n-font-size: 12px !important;
  --n-border-radius: var(--radius-sm) !important;
}

:deep(.n-tree-node-content__text) {
  font-size: 12px !important;
}
</style>
