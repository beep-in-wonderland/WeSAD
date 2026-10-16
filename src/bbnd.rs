use crate::ProgramPaths;
use crate::tokio;
use crate::dbus;
use crate::daemonize;

use crate::discord;
use crate::smtp;
use crate::database;
use crate::errors;

use crate::json;
use chrono::NaiveDate;


fn create_bbnd() {}


pub struct WebhookDescriptor {
    webhook_url : String,
    cron_descriptor: CronDescriptor,
    data: Vec<String>,
}


pub struct CronDescriptor {
    minutes : String,
    hours : String,
    days : String,
    next_execution : NaiveDate,
    last_execution : NaiveDate,
}


pub struct BBDaemon {
    paths : ProgramPaths,
    db : Option<database::Session>,
    webhooks : Option<Vec<WebhookDescriptor>>,
}

impl BBDaemon {

    /// Initialization function of the daemon only requires that the paths be specified for this runtime.
    pub fn init(paths: ProgramPaths) -> Result<Self, errors::Error> {
        Ok (Self {
            paths,
            db : None,
            webhooks : None,
        })

        
    }

    pub fn run(&mut self) {

        // Initialize connection persistent local data storage
        self.db = Some(database::Session::init(&self.paths).expect("An unexpected error occurred initializing the database."));
        
        
        // validate data within persist db (bbdb.rs)
        // test connections to 
        //      SMTP (smtp.rs)
        //      Discord webhooks (discord.rs) 

        // read, validate, and store configs (bbconf.rs)

        // initialize daemon with validated data
        //      daemon publishes to session D-Bus

        // external scripts use dbus method calls to send updates
        //      bbnd updates the stored data in the database
        // on set cron intervals as defined in bbnd.d/myfile.conf, the daemon
        //   will publish updates to the webhooks and/or send emails 


    }
}
