use image::RgbImage;
use serde::{Deserialize, Serialize};

/// 图像质量特征 (20维)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityFeatures {
    pub sharpness: f32,
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub noise_level: f32,
    pub edge_density: f32,
    pub color_richness: f32,
    pub local_contrast: f32,
    pub blur_score: f32,
    pub overexposed_ratio: f32,
    pub underexposed_ratio: f32,
    pub texture_complexity: f32,
    pub color_balance: f32,
    pub gradient_strength: f32,
    pub dynamic_range: f32,
    pub clarity: f32,
    pub detail_level: f32,
    pub smoothness: f32,
    pub sharpness_uniformity: f32,
    pub exposure_quality: f32,
}

impl QualityFeatures {
    pub fn to_vec(&self) -> Vec<f32> {
        vec![
            self.sharpness, self.brightness, self.contrast, self.saturation,
            self.noise_level, self.edge_density, self.color_richness, self.local_contrast,
            self.blur_score, self.overexposed_ratio, self.underexposed_ratio,
            self.texture_complexity, self.color_balance, self.gradient_strength,
            self.dynamic_range, self.clarity, self.detail_level, self.smoothness,
            self.sharpness_uniformity, self.exposure_quality,
        ]
    }

    pub fn dimension() -> usize { 20 }
}

/// 提取图像质量特征
pub fn extract_quality_features(img: &RgbImage) -> QualityFeatures {
    let pixels: Vec<_> = img.pixels().collect();

    let sharpness = calculate_laplacian_variance(img);

    let brightness: f32 = pixels.iter()
        .map(|p| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32)
        .sum::<f32>() / pixels.len() as f32 / 255.0;

    let mean_brightness = brightness * 255.0;
    let contrast = (pixels.iter()
        .map(|p| {
            let lum = 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
            (lum - mean_brightness).powi(2)
        })
        .sum::<f32>() / pixels.len() as f32).sqrt() / 128.0;

    let saturation = pixels.iter()
        .map(|p| {
            let max = p[0].max(p[1]).max(p[2]) as f32;
            let min = p[0].min(p[1]).min(p[2]) as f32;
            if max > 0.0 { (max - min) / max } else { 0.0 }
        })
        .sum::<f32>() / pixels.len() as f32;

    let noise_level = calculate_noise_level(img);
    let edge_density = calculate_edge_density(img);
    let color_richness = calculate_color_richness(img);
    let local_contrast = calculate_local_contrast(img);
    let blur_score = 1.0 - sharpness.min(1.0);
    let (overexposed_ratio, underexposed_ratio) = calculate_exposure_ratios(img);
    let texture_complexity = calculate_texture_complexity(img);
    let color_balance = calculate_color_balance(img);
    let gradient_strength = calculate_gradient_strength(img);
    let dynamic_range = calculate_dynamic_range(img);
    let clarity = calculate_clarity(img);
    let detail_level = calculate_detail_level(img);
    let smoothness = calculate_smoothness(img);
    let sharpness_uniformity = calculate_sharpness_uniformity(img);
    let exposure_quality = calculate_exposure_quality(img);

    QualityFeatures {
        sharpness: sharpness.min(1.0),
        brightness,
        contrast: contrast.min(1.0),
        saturation,
        noise_level,
        edge_density,
        color_richness,
        local_contrast,
        blur_score,
        overexposed_ratio,
        underexposed_ratio,
        texture_complexity,
        color_balance,
        gradient_strength,
        dynamic_range,
        clarity,
        detail_level,
        smoothness,
        sharpness_uniformity,
        exposure_quality,
    }
}

/// 计算区域的 Laplacian 方差（用于锐度评估）
pub fn calculate_laplacian_variance(img: &RgbImage) -> f32 {
    let gray = image::DynamicImage::ImageRgb8(img.clone()).to_luma8();
    let (w, h) = gray.dimensions();

    if w < 3 || h < 3 { return 0.0; }

    let mut sum = 0.0f64;
    let mut sum_sq = 0.0f64;
    let mut count = 0u64;

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let center = gray.get_pixel(x, y)[0] as f64;
            let laplacian =
                gray.get_pixel(x - 1, y)[0] as f64 +
                gray.get_pixel(x + 1, y)[0] as f64 +
                gray.get_pixel(x, y - 1)[0] as f64 +
                gray.get_pixel(x, y + 1)[0] as f64 -
                4.0 * center;

            sum += laplacian;
            sum_sq += laplacian * laplacian;
            count += 1;
        }
    }

    if count == 0 { return 0.0; }
    let mean = sum / count as f64;
    let variance = (sum_sq / count as f64) - (mean * mean);
    (variance.sqrt() / 255.0) as f32
}

