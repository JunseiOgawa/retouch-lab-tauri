use image::RgbImage;

#[cfg(feature = "onnx")]
use onnxruntime::{environment::Environment, GraphOptimizationLevel};

/// Run an ONNX model-based inference over the given image.
/// This is a light wrapper: actual tensor preparation and postprocessing
/// should be implemented for your model's input/output specifications.
#[cfg(feature = "onnx")]
pub fn run_model(image: &mut RgbImage, model_path: &str, _strength: f32) -> Result<String, String> {
    // Try to initialize ONNX Runtime environment and load the model file.
    let env = Environment::builder()
        .with_name("retouch-lab")
        .build()
        .map_err(|e| format!("ONNX env error: {}", e))?;

    let mut session_builder = env
        .new_session_builder()
        .map_err(|e| format!("ONNX session builder error: {}", e))?;

    // Optional: set optimization level if available.
    let _ = session_builder.set_optimization_level(GraphOptimizationLevel::All);

    let _session = session_builder
        .with_model_from_file(model_path)
        .map_err(|e| format!("Failed to load ONNX model: {}", e))?;

    // NOTE: This wrapper only confirms model load. Implement data conversion,
    // input tensor creation and session.run(...) according to your model.
    Ok(format!("onnx_model_loaded:{}", model_path))
}

#[cfg(not(feature = "onnx"))]
pub fn run_model(_image: &mut RgbImage, _model_path: &str, _strength: f32) -> Result<String, String> {
    Err("ONNX feature not enabled at compile time".to_string())
}
