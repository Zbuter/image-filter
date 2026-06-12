use std::path::Path;
use std::sync::Mutex;
use image::RgbImage;
use ort::session::Session;
use ort::value::Tensor;
use ndarray::Array4;
use serde::{Deserialize, Serialize};

const YUNET_INPUT_SIZE: usize = 320;
const CONFIDENCE_THRESHOLD: f32 = 0.5;

static YUNET_SESSION: Mutex<Option<Session>> = Mutex::new(None);

pub struct FaceDetection {
    pub bbox: [f32; 4],      // [x, y, w, h] 归一化到 0-1
    pub confidence: f32,
    pub landmarks: [[f32; 2]; 5], // 5个关键点: 0=左眼, 1=右眼, 2=鼻子, 3=左嘴角, 4=右嘴角
}

/// 人脸表情分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceExpression {
    pub eye_openness: f32,      // 眼睛开合度 (0-1, 1=完全睁开)
    pub eye_closed_ratio: f32,  // 闭眼比例 (0-1, 1=完全闭眼)
    pub mouth_openness: f32,    // 嘴巴张开度 (0-1, 1=完全张开)
    pub mouth_smile: f32,       // 微笑程度 (0-1, 1=大笑)
    pub face_symmetry: f32,     // 面部对称性 (0-1, 1=完全对称)
    pub head_tilt: f32,         // 头部倾斜角度 (-1 到 1)
    pub expression_quality: f32, // 表情质量 (0-1, 1=最佳)
    pub is_funny: bool,         // 是否表情包/怪表情
    pub is_smiling: bool,       // 是否微笑
    pub is_frowning: bool,      // 是否皱眉
    pub is_surprised: bool,     // 是否惊讶（嘴巴大张）
}

pub fn load_yunet_model(model_path: &Path) -> Result<(), String> {
    let session = Session::builder()
        .map_err(|e| format!("Failed to create ONNX session builder: {}", e))?
        .commit_from_file(model_path)
        .map_err(|e| format!("Failed to load YuNet model: {}", e))?;
    
    let mut guard = YUNET_SESSION.lock().map_err(|e| e.to_string())?;
    *guard = Some(session);
    
    Ok(())
}

pub fn detect_faces(img: &RgbImage) -> Result<Vec<FaceDetection>, String> {
    let mut guard = YUNET_SESSION.lock().map_err(|e| e.to_string())?;
    let session = guard.as_mut().ok_or("YuNet model not loaded")?;
    
    // 1. 缩放图像到 320x320
    let resized = image::imageops::resize(img, YUNET_INPUT_SIZE as u32, YUNET_INPUT_SIZE as u32, image::imageops::FilterType::Triangle);
    
    // 2. 转换为 NCHW 格式，使用 mean=[104, 117, 123] 归一化
    let mut input_data = vec![0.0f32; 3 * YUNET_INPUT_SIZE * YUNET_INPUT_SIZE];
    
    for y in 0..YUNET_INPUT_SIZE {
        for x in 0..YUNET_INPUT_SIZE {
            let pixel = resized.get_pixel(x as u32, y as u32);
            let idx = y * YUNET_INPUT_SIZE + x;
            
            // YuNet 使用 BGR 顺序
            input_data[idx] = (pixel[2] as f32 - 104.0) / 255.0; // B
            input_data[YUNET_INPUT_SIZE * YUNET_INPUT_SIZE + idx] = (pixel[1] as f32 - 117.0) / 255.0; // G
            input_data[2 * YUNET_INPUT_SIZE * YUNET_INPUT_SIZE + idx] = (pixel[0] as f32 - 123.0) / 255.0; // R
        }
    }
    
    // 3. 创建输入 tensor
    let input_array = Array4::from_shape_vec((1, 3, YUNET_INPUT_SIZE, YUNET_INPUT_SIZE), input_data)
        .map_err(|e| format!("Failed to create input tensor: {}", e))?;
    
    let input_tensor = Tensor::from_array(input_array)
        .map_err(|e| format!("Failed to create input tensor: {}", e))?;
    
    // 4. 运行推理
    let outputs = session.run(ort::inputs!["input" => input_tensor])
        .map_err(|e| format!("YuNet inference failed: {}", e))?;
    
    // 5. 解析输出
    // YuNet 输出格式: [1, num_detections, 15]
    // 每个检测: [x, y, w, h, confidence, 5*2 landmarks]
    let (_shape, output_slice) = outputs["output"]
        .try_extract_tensor::<f32>()
        .map_err(|e| format!("Failed to extract output: {}", e))?;
    
    // 输出数据是扁平的: num_detections * 15
    let num_detections = output_slice.len() / 15;
    let mut detections = Vec::new();
    
    for i in 0..num_detections {
        let base_idx = i * 15;
        let confidence = output_slice[base_idx + 4];
        
        if confidence >= CONFIDENCE_THRESHOLD {
            let bbox = [
                output_slice[base_idx + 0], // x
                output_slice[base_idx + 1], // y
                output_slice[base_idx + 2], // w
                output_slice[base_idx + 3], // h
            ];
            
            let mut landmarks = [[0.0f32; 2]; 5];
            for j in 0..5 {
                landmarks[j] = [
                    output_slice[base_idx + 5 + j * 2],
                    output_slice[base_idx + 5 + j * 2 + 1],
                ];
            }
            
            detections.push(FaceDetection {
                bbox,
                confidence,
                landmarks,
            });
        }
    }
    
    Ok(detections)
}

