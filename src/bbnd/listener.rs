use super::*;
use super::database::Session;


pub fn start(guard: Arc<Mutex<database::Session>>) -> Result<(), errors::Error> {
    Ok(())
}


fn listen_http() {}

