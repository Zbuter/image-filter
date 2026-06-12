use image::RgbImage;
use serde::{Deserialize, Serialize};
use crate::face_detector::{FaceDetection, detect_faces};
use crate::quality_analyzer;

/// 场景类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SceneType {
    Registration,    // 领证
    Ceremony,        // 婚礼仪式
    Portrait,        // 肖像照
    Group,           // 合照
    Outdoor,         // 外景
    Reception,       // 婚宴
    Unknown,
}

impl SceneType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "registration" => SceneType::Registration,
            "ceremony" => SceneType::Ceremony,
            "portrait" => SceneType::Portrait,
            "group" => SceneType::Group,
            "outdoor" => SceneType::Outdoor,
            "reception" => SceneType::Reception,
            _ => SceneType::Unknown,
        }
    }
}

/// 废片原因
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WasteReason {
    Overexposed,           // 过曝
    SkinOverexposed,       // 皮肤过曝
    Underexposed,          // 欠曝
    FaceBlur,              // 人脸模糊
    OverallBlur,           // 整体模糊
    FunnyExpression,       // 表情包/怪表情
    ClosedEyes,            // 闭眼
    MouthTooOpen,          // 嘴巴过度张开
    SurprisedExpression,   // 惊讶表情
    FrowningExpression,    // 皱眉表情
    HeadTilted,            // 头部过度倾斜
    SubjectTooSmall,       // 主体过小
    SubjectTooLarge,       // 主体过大（大头贴）
    HighNoise,             // 噪点过多
    Duplicate,             // 重复图
    BadComposition,        // 构图差
    LowRetouchValue,       // 无精修价值（截图等）
    MultipleFaces,         // 多人脸（非合照场景）
    MotionBlur,            // 运动模糊
    LowContrast,           // 低对比度
    Screenshot,            // 截图
}

impl WasteReason {
    pub fn label_cn(&self) -> &str {
        match self {
            WasteReason::Overexposed => "过曝",
            WasteReason::SkinOverexposed => "皮肤过曝",
            WasteReason::Underexposed => "欠曝",
            WasteReason::FaceBlur => "人脸模糊",
            WasteReason::OverallBlur => "整体模糊",
            WasteReason::FunnyExpression => "表情包/怪表情",
            WasteReason::ClosedEyes => "闭眼",
            WasteReason::MouthTooOpen => "嘴巴过度张开",
            WasteReason::SurprisedExpression => "惊讶表情",
            WasteReason::FrowningExpression => "皱眉表情",
            WasteReason::HeadTilted => "头部过度倾斜",
            WasteReason::SubjectTooSmall => "主体过小",
            WasteReason::SubjectTooLarge => "主体过大",
            WasteReason::HighNoise => "噪点过多",
            WasteReason::Duplicate => "重复图",
            WasteReason::BadComposition => "构图差",
            WasteReason::LowRetouchValue => "无精修价值",
            WasteReason::MultipleFaces => "多人脸",
            WasteReason::MotionBlur => "运动模糊",
            WasteReason::LowContrast => "低对比度",
            WasteReason::Screenshot => "截图",
        }
    }
}

/// 废片检测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasteConfig {
    pub overexposed_threshold: f32,
    pub skin_overexposed_threshold: f32,
    pub underexposed_threshold: f32,
    pub face_blur_threshold: f32,
    pub overall_blur_threshold: f32,
    pub mouth_open_threshold: f32,
    pub eye_closed_threshold: f32,
    pub min_face_ratio: f32,
    pub max_face_ratio: f32,
    pub noise_threshold: f32,
    pub duplicate_similarity: f32,
}

impl Default for WasteConfig {
    fn default() -> Self {
        Self {
            overexposed_threshold: 0.15,
            skin_overexposed_threshold: 0.3,
            underexposed_threshold: 0.1,
            face_blur_threshold: 0.03,
            overall_blur_threshold: 0.02,
            mouth_open_threshold: 0.4,
            eye_closed_threshold: 0.1,
            min_face_ratio: 0.05,
            max_face_ratio: 0.8,
            noise_threshold: 0.3,
            duplicate_similarity: 0.97,
        }
    }
}

