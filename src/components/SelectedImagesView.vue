<template>
  <div class="selected-view">
    <div class="view-header">
      <div class="header-info">
        <h3>已选图片</h3>
        <span class="count">{{ store.selectedCount }} 张</span>
      </div>
      <div class="header-actions">
        <select v-if="store.groups.length > 0" v-model="filterGroupId" class="group-filter">
          <option value="">全部</option>
          <option v-for="g in store.groups" :key="g.id" :value="g.id">{{ g.name }}</option>
        </select>
        <button 
          class="btn-export" 
          @click="showExportDialog = true" 
          :disabled="store.selectedCount === 0"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
          <span>导出</span>
        </button>
      </div>
    </div>

    <div v-if="filteredImages.length === 0" class="empty-state">
      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 11 12 14 22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>
      <span>未选择任何图片</span>
      <span class="hint">双击图片或 Ctrl+点击来选中</span>
    </div>

    <div v-else class="grid">
      <div 
        v-for="image in filteredImages" 
        :key="image.path"
        class="image-card selected"
        @click="handleClick(image, $event)"
        @dblclick="toggleSelection(image)"
      >
        <div class="card-image">
          <img :src="getImageUrl(image.path)" :alt="image.name" loading="lazy" />
          <span v-if="image.rawPath" class="raw-badge">RAW</span>
          <span v-if="store.getGroupForImage(image.path)" class="group-tag"
                :style="{ background: store.getGroupForImage(image.path)!.color }">
            {{ store.getGroupForImage(image.path)!.name }}
          </span>
          <button class="remove-btn" @click.stop="toggleSelection(image)" title="移除选中">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          </button>
        </div>
        <div class="card-info">
          <span class="card-name" :title="image.name">{{ image.name }}</span>
          <span class="card-dir" v-if="showDirHint" :title="getDirName(image.path)">{{ getDirName(image.path) }}</span>
        </div>
      </div>
    </div>

    <ExportDialog v-model:show="showExportDialog" :images="filteredImages" @exported="onExported" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useAppStore } from '../stores/app'
import type { ImageInfo } from '../types'
import { convertFileSrc } from '@tauri-apps/api/core'
import ExportDialog from './ExportDialog.vue'

const props = defineProps<{
  filterText: string
  fileTypeFilter: 'all' | 'raw' | 'regular' | 'custom'
  customExtensions: string
}>()

const store = useAppStore()
const showExportDialog = ref(false)
const filterGroupId = ref('')

const RAW_EXTENSIONS = ['cr2', 'cr3', 'nef', 'arw', 'dng', 'orf', 'rw2', 'pef', 'srw', 'raf']

const filteredImages = computed(() => {
  let images = store.allSelectedImages

  // 分组筛选
  if (filterGroupId.value) {
    images = images.filter(img => {
      const group = store.getGroupForImage(img.path)
      return group && group.id === filterGroupId.value
    })
  }

  if (props.fileTypeFilter === 'raw') {
    images = images.filter(img => RAW_EXTENSIONS.includes(img.extension.toLowerCase()))
  } else if (props.fileTypeFilter === 'regular') {
    images = images.filter(img => !RAW_EXTENSIONS.includes(img.extension.toLowerCase()))
  } else if (props.fileTypeFilter === 'custom') {
    const exts = getCustomExtList()
    if (exts.length > 0) {
      images = images.filter(img => exts.includes(img.extension.toLowerCase()))
    }
  }

  if (props.filterText) {
    const filter = props.filterText.toLowerCase()
    images = images.filter(img => img.name.toLowerCase().includes(filter))
  }

  return images
})

function getCustomExtList(): string[] {
  if (!props.customExtensions) return []
  return props.customExtensions
    .split(',')
    .map(e => e.trim().toLowerCase().replace(/^\./, ''))
    .filter(e => e.length > 0)
}

function getImageUrl(path: string): string {
  const ext = path.split('.').pop()?.toLowerCase() || ''
  if (RAW_EXTENSIONS.includes(ext)) {
    return store.rawPreviewCache?.get(path) || convertFileSrc(path)
  }
  return convertFileSrc(path)
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / 1024 / 1024).toFixed(1) + ' MB'
}

function handleClick(image: ImageInfo, event: MouseEvent) {
  if (event.ctrlKey || event.metaKey) {
    toggleSelection(image)
  } else {
    store.setPreviewImage(image)
  }
}

function toggleSelection(image: ImageInfo) {
  store.toggleImageSelection(image)
}

const showDirHint = computed(() => store.directories.length > 1)

function getDirName(path: string): string {
  const normalized = path.replace(/\\/g, '/')
  const parts = normalized.split('/')
  if (parts.length >= 2) return parts[parts.length - 2]
  return ''
}

function onExported() {
  // Could refresh or show success toast
}
</script>

<style scoped>
.selected-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  min-height: 0;
}

.view-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
  gap: 8px;
}

.header-info {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.header-info h3 {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.count {
  font-size: 12px;
  color: var(--accent);
  font-weight: 500;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.group-filter {
  padding: 4px 8px;
  background: var(--bg-base);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 11px;
  cursor: pointer;
}

.group-tag {
  position: absolute;
  bottom: 6px;
  left: 6px;
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 10px;
  font-weight: 500;
  color: #fff;
  z-index: 2;
  pointer-events: none;
}

.btn-export {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 12px;
  background: var(--accent-muted);
  border: 1px solid var(--accent-border);
  border-radius: var(--radius-sm);
  color: var(--accent);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-export:hover:not(:disabled) {
  background: rgba(212, 160, 83, 0.2);
  border-color: var(--accent);
}

.btn-export:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  flex: 1;
  color: var(--text-tertiary);
  font-size: 13px;
}

.hint {
  font-size: 11px;
  color: var(--text-disabled);
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 8px;
  padding: 12px;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
  align-content: start;
}

.image-card {
  position: relative;
  background: var(--bg-surface);
  border: 1px solid var(--accent-border);
  border-radius: var(--radius-md);
  overflow: hidden;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.image-card:hover {
  border-color: var(--accent);
  background: var(--bg-elevated);
}

.card-image {
  position: relative;
  aspect-ratio: 1;
  overflow: hidden;
  background: var(--bg-base);
}

.card-image img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform var(--transition-normal);
}

.image-card:hover .card-image img {
  transform: scale(1.03);
}

.raw-badge {
  position: absolute;
  top: 6px;
  left: 6px;
  padding: 1px 5px;
  background: var(--accent);
  color: var(--bg-base);
  font-size: 9px;
  font-weight: 700;
  border-radius: 3px;
  letter-spacing: 0.5px;
  line-height: 1.4;
  pointer-events: none;
  z-index: 2;
}

.remove-btn {
  position: absolute;
  top: 6px;
  right: 6px;
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(212, 83, 83, 0.8);
  border: none;
  border-radius: 50%;
  color: #fff;
  cursor: pointer;
  opacity: 0;
  transition: all var(--transition-fast);
  backdrop-filter: blur(4px);
  z-index: 2;
}

.image-card:hover .remove-btn {
  opacity: 1;
}

.remove-btn:hover {
  background: var(--danger);
}

.card-info {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 6px;
  padding: 6px 8px;
}

.card-name {
  font-size: 11px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.card-dir {
  font-size: 9px;
  color: var(--accent);
  flex-shrink: 0;
  opacity: 0.8;
}
</style>