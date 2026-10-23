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


pub fn placeholder_by_name(session: &Session, name: &String, placeholder: &String) -> Option<usize> {

    let query: &str = "SELECT id FROM placeholders WHERE webhook=? AND placeholder=? LIMIT 1";
    let mut statement = session.connection.prepare(query).unwrap();

    statement.bind((1,name.as_str())).unwrap();
    statement.bind((2,placeholder.as_str())).unwrap();
    statement.next().unwrap();
    statement.read::<String, usize>(0).map_or(
        None,
        |s| {
            Some(s.parse::<usize>().unwrap())
        }
    )
}


pub fn webhook_statuses(session: &Session) -> Result<Vec<WebhookStatus>, errors::Error> {
        
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

        let mut statement = session.connection.prepare(query)?;
        
        while let sqlite::State::Row = statement.next()? {
            let name = statement.read::<String, _>(COL_WEBHOOK_NAME)?;
            let url = statement.read::<String, _>(COL_WEBHOOK_URL)?;
            let period = statement.read::<f64, _>(COL_PERIOD)?;
            let last = statement.read::<String, _>(COL_LAST)?;
            let msg_id = statement.read::<String, _>(COL_MESSAGE_ID)?;
            result.push(
                WebhookStatus { 
                    name, url, period: period.into(), last : last.parse::<u64>().unwrap(), msg_id : msg_id.parse::<u64>().unwrap()
                }
            )

        }

        Ok(result)
    }


