use crate::domain::HeatmapData;
use crate::services::{AggregationService, CollectionService};
use axum::{
    extract::Path,
    http::StatusCode,
    Json,
};
use std::collections::HashMap;

pub async fn get_all_heatmaps() -> Result<Json<HashMap<String, HeatmapData>>, StatusCode> {
    let service = CollectionService::new()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let records = service.collect_all().await
        .map_err(|e| {
            tracing::error!("Failed to collect records: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let heatmaps = AggregationService::aggregate_by_agent(records);
    Ok(Json(heatmaps))
}

pub async fn get_agent_heatmap(Path(agent): Path<String>) -> Result<Json<HeatmapData>, StatusCode> {
    let service = CollectionService::new()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let records = match agent.to_lowercase().as_str() {
        "claude" => service.collect_claude().await,
        "gemini" => service.collect_gemini().await,
        "codex" => service.collect_codex().await,
        _ => return Err(StatusCode::NOT_FOUND),
    }.map_err(|e| {
        tracing::error!("Failed to collect {} records: {}", agent, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let heatmap = AggregationService::aggregate_by_date(records);
    Ok(Json(heatmap))
}
