use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use aw_domain::entities::{
    CreateEndorsementInput, CreateReviewInput,
    CreateHumanReviewInput, CreateAgentPeerReviewInput, UpsertCapabilityInput,
};

use crate::{error::ApiResult, state::AppState};

// ── Legacy endpoints (preserved) ─────────────────────────────────────────────

pub async fn upsert_review(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(mut input): Json<CreateReviewInput>,
) -> ApiResult<Json<serde_json::Value>> {
    input.agent_id = agent_id;
    let review = state.storage.upsert_review(input).await?;
    Ok(Json(serde_json::to_value(review).unwrap()))
}

pub async fn create_endorsement(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(mut input): Json<CreateEndorsementInput>,
) -> ApiResult<Json<serde_json::Value>> {
    input.to_agent_id = agent_id;
    let endorsement = state.storage.create_endorsement(input).await?;
    Ok(Json(serde_json::to_value(endorsement).unwrap()))
}

pub async fn get_reputation(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let reputation = state.storage.get_reputation(&agent_id).await?;
    Ok(Json(serde_json::to_value(reputation).unwrap()))
}

// ── Phase 1 — Human reviews ───────────────────────────────────────────────────

pub async fn upsert_human_review(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(mut input): Json<CreateHumanReviewInput>,
) -> ApiResult<Json<serde_json::Value>> {
    input.agent_id = agent_id;
    let review = state.storage.upsert_human_review(input).await?;
    Ok(Json(serde_json::to_value(review).unwrap()))
}

// ── Phase 1 — Agent peer reviews ─────────────────────────────────────────────

pub async fn upsert_agent_peer_review(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(mut input): Json<CreateAgentPeerReviewInput>,
) -> ApiResult<Json<serde_json::Value>> {
    input.to_agent_id = agent_id;
    let review = state.storage.upsert_agent_peer_review(input).await?;
    Ok(Json(serde_json::to_value(review).unwrap()))
}

// ── Phase 1 — Capabilities ────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct DomainPath {
    id: String,
    domain: String,
}

#[derive(Deserialize)]
pub struct CapabilityBody {
    level: u8,
    source: Option<String>,
    confidence: Option<f64>,
}

pub async fn upsert_capability(
    State(state): State<AppState>,
    Path(p): Path<DomainPath>,
    Json(body): Json<CapabilityBody>,
) -> ApiResult<Json<serde_json::Value>> {
    let input = UpsertCapabilityInput {
        agent_id: p.id,
        domain: p.domain,
        level: body.level,
        source: body.source,
        confidence: body.confidence,
    };
    let cap = state.storage.upsert_capability(input).await?;
    Ok(Json(serde_json::to_value(cap).unwrap()))
}

pub async fn list_capabilities(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let caps = state.storage.list_capabilities(&agent_id).await?;
    Ok(Json(serde_json::to_value(caps).unwrap()))
}

// ── Phase 1 — Full dual-channel reputation ───────────────────────────────────

pub async fn get_full_reputation(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let rep = state.storage.get_full_reputation(&agent_id).await?;
    Ok(Json(serde_json::to_value(rep).unwrap()))
}
