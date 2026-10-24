use super::*;
use super::database::Session;
use crate::TcpListener;
use crate::JoinHandle;


pub fn run(guard: Arc<Mutex<database::Session>>) -> Result<JoinHandle<()>, errors::Error> {
    Ok(
        thread::spawn(move||{
            listen_http(guard);
        })
    )
}


fn listen_http(guard: Arc<Mutex<database::Session>>) {}

