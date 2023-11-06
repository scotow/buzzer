use crate::packet::PacketOut;
use crate::registry::Registry;
use axum::extract::ws::{Message as WsMessage, WebSocket};
use axum::Error;
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashSet;
use std::sync::{Arc, Weak};
use std::time::Instant;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::Sender as BroadcastSender;
use tokio::sync::mpsc::Sender as MpscSender;
use tokio::sync::{broadcast, mpsc, Mutex};
use ulid::Ulid;

const CHANNEL_SIZE: usize = 1024;

#[derive(Debug)]
pub struct Room {
    main: MpscSender<RoomMessage>,
    broadcast: BroadcastSender<WsMessage>,
}

impl Room {
    pub fn new(id: Ulid, name: Box<str>, host: WebSocket, registry: Weak<Mutex<Registry>>) -> Self {
        let (main_tx, mut main_rx) = mpsc::channel::<RoomMessage>(CHANNEL_SIZE);
        let (broadcast_tx, _broadcast_rx) = broadcast::channel(CHANNEL_SIZE);
        let (mut host_tx, mut host_rx) = host.split();

        tokio::spawn(async move {
            let mut participants_count = 0;
            let mut run = Run::new();

            loop {
                let msg = main_rx.recv().await.expect("main channel closed");

                match msg {
                    RoomMessage::ParticipantJoin => {
                        participants_count += 1;
                        host_tx
                            .send(
                                PacketOut::ParticipantCount {
                                    count: participants_count,
                                }
                                .into(),
                            )
                            .await
                            .expect("send failed");
                    }
                    RoomMessage::Buzzed(buzzer, timestamp) => {
                        let timestamp_diff = match run.buzz(buzzer.id, timestamp) {
                            BuzzResult::Already => continue,
                            BuzzResult::First => None,
                            BuzzResult::TimeDifference(diff) => Some(diff),
                        };
                        host_tx
                            .send(
                                PacketOut::Buzzed {
                                    name: buzzer.name.clone(),
                                    timestamp_diff,
                                }
                                .into(),
                            )
                            .await
                            .expect("send failed");
                    }
                    RoomMessage::Clear => {
                        run = Run::new();
                    }
                    RoomMessage::ParticipantLeft => {
                        participants_count -= 1;
                        host_tx
                            .send(
                                PacketOut::ParticipantCount {
                                    count: participants_count,
                                }
                                .into(),
                            )
                            .await
                            .expect("send failed");
                    }
                    RoomMessage::HostLeft => {
                        registry
                            .upgrade()
                            .expect("registry deallocated")
                            .lock()
                            .await
                            .remove(id, name);
                        return;
                    }
                }
            }
        });

        let mut self_main_tx = main_tx.clone();
        tokio::spawn(async move {
            loop {
                match host_rx.next().await {
                    Some(Ok(_msg)) => {
                        // TODO: parse message and ensure it's a buzz.
                        self_main_tx
                            .send(RoomMessage::Clear)
                            .await
                            .expect("send failed");
                    }
                    Some(Err(_)) | None => {
                        // TODO: log error.
                        self_main_tx
                            .send(RoomMessage::HostLeft)
                            .await
                            .expect("send failed");
                        break;
                    }
                }
            }
        });

        Self {
            main: main_tx,
            broadcast: broadcast_tx,
        }
    }

    pub fn join(&self, socket: WebSocket, name: Box<str>) {
        let participant = Arc::new(Participant {
            id: Ulid::new(),
            name,
        });

        let (mut tx, mut rx) = socket.split();
        let mut main_tx = self.main.clone();
        let mut broadcast_rx = self.broadcast.subscribe();

        // TODO: check if either loop break ends the other.
        tokio::spawn(async move {
            loop {
                match broadcast_rx.recv().await {
                    Ok(msg) => tx.send(msg).await.expect("send failed"),
                    Err(_err) => {
                        // TODO: send "connection lost / room closed" error.
                        tx.close().await.expect("close failed");
                    }
                }
            }
        });
        tokio::spawn(async move {
            main_tx
                .send(RoomMessage::ParticipantJoin)
                .await
                .expect("send failed");
            loop {
                match rx.next().await {
                    Some(Ok(_msg)) =>
                    // TODO: parse message and ensure it's a buzz.
                    {
                        main_tx
                            .send(RoomMessage::Buzzed(
                                Arc::clone(&participant),
                                Instant::now(),
                            ))
                            .await
                            .expect("send failed")
                    }
                    Some(Err(_)) | None => {
                        // TODO: log error.
                        main_tx
                            .send(RoomMessage::ParticipantLeft)
                            .await
                            .expect("send failed");
                        break;
                    }
                }
            }
        });
    }
}

struct Run {
    buzzed: Vec<(Ulid, Instant)>,
}

impl Run {
    fn new() -> Self {
        Self { buzzed: Vec::new() }
    }

    fn buzz(&mut self, buzzer: Ulid, time: Instant) -> BuzzResult {
        // Start from the back because it's likely the last participant spamming the buzzer.
        if self.buzzed.iter().rev().any(|(b, _)| b == &buzzer) {
            return BuzzResult::Already;
        }
        let res = if self.buzzed.is_empty() {
            BuzzResult::First
        } else {
            BuzzResult::TimeDifference((time - self.buzzed[0].1).as_millis() as u64)
        };
        self.buzzed.push((buzzer, time));
        res
    }
}

enum BuzzResult {
    Already,
    First,
    TimeDifference(u64),
}

struct Participant {
    id: Ulid,
    name: Box<str>,
}

#[derive(Clone)]
pub enum RoomMessage {
    ParticipantJoin,
    Buzzed(Arc<Participant>, Instant),
    Clear,
    ParticipantLeft,
    HostLeft,
}
