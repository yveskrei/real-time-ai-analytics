use utoipa::ToSchema;
use axum::extract::Query;
use serde_json::json;
use serde::Deserialize;

// Custom modules
use crate::processing;
use crate::utils::config::SearchType;
use crate::handlers::api::{ApiResponse, ApiResult};
use crate::handlers::TAG_RETRIEVAL;
use crate::utils::elastic::SearchMetadata;

#[derive(ToSchema, Deserialize)]
pub struct ImageSearchRequest {
    pub image_id: String,
    pub search_type: SearchType,
    
    #[schema(nullable, example = json!(["1", "2"]))]
    pub channel_ids: Option<Vec<String>>,
    
    #[schema(nullable, example = json!(1767899017))]
    pub timestamp_start: Option<i64>,
    
    #[schema(nullable, example = json!(1767899017))]
    pub timestamp_end: Option<i64>,
}

/// Search images similar to a given one
#[utoipa::path(
    get,
    path = "/search",
    tag = TAG_RETRIEVAL,
    params(
        ("image_id" = String, Query, description = "ID of the reference image"),
        ("search_type" = SearchType, Query, description = "Type of search"),
        ("channel_ids" = Option<Vec<String>>, Query, description = "Channel IDs to filter"),
        ("timestamp_start" = Option<i64>, Query, description = "Start timestamp"),
        ("timestamp_end" = Option<i64>, Query, description = "End timestamp"),
    ),
    responses(
        (status = 200, description = "Search successful"),
        (status = 400, description = "Invalid input")
    )
)]
pub async fn search_image(
    Query(request): Query<ImageSearchRequest>
) -> ApiResult<serde_json::Value> {
    // Construct Elastic search parameters
    let metadata = SearchMetadata {
        channel_ids: request.channel_ids,
        timestamp_start: request.timestamp_start,
        timestamp_end: request.timestamp_end,
    };

    match processing::search::search_image(
        request.image_id,
        request.search_type, 
        metadata
    ).await {
        Ok(candidates) => {
            let results: Vec<serde_json::Value> = candidates.into_iter().map(|c| {
                json!({
                    "score": c["_score"],
                    "metadata": c["_source"]
                })
            }).collect();

            Ok(ApiResponse::success_with_message(
                "Image processed successfully",
                json!({
                    "count": results.len(),
                    "candidates": results
                })
            ))
        },
        Err(e) => {
            tracing::error!(
                error=%e,
                "Could not process search"
            );
            
            Ok(ApiResponse::bad_request("Error searching candidates"))
        }
    }
}