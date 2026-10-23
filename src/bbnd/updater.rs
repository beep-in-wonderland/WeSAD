use super::*;

pub fn start(guard: Arc<Mutex<Session>>) -> Result<(), errors::Error> {
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
    let mut last: DateTime<Local>;
    let mut next: DateTime<Local>;
    let mut time_delta: TimeDelta;

    loop {

        // 1. Lock the mutex
        let session = &guard.lock().expect("poisoned");

        // 2. Fetch the webhooks as a list sorted by next execution
        let webhooks = database::read::webhook_statuses(&session);

        for webhook in webhooks {
            // TODO
        }

        drop(session);
        break;
    }
    Ok(())
}


fn spawn_thread() {}