/// 计算噪点水平
fn calculate_noise_level(img: &RgbImage) -> f32 {
    let gray = image::DynamicImage::ImageRgb8(img.clone()).to_luma8();
    let (w, h) = gray.dimensions();

    if w < 3 || h < 3 { return 0.0; }

    let mut high_freq_sum = 0.0f64;
    let mut count = 0u64;

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let center = gray.get_pixel(x, y)[0] as f64;
            let avg_neighbor = (
                gray.get_pixel(x - 1, y)[0] as f64 +
                gray.get_pixel(x + 1, y)[0] as f64 +
                gray.get_pixel(x, y - 1)[0] as f64 +
                gray.get_pixel(x, y + 1)[0] as f64
            ) / 4.0;
            let diff = (center - avg_neighbor).abs();

            if diff > 10.0 {
                high_freq_sum += diff;
                count += 1;
            }
        }
    }

    if count == 0 { return 0.0; }
    ((high_freq_sum / count as f64) / 255.0) as f32
}

/// 计算边缘密度
fn calculate_edge_density(img: &RgbImage) -> f32 {
    let gray = image::DynamicImage::ImageRgb8(img.clone()).to_luma8();
    let (w, h) = gray.dimensions();

    if w < 3 || h < 3 { return 0.0; }

    let mut edge_count = 0u64;
    let threshold = 30.0;

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let gx = (gray.get_pixel(x + 1, y)[0] as f64 - gray.get_pixel(x - 1, y)[0] as f64).abs();
            let gy = (gray.get_pixel(x, y + 1)[0] as f64 - gray.get_pixel(x, y - 1)[0] as f64).abs();
            let gradient = (gx * gx + gy * gy).sqrt();

            if gradient > threshold {
                edge_count += 1;
            }
        }
    }

    let total_pixels = ((w - 2) * (h - 2)) as f32;
    (edge_count as f32 / total_pixels).min(1.0)
}

/// 计算颜色丰富度
fn calculate_color_richness(img: &RgbImage) -> f32 {
    use std::collections::HashSet;

    let mut color_set = HashSet::new();

    for p in img.pixels() {
        let key = (p[0] / 16, p[1] / 16, p[2] / 16);
        color_set.insert(key);
    }

    let unique_colors = color_set.len() as f32;
    let max_colors = 16.0 * 16.0 * 16.0;

    (unique_colors / max_colors).min(1.0)
}

/// 计算局部对比度
fn calculate_local_contrast(img: &RgbImage) -> f32 {
    let gray = image::DynamicImage::ImageRgb8(img.clone()).to_luma8();
    let (w, h) = gray.dimensions();

    if w < 16 || h < 16 { return 0.0; }

    let mut contrast_sum = 0.0f32;
    let mut count = 0u32;

    for y in (0..h).step_by(16) {
        for x in (0..w).step_by(16) {
            let mut min_val = 255u8;
            let mut max_val = 0u8;

            for dy in 0..16.min(h - y) {
                for dx in 0..16.min(w - x) {
                    let val = gray.get_pixel(x + dx, y + dy)[0];
                    min_val = min_val.min(val);
                    max_val = max_val.max(val);
                }
            }

            contrast_sum += (max_val - min_val) as f32 / 255.0;
            count += 1;
        }
    }

    if count == 0 { return 0.0; }
    contrast_sum / count as f32
}

/// 计算曝光比例 (过曝, 欠曝)
fn calculate_exposure_ratios(img: &RgbImage) -> (f32, f32) {
    let total = img.pixels().len() as f32;

    let mut overexposed = 0u32;
    let mut underexposed = 0u32;

    for p in img.pixels() {
        let lum = 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
        if lum > 240.0 { overexposed += 1; }
        if lum < 15.0 { underexposed += 1; }
    }

    (overexposed as f32 / total, underexposed as f32 / total)
}

/// 计算纹理复杂度
fn calculate_texture_complexity(img: &RgbImage) -> f32 {
    let gray = image::DynamicImage::ImageRgb8(img.clone()).to_luma8();
    let (w, h) = gray.dimensions();

    if w < 3 || h < 3 { return 0.0; }

    let mut complexity = 0.0f32;

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let center = gray.get_pixel(x, y)[0] as f32;
            let neighbors = [
                gray.get_pixel(x - 1, y - 1)[0] as f32,
                gray.get_pixel(x, y - 1)[0] as f32,
                gray.get_pixel(x + 1, y - 1)[0] as f32,
                gray.get_pixel(x - 1, y)[0] as f32,
                gray.get_pixel(x + 1, y)[0] as f32,
                gray.get_pixel(x - 1, y + 1)[0] as f32,
                gray.get_pixel(x, y + 1)[0] as f32,
                gray.get_pixel(x + 1, y + 1)[0] as f32,
            ];

            let avg: f32 = neighbors.iter().sum::<f32>() / 8.0;
            complexity += (center - avg).abs();
        }
    }

    let total_pixels = ((w - 2) * (h - 2)) as f32;
    (complexity / total_pixels / 128.0).min(1.0)
}

