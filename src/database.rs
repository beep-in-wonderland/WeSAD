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

struct Session {
    connection : sqlite::Connection,
}


impl Session {

    // /// Check for existing bbdb.db file and create a new one if none is found.
    // fn init(&self, paths: ProgramPaths) -> Result<Self, crate::errors::Error> {

    //     let db_path: &PathBuf = &paths.persist.join("bbnd.db");
    //     let connection: sqlite::Connection ;

    //     if exists(db_path).unwrap() {
    //         connection = self._init_from_existing(db_path).unwrap();
    //     } else {

    //         connection = self._init_from_fresh(db_path).unwrap();
    //     }

        
    //     Ok(Session { 
    //         connection : connection,
    //      })
    // }

    // TODO: Write in DB initialization logic depending on requirements as parsed by the config

    // fn _init_from_fresh(&self, db_path: &PathBuf) -> Result<sqlite::Connection, errors::Error> {

    //     let db_file = File::create_new(db_path).unwrap(); 

    //     Ok(sqlite::Connection::open())
    // }


    // fn _init_from_existing(&self, db_path: &PathBuf) -> Result<sqlite::Connection, errors::Error> {

    //     Ok(sqlite::Connection::open())
    // }

}
