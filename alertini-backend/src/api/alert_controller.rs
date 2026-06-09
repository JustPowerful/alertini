use std::{
    collections::HashSet,
    str::FromStr,
    sync::{Arc, OnceLock},
};

use axum::{
    Json, Router, extract::{
        Extension, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    }, http::StatusCode, middleware, response::IntoResponse, routing::get
};
use dashmap::DashMap;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, OptionalExtension, RunQueryDsl,
    SelectableHelper, query_dsl::methods::FilterDsl,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    common::claims::Claims, db::Pool, middleware::auth_middleware::auth_middleware, models::{
        alert::{Alert, GetVehicleAlertPayload, NewAlert},
        vehicle::Vehicle,
    }, responses::api_response::ApiResponse, schema::{alerts, vehicles}
};

pub struct AlertController;

// Types of Senders and Receivers for the alert controller
type Tx = mpsc::UnboundedSender<Message>;
type SubscriberMap = DashMap<Uuid, Tx>;

static WS_HUB: OnceLock<WsHub> = OnceLock::new();

#[derive(Clone)]
pub struct WsHub {
    pub channels: Arc<DashMap<String, SubscriberMap>>,
}

impl WsHub {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(DashMap::new()),
        }
    }

    pub fn global() -> &'static Self {
        WS_HUB.get_or_init(Self::new)
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
enum ServerMessage {
    Info { message: String },
    Error { message: String },
    Subscribed { license_plate: String },
    AlertCreated { alert: Alert },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum ClientMessage {
    #[serde(rename = "subscribe")]
    Subscribe { license_plate: String },

    #[serde(rename = "alert")]
    Alert {
        license_plate: String,
        message: String,
    },
}

impl AlertController {
    pub fn app() -> Router<Pool> {
        let protected_routes = Router::new()
            .route("/list", get(list_alerts))
            .route_layer(middleware::from_fn(auth_middleware));

        // WebSocket route is not protected by middleware - auth happens in the handler
        let public_routes = Router::new()
            .route("/ws", get(ws_handler));

        Router::new().merge(protected_routes).merge(public_routes)
    }

