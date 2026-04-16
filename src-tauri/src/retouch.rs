use image::{DynamicImage, Rgb, RgbImage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// UIが表示するカテゴリタブ。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetouchStrategyDefinition {
    pub id: String,
    pub label: String,
    pub description: String,
    pub tab: String,
    pub family: String,
    pub parameters: Vec<StrategyParameterDefinition>,
}

/// 各手法のパラメータ定義（スライダー描画用）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyParameterDefinition {
    pub key: String,
    pub label: String,
    pub description: String,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub default_value: f32,
}

/// フロントから受け取る処理リクエスト。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyRetouchRequest {
    pub input_path: String,
    pub strategy_id: String,
    pub params: HashMap<String, f32>,
}

/// 手法適用後にフロントへ返す処理結果。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyRetouchResponse {
    pub output_path: String,
    pub elapsed_ms: u128,
    pub applied_params: HashMap<String, f32>,
    pub model_info: Option<String>,
}

/// UI表示用に固定の戦略一覧を返す。
pub fn list_strategies() -> Vec<RetouchStrategyDefinition> {
    vec![
        RetouchStrategyDefinition {
            id: "classic-auto-balance".to_string(),
            label: "Auto Balance".to_string(),
            description: "Gray-worldホワイトバランスと軽いコントラスト補正。".to_string(),
            tab: "classic".to_string(),
            family: "classic".to_string(),
            parameters: vec![
                param(
                    "whiteBalanceStrength",
                    "WB Strength",
                    "ホワイトバランス補正の強さ",
                    0.0,
                    1.0,
                    0.01,
                    0.72,
                ),
                param(
                    "contrastBoost",
                    "Contrast Boost",
                    "最終コントラスト増加量",
                    0.0,
                    0.5,
                    0.01,
                    0.18,
                ),
            ],
        },
        RetouchStrategyDefinition {
            id: "classic-vibrance-boost".to_string(),
            label: "Vibrance Boost".to_string(),
            description: "彩度と微コントラストを同時に持ち上げる。".to_string(),
            tab: "classic".to_string(),
            family: "classic".to_string(),
            parameters: vec![
                param(
                    "vibrance",
                    "Vibrance",
                    "彩度ブースト量",
                    0.0,
                    0.8,
                    0.01,
                    0.26,
                ),
                param(
                    "contrastBoost",
                    "Contrast Boost",
                    "局所の見えを整える軽いコントラスト調整",
                    0.0,
                    0.5,
                    0.01,
                    0.10,
                ),
            ],
        },
        RetouchStrategyDefinition {
            id: "classic-highlight-recovery".to_string(),
            label: "Highlight Recovery".to_string(),
            description: "ハイライトを圧縮して白飛びを抑える。".to_string(),
            tab: "classic".to_string(),
            family: "classic".to_string(),
            parameters: vec![
                param(
                    "highlightThreshold",
                    "Highlight Threshold",
                    "圧縮を開始する明るさ閾値",
                    120.0,
                    250.0,
                    1.0,
                    198.0,
                ),
                param(
                    "recoveryStrength",
                    "Recovery Strength",
                    "ハイライト圧縮の強さ",
                    0.0,
                    1.0,
                    0.01,
                    0.56,
                ),
            ],
        },
        RetouchStrategyDefinition {
            id: "classic-gamma-lift".to_string(),
            label: "Gamma Lift".to_string(),
            description: "中間調を中心にトーンバランスを調整。".to_string(),
            tab: "classic".to_string(),
            family: "classic".to_string(),
            parameters: vec![param(
                "gamma",
                "Gamma",
                "ガンマ値（1.0より大きいほど持ち上げ）",
                0.6,
                1.8,
                0.01,
                1.20,
            )],
        },
        RetouchStrategyDefinition {
            id: "classic-local-clarity".to_string(),
            label: "Local Clarity".to_string(),
            description: "アンシャープマスクで局所的な明瞭感を強化。".to_string(),
            tab: "classic".to_string(),
            family: "classic".to_string(),
            parameters: vec![
                param(
                    "clarityAmount",
                    "Clarity Amount",
                    "明瞭感強調量",
                    0.0,
                    2.0,
                    0.01,
                    0.95,
                ),
                param(
                    "blurSigma",
                    "Blur Sigma",
                    "明瞭化に使うぼかし半径",
                    0.4,
                    4.0,
                    0.01,
                    1.60,
                ),
            ],
        },
        RetouchStrategyDefinition {
            id: "ai-kmeans-scene-model".to_string(),
            label: "AI Scene Model (K-Means)".to_string(),
            description: "色分布クラスタリングからシーン特性を推定して補正。".to_string(),
            tab: "ai".to_string(),
            family: "ai".to_string(),
            parameters: vec![param(
                "sceneStrength",
                "Scene Strength",
                "シーン推定補正の反映率",
                0.0,
                1.0,
                0.01,
                0.74,
            )],
        },
        RetouchStrategyDefinition {
            id: "hybrid-skin-aware-tone".to_string(),
            label: "Hybrid Skin-Aware Tone".to_string(),
            description: "肌色検知を使ってトーン調整を局所適用。".to_string(),
            tab: "ai".to_string(),
            family: "hybrid".to_string(),
            parameters: vec![param(
                "skinToneStrength",
                "Skin Tone Strength",
                "肌領域への補正強度",
                0.0,
                1.0,
                0.01,
                0.62,
            )],
        },
        RetouchStrategyDefinition {
            id: "hybrid-subject-pop".to_string(),
            label: "Hybrid Subject Pop".to_string(),
            description: "被写体らしい領域を推定し、彩度/コントラストを重点補正。".to_string(),
            tab: "ai".to_string(),
            family: "hybrid".to_string(),
            parameters: vec![param(
                "subjectPopStrength",
                "Subject Pop Strength",
                "推定被写体への強調量",
                0.0,
                1.0,
                0.01,
                0.70,
            )],
        },
    ]
}

