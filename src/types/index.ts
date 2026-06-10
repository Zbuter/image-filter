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
  modified: number
  width: number | null
  height: number | null
  /** Paired RAW file path when a same-name RAW exists */
  rawPath?: string
}

export interface ExportProgress {
  total: number
  completed: number
  failed: string[]
}

export interface AiAnalysisResult {
  path: string
  labels: string[]
  scores: number[]
  is_waste: boolean
}

export interface DuplicateEntry {
  path: string
  score: number
  similarity: number
}

export interface DuplicateGroup {
  best_path: string
  best_score: number
  duplicates: DuplicateEntry[]
}
