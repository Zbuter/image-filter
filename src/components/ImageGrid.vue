<template>
  <div class="image-grid">
    <div v-if="store.loading" class="loading-state">
      <div class="spinner"></div>
      <span>加载中...</span>
    </div>
    <div v-else-if="filteredImages.length === 0" class="empty-state">
      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
      <span>暂无图片</span>
    </div>
    <div v-else class="grid">
      <div 
        v-for="image in filteredImages" 
        :key="image.path"
        class="image-card"
        :class="{ selected: store.isImageSelected(image.path) }"
        @click="handleClick(image, $event)"
        @dblclick="handleDoubleClick(image)"
      >
        <div class="card-image">
          <img :src="getImageUrl(image.path)" :alt="image.name" loading="lazy" />
          <span v-if="image.rawPath" class="raw-badge">RAW</span>
          <button class="select-check" @click.stop="toggleSelection(image)">
            <svg v-if="store.isImageSelected(image.path)" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
          </button>
        </div>
        <div class="card-info">
          <span class="card-name" :title="image.name">{{ image.name }}</span>
          <span class="card-dir" v-if="showDirHint" :title="getDirName(image.path)">{{ getDirName(image.path) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue'
import { useAppStore } from '../stores/app'
import type { ImageInfo } from '../types'
import { convertFileSrc } from '@tauri-apps/api/core'

const props = defineProps<{
  filterText: string
  fileTypeFilter: 'all' | 'raw' | 'regular' | 'custom'
  customExtensions: string
}>()

const store = useAppStore()

const RAW_EXTENSIONS = ['cr2', 'cr3', 'nef', 'arw', 'dng', 'orf', 'rw2', 'pef', 'srw', 'raf']

const filteredImages = computed(() => {
  let images = store.images

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

function handleDoubleClick(image: ImageInfo) {
  toggleSelection(image)
}

function toggleSelection(image: ImageInfo) {
  store.toggleImageSelection(image)
}

let currentLoadGen = 0

async function loadRawPreviews() {
  currentLoadGen++
  const loadGen = currentLoadGen
  const scanGen = store.scanGeneration

  const rawImages = filteredImages.value.filter(
    img => RAW_EXTENSIONS.includes(img.extension.toLowerCase()) && !store.rawPreviewCache?.has(img.path)
  )
  const concurrency = 6
  let index = 0

  async function worker() {
    while (index < rawImages.length) {
      if (loadGen !== currentLoadGen || !store.isCurrentGeneration(scanGen)) return
      const i = index++
      const image = rawImages[i]
      if (store.rawPreviewCache?.has(image.path)) continue
      try {
        await store.getRawPreview(image.path)
      } catch (e) {
        console.error('Failed to load RAW preview:', image.path, e)
      }
    }
  }

  const workers = Array.from(
    { length: Math.min(concurrency, rawImages.length) },
    () => worker()
  )
  await Promise.all(workers)
}

// Show directory hint only when multiple directories are scanned
const showDirHint = computed(() => store.directories.length > 1)

function getDirName(path: string): string {
  const normalized = path.replace(/\\/g, '/')
  const parts = normalized.split('/')
  // Return parent folder name as short label
  if (parts.length >= 2) return parts[parts.length - 2]
  return ''
}

watch(filteredImages, () => {
  loadRawPreviews()
}, { immediate: true })
</script>

<style scoped>
.image-grid {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  height: 240px;
  color: var(--text-tertiary);
  font-size: 13px;
}

.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--border-default);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 8px;
}

.image-card {
  position: relative;
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  overflow: hidden;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.image-card:hover {
  border-color: var(--border-strong);
  background: var(--bg-elevated);
}

.image-card.selected {
  border-color: var(--accent);
  box-shadow: 0 0 0 1px var(--accent-border);
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

.select-check {
  position: absolute;
  top: 6px;
  right: 6px;
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.5);
  border: 1.5px solid rgba(255, 255, 255, 0.4);
  border-radius: 50%;
  color: #fff;
  cursor: pointer;
  opacity: 0;
  transition: all var(--transition-fast);
  backdrop-filter: blur(4px);
  z-index: 2;
}

.image-card:hover .select-check,
.image-card.selected .select-check {
  opacity: 1;
}

.image-card.selected .select-check {
  background: var(--accent);
  border-color: var(--accent);
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