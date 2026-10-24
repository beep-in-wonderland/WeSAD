use super::*;
use crate::thread;

/// Spawns a thread which continuously updates status type webhooks with session db values
pub fn run(guard: Arc<Mutex<Session>>) -> Result<(), errors::Error> {
    
    thread::spawn(move || {
        update_loop(guard);
    });

    Ok(())
}


fn update_loop(guard: Arc<Mutex<Session>>) -> (){
    // Each iteration, the loop should:
    // 1. Lock the mutex
    // 2. Fetch all status type webhooks
    // 3. Compare the `last` + `period` fields to the current DateTime
    // 4. Execute any updates where necessary, update the `last` fields
    // 5. Find the minimum seconds of the `last` fields for the next update
    // 6. Copy the data into a local variable `sleep_duration`
    // 7. Release the mutex
    // 8. Put the thread to sleep for `sleep_duration` seconds

    let mut webhooks: &Vec<WebhookDescriptor>;

    let mut period: f64;
    let mut last_millis: i64 = 0;
    let mut next_millis: i64 = 0;
    let mut time_delta: TimeDelta;
    let mut now_millis: i64;

    loop {

        // 1. Lock the mutex
        let session = guard.lock().expect("poisoned");

        // 2. Fetch the webhooks as a list sorted by next execution
        let webhooks = database::read::webhook_statuses(&session).unwrap();

        // 3. Release the mutex guard 
        drop(session);

        now_millis = chrono::Utc::now().timestamp_millis();

        for webhook in webhooks {
            if webhook.should_update(&now_millis) {
                webhook.send()
            }
        }



        break;
    }
    

}


fn sleep_to_retry(duration: Duration) {}