/// 人脸特征分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceFeatures {
    pub has_face: bool,           // 是否检测到人脸
    pub face_count: f32,
    pub face_size_ratio: f32,
    pub face_position: f32,
    pub face_quality: f32,
    pub eye_openness: f32,
    pub face_blur: f32,
    pub mouth_openness: f32,
    pub eye_closed_ratio: f32,
    pub mouth_smile: f32,
    pub head_tilt: f32,
    pub expression_quality: f32,
    pub is_funny: bool,
    pub is_smiling: bool,
    pub is_frowning: bool,
    pub is_surprised: bool,
    pub intentional_closed_eyes: bool,  // 是否故意闭眼
}

impl FaceFeatures {
    pub fn to_vec(&self) -> Vec<f32> {
        vec![
            if self.has_face { 1.0 } else { 0.0 },
            self.face_count,
            self.face_size_ratio,
            self.face_position,
            self.face_quality,
            self.eye_openness,
            self.face_blur,
            self.mouth_openness,
            self.eye_closed_ratio,
            self.mouth_smile,
            self.head_tilt,
            self.expression_quality,
            if self.is_funny { 1.0 } else { 0.0 },
            if self.is_smiling { 1.0 } else { 0.0 },
            if self.is_frowning { 1.0 } else { 0.0 },
            if self.is_surprised { 1.0 } else { 0.0 },
            if self.intentional_closed_eyes { 1.0 } else { 0.0 },
        ]
    }

    pub fn dimension() -> usize { 17 }
}

/// 婚礼场景特征（完整特征向量）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeddingFeatures {
    pub quality_features: crate::quality_analyzer::QualityFeatures,
    pub face_features: FaceFeatures,
}

impl WeddingFeatures {
    pub fn to_vec(&self) -> Vec<f32> {
        let mut v = self.quality_features.to_vec();
        v.extend(self.face_features.to_vec());
        v
    }

    pub fn dimension() -> usize {
        crate::quality_analyzer::QualityFeatures::dimension() + FaceFeatures::dimension()
    }
}

/// 分析人脸特征
pub fn analyze_face_features(img: &RgbImage) -> FaceFeatures {
    match detect_faces(img) {
        Ok(detections) if !detections.is_empty() => {
            let count = detections.len() as f32;
            let (img_w, img_h) = img.dimensions();
            let img_area = (img_w * img_h) as f32;

            let mut total_area = 0.0f32;
            let mut total_position = 0.0f32;
            let mut total_quality = 0.0f32;
            let mut total_eye_openness = 0.0f32;
            let mut total_face_blur = 0.0f32;
            let mut total_mouth_openness = 0.0f32;
            let mut total_eye_closed = 0.0f32;
            let mut total_mouth_smile = 0.0f32;
            let mut total_head_tilt = 0.0f32;
            let mut total_expression_quality = 0.0f32;
            let mut any_funny = false;
            let mut any_smiling = false;
            let mut any_frowning = false;
            let mut any_surprised = false;

            // 收集所有脸的表情信息，用于判断故意闭眼
            let mut face_expressions = Vec::new();

            for det in &detections {
                // 人脸面积占比
                let face_area = det.bbox[2] * det.bbox[3] * img_area;
                total_area += face_area / img_area;

                // 人脸位置（中心度）
                let center_x = det.bbox[0] + det.bbox[2] / 2.0;
                let center_y = det.bbox[1] + det.bbox[3] / 2.0;
                let position = 1.0 - ((center_x - 0.5).abs() + (center_y - 0.5).abs());
                total_position += position;

                // 人脸质量（基于置信度）
                total_quality += det.confidence;

                // 使用 face_detector 的表情分析
                let expr = crate::face_detector::analyze_expression(det);
                total_eye_openness += expr.eye_openness;
                total_eye_closed += expr.eye_closed_ratio;
                total_mouth_openness += expr.mouth_openness;
                total_mouth_smile += expr.mouth_smile;
                total_head_tilt += expr.head_tilt;
                total_expression_quality += expr.expression_quality;
                if expr.is_funny { any_funny = true; }
                if expr.is_smiling { any_smiling = true; }
                if expr.is_frowning { any_frowning = true; }
                if expr.is_surprised { any_surprised = true; }

                // 收集闭眼信息
                face_expressions.push((expr.eye_closed_ratio, expr.is_smiling, expr.expression_quality));

                // 人脸模糊度
                let face_blur = calculate_face_blur(img, det, img_w, img_h);
                total_face_blur += face_blur;
            }

            // 判断是否为故意闭眼
            let intentional_closed = is_intentional_closed_eyes(&face_expressions);

            FaceFeatures {
                has_face: true,
                face_count: count.min(5.0) / 5.0,
                face_size_ratio: (total_area / count).min(1.0),
                face_position: total_position / count,
                face_quality: total_quality / count,
                eye_openness: total_eye_openness / count,
                face_blur: total_face_blur / count,
                mouth_openness: total_mouth_openness / count,
                eye_closed_ratio: total_eye_closed / count,
                mouth_smile: total_mouth_smile / count,
                head_tilt: total_head_tilt / count,
                expression_quality: total_expression_quality / count,
                is_funny: any_funny,
                is_smiling: any_smiling,
                is_frowning: any_frowning,
                is_surprised: any_surprised,
                intentional_closed_eyes: intentional_closed,
            }
        }
        // 未检测到人脸（侧面照、背面照等）
        _ => FaceFeatures {
            has_face: false,
            face_count: 0.0,
            face_size_ratio: 0.0,
            face_position: 0.5,
            face_quality: 0.0,  // 无人脸时质量为0，不是默认的0.5
            eye_openness: 0.0,
            face_blur: 0.0,
            mouth_openness: 0.0,
            eye_closed_ratio: 0.0,
            mouth_smile: 0.0,
            head_tilt: 0.0,
            expression_quality: 0.0,
            is_funny: false,
            is_smiling: false,
            is_frowning: false,
            is_surprised: false,
            intentional_closed_eyes: false,
        },
    }
}

