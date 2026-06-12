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
    <RecycleScroller
      v-else
      ref="scrollerRef"
      class="scroller"
      :items="imageRows"
      :item-size="220"
      key-field="id"
    >
      <template #default="{ item: row }">
        <div class="grid-row">
          <div 
            v-for="image in row.images" 
            :key="image.path"
            class="image-card"
            :class="{ selected: store.isImageSelected(image.path), 'waste-highlight': props.highlightedWastePath === image.path || store.scrollTarget === image.path, 'marked-waste': store.isMarkedWaste(image.path), 'marked-good': store.isMarkedNotWaste(image.path) }"
            @click="handleClick(image, $event)"
            @dblclick="handleDoubleClick(image)"
            @contextmenu.prevent="showContextMenu($event, image)"
          >
            <div class="card-image">
              <img :src="getImageUrl(image.path)" :alt="image.name" loading="lazy" />
              <span v-if="store.isMarkedWaste(image.path)" class="waste-badge">
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
              </span>
              <span v-else-if="store.isMarkedNotWaste(image.path)" class="good-badge">
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
              </span>
              <div v-if="store.isMarkedWaste(image.path)" class="waste-overlay"></div>
              <button class="select-check" @click.stop="toggleSelection(image)">
                <svg v-if="store.isImageSelected(image.path)" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
              </button>
            </div>
            <div class="card-info">
              <span v-if="image.rawPath" class="raw-tag">RAW</span>
              <span class="card-name" :title="image.name">{{ image.name }}</span>
              <span class="card-dir" v-if="showDirHint" :title="getDirName(image.path)">{{ getDirName(image.path) }}</span>
            </div>
          </div>
        </div>
      </template>
    </RecycleScroller>

    <!-- Context Menu -->
    <Teleport to="body">
      <div 
        v-if="contextMenu.visible" 
        class="context-menu"
        :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
        @click.stop
      >
        <button class="ctx-item" @click="markFromCtx(true)">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
          标记为废片
        </button>
        <button class="ctx-item" @click="markFromCtx(false)">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
          标记为非废片
        </button>
        <div class="ctx-divider"></div>
        <button class="ctx-item ctx-item-danger" @click="deleteFromCtx()">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
          删除
        </button>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, watch, ref, onMounted, onUnmounted } from 'vue'
import { RecycleScroller } from 'vue-virtual-scroller'
import { useAppStore } from '../stores/app'
import type { ImageInfo } from '../types'
import { convertFileSrc } from '@tauri-apps/api/core'

const props = defineProps<{
  highlightedWastePath?: string | null

  filterText: string
  fileTypeFilter: 'all' | 'raw' | 'regular' | 'custom'
  customExtensions: string
}>()

const store = useAppStore()
const scrollerRef = ref<any>(null)

// 滚动到指定路径的图片
function scrollToPath(path: string) {
  const idx = filteredImages.value.findIndex(img => img.path === path)
  if (idx === -1) return
  const rowIndex = Math.floor(idx / COLS)
  if (scrollerRef.value) {
    scrollerRef.value.scrollToItem(rowIndex)
  }
  // 设置高亮
  store.setScrollTarget(path)
}

defineExpose({ scrollToPath })

const COLS = 5
const imageRows = computed(() => {
  const imgs = filteredImages.value
  const rows = []
  for (let i = 0; i < imgs.length; i += COLS) {
    rows.push({ id: i, images: imgs.slice(i, i + COLS) })
  }
  return rows
})

const contextMenu = ref({ visible: false, x: 0, y: 0, image: null as ImageInfo | null })

function showContextMenu(e: MouseEvent, image: ImageInfo) {
  contextMenu.value = { visible: true, x: e.clientX, y: e.clientY, image }
}

function hideContextMenu() {
  contextMenu.value.visible = false
}

async function markFromCtx(isWaste: boolean) {
  const img = contextMenu.value.image
  hideContextMenu()
  if (!img) return
  try {
    await store.markWasteFeedback(img.path, isWaste, [])
  } catch (e) {
    console.error('Failed to mark feedback:', e)
  }
}

async function deleteFromCtx() {
  const img = contextMenu.value.image
  hideContextMenu()
  if (!img) return
  // 从 images 列表中移除（仅前端）
  const idx = store.images.findIndex(i => i.path === img.path)
  if (idx !== -1) {
    store.images.splice(idx, 1)
  }
}

onMounted(() => {
  document.addEventListener('click', hideContextMenu)
})
onUnmounted(() => {
  document.removeEventListener('click', hideContextMenu)
})


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

.raw-tag {
  display: inline-block;
  padding: 0 4px;
  background: var(--accent);
  color: var(--bg-base);
  font-size: 9px;
  font-weight: 700;
  border-radius: 2px;
  letter-spacing: 0.5px;
  line-height: 1.6;
  margin-bottom: 2px;
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
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
  padding: 4px 8px 6px;
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

/* Context Menu */
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

.ctx-item-danger {
  color: var(--danger);
}

.ctx-item-danger:hover {
  background: rgba(212, 83, 83, 0.15);
}

.ctx-item-danger svg {
  color: var(--danger);
}

.ctx-divider {
  height: 1px;
  background: var(--border-subtle);
  margin: 4px 0;
}

.ctx-item svg {
  flex-shrink: 0;
  color: var(--text-secondary);
}


/* Waste Badge */
.waste-badge {
  position: absolute;
  top: 6px;
  left: 6px;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--danger);
  color: #fff;
  border-radius: 50%;
  z-index: 2;
}

/* Good Badge */
.good-badge {
  position: absolute;
  top: 6px;
  left: 6px;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--success);
  color: #fff;
  border-radius: 50%;
  opacity: 0;
  transition: opacity var(--transition-fast);
  z-index: 2;
}

.image-card.marked-good:hover .good-badge {
  opacity: 1;
}

/* Waste Overlay */
.waste-overlay {
  position: absolute;
  inset: 0;
  background: rgba(212, 83, 83, 0.25);
  z-index: 1;
  pointer-events: none;
}

/* Marked waste card style */
.image-card.marked-waste {
  border-color: rgba(212, 83, 83, 0.4);
}

.image-card.marked-waste .card-name {
  color: var(--text-tertiary);
  text-decoration: line-through;
}



/* Waste Highlight from AI Panel hover */
.image-card.waste-highlight {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-border);
}

.image-card.waste-highlight .card-image img {
  transform: scale(1.05);
}


.scroller {
  flex: 1;
  overflow-y: auto;
}

.grid-row {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 8px;
  padding: 0 0 8px 0;
}
</style>
