<template>
  <div v-if="store.previewImage" class="preview-overlay" @click="handleOverlayClick">
    <!-- Top Toolbar -->
    <div class="preview-toolbar">
      <div class="toolbar-group">
        <button class="tool-btn" @click="closePreview" title="关闭 (Esc)">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>

      <div class="toolbar-divider"></div>

      <div class="toolbar-group">
        <button class="tool-btn" @click="zoomOut" title="缩小">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/><line x1="8" y1="11" x2="14" y2="11"/></svg>
        </button>
        <span class="zoom-label">{{ Math.round(zoom * 100) }}%</span>
        <button class="tool-btn" @click="zoomIn" title="放大">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/><line x1="11" y1="8" x2="11" y2="14"/><line x1="8" y1="11" x2="14" y2="11"/></svg>
        </button>
        <button class="tool-btn" @click="resetZoom" title="适应窗口">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 3h6v6"/><path d="M9 21H3v-6"/><path d="M21 3l-7 7"/><path d="M3 21l7-7"/></svg>
        </button>
      </div>

      <div class="toolbar-divider"></div>

      <div class="toolbar-group">
        <button class="tool-btn" @click="rotateLeft" title="向左旋转 (R)">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="1 4 1 10 7 10"/><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"/></svg>
        </button>
        <button class="tool-btn" @click="rotateRight" title="向右旋转 (Shift+R)">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
        </button>
      </div>

      <div class="toolbar-group feedback-group">
        <button 
          class="tool-btn waste-btn"
          :class="{ active: store.previewImage && store.isMarkedWaste(store.previewImage.path) }"
          @click="markFeedback(true)"
          title="标记为废片 (D)"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
        <button 
          class="tool-btn good-btn"
          :class="{ active: store.previewImage && store.isMarkedNotWaste(store.previewImage.path) }"
          @click="markFeedback(false)"
          title="标记为非废片 (F)"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
        </button>
      </div>

      <div class="toolbar-divider"></div>

      <div class="toolbar-spacer"></div>

      <div class="file-meta">
        <span class="file-name">{{ store.previewImage.name }}</span>
        <span v-if="store.previewImage.width && store.previewImage.height" class="file-dims">
          {{ store.previewImage.width }} × {{ store.previewImage.height }}
        </span>
        <span class="file-position">{{ currentIndex + 1 }} / {{ previewList.length }}</span>
      </div>

      <div class="toolbar-divider"></div>

      <div class="toolbar-group">
        <button 
          class="tool-btn select-toggle"
          :class="{ selected: isSelected }"
          @click="toggleSelection"
          title="选中/取消选中"
        >
          <svg v-if="isSelected" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/></svg>
        </button>
      </div>

      <template v-if="!inSelectedView">
        <label class="auto-switch-label" title="浏览到末尾时自动跳转到已选视图">
          <input type="checkbox" v-model="switchToSelectedAtEnd" />
          <span class="auto-switch-text">末尾跳转已选</span>
        </label>
      </template>
    </div>

    <!-- Image Content -->
    <div class="preview-content" @wheel="handleWheel" @mousedown="handleMouseDown">
      <button class="nav-btn nav-prev" @click.stop="prevImage">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"/></svg>
      </button>

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

      <button class="nav-btn nav-next" @click.stop="nextImage">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"/></svg>
      </button>
    </div>

    <!-- Thumbnail Strip -->
    <div ref="thumbnailStripRef" class="thumbnail-strip">
      <div
        v-for="(image, index) in previewList"
        :key="image.path"
        class="thumb"
        :class="{ current: index === currentIndex }"
        @click.stop="goToImage(index)"
      >
        <img :src="getThumbnailUrl(image)" :alt="image.name" />
      </div>
    </div>
      <!-- Feedback Toast -->
    <Transition name="toast-fade">
      <div v-if="feedbackToast" class="feedback-toast">{{ feedbackToast }}</div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted, reactive } from 'vue'
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
const thumbnailStripRef = ref<HTMLElement | null>(null)

const RAW_EXTENSIONS = ['cr2', 'cr3', 'nef', 'arw', 'dng', 'orf', 'rw2', 'pef', 'srw', 'raf']
const placeholderUrl = 'data:image/svg+xml,%3Csvg xmlns="http://www.w3.org/2000/svg" width="100" height="100"%3E%3Crect fill="%23212125" width="100" height="100"/%3E%3C/svg%3E'

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
  transition: isDragging.value ? 'none' : 'transform 0.2s ease',
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

watch(() => store.previewImage, async (newImg) => {
  if (newImg && isRawFile(newImg.path) && !rawPreviewUrls[newImg.path]) {
    try {
      const url = await store.getRawPreview(newImg.path)
      if (url) rawPreviewUrls[newImg.path] = url
    } catch (e) {
      console.error('Failed to load RAW preview:', e)
    }
  }
  resetView()
})

