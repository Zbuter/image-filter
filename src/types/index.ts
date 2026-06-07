export interface DriveInfo {
  name: string
  path: string
  total_space: number
  available_space: number
}

export interface DirectoryContent {
  path: string
  directories: DirectoryEntry[]
}

export interface DirectoryEntry {
  name: string
  path: string
  is_dir: boolean
}

export interface ImageInfo {
  name: string
  path: string
  extension: string
  size: number
  width: number | null
  height: number | null
}

export interface ExportProgress {
  total: number
  completed: number
  failed: string[]
}