/// 画像読み込み・戦略適用・保存までを実施するエントリ。
pub fn apply_retouch(request: ApplyRetouchRequest) -> Result<ApplyRetouchResponse, String> {
    let source_path = PathBuf::from(&request.input_path);
    if !source_path.exists() {
        return Err(format!("Input file not found: {}", source_path.display()));
    }

    let strategies = list_strategies();
    let strategy = strategies
        .iter()
        .find(|item| item.id == request.strategy_id)
        .ok_or_else(|| format!("Unknown strategy_id: {}", request.strategy_id))?;

    let merged_params = merge_params(strategy, &request.params);
    let mut image =
        image::open(&source_path).map_err(|error| format!("Failed to open image: {error}"))?;

    let started = Instant::now();
    let model_info = run_strategy(&mut image, &request.strategy_id, &merged_params)?;
    let output_path = build_output_path(&source_path, &request.strategy_id)?;
    image
        .save(&output_path)
        .map_err(|error| format!("Failed to save image: {error}"))?;

    Ok(ApplyRetouchResponse {
        output_path: output_path.to_string_lossy().to_string(),
        elapsed_ms: started.elapsed().as_millis(),
        applied_params: merged_params,
        model_info,
    })
}

/// 手法ごとの分岐を一元管理する。
fn run_strategy(
    image: &mut DynamicImage,
    strategy_id: &str,
    params: &HashMap<String, f32>,
) -> Result<Option<String>, String> {
    let mut rgb = image.to_rgb8();

        if strategy_id == "ai-onnx-model" {
            let strength = get_param(params, "onnxStrength");
            let model_path = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("models").join("model.onnx")))
                .and_then(|p| p.to_str().map(|s| s.to_string()));
            let model_path = model_path.ok_or_else(|| "Unable to determine model path".to_string())?;
            #[cfg(feature = "onnx")]
            {
                let info = crate::onnx_inference::run_model(&mut rgb, &model_path, strength)
                    .map_err(|e| format!("ONNX inference error: {}", e))?;
                *image = DynamicImage::ImageRgb8(rgb);
                return Ok(Some(info));
            }
            #[cfg(not(feature = "onnx"))]
            {
                return Err("ONNX feature not enabled at compile time".to_string());
            }
        }
    let model_info = match strategy_id {
        "classic-auto-balance" => {
            let wb_strength = get_param(params, "whiteBalanceStrength");
            let contrast_boost = get_param(params, "contrastBoost");
            apply_gray_world_white_balance(&mut rgb, wb_strength);
            apply_contrast(&mut rgb, contrast_boost);
            None
        }
        "classic-vibrance-boost" => {
            let vibrance = get_param(params, "vibrance");
            let contrast_boost = get_param(params, "contrastBoost");
            apply_saturation_delta(&mut rgb, vibrance);
            apply_contrast(&mut rgb, contrast_boost);
            None
        }
        "classic-highlight-recovery" => {
            let threshold = get_param(params, "highlightThreshold");
            let strength = get_param(params, "recoveryStrength");
            apply_highlight_recovery(&mut rgb, threshold, strength);
            None
        }
        "classic-gamma-lift" => {
            let gamma = get_param(params, "gamma");
            apply_gamma(&mut rgb, gamma);
            None
        }
        "classic-local-clarity" => {
            let amount = get_param(params, "clarityAmount");
            let sigma = get_param(params, "blurSigma");
            apply_local_clarity(&mut rgb, sigma, amount);
            None
        }
        "ai-kmeans-scene-model" => {
            let strength = get_param(params, "sceneStrength");
            let info = apply_ai_kmeans_scene_model(&mut rgb, strength);
            Some(info)
        }
        "hybrid-skin-aware-tone" => {
            let strength = get_param(params, "skinToneStrength");
            let info = apply_hybrid_skin_aware_tone(&mut rgb, strength);
            Some(info)
        }
        "hybrid-subject-pop" => {
            let strength = get_param(params, "subjectPopStrength");
            let info = apply_hybrid_subject_pop(&mut rgb, strength);
            Some(info)
        }
        _ => return Err(format!("Unsupported strategy: {strategy_id}")),
    };

    *image = DynamicImage::ImageRgb8(rgb);
    Ok(model_info)
}

