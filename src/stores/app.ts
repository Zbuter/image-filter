import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ImageInfo, AiAnalysisResult, DuplicateGroup } from '../types'
import { invoke } from '@tauri-apps/api/core'

const RAW_EXTENSIONS = ['cr2', 'cr3', 'nef', 'arw', 'dng', 'orf', 'rw2', 'pef', 'srw', 'raf']

export const useAppStore = defineStore('app', () => {
  const directories = ref<string[]>([])
  const currentDirectory = ref<string>('')
  const images = ref<ImageInfo[]>([])
  const selectedImageMap = ref<Map<string, ImageInfo>>(new Map())
  const previewImage = ref<ImageInfo | null>(null)
  const loading = ref(false)
  const rawPreviewCache = ref<Map<string, string>>(new Map())
  const scanGeneration = ref(0)
  const aiResults = ref<AiAnalysisResult[]>([])
  const aiAnalyzing = ref(false)
  const aiProgress = ref(0)
  const aiTotal = ref(0)
  const aiModelLoaded = ref(false)
  const feedbackCount = ref(0)
  const feedbackMap = ref<Map<string, boolean>>(new Map())

  const selectedImages = computed(() => new Set(selectedImageMap.value.keys()))
  const selectedCount = computed(() => selectedImageMap.value.size)
  const totalImages = computed(() => images.value.length)
  const allSelectedImages = computed(() => Array.from(selectedImageMap.value.values()))

  async function loadDirectory(path: string) {
    // Single directory mode: replace directories list
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

  /** Get the effective export path: rawPath if available, otherwise the image path */
  function getExportPath(image: ImageInfo): string {
    return image.rawPath || image.path
  }

  async function exportImages(targetDir: string) {
    try {
      // Export both JPG and RAW when paired
      const sources: string[] = []
      for (const img of selectedImageMap.value.values()) {
        sources.push(img.path) // Always include the primary file (JPG)
        if (img.rawPath) {
          sources.push(img.rawPath) // Also include paired RAW
        }
      }
      const progress = await invoke<any>('export_images', { sources, targetDir: targetDir })
      return progress
    } catch (e) {
      console.error('Failed to export images:', e)
      throw e
    }
  }


  async function initAiModel(modelDir: string) {
    console.log('[AI] initAiModel called with:', modelDir);
    try {
      await invoke('init_ai_model', { modelDir });
      console.log('[AI] init_ai_model succeeded');
      aiModelLoaded.value = true;
      console.log('[AI] aiModelLoaded set to true')
    } catch (e) {
      console.error('[AI] Failed to init AI model:', e)
      throw e
    }
  }

  async function startAiAnalysis(paths: string[]) {
    if (!aiModelLoaded.value) {
      throw new Error('AI model not loaded')
    }
    aiAnalyzing.value = true
    aiProgress.value = 0
    aiTotal.value = paths.length
    aiResults.value = []

    const batchSize = 7
    for (let i = 0; i < paths.length; i += batchSize) {
      const batch = paths.slice(i, i + batchSize)
      try {
        const results = await invoke('analyze_images', { paths: batch })
        aiResults.value.push(...results)
        aiProgress.value = Math.min(i + batchSize, paths.length)
      } catch (e) {
        console.error('AI analysis batch failed:', e)
      }
    }

    aiAnalyzing.value = false

    // Auto-select waste images
    const wastePaths = new Set(aiResults.value.filter(r => r.is_waste).map(r => r.path))
    if (wastePaths.size > 0) {
      const newMap = new Map(selectedImageMap.value)
      for (const img of images.value) {
        if (wastePaths.has(img.path)) {
          newMap.set(img.path, img)
        }
      }
      selectedImageMap.value = newMap
    }
  }

  function getWasteImages() {
    return aiResults.value.filter(r => r.is_waste)
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

  
  async function loadFeedbackCount() {
    try {
      const count = await invoke<number>('get_feedback_count')
      feedbackCount.value = count
    } catch (e) {
      console.error('[AI] Failed to load feedback count:', e)
    }
  }

  async function markImageFeedback(path: string, isWaste: boolean) {
    try {
      const count = await invoke<number>('mark_image_feedback', { path, isWaste })
      feedbackCount.value = count
      // Update local map using path as key (for current session visual feedback)
      feedbackMap.value.set(path, isWaste)
      feedbackMap.value = new Map(feedbackMap.value)
      return count
    } catch (e) {
      console.error('[AI] Failed to mark feedback:', e)
      throw e
    }
  }

  
  function isMarkedWaste(path: string): boolean {
    return feedbackMap.value.get(path) === true
  }

  function isMarkedNotWaste(path: string): boolean {
    return feedbackMap.value.get(path) === false
  }

  function isMarked(path: string): boolean {
    return feedbackMap.value.has(path)
  }

  async function hydrateFeedback() {
    try {
      const count = await invoke<number>('get_feedback_count')
      feedbackCount.value = count
    } catch (e) {
      console.error('[AI] Failed to hydrate feedback:', e)
    }
  }

  
  const duplicateGroups = ref<DuplicateGroup[]>([])
  const dedupDetecting = ref(false)

  async function detectDuplicates(paths: string[]) {
    dedupDetecting.value = true
    try {
      const groups = await invoke<DuplicateGroup[]>('detect_duplicates', { paths })
      duplicateGroups.value = groups
      return groups
    } catch (e) {
      console.error('[AI] Duplicate detection failed:', e)
      throw e
    } finally {
      dedupDetecting.value = false
    }
  }

  async function markDuplicatesAsWaste(duplicatePaths: string[]) {
    try {
      const count = await invoke<number>('mark_duplicates_as_waste', { duplicatePaths })
      feedbackCount.value = count
      return count
    } catch (e) {
      console.error('[AI] Failed to mark duplicates:', e)
      throw e
    }
  }

  function resetAiResults() {
    aiResults.value = []
    aiProgress.value = 0
    aiTotal.value = 0
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
    selectAll,
    clearSelection,
    invertSelection,
    setPreviewImage,
    getExportPath,
    exportImages,
    aiResults,
    aiAnalyzing,
    aiProgress,
    aiTotal,
    aiModelLoaded,
    initAiModel,
    startAiAnalysis,
    getWasteImages,
    selectWasteImages,
    excludeWasteImages,
    resetAiResults,
    feedbackCount,
    loadFeedbackCount,
    markImageFeedback,
    feedbackMap,
    isMarkedWaste,
    isMarkedNotWaste,
    isMarked,
    hydrateFeedback,
    duplicateGroups,
    dedupDetecting,
    detectDuplicates,
    markDuplicatesAsWaste
  }
})
