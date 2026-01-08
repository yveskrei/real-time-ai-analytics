use anyhow::{Result, Context};

// Custom modules
use crate::utils::config::SearchType;
use crate::utils::elastic::SearchMetadata;
use crate::processing::RawFrame;

pub async fn search_image(
    raw_frame: RawFrame, 
    search_type: SearchType, 
    metadata: SearchMetadata
) -> Result<Vec<String>> {
    Ok(vec![])
}