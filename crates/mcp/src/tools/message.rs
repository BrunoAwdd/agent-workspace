use std::borrow::Cow;

use rmcp::{ErrorData, handler::server::router::tool::{AsyncTool, ToolBase}};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use aw_domain::entities::{InboxItem, Message, MessageKind, SendMessageInput};

use crate::server::WorkspaceServer;

// ── SendMessage ───────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema, Default)]
pub struct SendMessageParams {
    /// Workspace identifier (e.g. "main" or your project slug).
    pub workspace_id: String,
    /// Agent ID of the sender.
    pub from_agent_id: String,
    /// Agent ID of the recipient (omit for broadcast).
    pub to_agent_id: Option<String>,
    /// Channel to post to (optional).
    pub channel_id: Option<String>,
    /// UUID of the parent message for threading (optional).
    pub thread_id: Option<String>,
    /// Message kind: "chat_message" | "review_request" | "approval_request" |
    /// "handoff_note" | "alert" | "status_update" | "deferred_task" | "conditional_instruction".
    pub kind: String,
    /// Arbitrary JSON payload — the content of the message.
    pub payload: serde_json::Value,
    /// Also deliver this message to the recipient's inbox.
    pub deliver_to_inbox: bool,
}

pub struct SendMessageTool;
impl ToolBase for SendMessageTool {
    type Parameter = SendMessageParams;
    type Output = Message;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "workspace.send_message".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Send a message to another agent (or broadcast to a channel). Set deliver_to_inbox = true to also create an InboxItem the recipient will see on their next check-in.".into())
    }
}
impl AsyncTool<WorkspaceServer> for SendMessageTool {
    async fn invoke(service: &WorkspaceServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let thread_id = param.thread_id
            .map(|s| s.parse().map_err(|_| ErrorData::invalid_params("thread_id must be a valid UUID", None)))
            .transpose()?;
        let kind = parse_message_kind(&param.kind)?;
        service.storage.send_message(SendMessageInput {
            workspace_id: param.workspace_id,
            from_agent_id: param.from_agent_id,
            to_agent_id: param.to_agent_id,
            channel_id: param.channel_id,
            thread_id,
            kind,
            payload: param.payload,
            deliver_to_inbox: param.deliver_to_inbox,
        }).await.map_err(|e| ErrorData::internal_error(e.to_string(), None))
    }
}

// ── ReadInbox ─────────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema, Default)]
pub struct ReadInboxParams {
    /// Agent ID whose inbox to read.
    pub agent_id: String,
}

#[derive(Serialize, JsonSchema)]
pub struct InboxOutput {
    pub items: Vec<InboxItem>,
    pub count: usize,
}

pub struct ReadInboxTool;
impl ToolBase for ReadInboxTool {
    type Parameter = ReadInboxParams;
    type Output = InboxOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "workspace.read_inbox".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Read all pending inbox items for this agent. Items with deliver_on_checkin = true are returned here. After processing, call workspace.ack_inbox to mark them done.".into())
    }
}
impl AsyncTool<WorkspaceServer> for ReadInboxTool {
    async fn invoke(service: &WorkspaceServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let items = service.storage.list_inbox(&param.agent_id)
            .await.map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let count = items.len();
        Ok(InboxOutput { items, count })
    }
}

// ── AckInbox ──────────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema, Default)]
pub struct AckInboxParams {
    /// UUID of the inbox item to acknowledge.
    pub item_id: String,
    /// Agent ID (must match target_agent_id of the item).
    pub agent_id: String,
    /// New status: "done" | "failed" | "processing".
    pub status: String,
}

pub struct AckInboxTool;
impl ToolBase for AckInboxTool {
    type Parameter = AckInboxParams;
    type Output = super::agent::OkOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "workspace.ack_inbox".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Mark an inbox item as processed. Use status = \"done\" when handled successfully, \"failed\" on error.".into())
    }
}
impl AsyncTool<WorkspaceServer> for AckInboxTool {
    async fn invoke(service: &WorkspaceServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        use aw_domain::entities::{AckInboxItemInput, InboxStatus};
        let item_id = param.item_id.parse()
            .map_err(|_| ErrorData::invalid_params("item_id must be a valid UUID", None))?;
        let status = match param.status.as_str() {
            "done"       => InboxStatus::Done,
            "failed"     => InboxStatus::Failed,
            "processing" => InboxStatus::Processing,
            other        => return Err(ErrorData::invalid_params(format!("unknown status: {other}"), None)),
        };
        service.storage.ack_inbox_item(AckInboxItemInput {
            item_id,
            agent_id: param.agent_id,
            status,
        }).await.map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(super::agent::OkOutput { ok: true })
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn parse_message_kind(s: &str) -> Result<MessageKind, ErrorData> {
    match s {
        "chat_message"            => Ok(MessageKind::ChatMessage),
        "review_request"          => Ok(MessageKind::ReviewRequest),
        "approval_request"        => Ok(MessageKind::ApprovalRequest),
        "handoff_note"            => Ok(MessageKind::HandoffNote),
        "alert"                   => Ok(MessageKind::Alert),
        "status_update"           => Ok(MessageKind::StatusUpdate),
        "deferred_task"           => Ok(MessageKind::DeferredTask),
        "conditional_instruction" => Ok(MessageKind::ConditionalInstruction),
        other => Err(ErrorData::invalid_params(format!("unknown message kind: {other}"), None)),
    }
}
