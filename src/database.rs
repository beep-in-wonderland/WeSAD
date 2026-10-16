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
const COL_CRON_STRING: &str = "cron";

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
        let connection: sqlite::Connection ;

        if exists(db_path).unwrap() {
            new_session.connection = Some(init_db_from_existing(db_path).unwrap());
        } else {
            new_session.connection = Some(init_db_from_fresh(db_path).unwrap());
        }

        // new_session.insert_webhook_parent("test hook", "myurl")?;


        Ok(new_session)
    }


    pub fn add_webhook_parent(&self, webhook: &WebhookDescriptor) -> Result<(), errors::Error> {

        let query = format!("INSERT INTO 
                {TAB_WEBHOOKS} ({COL_WEBHOOK_NAME}, {COL_WEBHOOK_URL}, {COL_CRON_STRING}) 
                VALUES (:name, :url, :cron)");

        let mut statement = self.connection.as_ref().unwrap().prepare(query).unwrap();

        statement.bind::<&[(_, sqlite::Value)]>(&[
            (":name", webhook.webhook_name.clone().into()),
            (":url", webhook.webhook_url.clone().into()),
            (":cron", webhook.cron_descriptor.as_str().into())
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
                            {COL_CRON_STRING} VARCHAR(20)
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