/// Gray-world前提でチャンネル毎のゲインを合わせる。
fn apply_gray_world_white_balance(image: &mut RgbImage, strength: f32) {
    let mut sum_r = 0.0_f32;
    let mut sum_g = 0.0_f32;
    let mut sum_b = 0.0_f32;
    let mut count = 0.0_f32;

    for pixel in image.pixels() {
        sum_r += pixel[0] as f32;
        sum_g += pixel[1] as f32;
        sum_b += pixel[2] as f32;
        count += 1.0;
    }

    if count <= 0.0 {
        return;
    }

    let avg_r = sum_r / count;
    let avg_g = sum_g / count;
    let avg_b = sum_b / count;
    let gray = (avg_r + avg_g + avg_b) / 3.0;

    let scale_r = blend(1.0, gray / avg_r.max(1.0), strength);
    let scale_g = blend(1.0, gray / avg_g.max(1.0), strength);
    let scale_b = blend(1.0, gray / avg_b.max(1.0), strength);

    for pixel in image.pixels_mut() {
        pixel[0] = clamp_channel(pixel[0] as f32 * scale_r);
        pixel[1] = clamp_channel(pixel[1] as f32 * scale_g);
        pixel[2] = clamp_channel(pixel[2] as f32 * scale_b);
    }
}

/// 線形コントラスト補正。
fn apply_contrast(image: &mut RgbImage, amount: f32) {
    let multiplier = (1.0 + amount).max(0.0);
    for pixel in image.pixels_mut() {
        pixel[0] = clamp_channel(((pixel[0] as f32 - 128.0) * multiplier) + 128.0);
        pixel[1] = clamp_channel(((pixel[1] as f32 - 128.0) * multiplier) + 128.0);
        pixel[2] = clamp_channel(((pixel[2] as f32 - 128.0) * multiplier) + 128.0);
    }
}

/// EVベースの露出補正。
fn apply_exposure_ev(image: &mut RgbImage, ev: f32) {
    let factor = 2.0_f32.powf(ev);
    for pixel in image.pixels_mut() {
        pixel[0] = clamp_channel(pixel[0] as f32 * factor);
        pixel[1] = clamp_channel(pixel[1] as f32 * factor);
        pixel[2] = clamp_channel(pixel[2] as f32 * factor);
    }
}

