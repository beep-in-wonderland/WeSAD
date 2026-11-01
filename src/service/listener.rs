use super::*;

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
    content: Vec<TagValue>,
}

impl Display for StatusUpdate {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let mut values_str: String = String::from("\n");

        for pl in &self.content {
            values_str = values_str + format!("{} : \"{}\"", pl.tag, pl.value).as_str();
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
pub struct TagValue {
    pub tag: String,
    pub value: String
}

impl TagValue {
    pub fn new<T>(tag: T, value: T) -> Self 
    where T: Display + Clone,
    {
        TagValue { tag: tag.to_string().clone(), value: value.to_string().clone() }
    }
}


pub(super) fn run(guard: Arc<Mutex<database::Session>>, runtime_config: &io::RuntimeConfig) 
-> Result<ThreadHandle<Result<(), errors::Error>>, errors::Error> {
    
    let target_ip = format!("{}:{}", runtime_config.listener_addr, runtime_config.listener_port);
    
    Ok(
        ThreadHandle::Listener(
            thread::spawn(move||{
                listen_http(guard, target_ip)
            })
        )
    )
}



fn listen_http(guard: Arc<Mutex<database::Session>>, target_ip: String) -> Result<(), errors::Error> {

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build listener runtime");

    loop {

        // Continuously try and listen on the provided address.
        if let Err(e) = rt.block_on(build_listener(&guard, &target_ip)) {
            error!("HTTP listener failed: {e}");
        }

        // Only try and restart once per second.
        thread::sleep(std::time::Duration::from_secs(1));

        // TODO implement a dynamic error counter to decide when "too many recent errors"
        // means that the program should abort.
    }
}


async fn build_listener(guard: &Arc<Mutex<database::Session>>, target_ip: &String) -> Result<(), errors::Error> {

    let app: axum::Router = axum::Router::new()
        .route("/status", post(status_handler))
        .route("/health", get(health_check))
        .with_state(guard.clone());

    let listener = tokio::net::TcpListener::bind(target_ip).await?;
    axum::serve(listener, app).await?;

    Ok(())
}


/// --------------------------------- Endpoint functions

async fn status_handler(
    State(guard): State<Arc<Mutex<database::Session>>>, 
    Json(payload): Json<StatusUpdate>
) -> StatusCode {

    info!("POST received to HTTP listener for webhook: {}", &payload.webhook);

    if payload.update_type.to_lowercase().trim().ne("status") {
        error!("Incorrect webhook type for endpoint /status");
        return StatusCode::NOT_ACCEPTABLE;
    }

    let session = guard.lock().expect("Mutex poisoned in listener status_handler. Exiting thread.");

    for pl in &payload.content {
        if let Err(e) = database::write::update_webhook_tag(
            &session, 
            &payload.webhook, 
            &pl.tag, 
            &pl.value,
        ) {
            warn!("Tag {} : \"{}\" failed to update, error {e}", pl.tag, pl.value)
        } else {
            debug!("Tag value updated successfully: {}: \"{}\"", pl.tag, pl.value);
        }
    };

    StatusCode::OK

}

async fn health_check() -> StatusCode {
    info!("Health check received");
    StatusCode::OK
}