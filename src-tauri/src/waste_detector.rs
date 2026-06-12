use image::RgbImage;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{Manager, Emitter};

use crate::quality_analyzer;
use crate::wedding_analyzer::{self, WasteConfig, WasteReason, WeddingFeatures};

const RAW_EXTENSIONS: &[&str] = &["cr2", "cr3", "nef", "arw", "dng", "orf", "rw2", "pef", "srw", "raf"];

/// 废片检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasteResult {
    pub path: String,
    pub waste_score: f32,
    pub quality_score: f32,
    pub reasons: Vec<String>,
    pub confidence: f32,
    pub is_waste: bool,
    pub features: WeddingFeatures,
}

/// 简单线性分类器（逻辑回归）
#[derive(Debug, Clone)]
pub struct LinearClassifier {
    pub weights: Vec<f32>,
    pub bias: f32,
    pub feature_dim: usize,
}

impl LinearClassifier {
    pub fn new(feature_dim: usize) -> Self {
        Self {
            weights: vec![0.0; feature_dim],
            bias: 0.0,
            feature_dim,
        }
    }

    fn sigmoid(x: f32) -> f32 {
        1.0 / (1.0 + (-x).clamp(-10.0, 10.0).exp())
    }

    pub fn predict(&self, features: &[f32]) -> f32 {
        if features.len() != self.feature_dim {
            return 0.5;
        }
        let z: f32 = features.iter().zip(self.weights.iter()).map(|(f, w)| f * w).sum::<f32>() + self.bias;
        Self::sigmoid(z)
    }

    pub fn train_step(&mut self, features: &[f32], label: bool, learning_rate: f32) {
        if features.len() != self.feature_dim {
            return;
        }
        let pred = self.predict(features);
        let target = if label { 1.0 } else { 0.0 };
        let error = pred - target;
        for (w, f) in self.weights.iter_mut().zip(features.iter()) {
            *w -= learning_rate * error * f;
        }
        self.bias -= learning_rate * error;
    }