/// HSL空間で彩度を調整。
fn apply_saturation_delta(image: &mut RgbImage, delta: f32) {
    for pixel in image.pixels_mut() {
        let (h, s, l) = rgb_to_hsl(pixel[0], pixel[1], pixel[2]);
        let (r, g, b) = hsl_to_rgb(h, (s + delta).clamp(0.0, 1.0), l);
        pixel[0] = r;
        pixel[1] = g;
        pixel[2] = b;
    }
}

/// ガンマ補正（中間調の再配置）。
fn apply_gamma(image: &mut RgbImage, gamma: f32) {
    let safe_gamma = gamma.max(0.05);
    for pixel in image.pixels_mut() {
        pixel[0] = gamma_channel(pixel[0], safe_gamma);
        pixel[1] = gamma_channel(pixel[1], safe_gamma);
        pixel[2] = gamma_channel(pixel[2], safe_gamma);
    }
}

/// ハイライト圧縮（閾値より上だけを丸める）。
fn apply_highlight_recovery(image: &mut RgbImage, threshold: f32, strength: f32) {
    for pixel in image.pixels_mut() {
        for channel in &mut pixel.0 {
            if *channel as f32 > threshold {
                let compressed = threshold + ((*channel as f32 - threshold) * (1.0 - strength));
                *channel = clamp_channel(compressed);
            }
        }
    }
}

/// アンシャープマスクによる明瞭化。
fn apply_local_clarity(image: &mut RgbImage, sigma: f32, amount: f32) {
    let blurred = image::imageops::blur(image, sigma.max(0.1));
    for (pixel, blur_pixel) in image.pixels_mut().zip(blurred.pixels()) {
        pixel[0] = clamp_channel(pixel[0] as f32 + ((pixel[0] as f32 - blur_pixel[0] as f32) * amount));
        pixel[1] = clamp_channel(pixel[1] as f32 + ((pixel[1] as f32 - blur_pixel[1] as f32) * amount));
        pixel[2] = clamp_channel(pixel[2] as f32 + ((pixel[2] as f32 - blur_pixel[2] as f32) * amount));
    }
}

/// 色分布のK-Meansクラスタリング結果から補正値を推定する簡易AIモデル。
fn apply_ai_kmeans_scene_model(image: &mut RgbImage, strength: f32) -> String {
    let sample_step = 12_usize;
    let mut samples = Vec::<Vec3>::new();

    for y in (0..image.height() as usize).step_by(sample_step) {
        for x in (0..image.width() as usize).step_by(sample_step) {
            let p = image.get_pixel(x as u32, y as u32);
            samples.push(Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32));
        }
    }

    let clusters = kmeans_palette(&samples, 3, 7);
    if clusters.is_empty() {
        return "model=kmeans_scene;status=skipped".to_string();
    }

    let dominant = clusters
        .iter()
        .max_by_key(|item| item.count)
        .copied()
        .unwrap_or(clusters[0]);

    let scene_luma = 0.299 * dominant.center.r + 0.587 * dominant.center.g + 0.114 * dominant.center.b;
    let scene_temperature = dominant.center.r - dominant.center.b;
    let saturation_hint = {
        let (_, s, _) = rgb_to_hsl(
            dominant.center.r as u8,
            dominant.center.g as u8,
            dominant.center.b as u8,
        );
        s
    };

    if scene_luma < 96.0 {
        apply_exposure_ev(image, 0.35 * strength);
    } else if scene_luma > 190.0 {
        apply_exposure_ev(image, -0.22 * strength);
        apply_highlight_recovery(image, 205.0, 0.42 * strength);
    }

    if scene_temperature < -10.0 {
        apply_temperature_shift(image, 0.24 * strength);
    } else if scene_temperature > 20.0 {
        apply_temperature_shift(image, -0.18 * strength);
    }

    if saturation_hint < 0.25 {
        apply_saturation_delta(image, 0.16 * strength);
    }

    format!(
        "model=kmeans_scene;dominant=({:.0},{:.0},{:.0});luma={:.1};temp={:.1}",
        dominant.center.r, dominant.center.g, dominant.center.b, scene_luma, scene_temperature
    )
}

