use std::fs::File;

use sqlite::Connection;

use crate::bbnd::WebhookDescriptor;
use crate::exists;
use crate::PathBuf;
use crate::ProgramPaths;
use crate::sqlite;
use crate::errors;
use crate::{info,error,debug,warn};

const TAB_WEBHOOKS: &str = "webhooks";
const COL_WEBHOOK_NAME: &str = "webhook";
const COL_WEBHOOK_URL: &str = "url";
const COL_WEBHOOK_TYPE: &str = "webhook_type";
const COL_INT_TYPE: &str = "interval";
const COL_INT_VALUE: &str = "period";

const TAB_PLACEHOLDERS: &str = "placeholders";
const COL_WEBHOOK_PLACEHOLDER: &str = "placeholder";
const COL_WEBHOOK_PLACEHOLDER_VALUE: &str = "value";

enum BbdbLocations {
    VarLib,
    DebugState,
    None,
}

pub struct Session {
    connection : Option<sqlite::Connection>,
}


impl Session {

    /// Check for existing `bbdb.db` file and create a new one if none is found. This 
    /// function does not perform any validation on the database contents. It only
    /// establishes the connection and returns the object through which the database
    /// can be accessed.
    pub fn init(paths: &ProgramPaths) -> Result<Self, errors::Error> {

        let mut new_session = Self { connection : None };

        let db_path: &PathBuf = &paths.persist.join("bbnd.db");

        if exists(db_path).unwrap() {
            new_session.connection = Some(init_db_from_existing(db_path).unwrap());
        } else {
            new_session.connection = Some(init_db_from_fresh(db_path).unwrap());
        }
        Ok(new_session)
    }


    pub fn add_webhook_parent(&self, webhook: &WebhookDescriptor) -> Result<(), errors::Error> {

        let query = format!("INSERT INTO 
                {TAB_WEBHOOKS} ({COL_WEBHOOK_NAME}, {COL_WEBHOOK_URL}, {COL_WEBHOOK_TYPE}, {COL_INT_TYPE}, {COL_INT_VALUE}) 
                VALUES (:name, :url, :type, :interval, :period)");

        let mut statement = self.connection.as_ref().unwrap().prepare(query).unwrap();

        let w = webhook.inner.lock().expect("poisoned");

        statement.bind::<&[(_, sqlite::Value)]>(&[
            (":name", w.name.clone().into()),
            (":url", w.url.clone().into()),
            (":type", w.webhook_type.clone().to_string().into()),
            (":interval", w.interval.as_str().into()),
            (":period", w.period.into()),
        ])?;

        statement.next()?;

        Ok(())
    }

    pub fn add_webhook_placeholder(&self, webhook_name: &String, placeholder: &String) -> Result<(), errors::Error> {

        let query = format!("INSERT INTO 
                {TAB_PLACEHOLDERS} ({COL_WEBHOOK_NAME}, {COL_WEBHOOK_PLACEHOLDER}) 
                VALUES (:name, :placeholder)");

        let mut statement = self.connection.as_ref().unwrap().prepare(query).unwrap();

        statement.bind::<&[(_, sqlite::Value)]>(&[
            (":name", webhook_name.as_str().into()),
            (":placeholder", placeholder.as_str().into()),
        ])?;

        statement.next()?;

        Ok(())
    }


}


fn init_db_from_existing(db_path: &PathBuf) -> Result<Connection, errors::Error> {
        let connection = sqlite::Connection::open(db_path).unwrap();

        validate_existing_schema(&connection)?;

        Ok(connection)
}


fn init_db_from_fresh(db_path: &PathBuf) -> Result<Connection, errors::Error> {

    let _ = File::create_new(db_path).unwrap(); 
    let connection = sqlite::Connection::open(db_path).unwrap();

    create_webhook_parent_table(&connection)?;
    create_placeholder_table(&connection)?;

    Ok(connection)
}


fn create_webhook_parent_table(connection: &Connection) -> Result<(), errors::Error>{
    debug!("Creating webhook parent table");
    let query = format!("CREATE TABLE IF NOT EXISTS {TAB_WEBHOOKS} 
                            (
                            id INTEGER PRIMARY KEY AUTOINCREMENT,
                            {COL_WEBHOOK_NAME} VARCHAR(20),
                            {COL_WEBHOOK_URL} VARCHAR(150),
                            {COL_WEBHOOK_TYPE} VARCHAR(10),
                            {COL_INT_TYPE} VARCHAR(20),
                            {COL_INT_VALUE} REAL
                            )
                        ");

    connection.execute(query)?;

    Ok(())
}


fn create_placeholder_table(connection: &Connection) -> Result<(), errors::Error>{
    debug!("Creating webhook placeholder table");
    let query = format!("CREATE TABLE IF NOT EXISTS {TAB_PLACEHOLDERS} (
                            id INTEGER PRIMARY KEY AUTOINCREMENT,
                            {COL_WEBHOOK_NAME} VARCHAR(20),
                            {COL_WEBHOOK_PLACEHOLDER} VARCHAR(40),
                            {COL_WEBHOOK_PLACEHOLDER_VALUE} VARCHAR(10)
                            )
                        ");

    connection.execute(query)?;

    Ok(())
}


fn validate_existing_schema(connection: &Connection) -> Result<(), errors::Error> {

    // TODO implement table validation for columns once final architecture is built
    // If pass, return Ok(())
    // If fail, BbdbUnreadable() error

    Ok(())
}