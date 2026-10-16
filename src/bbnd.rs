use std::fs::read_dir;
use std::path::Display;
use std::path::PathBuf;

use crate::ProgramPaths;
use crate::tokio;
use crate::dbus;
use crate::daemonize;
use crate::regex;

use crate::discord;
use crate::smtp;
use crate::database;
use crate::errors;

use crate::json;
use chrono::NaiveDate;


const PATH_WEBHOOKS_ETC: &str = "bbnd/webhooks.d";


enum WebhookType {
    STATUS,
    ALERT,
}


pub struct WebhookDescriptor {
    pub webhook_name : String,
    pub webhook_url : String,
    pub webhook_type: WebhookType,
    pub cron_descriptor : CronDescriptor,
    pub placeholders: Vec<String>,
}



pub struct CronDescriptor {
    minutes : String,
    hours : String,
    days : String,
    next_execution : NaiveDate,
    last_execution : NaiveDate,
}

impl CronDescriptor {
    pub fn as_str(&self) -> String {
        format!("{} {} {}", self.minutes, self.minutes, self.days)
    }
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
            webhooks : Vec::new(),
        })

        
    }

    pub fn run(&mut self) {

        // Initialize connection persistent local data storage
        self.db = Some(database::Session::init(&self.paths).expect("An unexpected error occurred initializing the database."));
        
        self.read_webhook_configs()?;

        // Read configs



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


    fn read_webhook_configs(&mut self) -> Result<(), errors::Error> {

        let webhooks = read_dir(&self.paths.etc.join(PATH_WEBHOOKS_ETC))?;
        let mut webhook_path: String;

        let regex_conf = regex::Regex::new(".conf$")?;

        for webhook in webhooks {
            webhook_path = webhook.unwrap().path().display().to_string();
            
            
            // If the current file is a webhook .conf file
            if !(regex_conf.captures(&webhook_path).is_none()) {
                // Parse the conf file, i.e. the placeholders
                self.webhooks.as_ref().unwrap();

                parse_webhook_config(self.db.as_ref().unwrap(), &webhook_path);

            }

        }

        Ok(())
    }


}


fn parse_webhook_config(db: &database::Session, webhook_path: &String) -> Result<WebhookDescriptor, errors::Error> {
    // Do the conf stuff
    // etc etc etc...


    // If a conf file was found, there must be an associated .json file with the same name
    let json_path = &PathBuf::from(format!("{webhook_path}.json"));

}

