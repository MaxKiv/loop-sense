use crate::axumstate::AxumState;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::Response;
use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::time::{self, Duration};
use tracing::*;

const MEASUREMENT_SEND_PERIOD: Duration = Duration::from_millis(20);

/// Attempt to establish websocket
pub async fn handle_websocket_request(
    ws: WebSocketUpgrade,
    State(state): State<AxumState>,
) -> Response {
    ws.on_upgrade(|socket| handle_ws_measurements(socket, state))
}

/// Continously send latest measurements over websocket
async fn handle_ws_measurements(socket: WebSocket, state: AxumState) {
    let (mut sender, _) = socket.split();

    tokio::spawn(async move {
        let mut interval = time::interval(MEASUREMENT_SEND_PERIOD);

        loop {
            // Get the latest report, if it exists
            let report = {
                // Attempt to lock the report mutex in axumstate
                // Note: lock scope is strictly limited and cannot be held across an await (like
                // the one below)
                let guard = match state.report.lock() {
                    Ok(g) => g,
                    Err(_) => return,
                };

                // Was a report already created?
                match &*guard {
                    // Yes -> return it
                    Some(r) => r.clone(),
                    // No -> pass
                    _ => return,
                }
            };

            if let Ok(bytes) = serde_json::to_vec(&report) {
                let msg = Message::Binary(bytes.into());

                if let Err(e) = sender.send(msg).await {
                    error!("WebSocket send failed: {}", e);
                    return;
                }
            }

            interval.tick().await;
        }
    });
}