/// 计算颜色平衡
fn calculate_color_balance(img: &RgbImage) -> f32 {
    let total = img.pixels().len() as f32;

    let r_sum: f32 = img.pixels().map(|p| p[0] as f32).sum();
    let g_sum: f32 = img.pixels().map(|p| p[1] as f32).sum();
    let b_sum: f32 = img.pixels().map(|p| p[2] as f32).sum();

    let r_avg = r_sum / total;
    let g_avg = g_sum / total;
    let b_avg = b_sum / total;

    let overall_avg = (r_avg + g_avg + b_avg) / 3.0;
    let deviation = ((r_avg - overall_avg).abs() +
                     (g_avg - overall_avg).abs() +
                     (b_avg - overall_avg).abs()) / 3.0;

    1.0 - (deviation / 128.0).min(1.0)
}

/// 计算梯度强度
fn calculate_gradient_strength(img: &RgbImage) -> f32 {
    let gray = image::DynamicImage::ImageRgb8(img.clone()).to_luma8();
    let (w, h) = gray.dimensions();

    if w < 2 || h < 2 { return 0.0; }

    let mut gradient_sum = 0.0f32;

    for y in 0..h - 1 {
        for x in 0..w - 1 {
            let gx = (gray.get_pixel(x + 1, y)[0] as f32 - gray.get_pixel(x, y)[0] as f32).abs();
            let gy = (gray.get_pixel(x, y + 1)[0] as f32 - gray.get_pixel(x, y)[0] as f32).abs();
            gradient_sum += (gx + gy) / 2.0;
        }
    }

    let total_pixels = ((w - 1) * (h - 1)) as f32;
    (gradient_sum / total_pixels / 128.0).min(1.0)
}

/// 计算动态范围
fn calculate_dynamic_range(img: &RgbImage) -> f32 {
    let gray = image::DynamicImage::ImageRgb8(img.clone()).to_luma8();

    let min_val = gray.pixels().map(|p| p[0]).min().unwrap_or(0) as f32;
    let max_val = gray.pixels().map(|p| p[0]).max().unwrap_or(255) as f32;

    ((max_val - min_val) / 255.0).min(1.0)
}

/// 计算清晰度
fn calculate_clarity(img: &RgbImage) -> f32 {
    let gray = image::DynamicImage::ImageRgb8(img.clone()).to_luma8();
    let (w, h) = gray.dimensions();

    if w < 3 || h < 3 { return 0.0; }

    let mut clarity = 0.0f32;

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let center = gray.get_pixel(x, y)[0] as f32;
            let max_diff = [
                gray.get_pixel(x - 1, y)[0] as f32,
                gray.get_pixel(x + 1, y)[0] as f32,
                gray.get_pixel(x, y - 1)[0] as f32,
                gray.get_pixel(x, y + 1)[0] as f32,
            ].iter().map(|&n| (center - n).abs()).fold(0.0f32, |a, b| a.max(b));

            clarity += max_diff;
        }
    }

    let total_pixels = ((w - 2) * (h - 2)) as f32;
    (clarity / total_pixels / 128.0).min(1.0)
}

/// 计算细节水平
fn calculate_detail_level(img: &RgbImage) -> f32 {
    let gray = image::DynamicImage::ImageRgb8(img.clone()).to_luma8();
    let (w, h) = gray.dimensions();

    if w < 4 || h < 4 { return 0.0; }

    let mut detail = 0.0f32;

    for y in 2..h - 2 {
        for x in 2..w - 2 {
            let center = gray.get_pixel(x, y)[0] as f32;

            for dy in -2..=2 {
                for dx in -2..=2 {
                    if dx == 0 && dy == 0 { continue; }
                    let nx = (x as i32 + dx) as u32;
                    let ny = (y as i32 + dy) as u32;
                    let neighbor = gray.get_pixel(nx, ny)[0] as f32;
                    detail += (center - neighbor).abs();
                }
            }
        }
    }

    let total_pixels = ((w - 4) * (h - 4)) as f32;
    (detail / total_pixels / 24.0 / 128.0).min(1.0)
}

