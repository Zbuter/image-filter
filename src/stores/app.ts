import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ImageInfo, DuplicateGroup } from '../types'
import { invoke } from '@tauri-apps/api/core'

const RAW_EXTENSIONS = ['cr2', 'cr3', 'nef', 'arw', 'dng', 'orf', 'rw2', 'pef', 'srw', 'raf']

export interface WasteResult {
  path: string
  waste_score: number
  quality_score: number
  reasons: string[]
  confidence: number
  is_waste: boolean
  features: any
}

export interface WasteConfig {
  overexposed_threshold: number
  skin_overexposed_threshold: number
  underexposed_threshold: number
  face_blur_threshold: number
  overall_blur_threshold: number
  mouth_open_threshold: number
  eye_closed_threshold: number
  min_face_ratio: number
  max_face_ratio: number
  noise_threshold: number
  duplicate_similarity: number
}

export const useAppStore = defineStore('app', () => {
  const directories = ref<string[]>([])
  const currentDirectory = ref<string>('')
  const images = ref<ImageInfo[]>([])
  const selectedImageMap = ref<Map<string, ImageInfo>>(new Map())
  const previewImage = ref<ImageInfo | null>(null)
  const loading = ref(false)
  const rawPreviewCache = ref<Map<string, string>>(new Map())
  const scanGeneration = ref(0)

  // 废片检测状态
  const wasteResults = ref<WasteResult[]>([])
  const wasteAnalyzing = ref(false)
  const wasteProgress = ref(0)
  const wasteTotal = ref(0)
  const wasteFeedbackCount = ref(0)
  const wasteConfig = ref<WasteConfig | null>(null)
  const scrollTarget = ref<string | null>(null)

  const selectedImages = computed(() => new Set(selectedImageMap.value.keys()))
  const selectedCount = computed(() => selectedImageMap.value.size)
  const totalImages = computed(() => images.value.length)
  const allSelectedImages = computed(() => Array.from(selectedImageMap.value.values()))
  const wasteImages = computed(() => wasteResults.value.filter(r => r.is_waste))

  async function loadDirectory(path: string) {
    directories.value = [path]
    currentDirectory.value = path
    await rescanImages()
  }

  async function addDirectory(path: string) {
    if (!directories.value.includes(path)) {
      directories.value.push(path)
    }
    await rescanImages()
  }

  async function removeDirectory(path: string) {
    directories.value = directories.value.filter(d => d !== path)
    if (directories.value.length > 0) {
      currentDirectory.value = directories.value[directories.value.length - 1]
    } else {
      currentDirectory.value = ''
    }
    await rescanImages()
  }

  async function rescanImages() {
    scanGeneration.value++
    const gen = scanGeneration.value
    try {
      loading.value = true
      const dirs = directories.value.length > 0 ? directories.value : []
      if (dirs.length === 0) {
        images.value = []
        return
      }
      const imgs = await invoke<any[]>('scan_images', { directories: dirs })
      if (gen !== scanGeneration.value) return
      images.value = imgs
    } catch (e) {
      console.error('Failed to scan images:', e)
    } finally {
      if (gen === scanGeneration.value) {
        loading.value = false
      }
    }
  }

  async function getRawPreview(path: string): Promise<string> {
    if (rawPreviewCache.value.has(path)) {
      return rawPreviewCache.value.get(path)!
    }
    const gen = scanGeneration.value
    try {
      const dataUrl = await invoke<string>('get_raw_preview', { path })
      if (gen !== scanGeneration.value) return ''
      rawPreviewCache.value.set(path, dataUrl)
      return dataUrl
    } catch (e) {
      console.error('Failed to get RAW preview:', e)
      return ''
    }
  }

  function isCurrentGeneration(gen: number): boolean {
    return gen === scanGeneration.value
  }

  function toggleImageSelection(image: ImageInfo) {
    const newMap = new Map(selectedImageMap.value)
    if (newMap.has(image.path)) {
      newMap.delete(image.path)
    } else {
      newMap.set(image.path, image)
    }
    selectedImageMap.value = newMap
  }

  function isImageSelected(path: string): boolean {
    return selectedImageMap.value.has(path)
  }

  function hideImage(path: string) {
    const idx = images.value.findIndex(i => i.path === path)
    if (idx !== -1) {
      images.value.splice(idx, 1)
    }
  }

  function isImageHidden(path: string): boolean {
    return false
  }

  function selectAll() {
    const newMap = new Map(selectedImageMap.value)
    for (const img of images.value) {
      newMap.set(img.path, img)
    }
    selectedImageMap.value = newMap
  }

  function clearSelection() {
    selectedImageMap.value = new Map()
  }

  function invertSelection() {
    const newMap = new Map<string, ImageInfo>()
    for (const img of images.value) {
      if (!selectedImageMap.value.has(img.path)) {
        newMap.set(img.path, img)
      }
    }
    selectedImageMap.value = newMap
  }

  function setPreviewImage(image: ImageInfo | null) {
    previewImage.value = image
  }

  function getExportPath(image: ImageInfo): string {
    return image.rawPath || image.path
  }

  async function exportImages(targetDir: string) {
    try {
      const sources: string[] = []
      for (const img of selectedImageMap.value.values()) {
        sources.push(img.path)
        if (img.rawPath) {
          sources.push(img.rawPath)
        }
      }
      const progress = await invoke<any>('export_images', { sources, targetDir: targetDir })
      return progress
    } catch (e) {
      console.error('Failed to export images:', e)
      throw e
    }
  }

  // ── 废片检测 API ──

  async function autoLoadWasteModel() {
    try {
      await invoke('init_waste_detector')
    } catch (e) {
      console.log('[Waste] No classifier found')
    }
    await hydrateWasteFeedback()
  }

  async function batchAnalyzeWaste(paths: string[]) {
    wasteAnalyzing.value = true
    wasteProgress.value = 0
    wasteTotal.value = paths.length
    wasteResults.value = []

    // 监听进度事件
    const { listen } = await import('@tauri-apps/api/event')
    const unlistenProgress = await listen<{current: number, total: number}>('waste-progress', (event) => {
      wasteProgress.value = event.payload.current
    })

    // 监听单张结果（实时显示）
    const unlistenResult = await listen<WasteResult>('waste-result', (event) => {
      wasteResults.value = [...wasteResults.value, event.payload]
    })

    try {
      await invoke('analyze_waste_images', { paths })
      wasteProgress.value = paths.length
    } catch (e) {
      console.error('[Waste] Analysis failed:', e)
      throw e
    } finally {
      unlistenProgress()
      unlistenResult()
      wasteAnalyzing.value = false
    }
  }

  async function runWasteAnalysis() {
    const paths = images.value.map(img => img.path)
    if (paths.length === 0) return
    await batchAnalyzeWaste(paths)
  }

  function getWasteImages() {
    return wasteResults.value.filter(r => r.is_waste)
  }

  function selectWasteImages() {
    const wastePaths = new Set(getWasteImages().map(r => r.path))
    const newMap = new Map(selectedImageMap.value)
    for (const img of images.value) {
      if (wastePaths.has(img.path)) {
        newMap.set(img.path, img)
      }
    }
    selectedImageMap.value = newMap
  }

  function excludeWasteImages() {
    const wastePaths = new Set(getWasteImages().map(r => r.path))
    const newMap = new Map(selectedImageMap.value)
    for (const path of wastePaths) {
      newMap.delete(path)
    }
    selectedImageMap.value = newMap
  }

  function resetWasteResults() {
    wasteResults.value = []
    wasteProgress.value = 0
    wasteTotal.value = 0
  }

  async function markWasteFeedback(path: string, isWaste: boolean, reasons: string[]) {
    try {
      const count = await invoke<number>('mark_waste_feedback', { path, isWaste, reasons })
      wasteFeedbackCount.value = count
      // 更新本地标记状态
      markedStatus.value.set(path, isWaste)
      markedStatus.value = new Map(markedStatus.value)
      return count
    } catch (e) {
      console.error('[Waste] Failed to mark feedback:', e)
      throw e
    }
  }

  async function hydrateWasteFeedback() {
    try {
      const count = await invoke<number>('get_waste_feedback_count')
      wasteFeedbackCount.value = count
    } catch (e) {
      console.error('[Waste] Failed to hydrate feedback:', e)
    }
  }

  async function loadWasteConfig() {
    try {
      const config = await invoke<WasteConfig>('get_waste_config')
      wasteConfig.value = config
    } catch (e) {
      console.error('[Waste] Failed to load config:', e)
    }
  }

  async function updateWasteConfig(config: WasteConfig) {
    try {
      await invoke('update_waste_config', { config })
      wasteConfig.value = config
    } catch (e) {
      console.error('[Waste] Failed to update config:', e)
      throw e
    }
  }

  function setScrollTarget(path: string | null) {
    scrollTarget.value = path
    // 3秒后自动清除高亮
    if (path) {
      setTimeout(() => {
        if (scrollTarget.value === path) {
          scrollTarget.value = null
        }
      }, 3000)
    }
  }

  // ── 标记状态查询 ──
  const markedStatus = ref<Map<string, boolean>>(new Map())

  function isMarkedWaste(path: string): boolean {
    return markedStatus.value.get(path) === true
  }

  function isMarkedNotWaste(path: string): boolean {
    return markedStatus.value.get(path) === false
  }

  function isMarked(path: string): boolean {
    return markedStatus.value.has(path)
  }

  // Toast
  const toastMessage = ref('')
  const toastType = ref<'success' | 'error'>('success')
  let toastTimer: ReturnType<typeof setTimeout> | null = null

  function showToast(message: string, type: 'success' | 'error' = 'success') {
    toastMessage.value = message
    toastType.value = type
    if (toastTimer) clearTimeout(toastTimer)
    toastTimer = setTimeout(() => {
      toastMessage.value = ''
    }, 3000)
  }

  return {
    directories,
    currentDirectory,
    images,
    selectedImageMap,
    selectedImages,
    previewImage,
    loading,
    selectedCount,
    totalImages,
    allSelectedImages,
    scanGeneration,
    rawPreviewCache,
    loadDirectory,
    addDirectory,
    removeDirectory,
    rescanImages,
    getRawPreview,
    isCurrentGeneration,
    toggleImageSelection,
    isImageSelected,
    hideImage,
    selectAll,
    clearSelection,
    invertSelection,
    setPreviewImage,
    getExportPath,
    exportImages,
    // 废片检测
    wasteResults,
    wasteAnalyzing,
    wasteProgress,
    wasteTotal,
    wasteFeedbackCount,
    wasteConfig,
    wasteImages,
    autoLoadWasteModel,
    batchAnalyzeWaste,
    runWasteAnalysis,
    getWasteImages,
    selectWasteImages,
    excludeWasteImages,
    resetWasteResults,
    markWasteFeedback,
    hydrateWasteFeedback,
    isMarkedWaste,
    isMarkedNotWaste,
    isMarked,
    loadWasteConfig,
    updateWasteConfig,
    scrollTarget,
    setScrollTarget,
    // Toast
    toastMessage,
    toastType,
    showToast,
  }
})