/// 肌色領域を検知して、その領域だけへトーン補正を強めるハイブリッド手法。
fn apply_hybrid_skin_aware_tone(image: &mut RgbImage, strength: f32) -> String {
    let mut skin_pixels = 0_u64;
    let total_pixels = (image.width() as u64) * (image.height() as u64);

    for pixel in image.pixels_mut() {
        if is_skin_like(pixel[0], pixel[1], pixel[2]) {
            skin_pixels += 1;

            // 肌領域には彩度と明度を軽く持ち上げる。
            let (h, s, l) = rgb_to_hsl(pixel[0], pixel[1], pixel[2]);
            let boosted_s = (s + (0.10 * strength)).clamp(0.0, 1.0);
            let boosted_l = (l + (0.05 * strength)).clamp(0.0, 1.0);
            let (r, g, b) = hsl_to_rgb(h, boosted_s, boosted_l);
            pixel[0] = r;
            pixel[1] = g;
            pixel[2] = b;
        }
    }

    // 画像全体へは弱いWBとコントラスト補正を掛けて馴染ませる。
    apply_gray_world_white_balance(image, 0.25 * strength);
    apply_contrast(image, 0.08 * strength);

    let skin_ratio = if total_pixels == 0 {
        0.0
    } else {
        skin_pixels as f32 / total_pixels as f32
    };

    format!(
        "model=skin_detector+tone;skin_pixels={};skin_ratio={:.4}",
        skin_pixels, skin_ratio
    )
}

/// エッジ量が高いタイルを被写体候補として検知し、重点補正するハイブリッド手法。
fn apply_hybrid_subject_pop(image: &mut RgbImage, strength: f32) -> String {
    let width = image.width();
    let height = image.height();
    if width < 3 || height < 3 {
        return "model=subject_pop;status=skipped_small_image".to_string();
    }

    let grid = 8_u32;
    let tile_w = ((width as f32) / (grid as f32)).ceil() as u32;
    let tile_h = ((height as f32) / (grid as f32)).ceil() as u32;
    let mut tile_energy = vec![0.0_f32; (grid * grid) as usize];

    // 1ピクセル先との差分をエッジ量として集計し、被写体候補タイルを推定する。
    for y in 0..(height - 1) {
        for x in 0..(width - 1) {
            let p = image.get_pixel(x, y);
            let right = image.get_pixel(x + 1, y);
            let down = image.get_pixel(x, y + 1);

            let luma = rgb_luma(p[0], p[1], p[2]);
            let luma_right = rgb_luma(right[0], right[1], right[2]);
            let luma_down = rgb_luma(down[0], down[1], down[2]);
            let edge = (luma - luma_right).abs() + (luma - luma_down).abs();

            let tile_x = (x / tile_w).min(grid - 1);
            let tile_y = (y / tile_h).min(grid - 1);
            let idx = (tile_y * grid + tile_x) as usize;
            tile_energy[idx] += edge;
        }
    }

    let (best_idx, best_energy) = tile_energy
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .unwrap_or((0, &0.0_f32));
    let best_tile_x = (best_idx as u32) % grid;
    let best_tile_y = (best_idx as u32) / grid;

    let anchor_x = (((best_tile_x as f32) + 0.5) * tile_w as f32).min((width - 1) as f32);
    let anchor_y = (((best_tile_y as f32) + 0.5) * tile_h as f32).min((height - 1) as f32);
    let sigma = (width.min(height) as f32) * 0.32;
    let sigma2 = (sigma * sigma).max(1.0);

    // 検知中心からの距離に応じてマスクを作り、局所的に補正を強める。
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - anchor_x;
            let dy = y as f32 - anchor_y;
            let distance2 = dx * dx + dy * dy;
            let mask = (-distance2 / (2.0 * sigma2)).exp();
            let local = strength * mask;

            let pixel = image.get_pixel_mut(x, y);
            let adjusted = adjust_single_pixel(
                [pixel[0], pixel[1], pixel[2]],
                0.14 * local,
                0.12 * local,
                0.18 * local,
                0.08 * local,
            );
            *pixel = Rgb(adjusted);
        }
    }

    format!(
        "model=subject_tile_detector;tile=({},{});tile_energy={:.2}",
        best_tile_x, best_tile_y, best_energy
    )
}

