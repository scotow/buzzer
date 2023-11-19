use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;
use tower_http::set_header::SetResponseHeaderLayer;
use ulid::Ulid;

use crate::{error::Error, options::Options, registry::Registry};

mod asset;
mod error;
mod options;
mod packet;
mod registry;
mod room;
mod utils;

const USERNAME_MIN_LEN: usize = 2;

#[tokio::main]
async fn main() {
    let options = Options::parse();
    env_logger::Builder::new()
        .filter_level(options.log_level())
        .init();
    log_panics::init();

    let router = Router::new()
        .route("/rooms", post(reserve_room))
        .route("/rooms/id", get(find_room_by_name))
        .route("/rooms/:id/host", get(host_room))
        .route("/rooms/:id/participate", get(join_room))
        .with_state(Arc::new(Mutex::new(Registry::default())))
        .route("/", get(asset::handler))
        .route("/:asset", get(asset::handler))
        .layer(SetResponseHeaderLayer::overriding(
            header::SERVER,
            HeaderValue::from_static(concat!("Buzzer v", env!("CARGO_PKG_VERSION"))),
        ));

    axum::Server::bind(&SocketAddr::new(options.address, options.port))
        .serve(router.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize)]
struct ReserveRoom {
    name: String,
}

async fn reserve_room(
    State(registry): State<Arc<Mutex<Registry>>>,
    Json(request): Json<ReserveRoom>,
) -> Result<impl IntoResponse, Error> {
    let weak_registry = Arc::downgrade(&registry);
    let (id, name) = registry
        .lock()
        .await
        .reserve(&request.name, weak_registry)
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "id": id,
            "name": name,
        })),
    ))
}

async fn host_room(
    State(registry): State<Arc<Mutex<Registry>>>,
    Path(id): Path<Ulid>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let weak_registry = Arc::downgrade(&registry);
        let _ = registry.lock().await.create(id, socket, weak_registry);
    })
}

#[derive(Deserialize)]
struct FindRoomByNameQuery {
    name: String,
}

async fn find_room_by_name(
    State(registry): State<Arc<Mutex<Registry>>>,
    Query(FindRoomByNameQuery { name }): Query<FindRoomByNameQuery>,
) -> Result<impl IntoResponse, Error> {
    let (id, name) = registry.lock().await.find_room(&name)?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "id": id,
            "name": name,
        })),
    ))
}

#[derive(Deserialize)]
struct JoinRoomQuery {
    name: String,
}

async fn join_room(
    State(registry): State<Arc<Mutex<Registry>>>,
    Path(id): Path<Ulid>,
    Query(JoinRoomQuery { name }): Query<JoinRoomQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let name = utils::sanitize(&name).to_owned().into_boxed_str();
    if name.len() < USERNAME_MIN_LEN {
        return Err(Error::UsernameTooShort);
    }
    Ok(ws.on_upgrade(move |socket| async move {
        let _ = registry.lock().await.join_room(id, socket, name);
    }))
}
