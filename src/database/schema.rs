use super::*;

pub(super) fn init_db_from_existing(db_path: &PathBuf) -> Result<Connection, errors::Error> {
        let connection = sqlite::Connection::open(db_path).unwrap();

        // Verify the structure of the schema
        validate_schema_from_connection(&connection)?;
        // Set all records as orphaned until they can be corrected by the config pass
        write::set_all_orphaned(&connection, TAB_WEBHOOKS)?;
        write::set_all_orphaned(&connection, TAB_PLACEHOLDERS)?;

        Ok(connection)
}


pub(super) fn init_db_from_fresh(db_path: &PathBuf) -> Result<Connection, errors::Error> {

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


pub fn validate_schema_from_connection(connection: &Connection) -> Result<(), errors::Error> {

    // TODO implement table validation for columns once final architecture is built
    // If pass, return Ok(())
    // If fail, BbdbUnreadable() error
    debug!("Schema validated");

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