/// 判断是否为故意闭眼
/// 
/// 婚礼场景中常见：
/// - 新娘闭眼 pose
/// - 两人额头相碰闭眼
/// - 艺术感特写
/// 
/// 区分方法：
/// 1. 多人同时闭眼 + 表情自然 → 故意
/// 2. 闭眼 + 微笑 → 故意
/// 3. 闭眼 + 嘴巴张开/表情奇怪 → 意外
/// 4. 单人闭眼 + 无微笑 → 可能意外
fn is_intentional_closed_eyes(faces: &[(f32, bool, f32)]) -> bool {
    if faces.is_empty() {
        return false;
    }

    let closed_count = faces.iter().filter(|(ratio, _, _)| *ratio > 0.8).count();
    let total_count = faces.len();

    // 场景1：多人同时闭眼 → 大概率故意（如两人额头相碰）
    if total_count > 1 && closed_count == total_count {
        return true;
    }

    // 场景2：多人场景，部分人闭眼
    if total_count > 1 && closed_count > 0 && closed_count < total_count {
        // 如果闭眼的人都在微笑 → 故意
        let closed_smiling = faces.iter()
            .filter(|(ratio, _, _)| *ratio > 0.8)
            .filter(|(_, smile, _)| *smile)
            .count();
        if closed_smiling == closed_count {
            return true;
        }
        // 否则可能是意外（如有人眨眼）
        return false;
    }

    // 场景3：单人闭眼
    if total_count == 1 {
        let (eye_closed, is_smiling, expr_quality) = faces[0];
        if eye_closed > 0.8 {
            // 闭眼 + 微笑 + 表情质量好 → 故意
            if is_smiling && expr_quality > 0.4 {
                return true;
            }
            // 闭眼 + 嘴巴张开 → 意外（眨眼时嘴巴张开很少见）
            // 注意：这里没有 mouth_openness 信息，用 expression_quality 代替
            if expr_quality < 0.3 {
                return false;
            }
        }
    }

    false
}



/// 计算人脸区域模糊度
fn calculate_face_blur(img: &RgbImage, det: &FaceDetection, img_w: u32, img_h: u32) -> f32 {
    let face_x = (det.bbox[0] * img_w as f32) as u32;
    let face_y = (det.bbox[1] * img_h as f32) as u32;
    let face_w = (det.bbox[2] * img_w as f32) as u32;
    let face_h = (det.bbox[3] * img_h as f32) as u32;

    if face_w < 10 || face_h < 10 {
        return 0.5;
    }

    // 裁剪人脸区域
    let face_region = image::imageops::crop_imm(
        img,
        face_x.min(img_w - 1),
        face_y.min(img_h - 1),
        face_w.min(img_w - face_x),
        face_h.min(img_h - face_y),
    ).to_image();

    // 计算 Laplacian 方差作为锐度指标
    let sharpness = quality_analyzer::calculate_laplacian_variance(&face_region);

    // 模糊度 = 1 - 锐度
    (1.0 - sharpness.min(1.0)) as f32
}

