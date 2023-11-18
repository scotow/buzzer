use std::sync::{Arc, Weak};
use std::time::Instant;

use axum::extract::ws::{Message as WsMessage, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast::Sender as BroadcastSender;
use tokio::sync::mpsc::Sender as MpscSender;
use tokio::sync::{broadcast, mpsc, Mutex};
use ulid::Ulid;

use crate::packet::{PacketIn, PacketOut};
use crate::registry::Registry;

const CHANNEL_SIZE: usize = 1024;

#[derive(Debug)]
pub struct Room {
    pub name: Box<str>,
    main: MpscSender<RoomMessage>,
    broadcast: BroadcastSender<BroadcastMessage>,
}

impl Room {
    pub fn new(id: Ulid, name: Box<str>, host: WebSocket, registry: Weak<Mutex<Registry>>) -> Self {
        let (main_tx, mut main_rx) = mpsc::channel::<RoomMessage>(CHANNEL_SIZE);
        let (broadcast_tx, _broadcast_rx) = broadcast::channel(CHANNEL_SIZE);
        let (mut host_tx, mut host_rx) = host.split();

        let self_name = name.clone();
        let self_broadcast_tx = broadcast_tx.clone();
        tokio::spawn(async move {
            let mut participants_count = 0;
            let mut run = Run::new();

            loop {
                let msg = main_rx.recv().await.expect("main channel closed");

                match msg {
                    RoomMessage::ParticipantJoin => {
                        participants_count += 1;
                        let packet = WsMessage::from(PacketOut::ParticipantCount {
                            count: participants_count,
                        });
                        host_tx.send(packet.clone()).await.expect("send failed");
                        _ = self_broadcast_tx.send(BroadcastMessage::All(packet));
                    }
                    RoomMessage::Buzzed(buzzer, timestamp) => {
                        let buzz_result = run.buzz(buzzer.id, timestamp);
                        let timestamp_diff = match buzz_result {
                            BuzzResult::Already => continue,
                            BuzzResult::First => None,
                            BuzzResult::TimeDifference(diff) => Some(diff),
                        };
                        host_tx
                            .send(
                                PacketOut::Buzzed {
                                    id: buzzer.id,
                                    name: buzzer.name.clone(),
                                    timestamp_diff,
                                }
                                .into(),
                            )
                            .await
                            .expect("send failed");
                        if matches!(buzz_result, BuzzResult::First) {
                            _ = self_broadcast_tx.send(BroadcastMessage::Single(
                                run.first_unchecked(),
                                WsMessage::from(PacketOut::Select { id: None }),
                            ));
                        }
                    }
                    RoomMessage::SelectNext => {
                        let Some((to_clear, to_notify)) = run.select_next() else {
                            continue;
                        };
                        _ = self_broadcast_tx.send(BroadcastMessage::Single(
                            to_clear,
                            WsMessage::from(PacketOut::Deselect),
                        ));
                        _ = self_broadcast_tx.send(BroadcastMessage::Single(
                            to_notify,
                            WsMessage::from(PacketOut::Select { id: None }),
                        ));
                        host_tx
                            .send(WsMessage::from(PacketOut::Select {
                                id: Some(to_notify),
                            }))
                            .await
                            .expect("send failed");
                    }
                    RoomMessage::Clear => {
                        run = Run::new();
                        _ = self_broadcast_tx
                            .send(BroadcastMessage::All(WsMessage::from(PacketOut::Clear)));
                    }
                    RoomMessage::ParticipantLeft => {
                        participants_count -= 1;
                        let packet = WsMessage::from(PacketOut::ParticipantCount {
                            count: participants_count,
                        });
                        host_tx.send(packet.clone()).await.expect("send failed");
                        _ = self_broadcast_tx.send(BroadcastMessage::All(packet));
                    }
                    RoomMessage::HostLeft => {
                        registry
                            .upgrade()
                            .expect("registry deallocated")
                            .lock()
                            .await
                            .remove(id, self_name);
                        // If the host was alone, the broadcast channel is already partially closed.
                        _ = self_broadcast_tx
                            .send(BroadcastMessage::All(WsMessage::from(PacketOut::HostLeft)));
                        return;
                    }
                }
            }
        });

        let self_main_tx = main_tx.clone();
        tokio::spawn(async move {
            loop {
                match host_rx.next().await {
                    Some(Ok(msg)) => match PacketIn::try_from(msg) {
                        Ok(PacketIn::Clear) => {
                            self_main_tx
                                .send(RoomMessage::Clear)
                                .await
                                .expect("send failed");
                        }
                        Ok(PacketIn::SelectNext) => {
                            self_main_tx
                                .send(RoomMessage::SelectNext)
                                .await
                                .expect("send failed");
                        }
                        Ok(_) | Err(_) => {
                            self_main_tx
                                .send(RoomMessage::HostLeft)
                                .await
                                .expect("send failed");
                            return;
                        }
                    },
                    Some(Err(_)) | None => {
                        // TODO: log error.
                        self_main_tx
                            .send(RoomMessage::HostLeft)
                            .await
                            .expect("send failed");
                        return;
                    }
                }
            }
        });

        Self {
            name,
            main: main_tx,
            broadcast: broadcast_tx,
        }
    }

    pub fn join(&self, socket: WebSocket, name: Box<str>) {
        let id = Ulid::new();
        let participant = Arc::new(Participant { id, name });

        let (mut tx, mut rx) = socket.split();
        let main_tx = self.main.clone();
        let mut broadcast_rx = self.broadcast.subscribe();

        // TODO: if either loop breaks, end the other.
        let rx_handle = tokio::spawn(async move {
            loop {
                match broadcast_rx.recv().await {
                    Ok(msg) => {
                        if msg.is_target(&id) && tx.send(msg.inner()).await.is_err() {
                            return;
                        }
                    }
                    Err(_err) => {
                        // TODO: send "connection lost / room closed" error.
                        _ = tx.close().await;
                        return;
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
                    Some(Ok(WsMessage::Text(_))) => {
                        // TODO: parse message and ensure it's a buzz.
                        if main_tx
                            .send(RoomMessage::Buzzed(
                                Arc::clone(&participant),
                                Instant::now(),
                            ))
                            .await
                            .is_err()
                        {
                            rx_handle.abort();
                            return;
                        }
                    }
                    Some(Ok(WsMessage::Close(_))) | Some(Ok(_)) | Some(Err(_)) | None => {
                        rx_handle.abort();
                        _ = main_tx.send(RoomMessage::ParticipantLeft).await;
                        return;
                    }
                }
            }
        });
    }
}

#[derive(Debug)]
struct Run {
    buzzed: Vec<(Ulid, Instant)>,
    selection: usize,
}

impl Run {
    fn new() -> Self {
        Self {
            buzzed: Vec::new(),
            selection: 0,
        }
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

    fn select_next(&mut self) -> Option<(Ulid, Ulid)> {
        if self.buzzed.len() < 2 || self.selection == self.buzzed.len() - 1 {
            return None;
        }
        self.selection += 1;
        Some((
            self.buzzed[self.selection - 1].0,
            self.buzzed[self.selection].0,
        ))
    }

    fn first_unchecked(&self) -> Ulid {
        assert_eq!(self.buzzed.len(), 1);
        self.buzzed[0].0
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
enum RoomMessage {
    ParticipantJoin,
    Buzzed(Arc<Participant>, Instant),
    SelectNext,
    Clear,
    ParticipantLeft,
    HostLeft,
}

#[derive(Clone, Debug)]
enum BroadcastMessage {
    All(WsMessage),
    Single(Ulid, WsMessage),
}

impl BroadcastMessage {
    fn is_target(&self, id: &Ulid) -> bool {
        match self {
            BroadcastMessage::All(_) => true,
            BroadcastMessage::Single(target_id, _) => target_id == id,
        }
    }

    fn inner(self) -> WsMessage {
        match self {
            BroadcastMessage::All(msg) => msg,
            BroadcastMessage::Single(_, msg) => msg,
        }
    }
}