/// 赤青バランスで色温度を簡易シフトする。
fn apply_temperature_shift(image: &mut RgbImage, amount: f32) {
    for pixel in image.pixels_mut() {
        let adjusted = adjust_single_pixel([pixel[0], pixel[1], pixel[2]], 0.0, 0.0, 0.0, amount);
        pixel[0] = adjusted[0];
        pixel[1] = adjusted[1];
        pixel[2] = adjusted[2];
    }
}

/// 1ピクセル単位で露出・コントラスト・彩度・色温度を適用する共通関数。
fn adjust_single_pixel(
    rgb: [u8; 3],
    exposure_ev: f32,
    contrast: f32,
    saturation_delta: f32,
    temperature_shift: f32,
) -> [u8; 3] {
    let exposure_factor = 2.0_f32.powf(exposure_ev);
    let contrast_mul = (1.0 + contrast).max(0.0);

    let mut r = rgb[0] as f32 * exposure_factor;
    let mut g = rgb[1] as f32 * exposure_factor;
    let mut b = rgb[2] as f32 * exposure_factor;

    r *= 1.0 + (temperature_shift * 0.7);
    g *= 1.0 + (temperature_shift * 0.1);
    b *= 1.0 - (temperature_shift * 0.7);

    r = ((r - 128.0) * contrast_mul) + 128.0;
    g = ((g - 128.0) * contrast_mul) + 128.0;
    b = ((b - 128.0) * contrast_mul) + 128.0;

    let (h, s, l) = rgb_to_hsl(clamp_channel(r), clamp_channel(g), clamp_channel(b));
    let (nr, ng, nb) = hsl_to_rgb(h, (s + saturation_delta).clamp(0.0, 1.0), l);
    [nr, ng, nb]
}

/// 簡易K-Means（RGB空間）でクラスタ中心を推定。
fn kmeans_palette(samples: &[Vec3], k: usize, iterations: usize) -> Vec<Cluster> {
    if samples.is_empty() || k == 0 {
        return Vec::new();
    }

    let actual_k = k.min(samples.len());
    let mut centers = Vec::with_capacity(actual_k);
    for index in 0..actual_k {
        let sample_idx = index * samples.len() / actual_k;
        centers.push(samples[sample_idx]);
    }

    let mut assignments = vec![0_usize; samples.len()];
    for _ in 0..iterations {
        for (i, sample) in samples.iter().enumerate() {
            let mut best_idx = 0_usize;
            let mut best_distance = f32::MAX;
            for (center_idx, center) in centers.iter().enumerate() {
                let dist = sample.distance2(center);
                if dist < best_distance {
                    best_distance = dist;
                    best_idx = center_idx;
                }
            }
            assignments[i] = best_idx;
        }

        let mut sums = vec![Vec3::default(); actual_k];
        let mut counts = vec![0_u32; actual_k];
        for (sample, cluster_idx) in samples.iter().zip(assignments.iter()) {
            sums[*cluster_idx] = sums[*cluster_idx] + *sample;
            counts[*cluster_idx] += 1;
        }

        for cluster_idx in 0..actual_k {
            if counts[cluster_idx] > 0 {
                centers[cluster_idx] = sums[cluster_idx] / counts[cluster_idx] as f32;
            }
        }
    }

    let mut cluster_counts = vec![0_u32; actual_k];
    for assigned in assignments {
        cluster_counts[assigned] += 1;
    }

    centers
        .into_iter()
        .zip(cluster_counts)
        .map(|(center, count)| Cluster { center, count })
        .collect()
}

/// 肌色領域かどうかの簡易判定（YCbCr閾値）。
fn is_skin_like(r: u8, g: u8, b: u8) -> bool {
    let rf = r as f32;
    let gf = g as f32;
    let bf = b as f32;
    let cb = 128.0 - 0.168736 * rf - 0.331264 * gf + 0.5 * bf;
    let cr = 128.0 + 0.5 * rf - 0.418688 * gf - 0.081312 * bf;
    (77.0..=127.0).contains(&cb) && (133.0..=173.0).contains(&cr)
}

