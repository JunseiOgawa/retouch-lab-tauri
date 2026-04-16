use image::{RgbImage, imageops::FilterType};

// The ONNX code is feature-gated. Builds without `--features onnx` will use the
// fallback implementation below.
#[cfg(feature = "onnx")]
use onnxruntime::{
    environment::Environment,
    GraphOptimizationLevel,
    ndarray::Array,
    tensor::OrtOwnedTensor,
};

/// Run an ONNX model-based inference over the given image.
///
/// Behaviour and assumptions:
/// - Expects the model to have a single 4-D input tensor with layout [N, C, H, W].
/// - If the model expects a different H/W than the provided image, the image is
///   resized to the model input size before inference and then upscaled back.
/// - Input pixels are normalized to [0.0, 1.0] and fed in CHW (channels-first) order.
/// - If the model output shape matches [N, C, H, W] the output is written back to
///   the image (clamped to [0,255]). Otherwise the function returns the output shape
///   for inspection.
#[cfg(feature = "onnx")]
pub fn run_model(image: &mut RgbImage, model_path: &str, _strength: f32) -> Result<String, String> {
    // Initialize ONNX Runtime environment and session builder.
    let env = Environment::builder()
        .with_name("retouch-lab")
        .build()
        .map_err(|e| format!("ONNX env error: {}", e))?;

    let mut session_builder = env
        .new_session_builder()
        .map_err(|e| format!("ONNX session builder error: {}", e))?;

    // Try to enable graph optimizations where supported.
    let _ = session_builder.set_optimization_level(GraphOptimizationLevel::All);

    // Load model file from disk.
    let mut session = session_builder
        .with_model_from_file(model_path)
        .map_err(|e| format!("Failed to load ONNX model: {}", e))?;

    // Inspect model input / output shapes.
    let input_shape: Vec<usize> = session.inputs[0]
        .dimensions()
        .map(|d| d.unwrap())
        .collect();
    let output_shape: Vec<usize> = session.outputs[0]
        .dimensions()
        .map(|d| d.unwrap())
        .collect();

    // We expect a 4-D input (N,C,H,W). Reject otherwise.
    if input_shape.len() != 4 {
        return Err(format!("Unsupported model input rank: {}", input_shape.len()));
    }
    let (n, c, h, w) = (input_shape[0], input_shape[1], input_shape[2], input_shape[3]);

    // Prepare an image buffer that matches the model's H/W. If sizes differ, create
    // a resized copy to run inference on.
    let mut resized_buf: Option<RgbImage> = None;
    let used_image: &mut RgbImage = if (image.width() as usize) != w || (image.height() as usize) != h {
        let resized = image::imageops::resize(image, w as u32, h as u32, FilterType::Lanczos3).to_rgb8();
        resized_buf = Some(resized);
        // We will mutate the resized buffer and later scale results back.
        resized_buf.as_mut().unwrap()
    } else {
        image
    };

    // Prepare input data in CHW order, normalized to [0,1]. Assumes u8 input.
    let mut input_vals: Vec<f32> = Vec::with_capacity(n * c * h * w);
    for ch in 0..c {
        for yy in 0..h {
            for xx in 0..w {
                let px = used_image.get_pixel(xx as u32, yy as u32);
                let v = px[ch] as f32 / 255.0;
                input_vals.push(v);
            }
        }
    }

    // Build an ndarray with the model's expected shape.
    use onnxruntime::ndarray::IxDyn;
    let input_array = Array::from_shape_vec(IxDyn(&input_shape), input_vals)
        .map_err(|e| format!("ndarray shape error: {}", e))?;

    // Run inference. The session API accepts a Vec of ndarray arrays for inputs.
    let input_tensors = vec![input_array];
    let outputs: Vec<OrtOwnedTensor<f32, _>> = session
        .run(input_tensors)
        .map_err(|e| format!("ONNX run error: {}", e))?;

    if outputs.is_empty() {
        return Err("ONNX model returned no outputs".to_string());
    }

    // If the model output is the same rank/shape as input (N,C,H,W), copy back to image.
    let out_shape = outputs[0].shape().to_vec();
    if out_shape.len() == 4 && out_shape[0] == n && out_shape[1] == c && out_shape[2] == h && out_shape[3] == w {
        // Write pixels back into the used_image.
        for yy in 0..h {
            for xx in 0..w {
                // Clamp and convert to u8.
                let r = (outputs[0][[0, 0, yy, xx]] * 255.0).clamp(0.0, 255.0) as u8;
                let g = (outputs[0][[0, 1, yy, xx]] * 255.0).clamp(0.0, 255.0) as u8;
                let b = (outputs[0][[0, 2, yy, xx]] * 255.0).clamp(0.0, 255.0) as u8;
                let px = image::Rgb([r, g, b]);
                used_image.put_pixel(xx as u32, yy as u32, px);
            }
        }

        // If we resized for the model, scale the output back to the original and overwrite
        // the caller-supplied image buffer.
        if let Some(res_buf) = resized_buf {
            let up = image::imageops::resize(&res_buf, image.width(), image.height(), FilterType::Lanczos3).to_rgb8();
            *image = up;
        }

        Ok(format!("onnx_run_ok: output_shape={:?}", out_shape))
    } else {
        // Output shape is not directly mappable; return shape for debugging.
        Ok(format!("onnx_run_ok: output_shape={:?}", out_shape))
    }
}

#[cfg(not(feature = "onnx"))]
pub fn run_model(_image: &mut RgbImage, _model_path: &str, _strength: f32) -> Result<String, String> {
    Err("ONNX feature not enabled at compile time".to_string())
}
