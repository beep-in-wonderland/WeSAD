use std::fs::File;

use crate::exists;
use crate::PathBuf;
use crate::ProgramPaths;
use crate::sqlite;
use crate::errors;

enum BbdbLocations {
    VarLib,
    DebugState,
    None,
}

pub struct Session {
    connection : sqlite::Connection,
}


impl Session {

    /// Check for existing `bbdb.db` file and create a new one if none is found. This 
    /// function does not perform any validation on the database contents. It only
    /// establishes the connection and returns the object through which the database
    /// can be accessed.
    pub fn init(paths: &ProgramPaths) -> Result<Self, crate::errors::Error> {

        let db_path: &PathBuf = &paths.persist.join("bbnd.db");
        let connection: sqlite::Connection ;

        if exists(db_path).unwrap() {
            connection = Session::_init_from_existing(db_path).unwrap();
        } else {

            connection = Session::_init_from_fresh(db_path).unwrap();
        }

        
        Ok(Session { 
            connection : connection,
         })
    }


    fn _init_from_fresh(db_path: &PathBuf) -> Result<sqlite::Connection, errors::Error> {

        let _ = File::create_new(db_path).unwrap(); 

        Ok(sqlite::Connection::open(db_path)?)
    }


    fn _init_from_existing(db_path: &PathBuf) -> Result<sqlite::Connection, errors::Error> {

        Ok(sqlite::Connection::open(db_path)?)
    }
    

}
