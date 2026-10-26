use std::time::Duration;

use super::*;
use crate::errors::Error::HttpErr;
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
    // TODO - implement fallback logic for errors

    // Each iteration, the loop should:
    // 1. Lock the mutex
    // 2. Fetch all status type webhooks
    // 3. Compare the `last` + `period` fields to the current DateTime
    // 4. Execute any updates where necessary, update the `last` fields
    // 5. Find the minimum seconds of the `last` fields for the next update
    // 6. Copy the data into a local variable `sleep_duration`
    // 7. Release the mutex
    // 8. Put the thread to sleep for `sleep_duration` seconds

    let mut sleep_target_timestamp;
    let mut next_timestamp;
    let mut now_millis: i64;
    let mut sleep_duration_millis: u64 = 0;
    
    let rt = tokio::runtime::Runtime::new().unwrap();

    loop {
        debug!("Starting updater loop iteration");

        let session = guard.lock().expect("poisoned");
        let webhooks = database::read::webhook_statuses(&session).unwrap();

        now_millis = chrono::Utc::now().timestamp_millis();
        sleep_target_timestamp = 1<<60; // A big number to start with

        // Debug line for timestamping, remove when finished
        // let _ = write::update_webhook_placeholder(
        //     &session, 
        //     &"bbnd-status".to_string(), 
        //     &"timestamp".to_string(), 
        //     &now_millis.to_string());
    
        // Release the mutex guard 
        drop(session);

        for webhook in webhooks {
            if webhook.should_update(&now_millis) {
                let msg_id = rt.block_on(update_status_webhook(&guard, &webhook)).unwrap();


                let session = guard.lock().expect("poisoned");
                if msg_id.is_some() {
                    // If a new msg_id was returned, update it in the database
                    database::write::update_webhook_msg_id(&session, &webhook.name, &msg_id.unwrap()).unwrap();
                }
                // Update the last execution of the webhook to the current timestamp in milliseconds
                database::write::update_webhook_last(&session, &webhook.name, &now_millis.to_string()).unwrap();

                // Since we just executed, webhook.last is now stale. Use now_millis instead for finding next execution time
                next_timestamp = webhook.period + now_millis;

            } else {
                // If no update occurred, use the last execution from the webhook status
                next_timestamp = webhook.period + webhook.last;
            }

            // Lastly, check if the current time delta is the lowest, update if so
            if next_timestamp < sleep_target_timestamp {
                sleep_target_timestamp = next_timestamp;
                sleep_duration_millis = (sleep_target_timestamp - now_millis) as u64;
                debug!("New shortest sleep target for update loop: {}", &sleep_duration_millis / 1000)
            }

        }

        // With the shortest next sleep target found, put the thread to sleep for that long
        debug!("Putting updater thread to sleep for {} seconds", &sleep_duration_millis / 1000);
        thread::sleep(Duration::from_millis(sleep_duration_millis));

    }
    

}


async fn update_status_webhook(guard: &Arc<Mutex<Session>>, status: &webhook::Status) -> Result<Option<String>, errors::Error> {

    let mut content = open_as_string(&status.path, &status.name, ".json")?;
    update_content_string(&guard, &mut content, &status.name)?;

    dbg!(&content);

    if status.msg_id == 0 {
        // No existing message id, so we should send a new message rather than patch an existing one
        let msg_id = post_new_webhook(status, &content).await?;
        return Ok(Some(msg_id));
    } else {

        // There is a message id in the table. We should check it's valid before proceeding.
        if check_webhook_message_exists(guard, status).await? {

            // The message id is valid, proceed with a regular patch
            patch_existing_webhook(status, &content).await?;

        } else {

            // Invalid message id, so we should send a new message
            let msg_id = post_new_webhook(status, &content).await?;
            return Ok(Some(msg_id));
        }

    };
    
    Ok(None)
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


async fn post_new_webhook(status: &webhook::Status, content: &String) -> Result<String, errors::Error> {

    let uri = format!("{}?{}&{}", status.url, URI_FLAGS_COMPONENT, URI_FLAGS_WAIT);

    let client = reqwest::Client::new();

    let request = client.post(uri)
        .header("Content-Type", "application/json")
        .body(content.clone())
        .send()
        .await?;

    info!("POST sent for {}", &status.name);
    
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

    info!("PATCH sent for {}", &status.name);

    match request.status() {
        reqwest::StatusCode::OK => Ok(()),
        _ => Err(HttpErr(format!("PATCH failed with code: {} - {}",
            request.status().as_u16(),
            request.text().await.unwrap()
        )))
    }

}


async fn check_webhook_message_exists(guard: &Arc<Mutex<Session>>, status: &webhook::Status) -> Result<bool, errors::Error> {

    let uri = format!("{}/messages/{}?{}&{}", status.url, status.msg_id, URI_FLAGS_COMPONENT, URI_FLAGS_WAIT);

    let client = reqwest::Client::new();

    let request = client.get(uri)
        .header("Content-Type", "application/json")
        .send()
        .await?;

    debug!("GET sent for {}", &status.name);
    
    match request.status() {
        reqwest::StatusCode::OK => {
            debug!("Valid message id for {}", status.name);
            Ok(true)
        },
        _ => {
            debug!("Invalid message id for {}. Sending new message.", status.name);
            Ok(false)
        },
    }
    
}