/// 计算平滑度
fn calculate_smoothness(img: &RgbImage) -> f32 {
    let gray = image::DynamicImage::ImageRgb8(img.clone()).to_luma8();
    let (w, h) = gray.dimensions();

    if w < 3 || h < 3 { return 0.0; }

    let mut smooth_count = 0u64;
    let threshold = 10.0;

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let center = gray.get_pixel(x, y)[0] as f32;
            let avg = (
                gray.get_pixel(x - 1, y)[0] as f32 +
                gray.get_pixel(x + 1, y)[0] as f32 +
                gray.get_pixel(x, y - 1)[0] as f32 +
                gray.get_pixel(x, y + 1)[0] as f32
            ) / 4.0;

            if (center - avg).abs() < threshold {
                smooth_count += 1;
            }
        }
    }

    let total_pixels = ((w - 2) * (h - 2)) as f32;
    smooth_count as f32 / total_pixels
}

/// 计算锐度均匀性
fn calculate_sharpness_uniformity(img: &RgbImage) -> f32 {
    let gray = image::DynamicImage::ImageRgb8(img.clone()).to_luma8();
    let (w, h) = gray.dimensions();

    if w < 16 || h < 16 { return 0.0; }

    let block_size = 16;
    let mut sharpness_values = Vec::new();

    for y in (0..h).step_by(block_size as usize) {
        for x in (0..w).step_by(block_size as usize) {
            let bw = block_size.min(w - x);
            let bh = block_size.min(h - y);

            if bw < 3 || bh < 3 { continue; }

            let mut variance = 0.0f64;
            let mut count = 0u64;

            for dy in 1..bh - 1 {
                for dx in 1..bw - 1 {
                    let center = gray.get_pixel(x + dx, y + dy)[0] as f64;
                    let laplacian =
                        gray.get_pixel(x + dx - 1, y + dy)[0] as f64 +
                        gray.get_pixel(x + dx + 1, y + dy)[0] as f64 +
                        gray.get_pixel(x + dx, y + dy - 1)[0] as f64 +
                        gray.get_pixel(x + dx, y + dy + 1)[0] as f64 -
                        4.0 * center;

                    variance += laplacian * laplacian;
                    count += 1;
                }
            }

            if count > 0 {
                sharpness_values.push((variance / count as f64).sqrt());
            }
        }
    }

    if sharpness_values.len() < 2 { return 0.0; }

    let mean: f64 = sharpness_values.iter().sum::<f64>() / sharpness_values.len() as f64;
    let variance: f64 = sharpness_values.iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>() / sharpness_values.len() as f64;

    let cv = variance.sqrt() / mean;
    (1.0 - cv.min(1.0)) as f32
}

/// 计算曝光质量
fn calculate_exposure_quality(img: &RgbImage) -> f32 {
    let brightness: f32 = img.pixels()
        .map(|p| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32)
        .sum::<f32>() / img.pixels().len() as f32;

    let total = img.pixels().len() as f32;
    let overexposed = img.pixels()
        .filter(|p| {
            let lum = 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
            lum > 240.0
        })
        .count() as f32 / total;

    let underexposed = img.pixels()
        .filter(|p| {
            let lum = 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
            lum < 15.0
        })
        .count() as f32 / total;

    let brightness_score = 1.0 - ((brightness - 128.0).abs() / 128.0);
    let exposure_penalty = (overexposed + underexposed).min(1.0);

    brightness_score * (1.0 - exposure_penalty * 0.5)
}

/// 计算皮肤区域曝光比例（婚礼人像专用）
pub fn calculate_skin_exposure(img: &RgbImage) -> f32 {
    let total = img.pixels().len() as f32;
    if total == 0.0 { return 0.0; }

    let mut skin_overexposed = 0u32;
    let mut skin_pixels = 0u32;

    for p in img.pixels() {
        let r = p[0] as f32;
        let g = p[1] as f32;
        let b = p[2] as f32;

        let is_skin = r > 95.0 && g > 40.0 && b > 20.0
            && r > g && r > b
            && (r - g).abs() > 15.0
            && r - b > 15.0;

        if is_skin {
            skin_pixels += 1;
            let lum = 0.299 * r + 0.587 * g + 0.114 * b;
            if lum > 230.0 {
                skin_overexposed += 1;
            }
        }
    }

    if skin_pixels == 0 { return 0.0; }
    skin_overexposed as f32 / skin_pixels as f32
}

/// 计算图像质量综合分数 (0-10)
pub fn calculate_quality_score(features: &QualityFeatures) -> f32 {
    (features.sharpness * 2.0
        + features.contrast * 2.0
        + features.color_balance * 1.5
        + features.exposure_quality * 2.0
        + features.dynamic_range * 1.5
        + (1.0 - features.noise_level) * 1.0) * 5.0 / 10.0
}
