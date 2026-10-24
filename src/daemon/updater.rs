use super::*;
use crate::database::write;
use crate::errors::Error::HttpErr;
use crate::errors::ErrorObj;
use crate::{daemon::io::open_as_string};
use crate::thread::{self, JoinHandle};
use crate::HashMap;
use crate::reqwest;

const URI_FLAGS_COMPONENT: &str = "with_components=true";
const URI_FLAGS_WAIT: &str = "wait=true";

/// Spawns a thread which continuously updates status type webhooks with session db values
pub fn run(guard: Arc<Mutex<Session>>) -> Result<JoinHandle<()>, errors::Error> {
    
    Ok(thread::spawn(move || {
        update_loop(guard);
    }))
}


fn update_loop(guard: Arc<Mutex<Session>>) -> () {
    // Each iteration, the loop should:
    // 1. Lock the mutex
    // 2. Fetch all status type webhooks
    // 3. Compare the `last` + `period` fields to the current DateTime
    // 4. Execute any updates where necessary, update the `last` fields
    // 5. Find the minimum seconds of the `last` fields for the next update
    // 6. Copy the data into a local variable `sleep_duration`
    // 7. Release the mutex
    // 8. Put the thread to sleep for `sleep_duration` seconds

    let mut webhooks: &Vec<webhook::Status>;

    let mut period: f64;
    let mut last_millis: i64 = 0;
    let mut next_millis: i64 = 0;
    let mut time_delta: TimeDelta;
    let mut now_millis: i64;
    
    let rt = tokio::runtime::Runtime::new().unwrap();

    loop {

        let session = guard.lock().expect("poisoned");
        let webhooks = database::read::webhook_statuses(&session).unwrap();
        now_millis = chrono::Utc::now().timestamp_millis();

        // Debug line for timestamping, remove when finished
        let _ = write::update_webhook_placeholder(
            &session, 
            &"bbnd-status".to_string(), 
            &"timestamp".to_string(), 
            &now_millis.to_string());
    
        // Release the mutex guard 
        drop(session);

        for webhook in webhooks {
            if webhook.should_update(&now_millis) {
                rt.block_on(update_status_webhook(&guard, &webhook)).unwrap();
            }
        }

        break;
    }
    

}


fn sleep_to_retry(duration: Duration) {}


async fn update_status_webhook(guard: &Arc<Mutex<Session>>, status: &webhook::Status) -> Result<(), errors::Error> {

    let mut content = open_as_string(&status.path, &status.name, ".json")?;
    update_content_string(&guard, &mut content, &status.name)?;

    dbg!(&content);

    if status.msg_id == 0 {
        // No existing message id, so we should send a new message rather than patch an existing one
        let msg_id = send_new_webhook(status, &content).await?;
    } else {
        patch_existing_webhook(status, &content).await?;
    };

    Ok(())
}


fn update_content_string(guard: &Arc<Mutex<Session>>, content: &mut String, name: &String) -> Result<(), errors::Error> {
    
    let session = guard.lock().expect("poisoned");
    let placeholder_map = database::read::all_placeholders_by_webhook_name(&session, name)?;
    // Release the mutex lock
    drop(session);

    do_placeholder_replacements(placeholder_map, content)?;

    Ok(())
}


pub(super) fn do_placeholder_replacements(map: HashMap<String, String>, content: &mut String) -> Result<(), errors::Error> {

    let mut temp: String = content.clone();

    for (k, v) in map.iter() {
        temp = temp.replace(
            &format!("$${k}$$"),
            v
        );
    }

    content.clone_from(&temp);

    Ok(())
}


async fn send_new_webhook(status: &webhook::Status, content: &String) -> Result<String, errors::Error> {

    let uri = format!("{}?{}&{}", status.url, URI_FLAGS_COMPONENT, URI_FLAGS_WAIT);

    let client = reqwest::Client::new();

    let request = client.post(uri)
        .header("Content-Type", "application/json")
        .body(content.clone())
        .send()
        .await?;

    dbg!("POST sent. Response: {}",&request);
    
    match request.status() {
        reqwest::StatusCode::OK => {
            let message = json::parse(&request.text().await?.to_string())?;
            Ok(message["id"].to_string())
        },
        _ => Err(HttpErr(format!("PATCH failed with code: {}",
            request.status().as_u16()
        )))
    }
    
}


async fn patch_existing_webhook(status: &webhook::Status, content: &String) -> Result<(), errors::Error> {

    let uri = format!("{}/messages/{}?{}&{}", status.url, status.msg_id, URI_FLAGS_COMPONENT, URI_FLAGS_WAIT);

    let client = reqwest::Client::new();

    let request = client.patch(uri)
        .header("Content-Type", "application/json")
        .body(content.clone())
        .send()
        .await?;

    dbg!("PATCH sent. Response: {}",&request);

    match request.status() {
        reqwest::StatusCode::OK => Ok(()),
        _ => Err(HttpErr(format!("PATCH failed with code: {}",
            request.status().as_u16()
        )))
    }

}

