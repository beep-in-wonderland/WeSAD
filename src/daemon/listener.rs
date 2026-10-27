use super::*;

use crate::JoinHandle;
use crate::axum::{
    routing::{post, get},
    extract::{State, Json},
    http::StatusCode,
};
use crate::serde::Deserialize;


#[derive(Deserialize)]
struct StatusUpdate {
    webhook: String,
    update_type: String,
    content: Vec<PlaceholderValue>,
}

impl Display for StatusUpdate {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let mut values_str: String = String::from("\n");

        for pl in &self.content {
            values_str = values_str + format!("{} : \"{}\"", pl.placeholder, pl.value).as_str();
        }

        write!(f, "StatusUpdate:
        webhook: {}
        update_type: {}
        values:{}",
        self.webhook,
        self.update_type,
        values_str
    )
    }
}


#[derive(Deserialize)]
pub struct PlaceholderValue {
    pub placeholder: String,
    pub value: String
}

impl PlaceholderValue {
    pub fn new<T>(placeholder: T, value: T) -> Self 
    where T: Display + Clone,
    {
        PlaceholderValue { placeholder: placeholder.to_string().clone(), value: value.to_string().clone() }
    }
}


pub(super) fn run(guard: Arc<Mutex<database::Session>>, runtime_config: &RuntimeConfig) -> Result<JoinHandle<()>, errors::Error> {
    
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
        .route("/status", post(status_handler))
        .route("/health", get(health_check))
        .with_state(guard);

    let listener = tokio::net::TcpListener::bind(target_ip).await?;
    axum::serve(listener, app).await?;

    Ok(())
}


async fn status_handler(
    State(guard): State<Arc<Mutex<database::Session>>>, 
    Json(payload): Json<StatusUpdate>
) -> StatusCode {

    info!("POST received to HTTP listener for webhook: {}", &payload.webhook);

    if payload.update_type.to_lowercase().trim().ne("status") {
        error!("Incorrect webhook type for endpoint /status");
        return StatusCode::NOT_ACCEPTABLE;
    }

    let session = guard.lock().expect("poisoned");

    for pl in &payload.content {
        if let Err(e) = database::write::update_webhook_placeholder(
            &session, 
            &payload.webhook, 
            &pl.placeholder, 
            &pl.value,
        ) {
            warn!("Placeholder {} : \"{}\" failed to update, error {e}", pl.placeholder, pl.value)
        } else {
            debug!("Placeholder value updated successfully: {}: \"{}\"", pl.placeholder, pl.value);
        }
    };

    StatusCode::OK

}

async fn health_check() -> StatusCode {
    info!("Health check received");
    StatusCode::OK
}