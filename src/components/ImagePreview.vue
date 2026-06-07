<template>
  <div v-if="store.previewImage" class="preview-overlay" @click="handleOverlayClick">
    <div class="preview-toolbar">
      <button class="toolbar-btn" @click="closePreview">✕ 关闭</button>
      <div class="toolbar-divider"></div>
      <button class="toolbar-btn" @click="zoomOut">🔍−</button>
      <span class="zoom-level">{{ Math.round(zoom * 100) }}%</span>
      <button class="toolbar-btn" @click="zoomIn">🔍+</button>
      <button class="toolbar-btn" @click="resetZoom">适应</button>
      <div class="toolbar-divider"></div>
      <button class="toolbar-btn" @click="rotateLeft" title="向左旋转 (R)">↺</button>
      <button class="toolbar-btn" @click="rotateRight" title="向右旋转 (Shift+R)">↻</button>
      <div class="toolbar-spacer"></div>
      <span class="file-info">{{ store.previewImage.name }}</span>
      <span class="dimension-info" v-if="store.previewImage.width && store.previewImage.height">
        {{ store.previewImage.width }} × {{ store.previewImage.height }}
      </span>
      <span class="position-info">
        {{ currentIndex + 1 }} / {{ previewList.length }}
      </span>
      <div class="toolbar-divider"></div>
      <button 
        class="toolbar-btn select-btn"
        :class="{ selected: isSelected }"
        @click="toggleSelection"
      >
        {{ isSelected ? '✓ 已选中' : '○ 选中' }}
      </button>
      <template v-if="!inSelectedView">
        <div class="toolbar-divider"></div>
        <label class="toolbar-checkbox">
          <input type="checkbox" v-model="switchToSelectedAtEnd" />
          <span>末尾跳转到已选</span>
        </label>
      </template>
    </div>
    
    <div class="preview-content" @wheel="handleWheel" @mousedown="handleMouseDown">
      <button class="nav-btn prev" @click.stop="prevImage">‹</button>
      
      <div 
        class="image-wrapper"
        :style="imageWrapperStyle"
        @dblclick="toggleActualSize"
      >
        <img 
          :src="currentImageUrl"
          :alt="store.previewImage.name"
          @load="handleImageLoad"
          draggable="false"
        />
      </div>
      
      <button class="nav-btn next" @click.stop="nextImage">›</button>
      
      <div class="preview-hint">
        纵向滚轮缩放 | 横向滚轮切换 | 拖拽平移 | 双击 1:1 查看 | R 旋转
      </div>
    </div>
    
    <div class="thumbnail-strip">
      <div
        v-for="(image, index) in previewList"
        :key="image.path"
        class="thumbnail-item"
        :class="{ current: index === currentIndex }"
        @click.stop="goToImage(index)"
      >
        <img :src="getThumbnailUrl(image)" :alt="image.name" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, reactive } from 'vue'
import { useAppStore } from '../stores/app'
import { convertFileSrc } from '@tauri-apps/api/core'
import type { ImageInfo } from '../types'

const emit = defineEmits<{
  (e: 'switch-to-selected'): void
}>()

const props = defineProps<{
  previewList?: ImageInfo[]
  inSelectedView?: boolean
}>()

const store = useAppStore()

const zoom = ref(1)
const panX = ref(0)
const panY = ref(0)
const rotation = ref(0)
const isDragging = ref(false)
const dragStartX = ref(0)
const dragStartY = ref(0)
const imageLoaded = ref(false)
const switchToSelectedAtEnd = ref(false)
const rawPreviewUrls = reactive<Record<string, string>>({})

const RAW_EXTENSIONS = ['cr2', 'cr3', 'nef', 'arw', 'dng', 'orf', 'rw2', 'pef', 'srw', 'raf']
const placeholderUrl = 'data:image/svg+xml,%3Csvg xmlns="http://www.w3.org/2000/svg" width="100" height="100"%3E%3Crect fill="%23333" width="100" height="100"/%3E%3C/svg%3E'

const currentPreviewList = computed(() => props.previewList || store.images)

const currentIndex = computed(() => {
  if (!store.previewImage) return -1
  return currentPreviewList.value.findIndex(img => img.path === store.previewImage!.path)
})

const isSelected = computed(() => {
  if (!store.previewImage) return false
  return store.isImageSelected(store.previewImage.path)
})