/// 综合分析图像（质量 + 人脸）
pub fn analyze_wedding_image(img: &RgbImage) -> WeddingFeatures {
    let quality_features = quality_analyzer::extract_quality_features(img);
    let face_features = analyze_face_features(img);

    WeddingFeatures {
        quality_features,
        face_features,
    }
}

/// 计算废片概率和原因
pub fn calculate_waste_score(
    features: &WeddingFeatures,
    config: &WasteConfig,
) -> (f32, Vec<WasteReason>) {
    let mut score = 0.0f32;
    let mut reasons = Vec::new();

    // 曝光问题 (权重 20%)
    if features.quality_features.overexposed_ratio > config.overexposed_threshold {
        score += 0.20;
        reasons.push(WasteReason::Overexposed);
    }
    if features.quality_features.underexposed_ratio > config.underexposed_threshold {
        score += 0.15;
        reasons.push(WasteReason::Underexposed);
    }

    // 人脸模糊 (权重 25%) - 仅在检测到人脸时
    if features.face_features.has_face && features.face_features.face_blur > config.face_blur_threshold {
        score += 0.25;
        reasons.push(WasteReason::FaceBlur);
    }

    // 整体模糊 (权重 15%)
    if features.quality_features.sharpness < config.overall_blur_threshold {
        score += 0.15;
        reasons.push(WasteReason::OverallBlur);
    }

    // 运动模糊 (权重 15%) - 需要同时满足：严重模糊 + 极不均匀
    // 运动模糊特征：整体模糊但某些区域相对清晰（方向性模糊）
    if features.quality_features.blur_score > 0.85
        && features.quality_features.sharpness_uniformity < 0.15
        && features.quality_features.edge_density > 0.1 {
        score += 0.15;
        reasons.push(WasteReason::MotionBlur);
    }

    // 表情问题 (权重 25%) - 仅在检测到人脸时
    if features.face_features.has_face {
        // 怪表情
        if features.face_features.is_funny {
            score += 0.15;
            reasons.push(WasteReason::FunnyExpression);
        }
        // 嘴巴过度张开
        if features.face_features.mouth_openness > config.mouth_open_threshold {
            score += 0.05;
            reasons.push(WasteReason::MouthTooOpen);
        }
        // 闭眼 - 但要排除故意闭眼
        if features.face_features.eye_closed_ratio > config.eye_closed_threshold
            && !features.face_features.intentional_closed_eyes {
            score += 0.10;
            reasons.push(WasteReason::ClosedEyes);
        }
        // 惊讶表情
        if features.face_features.is_surprised {
            score += 0.05;
            reasons.push(WasteReason::SurprisedExpression);
        }
        // 皱眉
        if features.face_features.is_frowning {
            score += 0.05;
            reasons.push(WasteReason::FrowningExpression);
        }
        // 头部过度倾斜
        if features.face_features.head_tilt.abs() > 0.3 {
            score += 0.05;
            reasons.push(WasteReason::HeadTilted);
        }
    }

    // 构图问题 (权重 10%) - 仅在检测到人脸时
    if features.face_features.has_face {
        if features.face_features.face_size_ratio < config.min_face_ratio {
            score += 0.10;
            reasons.push(WasteReason::SubjectTooSmall);
        }
        if features.face_features.face_size_ratio > config.max_face_ratio {
            score += 0.08;
            reasons.push(WasteReason::SubjectTooLarge);
        }
    }

    // 无人脸场景（侧面照、背面照）- 不降低质量分，但标记为无人脸
    // 这类照片可能有精修价值，只是无法分析表情

    // 噪点 (权重 5%)
    if features.quality_features.noise_level > config.noise_threshold {
        score += 0.05;
        reasons.push(WasteReason::HighNoise);
    }

    // 低对比度 (权重 5%)
    if features.quality_features.contrast < 0.15 {
        score += 0.05;
        reasons.push(WasteReason::LowContrast);
    }

    (score.min(1.0), reasons)
}
