use crate::webhook;

use super::*;


/// Removes all records that are orphaned from the db
pub fn drop_orphaned_values(session: &Session) -> Result<(), errors::Error> {

    drop_orphaned_values_from_table(&session.connection, TAB_WEBHOOKS)?;
    drop_orphaned_values_from_table(&session.connection, TAB_TAGS)?;

    Ok(())
}


/// Takes the provided webhook and either updates the existing record or inserts it into the db.
pub fn init_webhook_parent(session: &Session, webhook: &webhook::Status) -> Result<(), errors::Error> {

    let name = &webhook.name;

    // If the webhook is already in the db table, update it. Otherwise, add it.
    if read::webhook_by_name(&session, &name).is_some() {
        init_update_webhook_parent(&session.connection, &webhook)
    } else {
        init_add_webhook_parent(&session.connection, &webhook)
    }
}


/// Takes the provided tags Vector and either updates or inserts it into the db depending if it exists already.
pub fn init_webhook_tags(session: &Session, name: &String, tags: &Vec<String>) -> Result<(), errors::Error> {
    
    for tag in tags {
        // If the tag is already in the db table, update it. Otherwise, add it.
        if read::tag_by_name(&session, name, &tag).is_some() {
            init_update_webhook_tag(&session.connection, &name, &tag)?;
        } else {
            init_add_webhook_tag(&session.connection, &name, &tag)?;
        }
    }
    Ok(())

}


pub fn update_webhook_tag(session: &Session, name: &String, tag: &String, value: &String) -> Result<(), errors::Error> {

    let query = format!("
            UPDATE 
                {TAB_TAGS} 
            SET  
                {COL_WEBHOOK_TAG_VALUE} = :value
            WHERE 
                {COL_WEBHOOK_NAME}=:name 
                AND {COL_WEBHOOK_TAG}=:tag;
        ");

    let mut statement = session.connection.prepare(query).unwrap();

    statement.bind::<&[(_, sqlite::Value)]>(&[
        (":value", value.as_str().into()),
        (":name", name.as_str().into()),
        (":tag", tag.as_str().into()),
    ])?;

    statement.next()?;

    Ok(())
}


pub fn update_webhook_msg_id(session: &Session, name: &String, msg_id: &String) -> Result<(), errors::Error> {

    let query = format!("
            UPDATE 
                {TAB_WEBHOOKS} 
            SET  
                {COL_MESSAGE_ID} = :value
            WHERE 
                {COL_WEBHOOK_NAME}=:name 
        ");

    let mut statement = session.connection.prepare(query).unwrap();

    statement.bind::<&[(_, sqlite::Value)]>(&[
        (":value", msg_id.as_str().into()),
        (":name", name.as_str().into()),
    ])?;

    statement.next()?;

    Ok(())

}


pub fn update_webhook_last(session: &Session, name: &String, last: &String) -> Result<(), errors::Error> {

    let query = format!("
            UPDATE 
                {TAB_WEBHOOKS} 
            SET  
                {COL_LAST} = :value
            WHERE 
                {COL_WEBHOOK_NAME}=:name 
        ");

    let mut statement = session.connection.prepare(query).unwrap();

    statement.bind::<&[(_, sqlite::Value)]>(&[
        (":value", last.as_str().into()),
        (":name", name.as_str().into()),
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


/// Helpder function for drop_orphaned_values() that removes all orphaned values from a provided table.
fn drop_orphaned_values_from_table(connection: &Connection, table: &str) -> Result<(), errors::Error>{

    let query = format!("DELETE FROM {} WHERE orphaned = 1", table);
    let mut statement = connection.prepare(query)?;

    statement.next()?;

    debug!("Dropped orpaned values from {}", table);
    Ok(())
}


/// Helper function for set_webhook_parent(). Inserts the webhook as a new entry in the table.
fn init_add_webhook_parent(connection: &Connection, webhook: &webhook::Status) -> Result<(), errors::Error> {

    let query = format!("INSERT INTO 
            {TAB_WEBHOOKS} ({COL_WEBHOOK_NAME}, {COL_WEBHOOK_URL}, {COL_WEBHOOK_TYPE},
            {COL_PATH}, {COL_PERIOD}, {COL_ORPHANED}, {COL_LAST}, {COL_MESSAGE_ID}) 
            VALUES (:name, :url, :type, :path, :period, '0', 0, :msg_id)");

    let mut statement = connection.prepare(query).unwrap();

    // Check if the incoming `msg_id` is valid, and if not, provide a dummy default
    let msg_id = match &webhook.msg_id {
        Some(msg) => msg,
        None => &"0".to_string(),
    };

    statement.bind::<&[(_, sqlite::Value)]>(&[
        (":name", webhook.name.clone().into()),
        (":url", webhook.url.clone().into()),
        (":type", webhook.webhook_type.to_string().into()),
        (":path", webhook.path.to_string().into()),
        (":period", webhook.period.get().into()),
        (":msg_id", msg_id.as_str().into())
    ])?;

    statement.next()?;

    Ok(())
}


/// Helper function for set_webhook_parent(). Updates the webhook instead of inserting a new entry.
/// Specifically avoids updating `msg_id`, since presumably if the webhook already exists, it has
/// already generated or consumed a message id.
fn init_update_webhook_parent(connection: &Connection, webhook: &webhook::Status) -> Result<(), errors::Error> {

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


/// Helper function for set_webhook_tag(). Updates the tag instead of inserting a new entry.
fn init_update_webhook_tag(connection: &Connection, name: &String, tag: &String) -> Result<(), errors::Error> {
    let query = format!("
            UPDATE 
                {TAB_TAGS} 
            SET  
                {COL_ORPHANED}='0'
            WHERE 
                {COL_WEBHOOK_NAME}=:name 
                AND {COL_WEBHOOK_TAG}=:tag;
        ");

    let mut statement = connection.prepare(query).unwrap();

    statement.bind::<&[(_, sqlite::Value)]>(&[
        (":name", name.as_str().into()),
        (":tag", tag.as_str().into()),
    ])?;

    statement.next()?;

    Ok(())
}


/// Helper function for set_webhook_tag(). Inserts the tag as a new entry in the table.
fn init_add_webhook_tag(connection: &Connection, name: &String, tag: &String) -> Result<(), errors::Error> {
    let query = format!("INSERT INTO 
            {TAB_TAGS} ({COL_WEBHOOK_NAME}, {COL_WEBHOOK_TAG}, {COL_WEBHOOK_TAG_VALUE}, {COL_ORPHANED}) 
            VALUES (:name, :tag, '*empty*' , '0')");

    let mut statement = connection.prepare(query).unwrap();

    statement.bind::<&[(_, sqlite::Value)]>(&[
        (":name", name.as_str().into()),
        (":tag", tag.as_str().into()),
    ])?;

    statement.next()?;

    Ok(())
}

