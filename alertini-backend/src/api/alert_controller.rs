use axum::{Extension, Router, extract::{State, WebSocketUpgrade, ws::Message}, middleware, response::IntoResponse, routing::get};

use crate::{common::claims::Claims, db::Pool, middleware::auth_middleware::auth_middleware};

pub struct AlertController;
impl AlertController {
    pub fn app() -> Router<Pool> {
        let protected_routes = Router::new()
        .route("/ws", get(Self::ws_handler))
        .route_layer(middleware::from_fn(auth_middleware));
    
        Router::new().merge(protected_routes)
    }

    pub async fn ws_handler(ws: WebSocketUpgrade, State(_pool): State<Pool>, Extension(claims): Extension<Claims>,) -> impl IntoResponse {
        ws.on_upgrade(|mut socket| async move {
            println!("New websocket connection established!");
            println!("The established user ID is: {}", claims.sub);
            // Loop in the handler to keep listening for websocket messages
            // The websocket will get json data so you can process later
            while let Some(msg) = socket.recv().await {
            match msg {
                // This will listen to text Utf8Bytes so and then read the Json from it
                Ok(Message::Text(text)) => {
                    println!("Received: {}", text);

                    if socket
                        .send(Message::Text(format!("Echo: {}", text).into()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }

                // This will trigger when the established websocket is finally disconnected
                Ok(Message::Close(_)) => {
                    println!("Client disconnected");
                    break;
                }

                _ => {}
            }
    }

        })
    }
}
