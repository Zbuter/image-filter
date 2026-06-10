use ndarray::{Array2, Array4};
use ort::session::Session;
use ort::value::Tensor;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;

/// Waste photo detection labels (must match labels.txt order)
const LABELS: &[&str] = &[
    "meme_emoji",
    "bad_expression_blur",
    "backlit",
    "lens_distortion",
    "normal",
];

/// Threshold: if a waste label scores above this AND higher than "normal", mark as waste
const WASTE_THRESHOLD: f32 = 0.25;

const RAW_EXTENSIONS: &[&str] = &["cr2", "cr3", "nef", "arw", "dng", "orf", "rw2", "pef", "srw", "raf"];

fn is_raw_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| RAW_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Load image as RGB, handling RAW and GIF formats
fn load_image_as_rgb(image_path: &Path) -> Result<image::RgbImage, String> {
    if is_raw_extension(image_path) {
        // Use embedded JPEG from RAW file
        crate::image_decoder::raw::decode_raw_to_rgb(image_path)
    } else {
        // Standard image open (handles GIF first frame automatically)
        let img = image::open(image_path)
            .map_err(|e| format!("Failed to open image: {}", e))?;
        Ok(img.to_rgb8())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResult {
    pub path: String,
    pub labels: Vec<String>,
    pub scores: Vec<f32>,
    pub is_waste: bool,
}

pub struct AiModel {
    session: Session,
    text_embeddings: Array2<f32>,
}

impl AiModel {
    /// Load model from the given directory containing clip_image_encoder.onnx and text_embeddings.bin
    pub fn load(model_dir: &Path) -> Result<Self, String> {
        let onnx_path = model_dir.join("clip_image_encoder.onnx");
        let embeddings_path = model_dir.join("text_embeddings.bin");

        if !onnx_path.exists() {
            return Err(format!("ONNX model not found: {}", onnx_path.display()));
        }
        if !embeddings_path.exists() {
            return Err(format!("Text embeddings not found: {}", embeddings_path.display()));
        }

        let session = Session::builder()
            .map_err(|e| format!("Failed to create ONNX session builder: {}", e))?
            .commit_from_file(&onnx_path)
            .map_err(|e| format!("Failed to load ONNX model: {}", e))?;

        // Load text embeddings: raw float32 binary, shape [N, 512]
        let emb_data = fs::read(&embeddings_path)
            .map_err(|e| format!("Failed to read text embeddings: {}", e))?;
        let num_elements = emb_data.len() / 4;
        let dim = 512;
        if num_elements % dim != 0 {
            return Err("Text embeddings file has invalid size".to_string());
        }
        let num_prompts = num_elements / dim;
        let floats: Vec<f32> = emb_data
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        let text_embeddings = Array2::from_shape_vec((num_prompts, dim), floats)
            .map_err(|e| format!("Failed to reshape text embeddings: {}", e))?;

        Ok(Self {
            session,
            text_embeddings,
        })
    }

    /// Analyze a single image, returning similarity scores for each label
    pub fn analyze_image(&mut self, image_path: &Path) -> Result<AiResult, String> {
    let rgb_img = load_image_as_rgb(image_path)?;
    let dyn_img = image::DynamicImage::ImageRgb8(rgb_img);
    let resized = dyn_img.resize_exact(224, 224, image::imageops::FilterType::Triangle);
    let rgb = resized.to_rgb8();

        // Convert to NCHW float tensor with CLIP normalization
        let mean = [0.48145466f32, 0.4578275, 0.40821073];
        let std_dev = [0.26862954f32, 0.26130258, 0.27577711];

        let mut data = vec![0.0f32; 3 * 224 * 224];
        for y in 0..224u32 {
            for x in 0..224u32 {
                let pixel = rgb.get_pixel(x, y);
                for c in 0..3usize {
                    let val = pixel[c] as f32 / 255.0;
                    data[c * 224 * 224 + y as usize * 224 + x as usize] =
                        (val - mean[c]) / std_dev[c];
                }
            }
        }

        let input_array = Array4::from_shape_vec((1, 3, 224, 224), data)
            .map_err(|e| format!("Failed to create input tensor: {}", e))?;

        // Create ort Tensor value from ndarray
        let input_value = Tensor::from_array(input_array)
            .map_err(|e| format!("Failed to create ORT tensor: {}", e))?;

        // Run inference
        let outputs = self
            .session
            .run(ort::inputs!["image" => input_value])
            .map_err(|e| format!("ONNX inference failed: {}", e))?;

        // Extract embedding: try_extract_tensor returns Result<(&Shape, &[f32])>
        let (_shape, emb_slice) = outputs["embedding"]
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Failed to extract embedding: {}", e))?;

        // Normalize embedding
        let norm: f32 = emb_slice.iter().map(|x| x * x).sum::<f32>().sqrt();
        let normalized: Vec<f32> = if norm > 0.0 {
            emb_slice.iter().map(|x| x / norm).collect()
        } else {
            vec![0.0; emb_slice.len()]
        };

        // Compute cosine similarity with each text embedding
        let mut scores: Vec<f32> = Vec::with_capacity(LABELS.len());
        for i in 0..self.text_embeddings.nrows() {
            let text_emb = self.text_embeddings.row(i);
            let sim: f32 = normalized
                .iter()
                .zip(text_emb.iter())
                .map(|(a, b)| a * b)
                .sum();
            scores.push(sim);
        }

        // Determine waste labels
        let normal_idx = LABELS.iter().position(|l| *l == "normal").unwrap_or(LABELS.len() - 1);
        let normal_score = scores[normal_idx];

        let mut waste_labels = Vec::new();
        for (i, label) in LABELS.iter().enumerate() {
            if *label != "normal" && scores[i] > WASTE_THRESHOLD && scores[i] > normal_score {
                waste_labels.push(label.to_string());
            }
        }

        let is_waste = !waste_labels.is_empty();

        Ok(AiResult {
            path: image_path.to_string_lossy().to_string(),
            labels: waste_labels,
            scores,
            is_waste,
        })
    }
}

/// Global AI model state
pub struct AiState {
    pub model: Mutex<Option<AiModel>>,
    pub classifier_weights: Mutex<Option<Vec<f32>>>,
    pub feedback_count: Mutex<u32>,
}

#[tauri::command]
pub async fn init_ai_model(
    app: tauri::AppHandle,
    state: tauri::State<'_, AiState>,
    model_dir: String,
) -> Result<(), String> {
    let dir = PathBuf::from(&model_dir);
    let model = AiModel::load(&dir)?;
    let mut guard = state.model.lock().map_err(|e| e.to_string())?;
    *guard = Some(model);
    drop(guard);

    // Load saved classifier weights if they exist
    if let Ok(wp) = weights_path(&app) {
        if wp.exists() {
            if let Ok(bytes) = fs::read(&wp) {
                if bytes.len() % 4 == 0 {
                    let weights: Vec<f32> = bytes.chunks_exact(4)
                        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                        .collect();
                    let mut w_guard = state.classifier_weights.lock().map_err(|e| e.to_string())?;
                    *w_guard = Some(weights);
                    eprintln!("[AI] Loaded saved classifier weights");
                }
            }
        }
    }

    // Load feedback count
    if let Ok(fb) = feedback_path(&app) {
        let samples = load_feedback_samples(&fb);
        let mut count_guard = state.feedback_count.lock().map_err(|e| e.to_string())?;
        *count_guard = samples.len() as u32;
    }

    Ok(())
}

#[tauri::command]
pub async fn analyze_images(
    state: tauri::State<'_, AiState>,
    app: tauri::AppHandle,
    paths: Vec<String>,
) -> Result<Vec<AiResult>, String> {
    let mut guard = state.model.lock().map_err(|e| e.to_string())?;
    let model = guard.as_mut().ok_or("AI model not initialized. Call init_ai_model first.")?;

    // Load user feedback to know which images are explicitly marked
    let fb_path = feedback_path(&app).unwrap_or_default();
    let feedback_samples = load_feedback_samples(&fb_path);
    
    // Build a map: embedding similarity -> label for quick lookup
    // We'll check each image's embedding against feedback embeddings
    
    // Check if custom classifier is available
    let weights = {
        let w_guard = state.classifier_weights.lock().map_err(|e| e.to_string())?;
        w_guard.clone()
    };

    let mut results = Vec::with_capacity(paths.len());
    for path_str in &paths {
        let path = Path::new(path_str);
        
        // Extract embedding once for both feedback check and analysis
        let embedding = extract_embedding(model, path).unwrap_or_default();
        
        // Check if user has explicitly marked this image
        let user_marked = if !embedding.is_empty() && !feedback_samples.is_empty() {
            let mut best_match: Option<bool> = None;
            let mut best_sim = 0.0f32;
            for sample in &feedback_samples {
                if sample.embedding.len() != embedding.len() { continue; }
                let dot: f32 = sample.embedding.iter().zip(embedding.iter())
                    .map(|(a, b)| a * b).sum();
                let norm_a: f32 = sample.embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                let norm_b: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                let sim = if norm_a > 0.0 && norm_b > 0.0 { dot / (norm_a * norm_b) } else { 0.0 };
                if sim > best_sim {
                    best_sim = sim;
                    best_match = Some(sample.label);
                }
            }
            if best_sim >= 0.99 { best_match } else { None }
        } else {
            None
        };

        // If user explicitly marked this image, use their judgment directly
        if let Some(is_waste) = user_marked {
            results.push(AiResult {
                path: path_str.clone(),
                labels: if is_waste { vec!["user-marked".to_string()] } else { vec![] },
                scores: vec![],
                is_waste,
            });
            continue;
        }

        // Otherwise run model analysis
        match model.analyze_image(path) {
            Ok(mut result) => {
                if let Some(ref w) = weights {
                    if !embedding.is_empty() {
                        let clf_prob = predict_waste(&embedding, w);
                        let zs_waste: f32 = result.scores.iter().enumerate()
                            .filter(|(i, _)| LABELS[*i] != "normal")
                            .map(|(_, s)| *s)
                            .fold(0.0f32, |a, b| a.max(b));
                        let sample_count = {
                            let cg = state.feedback_count.lock().unwrap_or_else(|e| e.into_inner());
                            *cg as f32
                        };
                        let alpha = (sample_count / 50.0).min(0.9).max(0.1);
                        let fused = alpha * clf_prob + (1.0 - alpha) * zs_waste;
                        result.is_waste = fused > 0.5;
                        if result.is_waste {
                            if clf_prob > 0.5 && zs_waste <= 0.5 {
                                result.labels = vec!["learned".to_string()];
                            } else if clf_prob <= 0.5 && zs_waste > 0.5 {
                                // Keep zero-shot labels
                            } else {
                                result.labels.push("learned".to_string());
                            }
                        } else {
                            result.labels.clear();
                        }
                    }
                }
                results.push(result);
            }
            Err(e) => {
                eprintln!("AI analysis failed for {}: {}", path_str, e);
            }
        }
    }

    Ok(results)
}


/// A single training sample: CLIP embedding + user label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSample {
    pub embedding: Vec<f32>,
    pub label: bool,
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String { "manual".to_string() }

/// Get the feedback file path
fn feedback_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    Ok(data_dir.join("feedback.jsonl"))
}

/// Get the classifier weights file path
fn weights_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    Ok(data_dir.join("classifier_weights.bin"))
}