    pub async fn health(pool: &Pool) -> bool {
        pool.get().is_ok()
    }
}

// Normal HTTP handlers
#[utoipa::path(
    get,
    path = "/alert/list",
    tag = "Alerts",
    security(("api_jwt_token" = [])),
    request_body = GetVehicleAlertPayload,
    responses(
        (status = 200, description = "List of alerts for the given vehicle"),
        (status = 403, description = "Forbidden — vehicle does not belong to user"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_alerts(
    State(pool): State<Pool>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<GetVehicleAlertPayload>,
) -> (StatusCode, Json<ApiResponse<Vec<Alert>>>) {
    let pool_clone = pool.clone();
    let mut conn = pool_clone.get().expect("Failed to get connection.");

    let user_uuid = Uuid::from_str(&claims.sub).expect("Failed to parse user ID");

    let user_has_vehicle = vehicles::table
        .filter(
            vehicles::license_plate
                .eq(&body.license_plate)
                .and(vehicles::user_id.eq(user_uuid)),
        )
        .first::<Vehicle>(&mut conn)
        .optional()
        .expect("There was a problem in deleting vehicle.");

    match user_has_vehicle {
        Some(vehicle) => {
            let alerts = alerts::table
                .filter(alerts::car_id.eq(vehicle.id))
                .load::<Alert>(&mut conn)
                .expect("There's a problem with fetching vehicle alerts.");
            (StatusCode::OK, Json(ApiResponse::success("Successfully loaded vehicle notifications", alerts)))
        }
        None => {
            (StatusCode::FORBIDDEN, Json(ApiResponse::error("You cannot see the alerts of this vehicle.")))
        }
    }
}

// WebSocket handlers
async fn handle_socket(mut socket: WebSocket, state: Pool) {
    let connection_id = Uuid::new_v4();
    let hub = WsHub::global();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    let mut subscriptions = HashSet::<String>::new();
    let mut user_uuid: Option<Uuid> = None;
    let mut authenticated = false;

    let _ = tx.send(Message::Text(
        serde_json::to_string(&ServerMessage::Info {
            message: "Websocket connected".to_string(),
        })
        .unwrap()
        .into(),
    ));

    loop {
        tokio::select! {
            Some(outgoing) = rx.recv() => {
                if socket.send(outgoing).await.is_err() {
                    break;
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        if !authenticated {
                            if let Ok(serde_json::Value::Object(obj)) = serde_json::from_str::<serde_json::Value>(text.as_str()) {
                                if let Some(action) = obj.get("action").and_then(|v| v.as_str()) {
                                    if action == "auth" {
                                        if let Some(token) = obj.get("token").and_then(|v| v.as_str()) {
                                            let secret = std::env::var("JWT_SECRET_KEY")
                                                .unwrap_or_default();
                                            match jsonwebtoken::decode::<Claims>(
                                                token,
                                                &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
                                                &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
                                            ) {
                                                Ok(token_data) => {
                                                    if let Ok(uuid) = Uuid::from_str(&token_data.claims.sub) {
                                                        user_uuid = Some(uuid);
                                                        authenticated = true;
                                                        let _ = tx.send(Message::Text(
                                                            serde_json::to_string(&ServerMessage::Info {
                                                                message: "Authenticated successfully".to_string(),
                                                            })
                                                            .unwrap()
                                                            .into(),
                                                        ));
                                                    }
                                                }
                                                Err(_) => {
                                                    let _ = tx.send(Message::Text(
                                                        serde_json::to_string(&ServerMessage::Error {
                                                            message: "Invalid token".to_string(),
                                                        })
                                                        .unwrap()
                                                        .into(),
                                                    ));
                                                }
                                            }
                                        }
                                        continue;
                                    }
                                }
                            }
                            // Not authenticated and didn't receive valid auth message
                            let _ = tx.send(Message::Text(
                                serde_json::to_string(&ServerMessage::Error {
                                    message: "Authentication required".to_string(),
                                })
                                .unwrap()
                                .into(),
                            ));
                            break;
                        }

                        // Process messages only if authenticated
                        if let Some(uuid) = user_uuid {
                            let payload = serde_json::from_str::<ClientMessage>(text.as_str());
                            match payload {
                                Ok(ClientMessage::Subscribe { license_plate }) => {
                                    if let Err(message) = subscribe_to_plate(&state, hub, &tx, connection_id, uuid, &license_plate).await {
                                        let _ = tx.send(Message::Text(
                                            serde_json::to_string(&ServerMessage::Error { message }).unwrap().into(),
                                        ));
                                    } else {
                                        subscriptions.insert(license_plate.clone());
                                        let _ = tx.send(Message::Text(
                                            serde_json::to_string(&ServerMessage::Subscribed { license_plate }).unwrap().into(),
                                        ));
                                    }
                                }
                                Ok(ClientMessage::Alert { license_plate, message }) => {
                                    match create_alert_and_notify(&state, hub, uuid, &license_plate, &message).await {
                                        Ok(alert) => {
                                            let _ = tx.send(Message::Text(
                                                serde_json::to_string(&ServerMessage::AlertCreated { alert }).unwrap().into(),
                                            ));
                                        }
                                        Err(message) => {
                                            let _ = tx.send(Message::Text(
                                                serde_json::to_string(&ServerMessage::Error { message }).unwrap().into(),
                                            ));
                                        }
                                    }
                                }
                                Err(_) => {
                                    let _ = tx.send(Message::Text(
                                        serde_json::to_string(&ServerMessage::Error {
                                            message: "Invalid websocket payload".to_string(),
                                        })
                                        .unwrap()
                                        .into(),
                                    ));
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = tx.send(Message::Pong(data));
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) => {
                        break;
                    }
                }
            }
        }
    }

    for plate in subscriptions {
        remove_subscriber(hub, &plate, &connection_id);
    }
}

#[utoipa::path(
    get,
    path = "/alert/ws",
    tag = "Alerts",
    responses(
        (status = 101, description = "WebSocket upgrade — send {\"action\":\"auth\",\"token\":\"<JWT>\"} first, then subscribe/alert messages"),
    )
)]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(_pool): State<Pool>,
) -> impl IntoResponse {
    let pool = _pool.clone();
    ws.on_upgrade(move |socket| async move {
        println!("New websocket connection established!");
        handle_socket(socket, pool).await;
    })
}

async fn subscribe_to_plate(
    pool: &Pool,
    hub: &WsHub,
    tx: &Tx,
    connection_id: Uuid,
    user_uuid: Uuid,
    license_plate: &str,
) -> Result<(), String> {
    let mut conn = pool
        .get()
        .map_err(|_| "Failed to get database connection".to_string())?;

    let vehicle = vehicles::table
        .filter(
            vehicles::license_plate
                .eq(license_plate)
                .and(vehicles::user_id.eq(user_uuid)),
        )
        .first::<Vehicle>(&mut conn)
        .optional()
        .map_err(|_| "Failed to validate vehicle ownership".to_string())?;

    if vehicle.is_none() {
        return Err("You cannot subscribe to a vehicle you do not own".to_string());
    }

    let subscriber_map = hub
        .channels
        .entry(license_plate.to_string())
        .or_insert_with(DashMap::new);
    subscriber_map.insert(connection_id, tx.clone());

    println!("WS: user {} subscribed connection {} to {}", user_uuid, connection_id, license_plate);
    let count = subscriber_map.len();
    println!("WS: current subscribers for {} = {}", license_plate, count);

    Ok(())
}

async fn create_alert_and_notify(
    pool: &Pool,
    hub: &WsHub,
    user_uuid: Uuid,
    license_plate: &str,
    note: &str,
) -> Result<Alert, String> {
    let mut conn = pool
        .get()
        .map_err(|_| "Failed to get database connection".to_string())?;

    let vehicle = vehicles::table
        .filter(vehicles::license_plate.eq(license_plate))
        .first::<Vehicle>(&mut conn)
        .optional()
        .map_err(|_| "Failed to search for the vehicle".to_string())?;

    let vehicle = match vehicle {
        Some(vehicle) => vehicle,
        None => return Err("Vehicle not found".to_string()),
    };

    let new_alert = NewAlert {
        car_id: vehicle.id,
        note: note.to_string(),
        reporter_id: user_uuid,
    };

    let alert: Alert = diesel::insert_into(alerts::table)
        .values(&new_alert)
        .returning(Alert::as_returning())
        .get_result(&mut conn)
        .map_err(|_| "Failed to store the alert".to_string())?;

    if let Some(subscribers) = hub.channels.get(license_plate) {
        let payload = serde_json::to_string(&ServerMessage::AlertCreated {
            alert: alert.clone(),
        })
        .map_err(|_| "Failed to serialize alert notification".to_string())?;

        let subscriber_count = subscribers.len();
        println!("WS: notifying {} subscribers for plate {}", subscriber_count, license_plate);

        let mut dead_connections: Vec<Uuid> = Vec::new();
        for entry in subscribers.iter() {
            let conn_id = *entry.key();
            let sender = entry.value();
            if sender.send(Message::Text(payload.clone().into())).is_err() {
                println!("WS: failed to send to connection {} for plate {}", conn_id, license_plate);
                dead_connections.push(conn_id);
            }
        }

        for connection_id in dead_connections {
            subscribers.remove(&connection_id);
        }
    } else {
        println!("WS: no subscribers for plate {}", license_plate);
    }

    Ok(alert)
}

fn remove_subscriber(hub: &WsHub, license_plate: &str, connection_id: &Uuid) {
    let should_remove = if let Some(subscribers) = hub.channels.get(license_plate) {
        subscribers.remove(connection_id);
        subscribers.is_empty()
    } else {
        false
    };

    if should_remove {
        hub.channels.remove(license_plate);
    }
}