// Scroll thumbnail strip to keep current image visible
watch(currentIndex, async () => {
  await nextTick()
  if (thumbnailStripRef.value) {
    const current = thumbnailStripRef.value.querySelector('.thumb.current') as HTMLElement
    if (current) {
      current.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'center' })
    }
  }
})

function resetView() {
  zoom.value = 1
  panX.value = 0
  panY.value = 0
  rotation.value = 0
  imageLoaded.value = false
}

function handleImageLoad() {
  imageLoaded.value = true
}

function closePreview() {
  store.setPreviewImage(null)
}

function handleOverlayClick(e: MouseEvent) {
  if (e.target === e.currentTarget) closePreview()
}

function zoomIn() { zoom.value = Math.min(zoom.value * 1.3, 10) }
function zoomOut() { zoom.value = Math.max(zoom.value / 1.3, 0.1) }
function resetZoom() { zoom.value = 1; panX.value = 0; panY.value = 0 }
function toggleActualSize() { zoom.value = zoom.value === 1 ? 2 : 1 }
function rotateLeft() { rotation.value -= 90 }
function rotateRight() { rotation.value += 90 }

function prevImage() {
  if (currentIndex.value > 0) {
    store.setPreviewImage(currentPreviewList.value[currentIndex.value - 1])
  } else if (switchToSelectedAtEnd.value && !props.inSelectedView) {
    emit('switch-to-selected')
  }
}

function nextImage() {
  if (currentIndex.value < currentPreviewList.value.length - 1) {
    store.setPreviewImage(currentPreviewList.value[currentIndex.value + 1])
  } else if (switchToSelectedAtEnd.value && !props.inSelectedView) {
    emit('switch-to-selected')
  }
}

function goToImage(index: number) {
  store.setPreviewImage(currentPreviewList.value[index])
}

function toggleSelection() {
  if (store.previewImage) store.toggleImageSelection(store.previewImage)
}

function handleWheel(e: WheelEvent) {
  e.preventDefault()
  if (e.ctrlKey || e.metaKey) {
    // Ctrl+scroll: zoom
    const delta = e.deltaY > 0 ? 0.9 : 1.1
    zoom.value = Math.max(0.1, Math.min(10, zoom.value * delta))
  } else if (Math.abs(e.deltaX) > Math.abs(e.deltaY)) {
    // Horizontal scroll: navigate
    if (e.deltaX > 0) nextImage()
    else prevImage()
  } else {
    // Vertical scroll: zoom
    const delta = e.deltaY > 0 ? 0.95 : 1.05
    zoom.value = Math.max(0.1, Math.min(10, zoom.value * delta))
  }
}

function handleMouseDown(e: MouseEvent) {
  if (e.button !== 0) return
  isDragging.value = true
  dragStartX.value = e.clientX - panX.value
  dragStartY.value = e.clientY - panY.value
}

function handleMouseMove(e: MouseEvent) {
  if (!isDragging.value) return
  panX.value = e.clientX - dragStartX.value
  panY.value = e.clientY - dragStartY.value
}

function handleMouseUp() {
  isDragging.value = false
}

function isInputFocused(): boolean {
  const el = document.activeElement
  if (!el) return false
  const tag = el.tagName.toLowerCase()
  return tag === 'input' || tag === 'textarea' || tag === 'select' || (el as HTMLElement).isContentEditable
}

function handleKeydown(e: KeyboardEvent) {
  if (!store.previewImage) return
  if (isInputFocused()) return
  switch (e.key) {
    case 'Escape': closePreview(); break
    case 'ArrowLeft': prevImage(); break
    case 'ArrowRight': nextImage(); break
    case ' ':
      e.preventDefault()
      toggleSelectionAndAdvance()
      break
    case 'd': case 'D':
        e.preventDefault()
        markFeedback(true)
        break
      case 'f': case 'F':
        e.preventDefault()
        markFeedback(false)
        break
      case 'r': case 'R':
      if (e.shiftKey) rotateRight()
      else rotateLeft()
      break
  }
}