/// Load all training samples from feedback.jsonl
fn load_feedback_samples(path: &Path) -> Vec<TrainingSample> {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    let reader = std::io::BufReader::new(file);
    let mut samples = Vec::new();
    for line in reader.lines() {
        if let Ok(line) = line {
            if let Ok(sample) = serde_json::from_str::<TrainingSample>(&line) {
                samples.push(sample);
            }
        }
    }
    samples
}

/// Train a logistic regression classifier on collected feedback
fn train_classifier(samples: &[TrainingSample]) -> Vec<f32> {
    let dim = 512usize;
    // Only use manual feedback for waste classifier training
    let manual_samples: Vec<&TrainingSample> = samples.iter()
        .filter(|s| s.source == "manual")
        .collect();
    let n = manual_samples.len();
    if n == 0 {
        return vec![0.0; dim + 1]; // weights + bias
    }

    // Initialize weights to zero
    let mut weights = vec![0.0f32; dim];
    let mut bias = 0.0f32;
    let lr = 0.01f32;
    let epochs = 50usize;

    for _epoch in 0..epochs {
        for sample in &manual_samples {
            // Forward pass: sigmoid(w . x + b)
            let z: f32 = weights.iter().zip(sample.embedding.iter())
                .map(|(w, x)| w * x).sum::<f32>() + bias;
            let pred = 1.0 / (1.0 + (-z).exp());

            // BCE gradient
            let target = if sample.label { 1.0f32 } else { 0.0f32 };
            let error = pred - target;

            // SGD update
            for (w, x) in weights.iter_mut().zip(sample.embedding.iter()) {
                *w -= lr * error * x;
            }
            bias -= lr * error;
        }
    }

    // Return weights + bias as single vec
    weights.push(bias);
    weights
}

