<template>
  <div class="app-container">
    <div class="toolbar">
      <button @click="selectDirectory">选择目录</button>
      <div class="toolbar-spacer"></div>
      <input 
        v-model="filterText" 
        type="text" 
        placeholder="搜索文件名..."
        class="search-input"
      />
      <select v-model="fileTypeFilter" class="filter-select">
        <option value="all">全部</option>
        <option value="raw">RAW 格式</option>
        <option value="regular">普通图片</option>
        <option value="custom">自定义后缀</option>
      </select>
      <input 
        v-if="fileTypeFilter === 'custom'"
        v-model="customExtensions"
        type="text"
        placeholder="输入后缀，如: jpg,png"
        class="custom-ext-input"
      />
      <div class="view-tabs">
        <button 
          :class="{ active: currentView === 'directory' }"
          @click="currentView = 'directory'"
        >
          目录浏览
        </button>
        <button 
          :class="{ active: currentView === 'selected' }"
          @click="currentView = 'selected'"
        >
          已选图片 ({{ store.selectedCount }})
        </button>
      </div>
      <button @click="checkUpdate" :disabled="checkingUpdate" class="update-btn">
        {{ checkingUpdate ? '检查中...' : '检查更新' }}
      </button>
    </div>

    <div class="main-content">
      <div class="sidebar">
        <DirectoryTree />
      </div>

      <div class="content">
        <div v-if="currentView === 'directory'">
          <Breadcrumb />
          <ImageGrid 
            :filter-text="filterText"
            :file-type-filter="fileTypeFilter"
            :custom-extensions="customExtensions"
          />
        </div>
        <SelectedImagesView 
          v-else
          :filter-text="filterText"
          :file-type-filter="fileTypeFilter"
          :custom-extensions="customExtensions"
        />
      </div>
    </div>

    <ImagePreview 
      :preview-list="currentPreviewList"
      :in-selected-view="currentView === 'selected'"
      @switch-to-selected="switchToSelected"
    />
    <StatusBar />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from './stores/app'
import DirectoryTree from './components/DirectoryTree.vue'
import Breadcrumb from './components/Breadcrumb.vue'
import ImageGrid from './components/ImageGrid.vue'
import SelectedImagesView from './components/SelectedImagesView.vue'
import ImagePreview from './components/ImagePreview.vue'
import StatusBar from './components/StatusBar.vue'
import { open } from '@tauri-apps/plugin-dialog'

const store = useAppStore()
const filterText = ref('')
const fileTypeFilter = ref<'all' | 'raw' | 'regular' | 'custom'>('all')
const customExtensions = ref('')
const currentView = ref<'directory' | 'selected'>('directory')
const checkingUpdate = ref(false)
const updateMessage = ref('')

const currentPreviewList = computed(() => {
  if (currentView.value === 'selected') {
    return store.allSelectedImages
  }
  return store.images
})

async function selectDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
    })
    if (selected) {
      await store.loadDirectory(selected as string)
    }
  } catch (e) {
    console.error('Failed to select directory:', e)
  }
}

interface UpdateInfo {
  version: string
  body?: string
}

function formatError(error: unknown) {
  if (error instanceof Error) {
    return error.message
  }
  if (typeof error === 'string') {
    return error
  }
  return String(error)
}

async function checkUpdate() {
  try {
    checkingUpdate.value = true
    updateMessage.value = ''
    
    const result = await invoke<UpdateInfo | null>('check_for_updates')
    
    if (result) {
      let message = `发现新版本: ${result.version}`
      if (result.body) {
        message += `\n\n更新内容:\n${result.body}`
      }
      message += '\n\n是否现在安装更新？'
      
      if (confirm(message)) {
        await installUpdateFunc()
      }
    } else {
      alert('当前已是最新版本')
    }
  } catch (e: unknown) {
    alert('检查更新失败: ' + formatError(e))
  } finally {
    checkingUpdate.value = false
  }
}

async function installUpdateFunc() {
  try {
    const result = await invoke<string>('install_update')
    alert(result)
  } catch (e: unknown) {
    alert('安装更新失败: ' + formatError(e))
  }
}

function switchToSelected() {
  currentView.value = 'selected'
}

function handleKeydown(e: KeyboardEvent) {
  if (e.ctrlKey || e.metaKey) {
    if (e.key === 'a') {
      e.preventDefault()
      store.selectAll()
    } else if (e.key === 'f') {
      e.preventDefault()
      const input = document.querySelector('.search-input') as HTMLInputElement
      input?.focus()
    }
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: #1e1e1e;
  color: #fff;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: #2a2a2a;
  border-bottom: 1px solid #3a3a3a;
}

.toolbar button {
  padding: 6px 12px;
  background: #3a3a3a;
  border: 1px solid #4a4a4a;
  border-radius: 4px;
  color: #fff;
  cursor: pointer;
  font-size: 13px;
}

.toolbar button:hover {
  background: #4a4a4a;
}

.toolbar-spacer {
  flex: 1;
}

.search-input {
  padding: 6px 12px;
  background: #3a3a3a;
  border: 1px solid #4a4a4a;
  border-radius: 4px;
  color: #fff;
  font-size: 13px;
  width: 200px;
}

.search-input:focus {
  outline: none;
  border-color: #007acc;
}

.filter-select {
  padding: 6px 12px;
  background: #3a3a3a;
  border: 1px solid #4a4a4a;
  border-radius: 4px;
  color: #fff;
  font-size: 13px;
  cursor: pointer;
}

.filter-select:focus {
  outline: none;
  border-color: #007acc;
}

.custom-ext-input {
  padding: 6px 12px;
  background: #3a3a3a;
  border: 1px solid #4a4a4a;
  border-radius: 4px;
  color: #fff;
  font-size: 13px;
  width: 150px;
}

.custom-ext-input:focus {
  outline: none;
  border-color: #007acc;
}

.view-tabs {
  display: flex;
  gap: 4px;
}

.view-tabs button {
  padding: 6px 12px;
  background: #3a3a3a;
  border: 1px solid #4a4a4a;
  border-radius: 4px;
  color: #999;
  cursor: pointer;
  font-size: 13px;
}

.view-tabs button.active {
  background: #007acc;
  border-color: #007acc;
  color: #fff;
}

.view-tabs button:hover:not(.active) {
  background: #4a4a4a;
}

.main-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.sidebar {
  width: 250px;
  background: #252525;
  border-right: 1px solid #3a3a3a;
  overflow-y: auto;
}

.content {
  flex: 1;
  overflow-y: auto;
}

.status-bar {
  background: #2a2a2a;
  padding: 6px 16px;
  border-top: 1px solid #3a3a3a;
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  height: 28px;
  flex-shrink: 0;
}

.status-left {
  display: flex;
  align-items: center;
  gap: 20px;
}

.status-indicator {
  color: #4ec9b0;
}

.status-indicator.loading {
  color: #dcdcaa;
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
.update-btn {
  padding: 6px 12px;
  background: #3a3a3a;
  border: 1px solid #4a4a4a;
  border-radius: 4px;
  color: #fff;
  cursor: pointer;
  font-size: 13px;
}

.update-btn:hover:not(:disabled) {
  background: #4a4a4a;
}

.update-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