    pub fn train(&mut self, samples: &[TrainingSample], epochs: usize, learning_rate: f32) {
        for _ in 0..epochs {
            for sample in samples {
                if sample.features.len() == self.feature_dim {
                    self.train_step(&sample.features, sample.label, learning_rate);
                }
            }
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let data = ClassifierData {
            weights: self.weights.clone(),
            bias: self.bias,
            feature_dim: self.feature_dim,
        };
        let json = serde_json::to_string(&data)
            .map_err(|e| format!("Failed to serialize classifier: {}", e))?;
        fs::write(path, json)
            .map_err(|e| format!("Failed to write classifier: {}", e))
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let json = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read classifier: {}", e))?;
        let data: ClassifierData = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse classifier: {}", e))?;
        Ok(Self {
            weights: data.weights,
            bias: data.bias,
            feature_dim: data.feature_dim,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClassifierData {
    weights: Vec<f32>,
    bias: f32,
    feature_dim: usize,
}

/// 训练样本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSample {
    pub features: Vec<f32>,
    pub label: bool,
    pub reasons: Vec<String>,
    pub timestamp: u64,
}

/// 全局废片检测器状态
pub struct WasteDetectorState {
    pub classifier: Mutex<Option<LinearClassifier>>,
    pub config: Mutex<WasteConfig>,
    pub feedback_count: Mutex<u32>,
}

impl WasteDetectorState {
    pub fn new() -> Self {
        Self {
            classifier: Mutex::new(None),
            config: Mutex::new(WasteConfig::default()),
            feedback_count: Mutex::new(0),
        }
    }
}

/// 加载图像为 RGB
pub fn load_image_as_rgb(image_path: &Path) -> Result<RgbImage, String> {
    let ext = image_path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if RAW_EXTENSIONS.contains(&ext.as_str()) {
        crate::image_decoder::raw::decode_raw_to_rgb(image_path)
    } else {
        let img = image::open(image_path)
            .map_err(|e| format!("Failed to open image: {}", e))?;
        Ok(img.to_rgb8())
    }
}

/// 检测是否为截图/图标（非人像/非风景）
fn is_screenshot(_img: &RgbImage, features: &WeddingFeatures) -> bool {
    // 有人脸的图肯定不是截图
    if features.face_features.has_face {
        return false;
    }

    // === 小图/图标检测 ===
    // 这个判断在图片扫描时已由前端处理（过滤小文件）

    // === UI 元素特征 ===
    // 高对比度 + 高边缘密度 + 低纹理复杂度 = 可能是 UI/图标
    let has_ui = features.quality_features.contrast > 0.25
        && features.quality_features.edge_density > 0.15
        && features.quality_features.texture_complexity < 0.1;

    // === 纯色/渐变背景检测 ===
    // 颜色丰富度极低 = 纯色背景截图
    let is_plain_background = features.quality_features.color_richness < 0.08;

    // === 文字密集型截图 ===
    // 高边缘密度 + 高对比度 + 低饱和度 = 文字截图
    let is_text_screenshot = features.quality_features.edge_density > 0.25
        && features.quality_features.contrast > 0.3
        && features.quality_features.saturation < 0.2;

    if has_ui || is_plain_background || is_text_screenshot {
        return true;
    }

    false
}

/// 分析单张图像
pub fn analyze_image(
    classifier: Option<&LinearClassifier>,
    image_path: &Path,
    config: &WasteConfig,
) -> Result<WasteResult, String> {
    let img = load_image_as_rgb(image_path)?;

    // 1. 图像质量 + 人脸分析
    let features = wedding_analyzer::analyze_wedding_image(&img);

    // 2. 计算废片分数（基于规则）
    let (mut score, mut reasons) = wedding_analyzer::calculate_waste_score(&features, config);

    // 3. 截图/图标检测
    if is_screenshot(&img, &features) {
        score = (score + 0.4).min(1.0);
        reasons.push(WasteReason::Screenshot);
    }

    // 4. 使用训练好的分类器调整分数
    if let Some(clf) = classifier {
        let feature_vec = features.to_vec();
        if feature_vec.len() == clf.feature_dim {
            let clf_prob = clf.predict(&feature_vec);
            // 分类器占 70% 权重，规则占 30%
            // 用户标记的学习结果应该有决定性影响
            let alpha = 0.7;
            score = (score * (1.0 - alpha) + clf_prob * alpha).min(1.0);

            // 如果分类器判定为废片但规则未触发，添加原因
            if clf_prob > 0.6 && reasons.is_empty() {
                reasons.push(WasteReason::LowRetouchValue);
            }
        }
    }

    // 5. 计算质量分数
    let quality_score = quality_analyzer::calculate_quality_score(&features.quality_features);

    // 6. 判断是否废片
    let is_waste = score > 0.5;
    let confidence = if is_waste {
        (score - 0.5) * 2.0
    } else {
        (0.5 - score) * 2.0
    };

    let reason_labels: Vec<String> = reasons.iter().map(|r| r.label_cn().to_string()).collect();

    Ok(WasteResult {
        path: image_path.to_string_lossy().to_string(),
        waste_score: score,
        quality_score,
        reasons: reason_labels,
        confidence,
        is_waste,
        features,
    })
}

/// 加载训练样本
pub fn load_training_samples(path: &Path) -> Vec<TrainingSample> {
    match fs::read_to_string(path) {
        Ok(content) => {
            content.lines()
                .filter_map(|line| serde_json::from_str(line).ok())
                .collect()
        }
        Err(_) => Vec::new(),
    }
}

/// 保存训练样本
pub fn save_training_sample(path: &Path, sample: &TrainingSample) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("Failed to open training file: {}", e))?;

    let line = serde_json::to_string(sample)
        .map_err(|e| format!("Failed to serialize sample: {}", e))?;

    writeln!(file, "{}", line)
        .map_err(|e| format!("Failed to write sample: {}", e))
}

/// 获取反馈文件路径
pub fn feedback_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    Ok(data_dir.join("waste_feedback.jsonl"))
}

/// 获取分类器文件路径
pub fn classifier_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    Ok(data_dir.join("waste_classifier.json"))
}

