<template>
  <div class="image-grid">
    <div v-if="store.loading" class="loading">加载中...</div>
    <div v-else-if="filteredImages.length === 0" class="empty">暂无图片</div>
    <div v-else class="grid">
      <div 
        v-for="image in filteredImages" 
        :key="image.path"
        class="image-item"
        :class="{ selected: store.isImageSelected(image.path) }"
        @click="handleClick(image, $event)"
        @dblclick="handleDoubleClick(image)"
      >
        <img :src="getImageUrl(image.path)" :alt="image.name" loading="lazy" />
        <div class="image-info">
          <div class="image-name">{{ image.name }}</div>
          <div class="image-size">{{ formatSize(image.size) }}</div>
        </div>
        <div class="checkmark" @click.stop="toggleSelection(image)">
          <span v-if="store.isImageSelected(image.path)">✓</span>
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

watch(filteredImages, () => {
  loadRawPreviews()
}, { immediate: true })
</script>

<style scoped>
.image-grid {
  padding: 16px;
}

.loading, .empty {
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
  border: 2px solid transparent;
  transition: border-color 0.2s;
}

.image-item:hover {
  border-color: #4a4a4a;
}

.image-item.selected {
  border-color: #007acc;
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
  background: rgba(0, 0, 0, 0.6);
  border: 2px solid #fff;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  color: #fff;
  cursor: pointer;
}

.image-item.selected .checkmark {
  background: #007acc;
  border-color: #007acc;
}
</style>
