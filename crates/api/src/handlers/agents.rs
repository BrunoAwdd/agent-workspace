use axum::{extract::{Path, State}, Json};
use aw_domain::entities::{Agent, CreateAgentInput};

use crate::{error::ApiResult, state::AppState};

pub async fn list_agents(State(state): State<AppState>) -> ApiResult<Json<Vec<Agent>>> {
    let agents = state.storage.list_agents().await?;
    Ok(Json(agents))
}

pub async fn get_agent(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> ApiResult<Json<Agent>> {
    let agent = state.storage.get_agent(&agent_id).await?
        .ok_or_else(|| aw_domain::error::WorkspaceError::NotFound(agent_id))?;
    Ok(Json(agent))
}

pub async fn create_agent(
    State(state): State<AppState>,
    Json(input): Json<CreateAgentInput>,
) -> ApiResult<Json<Agent>> {
    let agent = state.storage.create_agent(input).await?;
    Ok(Json(agent))
}

#[derive(serde::Deserialize)]
pub struct EligibilityQuery {
    pub task_kind: String,
    pub action: String,
}

#[derive(serde::Serialize)]
pub struct EligibilityResult {
    pub eligible: bool,
    pub missing: Vec<String>,
}

pub async fn get_agent_eligibility(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    axum::extract::Query(q): axum::extract::Query<EligibilityQuery>,
) -> ApiResult<Json<EligibilityResult>> {
    let mut missing = Vec::new();
    
    if let Some(policy) = state.storage.get_eligibility_policy(&q.task_kind).await? {
        let rule = match q.action.as_str() {
            "claim" => &policy.rules.claim,
            "review" => &policy.rules.review,
            "approve" => &policy.rules.approve,
            _ => &None,
        };
        
        if let Some(r) = rule {
            if !r.requires.is_empty() {
                let caps = state.storage.list_capabilities(&agent_id).await?;
                for req in &r.requires {
                    let current_level = caps.iter().find(|c| c.domain == req.domain).map(|c| c.level).unwrap_or(0);
                    if current_level < req.min {
                        missing.push(format!("{} >= {}", req.domain, req.min));
                    }
                }
            }
        }
    }
    
    Ok(Json(EligibilityResult {
        eligible: missing.is_empty(),
        missing,
    }))
}