/// Predict using trained classifier
fn predict_waste(embedding: &[f32], weights: &[f32]) -> f32 {
    let dim = embedding.len();
    if weights.len() != dim + 1 {
        return 0.5; // fallback
    }
    let z: f32 = weights[..dim].iter().zip(embedding.iter())
        .map(|(w, x)| w * x).sum::<f32>() + weights[dim];
    1.0 / (1.0 + (-z).exp())
}

/// Extract CLIP embedding for an image (without classification)
fn extract_embedding(model: &mut AiModel, image_path: &Path) -> Result<Vec<f32>, String> {
    let rgb_img = load_image_as_rgb(image_path)?;
    let dyn_img = image::DynamicImage::ImageRgb8(rgb_img);
    let resized = dyn_img.resize_exact(224, 224, image::imageops::FilterType::Triangle);
    let rgb = resized.to_rgb8();

    let mean = [0.48145466f32, 0.4578275, 0.40821073];
    let std_dev = [0.26862954f32, 0.26130258, 0.27577711];

    let mut data = vec![0.0f32; 3 * 224 * 224];
    for y in 0..224u32 {
        for x in 0..224u32 {
            let pixel = rgb.get_pixel(x, y);
            for ch in 0..3usize {
                let val = pixel[ch] as f32 / 255.0;
                data[ch * 224 * 224 + y as usize * 224 + x as usize] =
                    (val - mean[ch]) / std_dev[ch];
            }
        }
    }

    let input_array = Array4::from_shape_vec((1, 3, 224, 224), data)
        .map_err(|e| format!("Failed to create input tensor: {}", e))?;
    let input_value = Tensor::from_array(input_array)
        .map_err(|e| format!("Failed to create ORT tensor: {}", e))?;

    let outputs = model.session
        .run(ort::inputs!["image" => input_value])
        .map_err(|e| format!("ONNX inference failed: {}", e))?;

    let (_shape, emb_slice) = outputs["embedding"]
        .try_extract_tensor::<f32>()
        .map_err(|e| format!("Failed to extract embedding: {}", e))?;

    let norm: f32 = emb_slice.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        Ok(emb_slice.iter().map(|x| x / norm).collect())
    } else {
        Ok(vec![0.0; emb_slice.len()])
    }
}

