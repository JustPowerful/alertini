use std::{collections::HashSet, str::FromStr, sync::{Arc, OnceLock}};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Extension, State, WebSocketUpgrade,
    },
    middleware,
    response::IntoResponse,
    routing::get,
    Router,
};
use dashmap::DashMap;
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    common::claims::Claims,
    db::Pool,
    middleware::auth_middleware::auth_middleware,
    models::{alert::{Alert, NewAlert}, vehicle::Vehicle},
    schema::{alerts, vehicles},
};

pub struct AlertController;

// Types of Senders and Receivers for the alert controller
type Tx = mpsc::UnboundedSender<Message>;
// type Rx = mpsc::UnboundedReceiver<Message>;

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
    Subscribe {
        license_plate: String,
    },

    #[serde(rename = "alert")]
    Alert {
        license_plate: String,
        message: String,
    }
}


impl AlertController {
    pub fn app() -> Router<Pool> {
        let protected_routes = Router::new()
        .route("/ws", get(Self::ws_handler))
        .route_layer(middleware::from_fn(auth_middleware));
    
        Router::new().merge(protected_routes)
    }

        async fn handle_socket(mut socket: WebSocket, state: Pool, claims: Claims) {
            let user_uuid = match Uuid::from_str(&claims.sub) {
                Ok(uuid) => uuid,
                Err(_) => {
                    let _ = socket
                        .send(Message::Text(
                            serde_json::to_string(&ServerMessage::Error {
                                message: "Invalid user id in token".to_string(),
                            })
                            .unwrap()
                            .into(),
                        ))
                        .await;
                    return;
                }
            };

            let connection_id = Uuid::new_v4();
            let hub = WsHub::global();
            let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
            let mut subscriptions = HashSet::<String>::new();

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
                                let payload = serde_json::from_str::<ClientMessage>(text.as_str());
                                match payload {
                                    Ok(ClientMessage::Subscribe { license_plate }) => {
                                        if let Err(message) = Self::subscribe_to_plate(&state, hub, &tx, connection_id, user_uuid, &license_plate).await {
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
                                        match Self::create_alert_and_notify(&state, hub, user_uuid, &license_plate, &message).await {
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
                Self::remove_subscriber(hub, &plate, &connection_id);
            }
    }

    pub async fn ws_handler(ws: WebSocketUpgrade, State(_pool): State<Pool>, Extension(claims): Extension<Claims>,) -> impl IntoResponse {
            let pool = _pool.clone();
            ws.on_upgrade(move |socket| async move {
                println!("New websocket connection established!");
                println!("The established user ID is: {}", claims.sub);
                Self::handle_socket(socket, pool, claims).await;
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
            let mut conn = pool.get().map_err(|_| "Failed to get database connection".to_string())?;

            let vehicle = vehicles::table
                .filter(vehicles::license_plate.eq(license_plate).and(vehicles::user_id.eq(user_uuid)))
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

            Ok(())
        }

        async fn create_alert_and_notify(
            pool: &Pool,
            hub: &WsHub,
            user_uuid: Uuid,
            license_plate: &str,
            note: &str,
        ) -> Result<Alert, String> {
            let mut conn = pool.get().map_err(|_| "Failed to get database connection".to_string())?;

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
                let payload = serde_json::to_string(&ServerMessage::AlertCreated { alert: alert.clone() })
                    .map_err(|_| "Failed to serialize alert notification".to_string())?;

                let dead_connections: Vec<Uuid> = subscribers
                    .iter()
                    .filter_map(|entry| {
                        if entry.value().send(Message::Text(payload.clone().into())).is_err() {
                            Some(*entry.key())
                        } else {
                            None
                        }
                    })
                    .collect();

                for connection_id in dead_connections {
                    subscribers.remove(&connection_id);
                }
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
}