/// 出力ファイルパスをテンポラリ配下に生成する。
fn build_output_path(source: &Path, strategy_id: &str) -> Result<PathBuf, String> {
    let output_dir = std::env::temp_dir().join("retouch-lab-output");
    std::fs::create_dir_all(&output_dir)
        .map_err(|error| format!("Failed to create output directory: {error}"))?;

    let extension = source
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("png");
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("image");

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("Failed to read system time: {error}"))?
        .as_millis();

    let safe_strategy = strategy_id.replace('-', "_");
    let file_name = format!("{stem}_{safe_strategy}_{timestamp}.{extension}");
    Ok(output_dir.join(file_name))
}

/// 戦略デフォルトとUI入力値をマージする。
fn merge_params(
    strategy: &RetouchStrategyDefinition,
    user_params: &HashMap<String, f32>,
) -> HashMap<String, f32> {
    let mut merged = HashMap::new();
    for definition in &strategy.parameters {
        merged.insert(definition.key.clone(), definition.default_value);
    }
    for (key, value) in user_params {
        merged.insert(key.clone(), *value);
    }
    merged
}

/// パラメータ取得ヘルパ（未指定時は0.0）。
fn get_param(params: &HashMap<String, f32>, key: &str) -> f32 {
    params.get(key).copied().unwrap_or(0.0)
}

/// パラメータ定義生成ヘルパ。
fn param(
    key: &str,
    label: &str,
    description: &str,
    min: f32,
    max: f32,
    step: f32,
    default_value: f32,
) -> StrategyParameterDefinition {
    StrategyParameterDefinition {
        key: key.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        min,
        max,
        step,
        default_value,
    }
}

/// 線形補間。
fn blend(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// 8bitチャンネルへ丸める。
fn clamp_channel(value: f32) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

/// Luma推定。
fn rgb_luma(r: u8, g: u8, b: u8) -> f32 {
    0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32
}

/// ガンマ補正チャネル。
fn gamma_channel(value: u8, gamma: f32) -> u8 {
    let normalized = value as f32 / 255.0;
    let adjusted = normalized.powf(1.0 / gamma);
    clamp_channel(adjusted * 255.0)
}

/// RGB(0..255) -> HSL(0..1)。
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f32::EPSILON {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = d / (1.0 - (2.0 * l - 1.0).abs()).max(0.0001);
    let h = if (max - rf).abs() < f32::EPSILON {
        ((gf - bf) / d).rem_euclid(6.0)
    } else if (max - gf).abs() < f32::EPSILON {
        ((bf - rf) / d) + 2.0
    } else {
        ((rf - gf) / d) + 4.0
    } / 6.0;

    (h, s.clamp(0.0, 1.0), l.clamp(0.0, 1.0))
}

/// HSL(0..1) -> RGB(0..255)。
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    if s <= f32::EPSILON {
        let gray = clamp_channel(l * 255.0);
        return (gray, gray, gray);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    (
        clamp_channel(r * 255.0),
        clamp_channel(g * 255.0),
        clamp_channel(b * 255.0),
    )
}

/// HSL変換用の補助関数。
fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }

    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

/// K-Meansで扱う3次元ベクトル。
#[derive(Debug, Clone, Copy, Default)]
struct Vec3 {
    r: f32,
    g: f32,
    b: f32,
}

impl Vec3 {
    fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    fn distance2(&self, rhs: &Self) -> f32 {
        let dr = self.r - rhs.r;
        let dg = self.g - rhs.g;
        let db = self.b - rhs.b;
        (dr * dr) + (dg * dg) + (db * db)
    }
}

impl std::ops::Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl std::ops::Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output {
        Vec3::new(self.r / rhs, self.g / rhs, self.b / rhs)
    }
}

/// K-Means結果クラスタ。
#[derive(Debug, Clone, Copy)]
struct Cluster {
    center: Vec3,
    count: u32,
}

