use axum::extract::ws::{Message as WsMessage, Message};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(tag = "event", rename_all(serialize = "camelCase"))]
pub enum PacketOut {
    ParticipantCount {
        count: usize,
    },
    #[serde(rename_all(serialize = "camelCase"))]
    Buzzed {
        name: Box<str>,
        timestamp_diff: Option<u64>,
    },
    HostLeft,
}

impl From<PacketOut> for WsMessage {
    fn from(value: PacketOut) -> Self {
        Self::Text(serde_json::to_string(&value).expect("serialization failed"))
    }
}

#[derive(Deserialize)]
#[serde(tag = "event", rename_all(serialize = "camelCase"))]
pub enum PacketIn {
    Buzz,
    Clear,
}

impl TryFrom<WsMessage> for PacketIn {
    type Error = ();

    fn try_from(value: WsMessage) -> Result<Self, Self::Error> {
        match value {
            Message::Text(text) => serde_json::from_str(&text).map_err(|_err| ())?,
            _ => Err(()),
        }
    }
}
