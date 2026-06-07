<template>
  <div class="directory-tree">
    <div class="tree-header">目录</div>
    <n-tree
      :data="treeData"
      :selected-keys="selectedKeys"
      :on-load="handleLoad"
      :render-prefix="renderPrefix"
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

// Sync selected keys with store.currentDirectory
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

function renderPrefix({ option }: { option: TreeOption }) {
  return '📁'
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
  // Load drives/volumes as root nodes
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
  padding: 12px;
  color: #fff;
}

.tree-header {
  color: #fff;
  font-size: 13px;
  margin-bottom: 8px;
  font-weight: 500;
}
</style>
