<template>
  <div class="app-container">
    <!-- Toolbar -->
    <header class="toolbar">
      <div class="toolbar-left">
        <button class="btn btn-primary" @click="selectDirectory">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
          <span>打开目录</span>
        </button>
        <button class="btn btn-ghost" @click="addScanDirectory" title="添加额外扫描目录（双卡模式）">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        </button>
        <template v-if="store.directories.length > 1">
          <div class="dir-tags">
            <span 
              v-for="dir in store.directories" 
              :key="dir" 
              class="dir-tag"
            >
              <span class="dir-tag-name">{{ getDirLabel(dir) }}</span>
              <button class="dir-tag-remove" @click="store.removeDirectory(dir)" title="移除目录">×</button>
            </span>
          </div>
        </template>
        <div class="toolbar-divider"></div>
        <div class="search-box">
          <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
          <input 
            v-model="filterText" 
            type="text" 
            placeholder="搜索文件名..."
            class="search-input"
          />
        </div>
        <select v-model="fileTypeFilter" class="filter-select">
          <option value="all">全部类型</option>
          <option value="raw">RAW 格式</option>
          <option value="regular">普通图片</option>
          <option value="custom">自定义后缀</option>
        </select>
        <input 
          v-if="fileTypeFilter === 'custom'"
          v-model="customExtensions"
          type="text"
          placeholder="jpg,png"
          class="ext-input"
        />
      </div>

      <div class="toolbar-right">
        <!-- Extensible slot for future AI features -->
        <slot name="toolbar-actions"></slot>

        <button 
          class="btn btn-ghost"
          :class="{ active: sidebarMode === 'ai' }"
          @click="toggleAiPanel"
          title="AI 废片检测"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2a4 4 0 0 1 4 4c0 1.95-1.4 3.58-3.25 3.93L12 22"/><path d="M8 6a4 4 0 0 1 .65-2.18"/><path d="M17 12.5c1.77.64 3 2.34 3 4.28A4.5 4.5 0 0 1 15.5 21h-7A4.5 4.5 0 0 1 4 16.78c0-1.94 1.23-3.64 3-4.28"/></svg>
        </button>
        <div class="view-switcher">
          <button 
            class="switcher-btn"
            :class="{ active: currentView === 'directory' }"
            @click="currentView = 'directory'"
            title="目录浏览"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>
          </button>
          <button 
            class="switcher-btn"
            :class="{ active: currentView === 'selected' }"
            @click="currentView = 'selected'"
            title="已选图片"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 11 12 14 22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>
            <span v-if="store.selectedCount > 0" class="badge">{{ store.selectedCount }}</span>
          </button>
        </div>

        <div class="toolbar-divider"></div>

        <button 
          class="btn btn-ghost"
          @click="openNewGroupDialog()"
          title="新建分组"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          <span>分组</span>
        </button>
        <button 
          v-if="store.groups.length > 0"
          class="btn btn-ghost"
          @click="showGroupManager = true"
          title="管理分组"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
        </button>

        <button 
          class="btn btn-ghost" 
          @click="checkUpdate" 
          :disabled="checkingUpdate"
          title="检查更新"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>
        </button>
      </div>
    </header>

    <!-- Main Content -->
    <div class="main-content">
      <aside class="sidebar">
        <DirectoryTree v-if="sidebarMode === 'directory'" />
      <AiPanel 
        v-else 
         @preview="previewFromAi"
         @hover-waste="handleHoverWaste"
         @waste-context-menu="handleWasteContextMenu"
        />
      </aside>

      <!-- Waste Context Menu -->
      <Teleport to="body">
        <div
          v-if="wasteCtxMenu.visible"
          class="context-menu"
          :style="{ left: wasteCtxMenu.x + 'px', top: wasteCtxMenu.y + 'px' }"
          @click.stop
        >
         <button class="ctx-item" @click="markWasteAsGood">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
            标记为非废片
          </button>
        </div>
      </Teleport>

      <main class="content">
        <div v-if="currentView === 'directory'" class="content-inner">
          <Breadcrumb />
          <ImageGrid 
            ref="imageGridRef"
            :highlighted-waste-path="highlightedWastePath"
            :filter-text="filterText"
            :file-type-filter="fileTypeFilter"
            :custom-extensions="customExtensions"
            @new-group="openNewGroupDialog"
          />
        </div>
        <SelectedImagesView 
          v-else
          :filter-text="filterText"
          :file-type-filter="fileTypeFilter"
          :custom-extensions="customExtensions"
        />
      </main>
    </div>

    <ImagePreview 
      :preview-list="currentPreviewList"
      :in-selected-view="currentView === 'selected'"
      @switch-to-selected="switchToSelected"
    />
    <StatusBar />
    <GroupDialog 
      v-model:show="showGroupDialog"
      @confirm="handleGroupConfirm"
    />
    <GroupManager v-model:show="showGroupManager" />
    <!-- Global Toast -->
    <Transition name="global-toast">
      <div v-if="toastMessage" class="global-toast" :class="toastType">{{ toastMessage }}</div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from './stores/app'
