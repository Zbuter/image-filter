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

// 图片分组
export interface ImageGroup {
  id: string
  name: string
  shortcut: string
  color: string
}

export type ExportFormat = 'both' | 'raw' | 'regular'

// 废片检测结果
export interface WasteResult {
  path: string
  waste_score: number       // 0-1 废片概率
  quality_score: number     // 0-10 质量分数
  reasons: string[]         // 废片原因列表
  confidence: number        // 判断置信度
  is_waste: boolean
  features: WeddingFeatures
}

// 图像质量特征 (20维)
export interface QualityFeatures {
  sharpness: number
  brightness: number
  contrast: number
  saturation: number
  noise_level: number
  edge_density: number
  color_richness: number
  local_contrast: number
  blur_score: number
  overexposed_ratio: number
  underexposed_ratio: number
  texture_complexity: number
  color_balance: number
  gradient_strength: number
  dynamic_range: number
  clarity: number
  detail_level: number
  smoothness: number
  sharpness_uniformity: number
  exposure_quality: number
}

// 人脸特征 (17维)
export interface FaceFeatures {
  has_face: boolean        // 是否检测到人脸
  face_count: number
  face_size_ratio: number
  face_position: number
  face_quality: number
  eye_openness: number
  face_blur: number
  mouth_openness: number
  eye_closed_ratio: number
  mouth_smile: number
  head_tilt: number
  expression_quality: number
  is_funny: boolean
  is_smiling: boolean
  is_frowning: boolean
  is_surprised: boolean
  intentional_closed_eyes: boolean  // 是否故意闭眼
}

// 婚礼场景特征（完整特征向量）
export interface WeddingFeatures {
  quality_features: QualityFeatures
  face_features: FaceFeatures
}

// 废片检测配置
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
}

// 废片原因枚举
export type WasteReason =
  | '过曝'
  | '皮肤过曝'
  | '欠曝'
  | '人脸模糊'
  | '整体模糊'
  | '表情包/怪表情'
  | '闭眼'
  | '嘴巴过度张开'
  | '主体过小'
  | '主体过大'
  | '噪点过多'
  | '重复图'
  | '构图差'
  | '无精修价值'
  | '多人脸'
