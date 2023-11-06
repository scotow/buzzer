use axum::extract::ws::Message as WsMessage;
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
#[serde(tag = "event", rename_all(serialize = "camelCase"))]
pub enum PacketOut {
    ParticipantCount {
        count: usize,
    },
    Buzzed {
        name: Box<str>,
        timestamp_diff: Option<u64>,
    },
}

impl From<PacketOut> for WsMessage {
    fn from(value: PacketOut) -> Self {
        Self::Text(serde_json::to_string(&value).expect("serialization failed"))
    }
}