/// 分析人脸表情
pub fn analyze_expression(det: &FaceDetection) -> FaceExpression {
    // YuNet 5个关键点: 0=左眼, 1=右眼, 2=鼻子, 3=左嘴角, 4=右嘴角
    let left_eye = det.landmarks[0];
    let right_eye = det.landmarks[1];
    let nose = det.landmarks[2];
    let left_mouth = det.landmarks[3];
    let right_mouth = det.landmarks[4];

    // 眼睛开合度：基于眼睛关键点的垂直距离
    let eye_distance = (left_eye[1] - right_eye[1]).abs();
    let eye_openness = (eye_distance * 10.0).min(1.0);

    // 闭眼检测
    let eye_closed_ratio = if eye_openness < 0.05 { 1.0 } else { 0.0 };

    // 嘴巴张开度：基于嘴角到鼻子的垂直距离
    let mouth_center_y = (left_mouth[1] + right_mouth[1]) / 2.0;
    let mouth_to_nose = (mouth_center_y - nose[1]).abs();
    let mouth_width = (right_mouth[0] - left_mouth[0]).abs();

    let mouth_openness = if mouth_width > 0.01 {
        (mouth_to_nose / mouth_width).min(1.0)
    } else {
        0.0
    };

    // 微笑程度：基于嘴角上扬程度
    let mouth_y_offset = (left_mouth[1] - right_mouth[1]).abs();
    let smile_factor = if mouth_width > 0.01 {
        (mouth_y_offset / mouth_width).min(1.0)
    } else {
        0.0
    };
    let mouth_smile = smile_factor * 0.5 + (1.0 - mouth_openness) * 0.5;

    // 面部对称性：基于眼睛和嘴巴的对称性
    let eye_symmetry = 1.0 - (left_eye[1] - right_eye[1]).abs();
    let mouth_symmetry = 1.0 - (left_mouth[1] - right_mouth[1]).abs();
    let face_symmetry = (eye_symmetry + mouth_symmetry) / 2.0;

    // 头部倾斜角度
    let head_tilt = (right_eye[1] - left_eye[1]) / mouth_width.max(0.01);

    // 表情判断
    let is_funny = mouth_openness > 0.5 || eye_openness < 0.05;
    let is_smiling = mouth_smile > 0.6 && mouth_openness < 0.3;
    let is_frowning = eye_openness < 0.3 && mouth_openness < 0.2;
    let is_surprised = mouth_openness > 0.7;

    // 表情质量综合评分
    let expression_quality = {
        let mut score = 0.0;

        // 眼睛睁开且自然
        if eye_openness > 0.3 && eye_openness < 0.8 {
            score += 0.3;
        }

        // 嘴巴自然（不太开也不太闭）
        if mouth_openness > 0.1 && mouth_openness < 0.4 {
            score += 0.3;
        }

        // 面部对称
        score += face_symmetry * 0.2;

        // 头部不倾斜
        score += (1.0 - head_tilt.abs()) * 0.2;

        // 怪表情扣分
        if is_funny {
            score *= 0.3;
        }

        score.min(1.0)
    };

    FaceExpression {
        eye_openness,
        eye_closed_ratio,
        mouth_openness,
        mouth_smile,
        face_symmetry,
        head_tilt,
        expression_quality,
        is_funny,
        is_smiling,
        is_frowning,
        is_surprised,
    }
}

/// 批量分析多人脸表情
pub fn analyze_expressions(detections: &[FaceDetection]) -> Vec<FaceExpression> {
    detections.iter().map(analyze_expression).collect()
}

/// 检测是否为表情包/怪表情
pub fn is_funny_expression(expr: &FaceExpression) -> bool {
    // 嘴巴过度张开
    if expr.mouth_openness > 0.5 {
        return true;
    }

    // 完全闭眼
    if expr.eye_closed_ratio > 0.8 {
        return true;
    }

    // 表情质量过低
    if expr.expression_quality < 0.2 {
        return true;
    }

    false
}
