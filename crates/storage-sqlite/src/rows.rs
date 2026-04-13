//! SQLite row types — bridge between sqlx raw rows and domain entities.

use aw_domain::entities::*;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub fn parse_dt(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

pub fn parse_uuid(s: &str) -> Uuid {
    Uuid::parse_str(s).unwrap_or_else(|_| Uuid::nil())
}

pub fn parse_json(s: &str) -> serde_json::Value {
    serde_json::from_str(s).unwrap_or(serde_json::Value::Null)
}

pub fn parse_vec<T: serde::de::DeserializeOwned>(s: &str) -> Vec<T> {
    serde_json::from_str(s).unwrap_or_default()
}

pub fn parse_agent_status(s: &str) -> AgentStatus {
    match s {
        "active" => AgentStatus::Active,
        "idle" => AgentStatus::Idle,
        "suspended" => AgentStatus::Suspended,
        _ => AgentStatus::Offline,
    }
}

pub fn fmt_agent_status(s: &AgentStatus) -> &'static str {
    match s {
        AgentStatus::Active => "active",
        AgentStatus::Idle => "idle",
        AgentStatus::Offline => "offline",
        AgentStatus::Suspended => "suspended",
    }
}

pub fn parse_session_status(s: &str) -> SessionStatus {
    match s {
        "active" => SessionStatus::Active,
        "idle" => SessionStatus::Idle,
        "dead" => SessionStatus::Dead,
        _ => SessionStatus::CheckedOut,
    }
}

pub fn fmt_session_status(s: &SessionStatus) -> &'static str {
    match s {
        SessionStatus::Active => "active",
        SessionStatus::Idle => "idle",
        SessionStatus::Dead => "dead",
        SessionStatus::CheckedOut => "checked_out",
    }
}

pub fn parse_session_health(s: &str) -> SessionHealth {
    match s {
        "healthy" => SessionHealth::Healthy,
        "degraded" => SessionHealth::Degraded,
        _ => SessionHealth::Unknown,
    }
}

pub fn fmt_session_health(s: &SessionHealth) -> &'static str {
    match s {
        SessionHealth::Healthy => "healthy",
        SessionHealth::Degraded => "degraded",
        SessionHealth::Unknown => "unknown",
    }
}

pub fn parse_inbox_status(s: &str) -> InboxStatus {
    match s {
        "pending" => InboxStatus::Pending,
        "processing" => InboxStatus::Processing,
        "done" => InboxStatus::Done,
        "failed" => InboxStatus::Failed,
        _ => InboxStatus::Expired,
    }
}

pub fn fmt_inbox_status(s: &InboxStatus) -> &'static str {
    match s {
        InboxStatus::Pending => "pending",
        InboxStatus::Processing => "processing",
        InboxStatus::Done => "done",
        InboxStatus::Failed => "failed",
        InboxStatus::Expired => "expired",
    }
}

pub fn parse_task_status(s: &str) -> TaskStatus {
    match s {
        "open" => TaskStatus::Open,
        "claimed" => TaskStatus::Claimed,
        "in_progress" => TaskStatus::InProgress,
        "done" => TaskStatus::Done,
        "failed" => TaskStatus::Failed,
        _ => TaskStatus::Cancelled,
    }
}

pub fn fmt_task_status(s: &TaskStatus) -> &'static str {
    match s {
        TaskStatus::Open => "open",
        TaskStatus::Claimed => "claimed",
        TaskStatus::InProgress => "in_progress",
        TaskStatus::Done => "done",
        TaskStatus::Failed => "failed",
        TaskStatus::Cancelled => "cancelled",
    }
}

pub fn parse_task_priority(s: &str) -> TaskPriority {
    match s {
        "low" => TaskPriority::Low,
        "high" => TaskPriority::High,
        "critical" => TaskPriority::Critical,
        _ => TaskPriority::Normal,
    }
}

pub fn fmt_task_priority(s: &TaskPriority) -> &'static str {
    match s {
        TaskPriority::Low => "low",
        TaskPriority::Normal => "normal",
        TaskPriority::High => "high",
        TaskPriority::Critical => "critical",
    }
}

pub fn parse_lock_type(s: &str) -> LockType {
    match s {
        "write_lock" => LockType::WriteLock,
        "soft_lock" => LockType::SoftLock,
        "topic_lock" => LockType::TopicLock,
        "artifact_lock" => LockType::ArtifactLock,
        _ => LockType::LeaseLock,
    }
}

pub fn fmt_lock_type(s: &LockType) -> &'static str {
    match s {
        LockType::WriteLock => "write_lock",
        LockType::SoftLock => "soft_lock",
        LockType::TopicLock => "topic_lock",
        LockType::ArtifactLock => "artifact_lock",
        LockType::LeaseLock => "lease_lock",
    }
}

pub fn parse_dep_state(s: &str) -> DependencyState {
    match s {
        "healthy" => DependencyState::Healthy,
        "degraded" => DependencyState::Degraded,
        "unhealthy" => DependencyState::Unhealthy,
        _ => DependencyState::Unknown,
    }
}

pub fn fmt_dep_state(s: &DependencyState) -> &'static str {
    match s {
        DependencyState::Healthy => "healthy",
        DependencyState::Degraded => "degraded",
        DependencyState::Unhealthy => "unhealthy",
        DependencyState::Unknown => "unknown",
    }
}

pub fn parse_message_kind(s: &str) -> MessageKind {
    match s {
        "chat_message" => MessageKind::ChatMessage,
        "review_request" => MessageKind::ReviewRequest,
        "approval_request" => MessageKind::ApprovalRequest,
        "handoff_note" => MessageKind::HandoffNote,
        "alert" => MessageKind::Alert,
        "status_update" => MessageKind::StatusUpdate,
        "deferred_task" => MessageKind::DeferredTask,
        _ => MessageKind::ConditionalInstruction,
    }
}

pub fn fmt_message_kind(s: &MessageKind) -> &'static str {
    match s {
        MessageKind::ChatMessage => "chat_message",
        MessageKind::ReviewRequest => "review_request",
        MessageKind::ApprovalRequest => "approval_request",
        MessageKind::HandoffNote => "handoff_note",
        MessageKind::Alert => "alert",
        MessageKind::StatusUpdate => "status_update",
        MessageKind::DeferredTask => "deferred_task",
        MessageKind::ConditionalInstruction => "conditional_instruction",
    }
}
