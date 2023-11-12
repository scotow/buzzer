use crate::room::Room;
use axum::extract::ws::WebSocket;
use log::{as_display, info};
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use std::time::Instant;
use tokio::sync::Mutex;
use ulid::Ulid;

#[derive(Default)]
pub struct Registry {
    pending_rooms: HashMap<Ulid, PendingRoom>,
    rooms: HashMap<Ulid, Room>,
    rooms_name_mapping: HashMap<Box<str>, Ulid>,
}

impl Registry {
    pub async fn reserve(&mut self, name: Box<str>) -> Option<(Ulid, Box<str>)> {
        // TODO: cleanup unclaimed rooms after a while.
        // TODO: sanitize room names.

        let id = Ulid::new();
        assert!(self
            .pending_rooms
            .insert(
                id,
                PendingRoom {
                    name: name.clone(),
                    creation: Instant::now(),
                },
            )
            .is_none());

        info!(id = as_display!(id), room = as_display!(name); "room reserved");
        Some((id, name))
    }

    pub fn create(&mut self, id: Ulid, socket: WebSocket, weak_self: Weak<Mutex<Self>>) -> bool {
        let Some(name) = self.pending_rooms.remove(&id).map(|r| r.name) else {
            return false;
        };

        assert!(self.rooms_name_mapping.insert(name.clone(), id).is_none());
        info!(id = as_display!(id), room = as_display!(name); "room created");
        self.rooms
            .insert(id, Room::new(id, name, socket, weak_self));

        true
    }

    pub fn remove(&mut self, id: Ulid, name: Box<str>) {
        self.rooms.remove(&id);
        self.rooms_name_mapping.remove(&name);
        info!(id = as_display!(id), room = as_display!(name); "room removed");
    }

    pub fn find_room(&self, name: &str) -> Option<(Ulid, Box<str>)> {
        self.rooms_name_mapping
            .get(name)
            .copied()
            .map(|id| (id, Box::from(name)))
    }

    pub fn join_room(&self, id: Ulid, socket: WebSocket, name: Box<str>) -> bool {
        match self.rooms.get(&id) {
            Some(room) => {
                room.join(socket, name);
                true
            }
            None => false,
        }
    }
}

struct PendingRoom {
    name: Box<str>,
    creation: Instant,
}
