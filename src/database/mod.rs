pub mod read;
pub mod write;
mod schema;

#[cfg(test)]
mod tests;

use std::fs::File;
use std::ops::Not;

use dbus::arg::RefArg;

use crate::bbnd::WebhookDescriptor;
use crate::{Path,PathBuf};
use crate::ProgramPaths;
use crate::sqlite::{self, Connection};
use crate::errors;
use crate::{info,error,debug,warn};

pub const TAB_WEBHOOKS: &str = "webhooks";
pub const COL_WEBHOOK_NAME: &str = "webhook";
pub const COL_WEBHOOK_URL: &str = "url";
pub const COL_WEBHOOK_TYPE: &str = "webhook_type";
pub const COL_PERIOD: &str = "period";
pub const COL_ORPHANED: &str = "orphaned";
pub const COL_MESSAGE_ID: &str = "messageId";
pub const COL_LAST: &str = "last";

pub const TAB_PLACEHOLDERS: &str = "placeholders";
pub const COL_WEBHOOK_PLACEHOLDER: &str = "placeholder";
pub const COL_WEBHOOK_PLACEHOLDER_VALUE: &str = "value";


pub struct Session {
    connection : sqlite::Connection,
}


pub struct WebhookStatus {
    pub name: String,
    pub url: String,
    pub period: f64,
    pub last: u64,
    pub msg_id: u64,
}


impl Session {

    /// Check for existing `bbdb.db` file and create a new one if none is found. This 
    /// function does not perform any validation on the database contents. It only
    /// establishes the connection and returns the object through which the database
    /// can be accessed.
    pub fn init(paths: &ProgramPaths) -> Result<Self, errors::Error> {

        let db_path: &PathBuf = &paths.persist.join("bbnd.db");
        let connection = Session::open_db_at_path(db_path)?;

        debug!("Database initialization complete");
        Ok(Self {connection,})
    }

    pub fn init_from_path(path_str: &str) -> Result<Self, errors::Error> {

        let db_path: &PathBuf = &PathBuf::from(path_str);
        let connection: Connection = Session::open_db_at_path(db_path)?;

        debug!("Database initialization complete");
        Ok(Self {connection,})
    }

    fn open_db_at_path(db_path: &PathBuf) -> Result<Connection, errors::Error> {
        if Path::exists(db_path) {
            schema::init_db_from_existing(db_path)
        } else {
            debug!("No existing database found... Building from fresh.");
            schema::init_db_from_fresh(db_path)
        }
    }

}