import DirectoryTree from './components/DirectoryTree.vue'
import AiPanel from './components/AiPanel.vue'
import Breadcrumb from './components/Breadcrumb.vue'
import ImageGrid from './components/ImageGrid.vue'
import SelectedImagesView from './components/SelectedImagesView.vue'
import ImagePreview from './components/ImagePreview.vue'
import StatusBar from './components/StatusBar.vue'
import GroupDialog from './components/GroupDialog.vue'
import GroupManager from './components/GroupManager.vue'
import { open } from '@tauri-apps/plugin-dialog'

const store = useAppStore()
const filterText = ref('')
const fileTypeFilter = ref<'all' | 'raw' | 'regular' | 'custom'>('all')
const customExtensions = ref('')
const currentView = ref<'directory' | 'selected'>('directory')

store.autoLoadWasteModel()
store.loadGroups()

const toastMessage = computed(() => store.toastMessage)
const toastType = computed(() => store.toastType)
const showToast = store.showToast

const sidebarMode = ref<'directory' | 'ai'>('directory')
const imageGridRef = ref<any>(null)
const aiModelDir = ref('')
const checkingUpdate = ref(false)
const updateMessage = ref('')

const currentPreviewList = computed(() => {
  if (currentView.value === 'selected') {
    return store.allSelectedImages
  }
  return store.images
})

// 分组弹窗
const showGroupDialog = ref(false)
const showGroupManager = ref(false)
const groupDialogImage = ref<string | null>(null)

function openNewGroupDialog(imagePath?: string) {
  groupDialogImage.value = imagePath || null
  showGroupDialog.value = true
}

function handleGroupConfirm(name: string, shortcut: string) {
  const group = store.createGroup(name, shortcut)
  if (groupDialogImage.value) {
    store.addToGroup(groupDialogImage.value, group.id)
  }
  showToast(`分组「${name}」已创建`)
}

async function selectDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
    })
    console.log('[AI] Directory selected:', selected, typeof selected);
        if (selected) {
      await store.loadDirectory(selected as string)
    }
  } catch (e) {
    console.error('Failed to select directory:', e)
  }
}

async function addScanDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
    })
    if (selected) {
      await store.addDirectory(selected as string)
    }
  } catch (e) {
    console.error('Failed to add directory:', e)
  }
}

function getDirLabel(path: string): string {
  // Show last two segments for readability, e.g. "D:\DCIM" or "E:\RAW\2024"
  const normalized = path.replace(/\\/g, '/').replace(/\/$/, '')
  const parts = normalized.split('/')
  if (parts.length <= 2) return path
  return parts.slice(-2).join('/')
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


async function toggleAiPanel() {
    if (sidebarMode.value === 'ai') {
      sidebarMode.value = 'directory';
    } else {
      sidebarMode.value = 'ai';
    }
  }


function previewFromAi(path: string) {
  const img = store.images.find(i => i.path === path);
  if (img) {
    store.setPreviewImage(img);
    // 滚动到网格中的对应图片
    if (imageGridRef.value) {
      imageGridRef.value.scrollToPath(path)
    }
  }
}


  const highlightedWastePath = ref<string | null>(null)
  const wasteCtxMenu = ref({ visible: false, x: 0, y: 0, path: '' })

  function handleHoverWaste(path: string | null) {
    highlightedWastePath.value = path
  }

  function handleWasteContextMenu(e: MouseEvent, path: string) {
    wasteCtxMenu.value = { visible: true, x: e.clientX, y: e.clientY, path }
  }

  async function markWasteAsGood() {
    const path = wasteCtxMenu.value.path
    wasteCtxMenu.value.visible = false
    if (!path) return
    try {
      await store.markWasteAsGood(path)
    } catch (e) {
      console.error('Failed to mark:', e)
    }
  }

  function hideWasteCtxMenu() {
    wasteCtxMenu.value.visible = false
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
      showToast('当前已是最新版本')
    }
  } catch (e: unknown) {
    showToast('检查更新失败: ' + formatError(e))
  } finally {
    checkingUpdate.value = false
  }
}

async function installUpdateFunc() {
  try {
    const result = await invoke<string>('install_update')
    showToast(result)
  } catch (e: unknown) {
    showToast('安装更新失败: ' + formatError(e))
  }
}

function switchToSelected() {
  store.setPreviewImage(null)
  currentView.value = 'selected'
}

function isInputFocused(): boolean {
  const el = document.activeElement
  if (!el) return false
  const tag = el.tagName.toLowerCase()
  return tag === 'input' || tag === 'textarea' || tag === 'select' || (el as HTMLElement).isContentEditable
}

function handleKeydown(e: KeyboardEvent) {
  // Let input elements handle their own shortcuts
  if (isInputFocused()) return

  if (e.ctrlKey || e.metaKey) {
    if (e.key === 'a') {
      e.preventDefault()
      store.selectAll()
    } else if (e.key === 'i') {
      e.preventDefault()
      store.invertSelection()
    } else if (e.key === 'f') {
      e.preventDefault()
      const input = document.querySelector('.search-input') as HTMLInputElement
      input?.focus()
    }
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
  document.addEventListener('click', hideWasteCtxMenu)
})