/// Tauri 命令：初始化废片检测器
#[tauri::command]
pub async fn init_waste_detector(
    app: tauri::AppHandle,
    state: tauri::State<'_, WasteDetectorState>,
) -> Result<(), String> {
    let clf_path = classifier_path(&app)?;
    if clf_path.exists() {
        match LinearClassifier::load(&clf_path) {
            Ok(clf) => {
                let mut guard = state.classifier.lock().map_err(|e| e.to_string())?;
                *guard = Some(clf);
            }
            Err(e) => {
                eprintln!("[Waste] Failed to load classifier: {}", e);
            }
        }
    }

    let fb_path = feedback_path(&app)?;
    let samples = load_training_samples(&fb_path);
    let mut count_guard = state.feedback_count.lock().map_err(|e| e.to_string())?;
    *count_guard = samples.len() as u32;

    Ok(())
}

/// Tauri 命令：分析图像（并发处理，实时推送结果）
#[tauri::command]
pub async fn analyze_waste_images(
    app: tauri::AppHandle,
    state: tauri::State<'_, WasteDetectorState>,
    paths: Vec<String>,
) -> Result<Vec<WasteResult>, String> {
    let classifier = {
        let guard = state.classifier.lock().map_err(|e| e.to_string())?;
        guard.clone()
    };
    let config = {
        let guard = state.config.lock().map_err(|e| e.to_string())?;
        guard.clone()
    };

    let total = paths.len();
    let completed = std::sync::atomic::AtomicUsize::new(0);
    let results = std::sync::Mutex::new(Vec::new());

    // 使用 rayon 并发处理，每张图实时推送
    paths.par_iter().for_each(|path_str| {
        let path = Path::new(path_str);
        if let Ok(result) = analyze_image(classifier.as_ref(), path, &config) {
            // 推送单张结果
            let _ = app.emit("waste-result", &result);
            results.lock().unwrap().push(result);
        }

        // 报告进度
        let current = completed.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        let _ = app.emit("waste-progress", serde_json::json!({
            "current": current,
            "total": total,
        }));
    });

    Ok(results.into_inner().unwrap_or_default())
}

/// Tauri 命令：标记图像反馈
#[tauri::command]
pub async fn mark_waste_feedback(
    app: tauri::AppHandle,
    state: tauri::State<'_, WasteDetectorState>,
    path: String,
    is_waste: bool,
    reasons: Vec<String>,
) -> Result<u32, String> {
    let features = {
        let img = load_image_as_rgb(Path::new(&path))?;
        let features = wedding_analyzer::analyze_wedding_image(&img);
        features.to_vec()
    };

    let sample = TrainingSample {
        features,
        label: is_waste,
        reasons,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    let fb_path = feedback_path(&app)?;
    save_training_sample(&fb_path, &sample)?;

    // 重新训练分类器（只要有样本就训练）
    let samples = load_training_samples(&fb_path);
    if !samples.is_empty() {
        let mut clf = LinearClassifier::new(WeddingFeatures::dimension());
        let lr = (0.1 / (samples.len() as f32).sqrt()).max(0.01);
        // 多轮训练，确保学习到模式
        clf.train(&samples, 200, lr);

        let clf_path = classifier_path(&app)?;
        clf.save(&clf_path)?;

        let mut clf_guard = state.classifier.lock().map_err(|e| e.to_string())?;
        *clf_guard = Some(clf);
    }

    let count = samples.len() as u32;
    let mut count_guard = state.feedback_count.lock().map_err(|e| e.to_string())?;
    *count_guard = count;

    Ok(count)
}

/// Tauri 命令：获取反馈数量
#[tauri::command]
pub fn get_waste_feedback_count(
    state: tauri::State<'_, WasteDetectorState>,
) -> u32 {
    *state.feedback_count.lock().unwrap_or_else(|e| e.into_inner())
}

/// Tauri 命令：获取配置
#[tauri::command]
pub fn get_waste_config(
    state: tauri::State<'_, WasteDetectorState>,
) -> WasteConfig {
    state.config.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// Tauri 命令：更新配置
#[tauri::command]
pub fn update_waste_config(
    state: tauri::State<'_, WasteDetectorState>,
    config: WasteConfig,
) -> Result<(), String> {
    let mut guard = state.config.lock().map_err(|e| e.to_string())?;
    *guard = config;
    Ok(())
}
