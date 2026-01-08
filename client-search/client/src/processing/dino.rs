/// Module for DINOv3 model pre/post processing

use anyhow::{Result, Context};
use std::sync::Arc;
use std::time::Instant;

// Custom modules
use crate::inference::InferenceModel;
use crate::statistics::FrameProcessStats;
use crate::processing::{self, RawFrame, ResultEmbedding};
use crate::utils::config::InferencePrecision;

/// Performs pre-processing on raw RGB frame for DINOv3 model
/// 
/// This function performs pre-processing steps including resizing, center cropping,
/// and normalization(pixel & ImageNet) to prepare the frame for inference with DINOv3 models.
pub fn preprocess_frame(
    frame: &RawFrame,
    precision: InferencePrecision,
) -> Result<Vec<u8>> {
    // Validate input
    let frame_target_size = (frame.height * frame.width * 3) as usize;
    if frame.data.len() != frame_target_size {
        anyhow::bail!(
            "Got unexpected size of frame input. Got {}, expected {}",
            frame.data.len(),
            frame_target_size
        );
    }

    // Preprocess with letterbox resize + ImageNet normalization
    const TARGET_SIZE: u32 = 224;
    processing::resize_letterbox_and_normalize_imagenet(
        &frame.data,
        frame.height,
        frame.width,
        TARGET_SIZE,
        TARGET_SIZE,
        precision
    )
}

/// Performs post-processing on multiple raw inference results from DINOv3 models
/// 
/// Takes a Vec of raw Vec<u8> outputs from batch model inference and converts them to 
/// a Vec of ResultEmbedding containing the feature vectors.
pub fn postprocess(
    raw_results: Vec<Vec<u8>>,
    precision: InferencePrecision,
) -> Result<Vec<ResultEmbedding>> {
    let mut embeddings = Vec::with_capacity(raw_results.len());
    
    for raw_result in raw_results {
        let num_elements = match precision {
            InferencePrecision::FP16 => raw_result.len() / 2,
            InferencePrecision::FP32 => raw_result.len() / 4,
        };
        
        let embedding = match precision {
            InferencePrecision::FP16 => {
                let raw_ptr = raw_result.as_ptr() as *const u16;
                let mut data = Vec::with_capacity(num_elements);
                unsafe {
                    for i in 0..num_elements {
                        data.push(processing::get_f16_to_f32_lut(*raw_ptr.add(i)));
                    }
                }
                ResultEmbedding { data }
            }
            InferencePrecision::FP32 => {
                let raw_ptr = raw_result.as_ptr() as *const f32;
                let data = unsafe {
                    Vec::from_raw_parts(
                        raw_ptr as *mut f32,
                        num_elements,
                        num_elements
                    )
                };
                std::mem::forget(raw_result);
                ResultEmbedding { data }
            }
        };
        
        embeddings.push(embedding);
    }
    
    Ok(embeddings)
}

/// Performs operations on a given frame, including pre/post processing, inference on the given frame
pub async fn process_frame(
    inference_model: &InferenceModel,
    frame: Arc<RawFrame>
) -> Result<(FrameProcessStats, ResultEmbedding)> {
    let processing_start = Instant::now();

    // Pre process
    let measure_start = Instant::now();
    let precision = inference_model.model_config().precision;
    let frame_clone = Arc::clone(&frame);
    let pre_frame = tokio::task::spawn_blocking(move || {
        preprocess_frame(&frame_clone, precision)
    })
        .await
        .context("Preprocess task failed")?
        .context("Error preprocessing frame for DINOv3")?;
    let pre_proc_time = measure_start.elapsed();

    // Inference
    let measure_start = Instant::now();
    let raw_results = inference_model.infer(vec![pre_frame])
        .await
        .context("Error performing inference for DINOv3")?;
    let inference_time = measure_start.elapsed();

    // Post process
    let measure_start = Instant::now();
    let embeddings = tokio::task::spawn_blocking(move || {
        postprocess(raw_results, precision)
    })
        .await
        .context("Postprocess task failed")?
        .context("Error postprocessing frame for DINOv3")?;

    let embedding = match embeddings.into_iter().next() {
        Some(res) => res,
        None => anyhow::bail!("No inference results returned for DINOv3"),
    };

    let post_proc_time = measure_start.elapsed();

    // Statistics
    let mut stats = FrameProcessStats::default();
    stats.pre_processing = pre_proc_time.as_micros() as u64;
    stats.inference = inference_time.as_micros() as u64;
    stats.post_processing = post_proc_time.as_micros() as u64;
    stats.processing = processing_start.elapsed().as_micros() as u64;

    Ok((stats, embedding))
}