import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ImageInfo } from '../types'
import { invoke } from '@tauri-apps/api/core'

const IMAGE_EXTENSIONS = [
  'jpg', 'jpeg', 'png', 'gif', 'webp', 'tiff', 'tif', 'bmp',
  'cr2', 'cr3', 'nef', 'arw', 'dng', 'orf', 'rw2', 'pef', 'srw', 'raf',
]
const RAW_EXTENSIONS = ['cr2', 'cr3', 'nef', 'arw', 'dng', 'orf', 'rw2', 'pef', 'srw', 'raf']

export const useAppStore = defineStore('app', () => {
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
    scanGeneration.value++
    const gen = scanGeneration.value
    try {
      loading.value = true
      currentDirectory.value = path
      const imgs = await invoke<any[]>('scan_images', { directory: path })
      if (gen !== scanGeneration.value) return
      images.value = imgs
    } catch (e) {
      console.error('Failed to load directory:', e)
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
      // If the deselected image is being previewed, jump to next/prev
      if (previewImage.value?.path === image.path) {
        const remaining = Array.from(newMap.values())
        if (remaining.length === 0) {
          previewImage.value = null
        } else {
          const idx = allSelectedImages.value.findIndex(img => img.path === image.path)
          const nextIdx = idx >= remaining.length ? remaining.length - 1 : idx
          previewImage.value = remaining[nextIdx]
        }
      }
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

  function setPreviewImage(image: ImageInfo | null) {
    previewImage.value = image
  }

  async function exportImages(targetDir: string) {
    try {
      const sources = Array.from(selectedImageMap.value.keys())
      const progress = await invoke<any>('export_images', { sources, targetDir })
      return progress
    } catch (e) {
      console.error('Failed to export images:', e)
      throw e
    }
  }

  return {
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
    getRawPreview,
    isCurrentGeneration,
    toggleImageSelection,
    isImageSelected,
    selectAll,
    clearSelection,
    setPreviewImage,
    exportImages
  }
})
