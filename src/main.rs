mod account;

use account::account_update::{AccountUpdate ,account_update_service_client::AccountUpdateServiceClient};
use account::account_update::SubscribeRequest;
use axum::extract::State;
use axum::extract::ws::Message;
use axum::{Router, serve};
use axum::response::Response;
use tokio::net::TcpListener;
use tokio::sync::broadcast::{self, channel};
use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket},
    routing::any,
    };

#[derive(Clone)]
struct AppState {
    broadcast_sender: broadcast::Sender<AccountUpdate>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (sender, _reciever) = channel::<AccountUpdate>(10_000);
    let grpc_sender = sender.clone();

    let app_state = AppState {
        broadcast_sender: sender
    };

    let grpc_process = tokio::spawn(async move {

        let mut client = AccountUpdateServiceClient::connect("http://0.0.0.0:50051".parse::<String>().unwrap()).await?;
        let subscribe_request = SubscribeRequest {};
        let mut stream = client.subscribe_account_updates(subscribe_request).await?.into_inner();
    
        while let Some(acc_update) = stream.message().await? {
            grpc_sender.send(acc_update).ok();
        }

        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    });
    let websocket_process = tokio::spawn(async move {
        let router = Router::new().route("/ws", any(ws_upgrader)).with_state(app_state);

        let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
        serve(listener, router).await.unwrap();

    });

    async fn ws_upgrader(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
        ws.on_upgrade(move |socket| websocket_handler(socket, state))
    }

    async fn websocket_handler(mut socket: WebSocket, state: AppState) {
        let mut subscriber = state.broadcast_sender.subscribe();
        
        while let Ok(acc_update) = subscriber.recv().await {
            let acc_update = Message::Text(serde_json::to_string(&acc_update).unwrap().into());
            if socket.send(acc_update).await.is_err() {
                break;
            }
        }
    }

    grpc_process.await?;
    websocket_process.await?;

    Ok(())
}
