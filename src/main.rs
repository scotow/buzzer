use crate::registry::Registry;
use axum::extract::ws::WebSocket;
use axum::extract::{Path, Query, State, WebSocketUpgrade};
use axum::http::{header, HeaderValue};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::set_header::SetResponseHeaderLayer;
use ulid::Ulid;

mod asset;
mod packet;
mod registry;
mod room;

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/rooms", post(reserve_room))
        .route("/rooms/id", get(find_room_by_name))
        .route("/rooms/:id/host", get(host_room))
        .route("/rooms/:id/participate", get(participate_room))
        .with_state(Arc::new(Mutex::new(Registry::default())))
        .route("/", get(asset::handler))
        .route("/:asset", get(asset::handler))
        .layer(SetResponseHeaderLayer::overriding(
            header::SERVER,
            HeaderValue::from_static(concat!("Buzzer v", env!("CARGO_PKG_VERSION"))),
        ));

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
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
) -> impl IntoResponse {
    registry
        .lock()
        .await
        .reserve(request.name.into_boxed_str())
        .await
        .unwrap()
        .to_string()
}

async fn host_room(
    State(registry): State<Arc<Mutex<Registry>>>,
    Path(id): Path<Ulid>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let weak_registry = Arc::downgrade(&registry);
        registry.lock().await.create(id, socket, weak_registry);
    })
}

#[derive(Deserialize)]
struct FindRoomByNameQuery {
    name: String,
}

async fn find_room_by_name(
    State(registry): State<Arc<Mutex<Registry>>>,
    Query(FindRoomByNameQuery { name }): Query<FindRoomByNameQuery>,
) -> impl IntoResponse {
    registry.lock().await.find_room(&name).unwrap().to_string()
}

#[derive(Deserialize)]
struct JoinRoomQuery {
    name: String,
}

async fn participate_room(
    State(registry): State<Arc<Mutex<Registry>>>,
    Path(id): Path<Ulid>,
    Query(JoinRoomQuery { name }): Query<JoinRoomQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        registry
            .lock()
            .await
            .join_room(id, socket, name.into_boxed_str());
    })
}
