use super::*;
use super::database::Session;
use crate::TcpListener;
use crate::JoinHandle;
use crate::axum::{
    routing::{post},
    extract::{State, Json},
};
use crate::serde::Deserialize;


#[derive(Deserialize)]
struct PlaceholderUpdate {
    webhook: String,
    placeholder: String,
    value: String,
}

impl Display for PlaceholderUpdate {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PlaceholderUpdate:
        webhook: {}
        placeholder: {}
        value: {}",
        self.webhook,
        self.placeholder,
        self.value
    )
    }
}


pub fn run(guard: Arc<Mutex<database::Session>>, runtime_config: &RuntimeConfig) -> Result<JoinHandle<()>, errors::Error> {
    
    let target_ip = format!("{}:{}", runtime_config.listener_addr, runtime_config.listener_port);
    
    Ok(
        thread::spawn(move||{
            listen_http(guard, target_ip);
        })
    )
}

fn listen_http(guard: Arc<Mutex<database::Session>>, target_ip: String) -> () {

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build listener runtime");

    if let Err(e) = rt.block_on(build_listener(guard, target_ip)) {
        error!("HTTP listener failed with {e}");
    }
}


async fn build_listener(guard: Arc<Mutex<database::Session>>, target_ip: String) -> Result<(), errors::Error> {

    let app: axum::Router = axum::Router::new()
        .route("/placeholder", post(placeholder_handler))
        .with_state(guard);

    let listener = tokio::net::TcpListener::bind(target_ip).await?;
    axum::serve(listener, app).await?;

    Ok(())
}


async fn placeholder_handler(
    State(guard): State<Arc<Mutex<database::Session>>>, 
    Json(payload): Json<PlaceholderUpdate>
) -> axum::http::StatusCode {

    debug!("POST received to HTTP listener:\n{}", payload);

    let session = guard.lock().expect("poisoned");

    if let Ok(_) = database::write::update_webhook_placeholder(
        &session, 
        &payload.webhook, 
        &payload.placeholder, 
        &payload.value,
    ) {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    }

}