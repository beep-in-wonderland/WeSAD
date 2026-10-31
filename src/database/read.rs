use crate::webhook;
use crate::HashMap;
use crate::FromStr;

use super::*;


pub fn webhook_by_name(session: &Session, name: &String) -> Option<usize> {

    let query: &str = "SELECT id FROM webhooks WHERE webhook=:name LIMIT 1";
    let mut statement = session.connection.prepare(query).unwrap();

    statement.bind((":name",name.as_str())).unwrap();
    statement.next().unwrap();
    statement.read::<String, usize>(0).map_or(
        None, 
        |s| {
        Some(s.parse::<usize>().unwrap())
    })
    
}


pub fn placeholder_by_name(session: &Session, name: &String, placeholder: &String) -> Option<String> {

    let query: &str = "SELECT id FROM placeholders WHERE webhook=? AND placeholder=? LIMIT 1";
    let mut statement = session.connection.prepare(query).unwrap();

    statement.bind((1,name.as_str())).unwrap();
    statement.bind((2,placeholder.as_str())).unwrap();
    statement.next().unwrap();
    statement.read::<String, usize>(0).map_or(
        None,
        |s| {
            Some(s.parse::<String>().unwrap())
        }
    )
}


pub fn all_placeholders_by_webhook_name(session: &Session, name: &String) -> Result<HashMap<String, String>, errors::Error> {
    let mut result:HashMap<String, String> = HashMap::new();

    let query = format!("SELECT {COL_WEBHOOK_PLACEHOLDER}, {COL_WEBHOOK_PLACEHOLDER_VALUE} FROM {TAB_PLACEHOLDERS} WHERE {COL_WEBHOOK_NAME}=?");
    let mut statement = session.connection.prepare(query).unwrap();

    statement.bind((1,name.as_str())).unwrap();

    while let sqlite::State::Row = statement.next()? {
        let k = statement.read::<String, usize>(0)?;
        let v = statement.read::<String, usize>(1)?;

        result.insert(k, v);
    }

    Ok(result)
}


pub fn webhook_statuses(session: &Session) -> Result<Vec<webhook::Status>, errors::Error> {
        
        let mut result = Vec::new();
        let query:String = format!("
        SELECT
            {COL_WEBHOOK_NAME},
            {COL_WEBHOOK_URL},
            {COL_PATH},
            {COL_WEBHOOK_TYPE},
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

        let mut statement = session.connection.prepare(query)?;
        
        while let sqlite::State::Row = statement.next()? {
            let name = statement.read::<String, _>(COL_WEBHOOK_NAME)?;
            let url = statement.read::<String, _>(COL_WEBHOOK_URL)?;
            let path = statement.read::<String, _>(COL_PATH)?;

            let webhook_type = webhook::Type::from_str(
                statement.read::<String, _>(COL_WEBHOOK_TYPE)?.as_str())
                .expect("The webhook status should have been validated when it was pushed into the db");

            let period = statement.read::<i64, _>(COL_PERIOD)?;
            let last = statement.read::<String, _>(COL_LAST)?;
            let msg_id = statement.read::<String, _>(COL_MESSAGE_ID)?;
            result.push(
                webhook::Status { 
                    name, url, path, 
                    webhook_type,
                    period: period.into(), 
                    last : last.parse::<i64>().unwrap(), 
                    msg_id : Some(msg_id)
                }
            )

        }

        Ok(result)
    }