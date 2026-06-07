<template>
  <div class="selected-images-view">
    <ExportDialog 
      v-if="store.selectedCount > 0"
      v-model:show="showExportDialog"
      @exported="handleExported"
    />
    <div v-if="store.selectedCount > 0" class="export-bar">
      <button class="export-btn" @click="showExportDialog = true">
        导出选中图片 ({{ store.selectedCount }})
      </button>
    </div>
    <div v-if="filteredImages.length === 0" class="empty">暂无选中图片</div>
    <div v-else class="grid">
      <div 
        v-for="image in filteredImages" 
        :key="image.path"
        class="image-item"
        :class="{ selected: true }"
        @click="handleClick(image, $event)"
        @dblclick="handleDoubleClick(image)"
      >
        <img :src="getImageUrl(image.path)" :alt="image.name" loading="lazy" />
        <div class="image-info">
          <div class="image-name">{{ image.name }}</div>
          <div class="image-size">{{ formatSize(image.size) }}</div>
        </div>
        <div class="checkmark" @click.stop="toggleSelection(image)">
          <span>✓</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import ExportDialog from './ExportDialog.vue'
import { useAppStore } from '../stores/app'
import type { ImageInfo } from '../types'
import { convertFileSrc } from '@tauri-apps/api/core'

const props = defineProps<{
  filterText: string
  fileTypeFilter: 'all' | 'raw' | 'regular' | 'custom'
  customExtensions: string
}>()

const store = useAppStore()
const showExportDialog = ref(false)

const RAW_EXTENSIONS = ['cr2', 'cr3', 'nef', 'arw', 'dng', 'orf', 'rw2', 'pef', 'srw', 'raf']

const filteredImages = computed(() => {
  let images = store.allSelectedImages

  // 类型筛选
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

  // 名称搜索
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

function handleDoubleClick(image: ImageInfo) {
  toggleSelection(image)
}

function toggleSelection(image: ImageInfo) {
  store.toggleImageSelection(image)
}

function handleExported() {
  store.clearSelection()
}
</script>

<style scoped>
.selected-images-view {
  padding: 16px;
}

.empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: #999;
  font-size: 14px;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 12px;
}

.image-item {
  position: relative;
  aspect-ratio: 1;
  background: #2a2a2a;
  border-radius: 4px;
  overflow: hidden;
  cursor: pointer;
  border: 2px solid #007acc;
  transition: border-color 0.2s;
}

.image-item:hover {
  border-color: #0098ff;
}

.image-item img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.image-info {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.8));
  padding: 20px 8px 8px;
}

.image-name {
  font-size: 11px;
  color: #fff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.image-size {
  font-size: 10px;
  color: #999;
  margin-top: 2px;
}

.checkmark {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 20px;
  height: 20px;
  background: #007acc;
  border: 2px solid #007acc;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  color: #fff;
  cursor: pointer;
}

.export-bar {
  position: fixed;
  bottom: 40px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(42, 42, 42, 0.95);
  padding: 12px 20px;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  z-index: 100;
}

.export-btn {
  background: #007acc;
  border: none;
  border-radius: 4px;
  padding: 8px 20px;
  color: #fff;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.2s;
}

.export-btn:hover {
  background: #0098ff;
}
</style>
