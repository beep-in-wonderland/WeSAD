mod constants;
mod checker;
mod execute;
mod session;

use constants::*;

use std::fs::File;
use std::ops::Not;

use dbus::arg::RefArg;
use sqlite::Connection;

use crate::bbnd::WebhookDescriptor;
use crate::exists;
use crate::PathBuf;
use crate::ProgramPaths;
use crate::sqlite;
use crate::errors;
use crate::{info,error,debug,warn};


enum BbdbLocations {
    VarLib,
    DebugState,
    None,
}

pub struct Session {
    connection : Option<sqlite::Connection>,
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

        let mut new_session = Self { connection : None };

        let db_path: &PathBuf = &paths.persist.join("bbnd.db");

        if exists(db_path).unwrap() {
            new_session.connection = Some(init_db_from_existing(db_path).unwrap());
        } else {
            new_session.connection = Some(init_db_from_fresh(db_path).unwrap());
        }
        debug!("Database initialization complete");
        Ok(new_session)
    }


    /// Takes the provided webhook and either updates or inserts it into the db depending if it exists already.
    pub fn set_webhook_parent(&self, webhook: &WebhookDescriptor) -> Result<(), errors::Error> {

        let name = &webhook.name;

        // If the webhook is already in the db table, update the record instead of inserting.
        if check_webhook_exists(self.connection.as_ref().unwrap(), &name) { 
            debug!("Webhook `{}` found in db. Removing orphaned flag and updating.", &name);
            self.update_webhook_parent(&webhook)

        } else {
            debug!("Webhook `{}` not found in db. Inserting.", &name);
            self.add_webhook_parent(&webhook)
        }
    }

    /// Helper function for set_webhook_parent(). Inserts the webhook as a new entry in the table.
    fn add_webhook_parent(&self, webhook: &WebhookDescriptor) -> Result<(), errors::Error> {

        let query = format!("INSERT INTO 
                {TAB_WEBHOOKS} ({COL_WEBHOOK_NAME}, {COL_WEBHOOK_URL}, {COL_WEBHOOK_TYPE}, {COL_PERIOD}, {COL_ORPHANED}, {COL_LAST}, {COL_MESSAGE_ID}) 
                VALUES (:name, :url, :type, :period, '0', '0', '0')");

        let mut statement = self.connection.as_ref().unwrap().prepare(query).unwrap();

        

        statement.bind::<&[(_, sqlite::Value)]>(&[
            (":name", webhook.name.clone().into()),
            (":url", webhook.url.clone().into()),
            (":type", webhook.webhook_type.to_string().into()),
            (":period", webhook.period.into()),
        ])?;

        statement.next()?;

        Ok(())
    }

    /// Helper function for set_webhook_parent(). Updates the webhook instead of inserting a new entry.
    fn update_webhook_parent(&self, webhook: &WebhookDescriptor) -> Result<(), errors::Error> {

        let query = format!("
                UPDATE 
                    {TAB_WEBHOOKS} 
                SET
                    {COL_WEBHOOK_URL}=:url, 
                    {COL_WEBHOOK_TYPE}=:type, 
                    {COL_PERIOD}=:period, 
                    {COL_ORPHANED}='0'
                WHERE 
                    {COL_WEBHOOK_NAME} = :name;
            ");

        let mut statement = self.connection.as_ref().unwrap().prepare(query).unwrap();

        statement.bind::<&[(_, sqlite::Value)]>(&[
            (":name", webhook.name.clone().into()),
            (":url", webhook.url.clone().into()),
            (":type", webhook.webhook_type.to_string().into()),
            (":period", webhook.period.into()),
        ])?;

        statement.next()?;

        Ok(())
    }


    /// Takes the provided placeholder and either updates or inserts it into the db depending if it exists already.
    pub fn set_webhook_placeholder(&self, name: &String, placeholder: &String) -> Result<(), errors::Error> {
        
        // If the placeholder is already in the db table, return early.
        if check_plceholder_exists(self.connection.as_ref().unwrap(), name, placeholder) { 

            debug!("Webhook `{}`, Placeholder `{}` found. Removing orphaned flag.", name, placeholder);
            self.update_webhook_placeholder(&name, &placeholder)
        } else {
            debug!("Webhook `{}`, Placeholder `{}` not found. Inserting.", name, placeholder);
            self.add_webhook_placeholder(&name, &placeholder)
        }
        
    }

    /// Helper function for set_webhook_placeholder(). Updates the placeholder instead of inserting a new entry.
    fn update_webhook_placeholder(&self, name: &String, placeholder: &String) -> Result<(), errors::Error> {
        let query = format!("
                UPDATE 
                    {TAB_PLACEHOLDERS} 
                SET  
                    {COL_ORPHANED}='0'
                WHERE 
                    {COL_WEBHOOK_NAME}=:name 
                    AND {COL_WEBHOOK_PLACEHOLDER}=:placeholder;
            ");

        let mut statement = self.connection.as_ref().unwrap().prepare(query).unwrap();

        statement.bind::<&[(_, sqlite::Value)]>(&[
            (":name", name.as_str().into()),
            (":placeholder", placeholder.as_str().into()),
        ])?;

        statement.next()?;

        Ok(())
    }

    /// Helper function for set_webhook_placeholder(). Inserts the placeholder as a new entry in the table.
    fn add_webhook_placeholder(&self, name: &String, placeholder: &String) -> Result<(), errors::Error> {
        let query = format!("INSERT INTO 
                {TAB_PLACEHOLDERS} ({COL_WEBHOOK_NAME}, {COL_WEBHOOK_PLACEHOLDER}, {COL_ORPHANED}) 
                VALUES (:name, :placeholder, '0')");

        let mut statement = self.connection.as_ref().unwrap().prepare(query).unwrap();

        statement.bind::<&[(_, sqlite::Value)]>(&[
            (":name", name.as_str().into()),
            (":placeholder", placeholder.as_str().into()),
        ])?;

        statement.next()?;

        Ok(())
    }


    /// Removes all records that are orphaned from the db
    pub fn drop_orphaned_values(&self) -> Result<(), errors::Error> {

        self.drop_orphaned_values_from_table(TAB_WEBHOOKS)?;
        self.drop_orphaned_values_from_table(TAB_PLACEHOLDERS)?;

        Ok(())
    }

    /// Helpder function for drop_orphaned_values() that removes all orphaned values from a provided table.
    fn drop_orphaned_values_from_table(&self, table: &str) -> Result<(), errors::Error>{

        
        let connection = self.connection.as_ref().unwrap();

        let query = format!("DELETE FROM {} WHERE orphaned = 1", table);
        let mut statement = connection.prepare(query)?;

        statement.next()?;
    
        debug!("Dropped orpaned values from {}", table);
        Ok(())
    }


    pub fn get_webhook_status_data(&self) -> Result<Vec<WebhookStatus>, errors::Error> {
        
        let mut result = Vec::new();
        let query:String = format!("
        SELECT
            {COL_WEBHOOK_NAME},
            {COL_WEBHOOK_URL},
            {COL_PERIOD},
            {COL_LAST},
            {COL_MESSAGE_ID}        
        FROM
            {TAB_WEBHOOKS}
        WHERE
            {COL_WEBHOOK_TYPE} = 'status'
        ORDER BY 
            {COL_LAST}
        ASC;
        ");

        let mut statement = self.connection.as_ref().unwrap().prepare(query)?;
        
        while let sqlite::State::Row = statement.next()? {
            let name = statement.read::<String, _>(COL_WEBHOOK_NAME)?;
            let url = statement.read::<String, _>(COL_WEBHOOK_URL)?;
            let period = statement.read::<f64, _>(COL_PERIOD)?;
            let last = statement.read::<String, _>(COL_LAST)?;
            let msg_id = statement.read::<String, _>(COL_MESSAGE_ID)?;

            result.push(
                WebhookStatus { 
                    name, url, period, last : last.as_u64().unwrap(), msg_id : msg_id.as_u64().unwrap()
                }
            )

        }

        Ok(result)
    }

}



fn init_db_from_existing(db_path: &PathBuf) -> Result<Connection, errors::Error> {
        let connection = sqlite::Connection::open(db_path).unwrap();

        // Verify the structure of the schema
        validate_existing_schema(&connection)?;
        // Set all records as orphaned until they can be corrected by the config pass
        set_all_orphaned(&connection, TAB_WEBHOOKS)?;
        set_all_orphaned(&connection, TAB_PLACEHOLDERS)?;

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
                            {COL_PERIOD} REAL,
                            {COL_ORPHANED} INTEGER,
                            {COL_MESSAGE_ID} TEXT,
                            {COL_LAST} TEXT
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
                            {COL_WEBHOOK_PLACEHOLDER_VALUE} VARCHAR(10),
                            {COL_ORPHANED} INTEGER
                            )
                        ");

    connection.execute(query)?;

    Ok(())
}


fn validate_existing_schema(connection: &Connection) -> Result<(), errors::Error> {

    // TODO implement table validation for columns once final architecture is built
    // If pass, return Ok(())
    // If fail, BbdbUnreadable() error
    debug!("Schema validated");

    Ok(())
}


fn check_webhook_exists(connection: &Connection, name: &String) -> bool {

    let query: &str = "SELECT id FROM webhooks WHERE webhook=:name LIMIT 1";
    let mut statement = connection.prepare(query).unwrap();

    statement.bind((":name",name.as_str())).unwrap();
    statement.next().unwrap();
    statement.read::<String, usize>(0).is_err().not()
}


fn check_plceholder_exists(connection: &Connection, name: &String, placeholder: &String) -> bool {

    let query: &str = "SELECT id FROM placeholders WHERE webhook=? AND placeholder=? LIMIT 1";
    let mut statement = connection.prepare(query).unwrap();

    statement.bind((1,name.as_str())).unwrap();
    statement.bind((2,placeholder.as_str())).unwrap();
    statement.next().unwrap();
    statement.read::<String, usize>(0).is_err().not()
}


/// Called at initialization to set all the existing nodes in the db as `orphaned=true` by
/// default. When the configs are read, any values that are still valid and required will
/// have `orphaned=false` and remain in the tables.
fn set_all_orphaned(connection: &Connection, table: &str) -> Result<(), errors::Error> {

    let query = format!("UPDATE {} SET orphaned=:val", table);
    let mut statement = connection.prepare(query)?;

    statement.bind((":val", 1))?;
    statement.next()?;

    debug!("Table {} set all orphaned", table);
    Ok(())
}


/// Sets the `orphaned` flag of a specific table record to `val`
fn set_webhook_orphaned_by_name(connection: &Connection, name: &String, val: bool) -> Result<(), errors::Error> {

    let query = format!("UPDATE {} SET orphaned = :val WHERE webhook = :name", TAB_WEBHOOKS);
    let mut statement = connection.prepare(query)?;

    statement.bind((":val", 
        match val {
            true  => "1",
            false => "0"
        }
    ))?;
    statement.bind((":name", name.as_str()))?;

    statement.next()?;
    Ok(())
}

/// Sets the `orphaned` flag of a specific table record to `val`
fn set_placeholder_orphaned_by_name(connection: &Connection, name: &String, placeholder: &String, val: bool) -> Result<(), errors::Error> {

    let query = format!("UPDATE {} SET orphaned = :val WHERE webhook = :name AND placeholder = :placeholder", TAB_PLACEHOLDERS);
    let mut statement = connection.prepare(query)?;

    statement.bind((":val", 
        match val {
            true  => "1",
            false => "0"
        }
    ))?;
    statement.bind((":name", name.as_str()))?;
    statement.bind((":placeholder", placeholder.as_str()))?;

    statement.next()?;
    Ok(())
}

