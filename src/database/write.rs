use crate::{database::read::webhook_by_name, webhook};

use super::*;


/// Removes all records that are orphaned from the db
pub fn drop_orphaned_values(session: &Session) -> Result<(), errors::Error> {

    drop_orphaned_values_from_table(&session.connection, TAB_WEBHOOKS)?;
    drop_orphaned_values_from_table(&session.connection, TAB_PLACEHOLDERS)?;

    Ok(())
}


/// Takes the provided webhook and either updates the existing record or inserts it into the db.
pub fn init_webhook_parent(session: &Session, webhook: &webhook::Descriptor) -> Result<(), errors::Error> {

    let name = &webhook.name;

    // If the webhook is already in the db table, update it. Otherwise, add it.
    if read::webhook_by_name(&session, &name).is_some() {
        init_update_webhook_parent(&session.connection, &webhook)
    } else {
        init_add_webhook_parent(&session.connection, &webhook)
    }
}


/// Takes the provided placeholders Vector and either updates or inserts it into the db depending if it exists already.
pub fn init_webhook_placeholders(session: &Session, name: &String, placeholders: &Vec<String>) -> Result<(), errors::Error> {
    
    for placeholder in placeholders {
        // If the placeholder is already in the db table, update it. Otherwise, add it.
        if read::placeholder_by_name(&session, name, &placeholder).is_some() {
            init_update_webhook_placeholder(&session.connection, &name, &placeholder)?;
        } else {
            init_add_webhook_placeholder(&session.connection, &name, &placeholder)?;
        }
    }
    Ok(())

}


pub fn update_webhook_placeholder(session: &Session, name: &String, placeholder: &String, value: &String) -> Result<(), errors::Error> {

    let query = format!("
            UPDATE 
                {TAB_PLACEHOLDERS} 
            SET  
                {COL_WEBHOOK_PLACEHOLDER_VALUE} = :value
            WHERE 
                {COL_WEBHOOK_NAME}=:name 
                AND {COL_WEBHOOK_PLACEHOLDER}=:placeholder;
        ");

    let mut statement = session.connection.prepare(query).unwrap();

    statement.bind::<&[(_, sqlite::Value)]>(&[
        (":value", value.as_str().into()),
        (":name", name.as_str().into()),
        (":placeholder", placeholder.as_str().into()),
    ])?;

    statement.next()?;

    Ok(())
}


/// Called at initialization to set all the existing nodes in the db as `orphaned=true` by
/// default. When the configs are read, any values that are still valid and required will
/// have `orphaned=false` and remain in the tables.
pub(super) fn set_all_orphaned(connection: &Connection, table: &str) -> Result<(), errors::Error> {

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


/// Helpder function for drop_orphaned_values() that removes all orphaned values from a provided table.
fn drop_orphaned_values_from_table(connection: &Connection, table: &str) -> Result<(), errors::Error>{

    let query = format!("DELETE FROM {} WHERE orphaned = 1", table);
    let mut statement = connection.prepare(query)?;

    statement.next()?;

    debug!("Dropped orpaned values from {}", table);
    Ok(())
}


/// Helper function for set_webhook_parent(). Inserts the webhook as a new entry in the table.
fn init_add_webhook_parent(connection: &Connection, webhook: &webhook::Descriptor) -> Result<(), errors::Error> {

    let query = format!("INSERT INTO 
            {TAB_WEBHOOKS} ({COL_WEBHOOK_NAME}, {COL_WEBHOOK_URL}, {COL_WEBHOOK_TYPE},
            {COL_PATH}, {COL_PERIOD}, {COL_ORPHANED}, {COL_LAST}, {COL_MESSAGE_ID}) 
            VALUES (:name, :url, :type, :path, :period, '0', 0, '0')");

    let mut statement = connection.prepare(query).unwrap();

    

    statement.bind::<&[(_, sqlite::Value)]>(&[
        (":name", webhook.name.clone().into()),
        (":url", webhook.url.clone().into()),
        (":type", webhook.webhook_type.to_string().into()),
        (":path", webhook.path.to_string().into()),
        (":period", webhook.period.get().into()),
    ])?;

    statement.next()?;

    Ok(())
}


/// Helper function for set_webhook_parent(). Updates the webhook instead of inserting a new entry.
fn init_update_webhook_parent(connection: &Connection, webhook: &webhook::Descriptor) -> Result<(), errors::Error> {

    let query = format!("
            UPDATE 
                {TAB_WEBHOOKS} 
            SET
                {COL_WEBHOOK_URL}=:url, 
                {COL_WEBHOOK_TYPE}=:type,
                {COL_PATH}=:path,
                {COL_PERIOD}=:period, 
                {COL_ORPHANED}='0'
            WHERE 
                {COL_WEBHOOK_NAME} = :name;
        ");

    let mut statement = connection.prepare(query).unwrap();

    statement.bind::<&[(_, sqlite::Value)]>(&[
        (":name", webhook.name.clone().into()),
        (":url", webhook.url.clone().into()),
        (":type", webhook.webhook_type.to_string().into()),
        (":path", webhook.path.to_string().into()),
        (":period", webhook.period.get().into()),
    ])?;

    statement.next()?;

    Ok(())
}


/// Helper function for set_webhook_placeholder(). Updates the placeholder instead of inserting a new entry.
fn init_update_webhook_placeholder(connection: &Connection, name: &String, placeholder: &String) -> Result<(), errors::Error> {
    let query = format!("
            UPDATE 
                {TAB_PLACEHOLDERS} 
            SET  
                {COL_ORPHANED}='0'
            WHERE 
                {COL_WEBHOOK_NAME}=:name 
                AND {COL_WEBHOOK_PLACEHOLDER}=:placeholder;
        ");

    let mut statement = connection.prepare(query).unwrap();

    statement.bind::<&[(_, sqlite::Value)]>(&[
        (":name", name.as_str().into()),
        (":placeholder", placeholder.as_str().into()),
    ])?;

    statement.next()?;

    Ok(())
}


/// Helper function for set_webhook_placeholder(). Inserts the placeholder as a new entry in the table.
fn init_add_webhook_placeholder(connection: &Connection, name: &String, placeholder: &String) -> Result<(), errors::Error> {
    let query = format!("INSERT INTO 
            {TAB_PLACEHOLDERS} ({COL_WEBHOOK_NAME}, {COL_WEBHOOK_PLACEHOLDER}, {COL_ORPHANED}) 
            VALUES (:name, :placeholder, '0')");

    let mut statement = connection.prepare(query).unwrap();

    statement.bind::<&[(_, sqlite::Value)]>(&[
        (":name", name.as_str().into()),
        (":placeholder", placeholder.as_str().into()),
    ])?;

    statement.next()?;

    Ok(())
}

