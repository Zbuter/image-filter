import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ImageInfo } from '../types'
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
    exportImages
  }
})