#[tauri::command]
pub async fn mark_image_feedback(
    app: tauri::AppHandle,
    state: tauri::State<'_, AiState>,
    path: String,
    is_waste: bool,
) -> Result<u32, String> {
    // Extract embedding
    let embedding = {
        let mut guard = state.model.lock().map_err(|e| e.to_string())?;
        let model = guard.as_mut().ok_or("AI model not initialized")?;
        extract_embedding(model, Path::new(&path))?
    };

    let sample = TrainingSample {
        embedding: embedding.clone(),
        label: is_waste,
        source: "manual".to_string(),
    };

    // Read existing samples, deduplicate by embedding similarity (cosine > 0.999 = same image)
    let fb_path = feedback_path(&app)?;
    let mut samples = load_feedback_samples(&fb_path);
    samples.retain(|s| {
        if s.embedding.len() != embedding.len() { return true; }
        let dot: f32 = s.embedding.iter().zip(embedding.iter()).map(|(a, b)| a * b).sum();
        let norm_a: f32 = s.embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        let sim = if norm_a > 0.0 && norm_b > 0.0 { dot / (norm_a * norm_b) } else { 0.0 };
        sim < 0.99 // keep if NOT the same image
    });
    samples.push(sample);

    // Rewrite entire file (deduped)
    if let Some(parent) = fb_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let mut file = fs::File::create(&fb_path)
        .map_err(|e| format!("Failed to open feedback file: {}", e))?;
    for s in &samples {
        let json = serde_json::to_string(s)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        writeln!(file, "{}", json)
            .map_err(|e| format!("Failed to write: {}", e))?;
    }

    let count = samples.len() as u32;
    let mut count_guard = state.feedback_count.lock().map_err(|e| e.to_string())?;
    *count_guard = count;

    // Auto-retrain if >= 20 samples
    {
        let new_weights = train_classifier(&samples);
        let mut w_guard = state.classifier_weights.lock().map_err(|e| e.to_string())?;
        *w_guard = Some(new_weights.clone());

        if let Ok(wp) = weights_path(&app) {
            let bytes: Vec<u8> = new_weights.iter()
                .flat_map(|f| f.to_le_bytes())
                .collect();
            fs::write(&wp, &bytes).ok();
        }
        eprintln!("[AI] Classifier retrained with {} samples", count);
    }

    Ok(count)
}