const imageWrapperStyle = computed(() => ({
  transform: `translate(${panX.value}px, ${panY.value}px) scale(${zoom.value}) rotate(${rotation.value}deg)`,
  transition: isDragging.value ? 'none' : 'transform 0.2s',
  transformOrigin: 'center center'
}))

function isRawFile(path: string): boolean {
  const ext = path.split('.').pop()?.toLowerCase() || ''
  return RAW_EXTENSIONS.includes(ext)
}

const currentImageUrl = computed(() => {
  if (!store.previewImage) return placeholderUrl
  const path = store.previewImage.path
  if (isRawFile(path)) {
    return rawPreviewUrls[path] || placeholderUrl
  }
  return convertFileSrc(path)
})

function getThumbnailUrl(image: ImageInfo): string {
  if (isRawFile(image.path)) {
    return rawPreviewUrls[image.path] || placeholderUrl
  }
  return convertFileSrc(image.path)
}

async function loadRawPreview(image: ImageInfo | null) {
  if (!image || !isRawFile(image.path)) return
  if (rawPreviewUrls[image.path]) return
  
  try {
    const dataUrl = await store.getRawPreview(image.path)
    if (dataUrl) {
      rawPreviewUrls[image.path] = dataUrl
    }
  } catch (e) {
    console.error('Failed to load RAW preview:', image.path, e)
  }
}

async function preloadNearbyRawPreviews() {
  const idx = currentIndex.value
  if (idx < 0) return
  
  const range = 3
  const start = Math.max(0, idx - range)
  const end = Math.min(currentPreviewList.value.length, idx + range + 1)
  
  for (let i = start; i < end; i++) {
    const img = currentPreviewList.value[i]
    if (isRawFile(img.path) && !rawPreviewUrls[img.path]) {
      try {
        const dataUrl = await store.getRawPreview(img.path)
        if (dataUrl) {
          rawPreviewUrls[img.path] = dataUrl
        }
      } catch (e) {
        // Skip errors for preloading
      }
    }
  }
}

function handleOverlayClick(event: MouseEvent) {
  if (event.target === event.currentTarget) {
    closePreview()
  }
}

function closePreview() {
  store.setPreviewImage(null)
  resetZoom()
}

function zoomIn() {
  zoom.value = Math.min(zoom.value * 1.2, 20)
}

function zoomOut() {
  zoom.value = Math.max(zoom.value / 1.2, 0.1)
}

function resetZoom() {
  zoom.value = 1
  panX.value = 0
  panY.value = 0
  rotation.value = 0
}

function toggleActualSize() {
  if (zoom.value === 1) {
    zoom.value = 2
  } else {
    resetZoom()
  }
}

function rotateLeft() {
  rotation.value = (rotation.value - 90) % 360
}

function rotateRight() {
  rotation.value = (rotation.value + 90) % 360
}

function handleWheel(event: WheelEvent) {
  event.preventDefault()
  
  // 横向滚动切换图片
  if (Math.abs(event.deltaX) > Math.abs(event.deltaY)) {
    if (event.deltaX > 0) {
      nextImage()
    } else {
      prevImage()
    }
    return
  }
  
  // 纵向滚动缩放
  const delta = event.deltaY > 0 ? 0.9 : 1.1
  zoom.value = Math.max(0.1, Math.min(20, zoom.value * delta))
}

function handleMouseDown(event: MouseEvent) {
  if (event.button !== 0) return
  
  isDragging.value = true
  dragStartX.value = event.clientX - panX.value
  dragStartY.value = event.clientY - panY.value
  
  const handleMouseMove = (e: MouseEvent) => {
    if (!isDragging.value) return
    panX.value = e.clientX - dragStartX.value
    panY.value = e.clientY - dragStartY.value
  }
  
  const handleMouseUp = () => {
    isDragging.value = false
    document.removeEventListener('mousemove', handleMouseMove)
    document.removeEventListener('mouseup', handleMouseUp)
  }
  
  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', handleMouseUp)
}

function prevImage() {
  const list = currentPreviewList.value
  if (list.length === 0) return
  const newIndex = currentIndex.value <= 0 ? list.length - 1 : currentIndex.value - 1
  store.setPreviewImage(list[newIndex])
  resetZoom()
}