onUnmounted(() => {
  document.removeEventListener('click', hideWasteCtxMenu)
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg-base);
  color: var(--text-primary);
}

/* ── Toolbar ─────────────────────────────────────── */
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--toolbar-height);
  padding: 0 12px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
  gap: 8px;
}

.toolbar-left,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 6px;
}

.toolbar-divider {
  width: 1px;
  height: 20px;
  background: var(--border-subtle);
  margin: 0 4px;
  flex-shrink: 0;
}

/* Buttons */
.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 12px;
  cursor: pointer;
  transition: all var(--transition-fast);
  white-space: nowrap;
  line-height: 1;
}

.btn:hover {
  background: var(--bg-hover);
  border-color: var(--border-strong);
}

.btn-primary {
  background: var(--accent-muted);
  border-color: var(--accent-border);
  color: var(--accent);
}

.btn-primary:hover {
  background: rgba(212, 160, 83, 0.2);
  border-color: var(--accent);
}

.btn-ghost {
  background: transparent;
  border-color: transparent;
  color: var(--text-secondary);
  padding: 5px 6px;
}

.btn-ghost:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.btn-ghost.active {
  color: var(--accent);
  background: var(--accent-muted);
}

.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* Search */
.search-box {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 8px;
  color: var(--text-tertiary);
  pointer-events: none;
}

.search-input {
  padding: 5px 8px 5px 28px;
  background: var(--bg-base);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  width: 180px;
  transition: border-color var(--transition-fast);
}

.search-input::placeholder {
  color: var(--text-tertiary);
}

.search-input:focus {
  border-color: var(--accent-border);
}

/* Filter Select */
.filter-select {
  padding: 5px 24px 5px 8px;
  background: var(--bg-base);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  cursor: pointer;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6'%3E%3Cpath d='M0 0l5 6 5-6z' fill='%236b6a68'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 8px center;
  transition: border-color var(--transition-fast);
}

.filter-select:focus {
  border-color: var(--accent-border);
}

.ext-input {
  padding: 5px 8px;
  background: var(--bg-base);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  width: 100px;
  transition: border-color var(--transition-fast);
}

.ext-input::placeholder {
  color: var(--text-tertiary);
}

.ext-input:focus {
  border-color: var(--accent-border);
}

/* View Switcher */
.view-switcher {
  display: flex;
  background: var(--bg-base);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.switcher-btn {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 4px 8px;
  background: transparent;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  transition: all var(--transition-fast);
  font-size: 11px;
}

.switcher-btn:first-child {
  border-right: 1px solid var(--border-subtle);
}

.switcher-btn:hover {
  color: var(--text-secondary);
  background: var(--bg-hover);
}

.switcher-btn.active {
  color: var(--accent);
  background: var(--accent-muted);
}

.badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  background: var(--accent);
  color: var(--bg-base);
  border-radius: 8px;
  font-size: 10px;
  font-weight: 600;
}

/* ── Main Layout ─────────────────────────────────── */
.main-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.sidebar {
  width: var(--sidebar-width);
  background: var(--bg-surface);
  border-right: 1px solid var(--border-subtle);
  overflow-y: auto;
  flex-shrink: 0;
}

.content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.content-inner {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

/* Directory Tags */
.dir-tags {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: 4px;
}

.dir-tag {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 2px 6px 2px 8px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: 10px;
  font-size: 10px;
  color: var(--text-secondary);
  white-space: nowrap;
}

.dir-tag-name {
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.dir-tag-remove {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 14px;
  height: 14px;
  background: transparent;
  border: none;
  border-radius: 50%;
  color: var(--text-tertiary);
  font-size: 12px;
  line-height: 1;
  cursor: pointer;
  transition: all var(--transition-fast);
  padding: 0;
}

.dir-tag-remove:hover {
  background: var(--danger);
  color: #fff;
}

/* Waste Context Menu (global, used by Teleport) */
.context-menu {
  position: fixed;
  z-index: 2000;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  padding: 4px 0;
  min-width: 160px;
}

.ctx-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 12px;
  background: transparent;
  border: none;
  color: var(--text-primary);
  font-size: 12px;
  cursor: pointer;
  transition: background var(--transition-fast);
  text-align: left;
}

.ctx-item:hover {
  background: var(--bg-hover);
}

.ctx-item svg {
  flex-shrink: 0;
  color: var(--text-secondary);
}

/* Global Toast */
.global-toast {
  position: fixed;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  padding: 8px 16px;
  border-radius: var(--radius-md);
  font-size: 12px;
  color: #fff;
  z-index: 9999;
  pointer-events: none;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
}
.global-toast.error {
  background: var(--danger);
}
.global-toast.success {
  background: var(--success);
}

.global-toast-enter-active, .global-toast-leave-active {
  transition: all 0.3s ease;
  opacity: 1;
}

.global-toast-enter-from, .global-toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(10px);
}
</style>