#[tauri::command]
pub async fn retrain_classifier(
    app: tauri::AppHandle,
    state: tauri::State<'_, AiState>,
) -> Result<u32, String> {
    let fb_path = feedback_path(&app)?;
    let samples = load_feedback_samples(&fb_path);
    if samples.is_empty() {
        return Err("No feedback samples available".to_string());
    }

    let new_weights = train_classifier(&samples);
    let mut w_guard = state.classifier_weights.lock().map_err(|e| e.to_string())?;
    *w_guard = Some(new_weights.clone());

    // Save weights to disk
    let wp = weights_path(&app)?;
    let bytes: Vec<u8> = new_weights.iter()
        .flat_map(|f| f.to_le_bytes())
        .collect();
    fs::write(&wp, &bytes)
        .map_err(|e| format!("Failed to save weights: {}", e))?;

    Ok(samples.len() as u32)
}

#[tauri::command]
pub fn get_feedback_count(
    state: tauri::State<'_, AiState>,
) -> u32 {
    *state.feedback_count.lock().unwrap_or_else(|e| e.into_inner())
}



/// Compute image quality score: sharpness (Laplacian variance) + exposure balance
fn compute_quality_score(image_path: &Path) -> Result<f64, String> {
    let rgb = load_image_as_rgb(image_path)?;
    let (w, h) = (rgb.width(), rgb.height());
    
    // Sharpness: Laplacian variance on grayscale
    let gray: Vec<f64> = rgb.pixels().map(|p| {
        0.299 * p[0] as f64 + 0.587 * p[1] as f64 + 0.114 * p[2] as f64
    }).collect();
    
    let mut lap_sum = 0.0f64;
    let mut lap_sq_sum = 0.0f64;
    let mut count = 0u64;
    
    for y in 1..(h.saturating_sub(1)) {
        for x in 1..(w.saturating_sub(1)) {
            let idx = (y * w + x) as usize;
            let up = ((y - 1) * w + x) as usize;
            let down = ((y + 1) * w + x) as usize;
            let left = (y * w + x - 1) as usize;
            let right = (y * w + x + 1) as usize;
            
            let lap = gray[up] + gray[down] + gray[left] + gray[right] - 4.0 * gray[idx];
            lap_sum += lap;
            lap_sq_sum += lap * lap;
            count += 1;
        }
    }
    
    let sharpness = if count > 0 {
        let mean = lap_sum / count as f64;
        (lap_sq_sum / count as f64 - mean * mean).max(0.0)
    } else {
        0.0
    };
    
    // Exposure: histogram std dev of luminance (higher = better dynamic range)
    let mut hist = [0u64; 256];
    for &g in &gray {
        let bin = (g.clamp(0.0, 255.0)) as usize;
        hist[bin] += 1;
    }
    let total = gray.len() as f64;
    let hist_mean: f64 = hist.iter().enumerate().map(|(i, &c)| i as f64 * c as f64).sum::<f64>() / total;
    let hist_var: f64 = hist.iter().enumerate()
        .map(|(i, &c)| c as f64 * (i as f64 - hist_mean).powi(2))
        .sum::<f64>() / total;
    let exposure_score = hist_var.sqrt();
    
    // Combined score (sharpness dominates)
    Ok(sharpness * 0.7 + exposure_score * 100.0 * 0.3)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub best_path: String,
    pub best_score: f64,
    pub duplicates: Vec<DuplicateEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateEntry {
    pub path: String,
    pub score: f64,
    pub similarity: f32,
}

#[tauri::command]
pub async fn detect_duplicates(
    state: tauri::State<'_, AiState>,
    paths: Vec<String>,
) -> Result<Vec<DuplicateGroup>, String> {
    let mut guard = state.model.lock().map_err(|e| e.to_string())?;
    let model = guard.as_mut().ok_or("AI model not initialized")?;

    // Extract embeddings for all images
    let mut embeddings: Vec<(String, Vec<f32>)> = Vec::with_capacity(paths.len());
    for path_str in &paths {
        match extract_embedding(model, Path::new(path_str)) {
            Ok(emb) => embeddings.push((path_str.clone(), emb)),
            Err(e) => eprintln!("[AI] Failed to extract embedding for {}: {}", path_str, e),
        }
    }

    // Cluster by cosine similarity > 0.95
    let n = embeddings.len();
    let mut visited = vec![false; n];
    let mut groups: Vec<DuplicateGroup> = Vec::new();

    for i in 0..n {
        if visited[i] { continue; }
        
        let mut cluster = vec![i];
        visited[i] = true;
        
        for j in (i+1)..n {
            if visited[j] { continue; }
            let dot: f32 = embeddings[i].1.iter().zip(embeddings[j].1.iter())
                .map(|(a, b)| a * b).sum();
            let norm_a: f32 = embeddings[i].1.iter().map(|x| x * x).sum::<f32>().sqrt();
            let norm_b: f32 = embeddings[j].1.iter().map(|x| x * x).sum::<f32>().sqrt();
            let sim = if norm_a > 0.0 && norm_b > 0.0 { dot / (norm_a * norm_b) } else { 0.0 };
            
            if sim >= 0.97 {
                cluster.push(j);
                visited[j] = true;
            }
        }
        
        // Only report groups with duplicates (size > 1)
        if cluster.len() > 1 {
            // Score each image in the group
            let mut scored: Vec<(usize, f64)> = Vec::new();
            for &idx in &cluster {
                let score = compute_quality_score(Path::new(&embeddings[idx].0)).unwrap_or(0.0);
                scored.push((idx, score));
            }
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            let best_idx = scored[0].0;
            let best_score = scored[0].1;
            
            let duplicates: Vec<DuplicateEntry> = scored[1..].iter().map(|&(idx, score)| {
                // Compute similarity to best
                let dot: f32 = embeddings[best_idx].1.iter().zip(embeddings[idx].1.iter())
                    .map(|(a, b)| a * b).sum();
                let norm_a: f32 = embeddings[best_idx].1.iter().map(|x| x * x).sum::<f32>().sqrt();
                let norm_b: f32 = embeddings[idx].1.iter().map(|x| x * x).sum::<f32>().sqrt();
                let sim = if norm_a > 0.0 && norm_b > 0.0 { dot / (norm_a * norm_b) } else { 0.0 };
                
                DuplicateEntry {
                    path: embeddings[idx].0.clone(),
                    score,
                    similarity: sim,
                }
            }).collect();
            
            groups.push(DuplicateGroup {
                best_path: embeddings[best_idx].0.clone(),
                best_score,
                duplicates,
            });
        }
    }

    Ok(groups)
}

#[tauri::command]
pub async fn mark_duplicates_as_waste(
    app: tauri::AppHandle,
    state: tauri::State<'_, AiState>,
    duplicate_paths: Vec<String>,
) -> Result<u32, String> {
    // Mark all duplicate paths as waste with source="dedup"
    let fb_path = feedback_path(&app)?;
    let mut samples = load_feedback_samples(&fb_path);

    for path_str in &duplicate_paths {
        let path = Path::new(path_str);
        let embedding = {
            let mut guard = state.model.lock().map_err(|e| e.to_string())?;
            let model = guard.as_mut().ok_or("AI model not initialized")?;
            extract_embedding(model, path)?
        };

        // Dedup by embedding similarity
        samples.retain(|s| {
            if s.embedding.len() != embedding.len() { return true; }
            let dot: f32 = s.embedding.iter().zip(embedding.iter()).map(|(a, b)| a * b).sum();
            let norm_a: f32 = s.embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            let norm_b: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            let sim = if norm_a > 0.0 && norm_b > 0.0 { dot / (norm_a * norm_b) } else { 0.0 };
            sim < 0.99
        });

        samples.push(TrainingSample {
            embedding,
            label: true,
            source: "dedup".to_string(),
        });
    }

    // Rewrite file
    if let Some(parent) = fb_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let mut file = fs::File::create(&fb_path)
        .map_err(|e| format!("Failed to open feedback file: {}", e))?;
    for s in &samples {
        let json = serde_json::to_string(s)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        writeln!(file, "{}", json)
            .map_err(|e| format!("Failed to write: {}", e))?;
    }

    let count = samples.len() as u32;
    let mut count_guard = state.feedback_count.lock().map_err(|e| e.to_string())?;
    *count_guard = count;

    Ok(count)
}

/// Get all feedback data for frontend hydration
#[tauri::command]
pub fn get_feedback_data(
    app: tauri::AppHandle,
) -> Result<Vec<TrainingSample>, String> {
    let fb_path = feedback_path(&app)?;
    Ok(load_feedback_samples(&fb_path))
}


/// Save the model directory path to config
#[tauri::command]
pub fn save_model_path(app: tauri::AppHandle, model_dir: String) -> Result<(), String> {
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    std::fs::create_dir_all(&data_dir).ok();
    let config_path = data_dir.join("config.json");
    
    // Read existing config or create new
    let mut config: serde_json::Value = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    
    config["model_dir"] = serde_json::json!(model_dir);
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(&config_path, json)
        .map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}

/// Load the saved model directory path from config
#[tauri::command]
pub fn load_model_path(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let config_path = data_dir.join("config.json");
    
    if !config_path.exists() {
        return Ok(None);
    }
    
    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    let config: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;
    
    match config.get("model_dir").and_then(|v| v.as_str()) {
        Some(path) => {
            // Verify the path still exists
            if std::path::Path::new(path).join("clip_image_encoder.onnx").exists() {
                Ok(Some(path.to_string()))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

/// Open browser to download AI model, then user extracts to app data directory
#[tauri::command]
pub async fn download_ai_model(
    app: tauri::AppHandle,
) -> Result<String, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let model_dir = data_dir.join("ai-models");

    // If model already exists, return path
    if model_dir.join("clip_image_encoder.onnx").exists()
        && model_dir.join("text_embeddings.bin").exists()
    {
        return Ok(model_dir.to_string_lossy().to_string());
    }

    // Open browser for manual download
    let url = "https://nas.hosee.icu:8888/sharing/VVLuOtym3";
    open::that(url).map_err(|e| format!("Failed to open browser: {}", e))?;

    Err(format!(
        "请在浏览器中下载模型文件并解压到: {}",
        model_dir.to_string_lossy()
    ))
}

/// Get the expected model directory path
#[tauri::command]
pub fn get_ai_model_dir(app: tauri::AppHandle) -> Result<String, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    Ok(data_dir.join("ai-models").to_string_lossy().to_string())
}

/// Check if AI model files exist
#[tauri::command]
pub fn check_ai_model_exists(app: tauri::AppHandle) -> bool {
    let data_dir = match app.path().app_data_dir() {
        Ok(d) => d,
        Err(_) => return false,
    };
    let model_dir = data_dir.join("ai-models");
    model_dir.join("clip_image_encoder.onnx").exists()
        && model_dir.join("text_embeddings.bin").exists()
}

/// Extract AI model zip to app data directory
#[tauri::command]
pub async fn extract_ai_model_zip(
    app: tauri::AppHandle,
    zip_path: String,
) -> Result<String, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let model_dir = data_dir.join("ai-models");
    std::fs::create_dir_all(&model_dir)
        .map_err(|e| format!("Failed to create model dir: {}", e))?;

    let file = std::fs::File::open(&zip_path)
        .map_err(|e| format!("Failed to open zip: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read zip: {}", e))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;
        let name = entry.name().to_string();
        if name.ends_with('/') || name.starts_with("__MACOSX") { continue; }
        let out_name = if let Some(pos) = name.find('/') {
            &name[pos + 1..]
        } else {
            &name
        };
        if out_name.is_empty() { continue; }
        let out_path = model_dir.join(out_name);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let mut out_file = std::fs::File::create(&out_path)
            .map_err(|e| format!("Failed to create {}: {}", out_name, e))?;
        std::io::copy(&mut entry, &mut out_file)
            .map_err(|e| format!("Failed to extract {}: {}", out_name, e))?;
    }

    Ok(model_dir.to_string_lossy().to_string())
}