function nextImage() {
  const list = currentPreviewList.value
  if (list.length === 0) return
  
  if (currentIndex.value >= list.length - 1) {
    // At the last image
    if (switchToSelectedAtEnd.value && store.selectedCount > 0) {
      // Switch to selected images view
      closePreview()
      // Emit event to switch view (will be handled by parent)
      emit('switch-to-selected')
      return
    }
    // Otherwise wrap around
    store.setPreviewImage(list[0])
  } else {
    store.setPreviewImage(list[currentIndex.value + 1])
  }
  resetZoom()
}

function goToImage(index: number) {
  if (index >= 0 && index < currentPreviewList.value.length) {
    store.setPreviewImage(currentPreviewList.value[index])
    resetZoom()
  }
}

function toggleSelection() {
  if (store.previewImage) {
    store.toggleImageSelection(store.previewImage)
  }
}

function handleImageLoad() {
  imageLoaded.value = true
}

function handleKeydown(event: KeyboardEvent) {
  if (!store.previewImage) return
  
  switch (event.key) {
    case 'Escape':
      closePreview()
      break
    case 'ArrowLeft':
      prevImage()
      break
    case 'ArrowRight':
      nextImage()
      break
    case ' ':
      event.preventDefault()
      toggleSelection()
      break
    case 'r':
    case 'R':
      if (event.shiftKey) {
        rotateRight()
      } else {
        rotateLeft()
      }
      break
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})

watch(() => store.previewImage, async (newImage) => {
  resetZoom()
  imageLoaded.value = false
  if (newImage) {
    await loadRawPreview(newImage)
    preloadNearbyRawPreviews()
  }
}, { immediate: true })
</script>

<style scoped>
.preview-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: #0a0a0a;
  z-index: 1000;
  display: flex;
  flex-direction: column;
}

.preview-toolbar {
  background: rgba(30, 30, 30, 0.95);
  padding: 10px 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  z-index: 10;
}

.toolbar-btn {
  background: transparent;
  border: none;
  color: #ccc;
  font-size: 13px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: background 0.2s;
}

.toolbar-btn:hover {
  background: #3a3a3a;
}

.toolbar-btn.select-btn.selected {
  background: #007acc;
  color: #fff;
}

.toolbar-divider {
  width: 1px;
  height: 20px;
  background: #444;
}

.toolbar-spacer {
  flex: 1;
}

.zoom-level, .file-info, .dimension-info, .position-info {
  color: #ccc;
  font-size: 13px;
}

.dimension-info, .position-info {
  color: #888;
  font-size: 12px;
  margin-left: 12px;
}

.preview-content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

.nav-btn {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  width: 44px;
  height: 44px;
  background: rgba(40, 40, 40, 0.8);
  border: 1px solid #444;
  border-radius: 50%;
  color: #fff;
  font-size: 20px;
  cursor: pointer;
  z-index: 5;
  transition: background 0.2s;
}

.nav-btn:hover {
  background: rgba(60, 60, 60, 0.9);
}

.nav-btn.prev {
  left: 16px;
}

.nav-btn.next {
  right: 16px;
}

.image-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
}

.image-wrapper img {
  display: block;
  max-width: 90vw;
  max-height: calc(100vh - 140px);
  width: auto;
  height: auto;
  object-fit: contain;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}

.preview-hint {
  position: absolute;
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(30, 30, 30, 0.9);
  padding: 6px 12px;
  border-radius: 4px;
  color: #888;
  font-size: 12px;
}

.thumbnail-strip {
  background: rgba(30, 30, 30, 0.95);
  padding: 8px 16px;
  display: flex;
  align-items: center;
  gap: 8px;
  overflow-x: auto;
}

.thumbnail-item {
  width: 48px;
  height: 48px;
  flex-shrink: 0;
  border-radius: 3px;
  overflow: hidden;
  cursor: pointer;
  opacity: 0.5;
  border: 2px solid transparent;
  transition: all 0.2s;
}

.thumbnail-item:hover {
  opacity: 0.8;
}

.thumbnail-item.current {
  opacity: 1;
  border-color: #007acc;
}

.thumbnail-item img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.toolbar-checkbox {
  display: flex;
  align-items: center;
  gap: 6px;
  color: #ccc;
  font-size: 13px;
  cursor: pointer;
  user-select: none;
}

.toolbar-checkbox input[type="checkbox"] {
  cursor: pointer;
}
</style>
