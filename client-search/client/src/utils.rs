use anyhow::{Result, Context};
use image::GenericImageView;
use nvml_wrapper::Nvml;

// Custom modules
pub mod config;
pub mod elastic;

/// Parses image bytes to extract an image
/// Returns (image_bytes, width, height)
pub fn parse_image(data: &[u8]) -> Result<(Vec<u8>, u32, u32)> {
    let img = image::load_from_memory(data)
        .context("Failed to load image from memory")?;
    
    let (width, height) = img.dimensions();
    let rgb_img = img.to_rgb8();
    
    Ok((rgb_img.into_raw(), width, height))
}

/// Get GPU name
pub fn get_gpu_name() -> Result<String> {
    let nvml = Nvml::init()
        .context("Error initiating NVML wrapper")?;
    let gpu = nvml.device_by_index(0)
        .context("Error getting GPU")?;
    let gpu_name = gpu.name()
        .context("Error getting GPU name")?;
    Ok(gpu_name)
}
    