use std::collections::HashMap;
use std::sync::Weak;
use std::time::Instant;

use axum::extract::ws::WebSocket;
use log::{as_display, info};
use tokio::sync::Mutex;
use ulid::Ulid;

use crate::error::Error;
use crate::room::Room;
use crate::utils;

const ROOM_NAME_MIN_LEN: usize = 3;

#[derive(Default)]
pub struct Registry {
    pending_rooms: HashMap<Ulid, PendingRoom>,
    pending_rooms_name_mapping: HashMap<Box<str>, Ulid>,
    rooms: HashMap<Ulid, Room>,
    rooms_name_mapping: HashMap<Box<str>, Ulid>,
}

impl Registry {
    pub async fn reserve(&mut self, name: &str) -> Result<(Ulid, Box<str>), Error> {
        // TODO: cleanup unclaimed rooms after a while.

        let search_sanitized = utils::sanitize_for_search(name);
        if search_sanitized.len() < ROOM_NAME_MIN_LEN {
            return Err(Error::RoomNameTooShort);
        }

        if self.rooms_name_mapping.contains_key(&search_sanitized)
            || self
                .pending_rooms_name_mapping
                .contains_key(&search_sanitized)
        {
            return Err(Error::RoomAlreadyExist);
        }

        let id = Ulid::new();
        let name = utils::sanitize(name).to_owned().into_boxed_str();

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
        assert!(self
            .pending_rooms_name_mapping
            .insert(search_sanitized, id)
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
        let search_sanitized = utils::sanitize_for_search(&name);
        assert_eq!(
            self.pending_rooms_name_mapping.remove(&search_sanitized),
            Some(id)
        );

        assert!(self
            .rooms_name_mapping
            .insert(search_sanitized, id)
            .is_none());
        info!(id = as_display!(id), room = as_display!(name); "room created");
        self.rooms
            .insert(id, Room::new(id, name, socket, weak_self));

        Ok(())
    }

    pub fn remove(&mut self, id: Ulid, name: Box<str>) {
        assert!(self.rooms.remove(&id).is_some());
        assert_eq!(
            self.rooms_name_mapping
                .remove(&utils::sanitize_for_search(&name)),
            Some(id)
        );
        info!(id = as_display!(id), room = as_display!(name); "room removed");
    }

    pub fn find_room(&self, name: &str) -> Result<(Ulid, Box<str>), Error> {
        self.rooms_name_mapping
            .get(&utils::sanitize_for_search(name))
            .copied()
            .and_then(|id| self.rooms.get(&id).map(|r| (id, r.name.clone())))
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