function toggleSelectionAndAdvance() {
  // Use the computed ref which reflects current state before toggle
  const wasSelected = isSelected.value
  toggleSelection()
  if (!wasSelected) {
    nextImage()
  }
}


  // AI feedback toast
  const feedbackToast = ref('')
  let toastTimer: ReturnType<typeof setTimeout> | null = null

  async function markFeedback(isWaste: boolean) {
    if (!store.previewImage) return
    try {
      const count = await store.markImageFeedback(store.previewImage.path, isWaste)
      feedbackToast.value = isWaste ? `已标记废片 (共 ${count} 条)` : `已标记非废片 (共 ${count} 条)`
      if (toastTimer) clearTimeout(toastTimer)
      toastTimer = setTimeout(() => { feedbackToast.value = '' }, 1500)
    } catch (e) {
      console.error('Failed to mark feedback:', e)
    }
  }

  onMounted(() => {
  window.addEventListener('mousemove', handleMouseMove)
  window.addEventListener('mouseup', handleMouseUp)
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('mousemove', handleMouseMove)
  window.removeEventListener('mouseup', handleMouseUp)
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.preview-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  flex-direction: column;
  background: rgba(10, 10, 12, 0.95);
  backdrop-filter: blur(8px);
}

/* ── Toolbar ─────────────────────────────────────── */
.preview-toolbar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 12px;
  height: 40px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: 2px;
}

.toolbar-divider {
  width: 1px;
  height: 18px;
  background: var(--border-subtle);
  margin: 0 4px;
  flex-shrink: 0;
}

.toolbar-spacer {
  flex: 1;
}

.tool-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  background: transparent;
  border: none;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.tool-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.select-toggle.selected {
  color: var(--accent);
}

.select-toggle.selected:hover {
  background: var(--accent-muted);
}

.zoom-label {
  font-size: 11px;
  color: var(--text-secondary);
  min-width: 36px;
  text-align: center;
  font-variant-numeric: tabular-nums;
}

.file-meta {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 12px;
}

.file-name {
  color: var(--text-primary);
  max-width: 240px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-dims {
  color: var(--text-tertiary);
  font-variant-numeric: tabular-nums;
}

.file-position {
  color: var(--text-tertiary);
  font-variant-numeric: tabular-nums;
}

.auto-switch-label {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: 4px;
  cursor: pointer;
}

.auto-switch-label input {
  accent-color: var(--accent);
  width: 12px;
  height: 12px;
}

.auto-switch-text {
  font-size: 11px;
  color: var(--text-tertiary);
  white-space: nowrap;
}

/* ── Image Content ───────────────────────────────── */
.preview-content {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  user-select: none;
}

.image-wrapper {
  max-width: 90%;
  max-height: 90%;
}

.image-wrapper img {
  max-width: 100%;
  max-height: calc(100vh - 140px);
  object-fit: contain;
  border-radius: var(--radius-sm);
  box-shadow: var(--shadow-lg);
}

.nav-btn {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 50%;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: all var(--transition-fast);
  backdrop-filter: blur(4px);
  z-index: 10;
}

.nav-btn:hover {
  background: rgba(0, 0, 0, 0.6);
  color: #fff;
  border-color: rgba(255, 255, 255, 0.15);
}

.nav-prev { left: 16px; }
.nav-next { right: 16px; }

/* ── Thumbnail Strip ─────────────────────────────── */
.thumbnail-strip {
  display: flex;
  gap: 4px;
  padding: 8px 12px;
  background: var(--bg-surface);
  border-top: 1px solid var(--border-subtle);
  overflow-x: auto;
  flex-shrink: 0;
}

.thumb {
  width: 48px;
  height: 48px;
  flex-shrink: 0;
  border-radius: var(--radius-sm);
  overflow: hidden;
  cursor: pointer;
  border: 2px solid transparent;
  opacity: 0.5;
  transition: all var(--transition-fast);
}

.thumb:hover {
  opacity: 0.8;
}

.thumb.current {
  border-color: var(--accent);
  opacity: 1;
}

.thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

/* Feedback Toast */
.feedback-toast {
  position: fixed;
  bottom: 80px;
  left: 50%;
  transform: translateX(-50%);
  padding: 8px 20px;
  background: var(--accent);
  color: var(--bg-base);
  border-radius: var(--radius-md);
  font-size: 13px;
  font-weight: 600;
  z-index: 1001;
  pointer-events: none;
  box-shadow: var(--shadow-md);
}

.toast-fade-enter-active,
.toast-fade-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.toast-fade-enter-from,
.toast-fade-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(10px);
}


/* Feedback Toggle Buttons */
.feedback-group {
  display: flex;
  gap: 2px;
  background: var(--bg-base);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: 1px;
}

.waste-btn,
.good-btn {
  width: 28px;
  height: 24px;
  border-radius: 3px;
}

.waste-btn {
  color: var(--text-tertiary);
}

.waste-btn:hover {
  color: var(--danger);
  background: rgba(212, 83, 83, 0.1);
}

.waste-btn.active {
  color: #fff;
  background: var(--danger);
  border-color: var(--danger);
}

.good-btn {
  color: var(--text-tertiary);
}

.good-btn:hover {
  color: var(--success);
  background: rgba(92, 184, 122, 0.1);
}

.good-btn.active {
  color: #fff;
  background: var(--success);
  border-color: var(--success);
}

</style>
