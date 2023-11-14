use std::collections::HashMap;
use std::sync::Weak;
use std::time::Instant;

use axum::extract::ws::WebSocket;
use log::{as_display, info};
use tokio::sync::Mutex;
use ulid::Ulid;

use crate::error::Error;
use crate::room::Room;

#[derive(Default)]
pub struct Registry {
    pending_rooms: HashMap<Ulid, PendingRoom>,
    pending_rooms_name_mapping: HashMap<Box<str>, Ulid>,
    rooms: HashMap<Ulid, Room>,
    rooms_name_mapping: HashMap<Box<str>, Ulid>,
}

impl Registry {
    pub async fn reserve(&mut self, name: Box<str>) -> Result<(Ulid, Box<str>), Error> {
        // TODO: cleanup unclaimed rooms after a while.
        // TODO: sanitize room names.

        if self.rooms_name_mapping.contains_key(&name)
            || self.pending_rooms_name_mapping.contains_key(&name)
        {
            return Err(Error::RoomAlreadyExist);
        }

        let id = Ulid::new();
        assert!(self
            .pending_rooms_name_mapping
            .insert(name.clone(), id)
            .is_none());
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
        Ok((id, name))
    }

    pub fn create(
        &mut self,
        id: Ulid,
        socket: WebSocket,
        weak_self: Weak<Mutex<Self>>,
    ) -> Result<(), Error> {
        let Some(name) = self.pending_rooms.remove(&id).map(|r| r.name) else {
            return Err(Error::RoomNotFound);
        };
        assert_eq!(self.pending_rooms_name_mapping.remove(&name), Some(id));

        assert!(self.rooms_name_mapping.insert(name.clone(), id).is_none());
        info!(id = as_display!(id), room = as_display!(name); "room created");
        self.rooms
            .insert(id, Room::new(id, name, socket, weak_self));

        Ok(())
    }

    pub fn remove(&mut self, id: Ulid, name: Box<str>) {
        assert!(self.rooms.remove(&id).is_some());
        assert_eq!(self.rooms_name_mapping.remove(&name), Some(id));
        info!(id = as_display!(id), room = as_display!(name); "room removed");
    }

    pub fn find_room(&self, name: &str) -> Result<(Ulid, Box<str>), Error> {
        self.rooms_name_mapping
            .get(name)
            .copied()
            .map(|id| (id, Box::from(name)))
            .ok_or(Error::RoomNotFound)
    }

    pub fn join_room(&self, id: Ulid, socket: WebSocket, name: Box<str>) -> Result<(), Error> {
        self.rooms
            .get(&id)
            .ok_or(Error::RoomNotFound)?
            .join(socket, name);
        Ok(())
    }
}

struct PendingRoom {
    name: Box<str>,
    creation: Instant,
}
