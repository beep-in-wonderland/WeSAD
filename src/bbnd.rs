use std::io::Read;

use crate::read_dir;
use crate::PathBuf;
use crate::File;

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

/// Regex capture which will split a fully qualified file path with extension into three non-overlapping captures:
/// example: ``/this/is/my/file.conf``
/// `dir = /this/is/my/`
/// `name = file`
/// `ext = .conf`
const REGEX_CONF: &str = "(?<dir>^[\\\\0-9a-zA-Z_\\- ]+\\\\(?!\\\\[0-9a-zA-Z_\\- ]+\\.))(?<name>[0-9a-zA-Z_\\-. ]+)(?<ext>\\.conf)$";


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


struct WebhookConfigContents {
    webhook_name : String,
    webhook_url : String,
    webhook_type: WebhookType,
    cron_descriptor : CronDescriptor,
}


struct WebhookJsonPlaceholders {
    placeholders : Vec<String>,
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
            webhooks : None,
        })

        
    }

    pub fn run(&mut self) -> Result<(), errors::Error> {

        // Initialize connection persistent local data storage
        self.db = Some(database::Session::init(&self.paths).expect("An unexpected error occurred initializing the database."));
        
        self.parse_all_webhook_configs()?;

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

        Ok (())
    }


    /// Spawns a thread that continuously updates embeds based on cron intervals
    /// and current database information
    fn update() -> Result<(), errors::Error> {
        Ok(())
    }

    
    /// Spawns a thread that continuously listens for updates via user D-BUS in 
    /// order to update database based on external signals
    fn listen() -> Result<(), errors::Error> {
        Ok(())
    }


    fn parse_all_webhook_configs(&mut self) -> Result<(), errors::Error> {

        let webhooks = read_dir(&self.paths.etc.join(PATH_WEBHOOKS_ETC))?;
        let mut webhook_path: PathBuf;

        let regex_conf = regex::Regex::new(REGEX_CONF)?;
        let mut captures: Option<regex::Captures>;
        let mut path: String;
        let mut name: String;

        let mut webhook_config: WebhookConfigContents;
        let mut webhook_placeholders: WebhookJsonPlaceholders;


        for webhook in webhooks {
            webhook_path = webhook.unwrap().path();

            let target_string = webhook_path.display().to_string();
            captures = regex_conf.captures(&target_string);
            
            // If the regex found any captures
            if !captures.is_none() {
                // Extract the path & filename from the regex capture group
                path = captures.as_ref().unwrap().get(1).map_or("", |s| s.as_str()).to_string();
                name = captures.as_ref().unwrap().get(2).map_or("", |s| s.as_str()).to_string();


                webhook_config = parse_webhook_config(&path, &name)?;
                webhook_placeholders = parse_webhook_json_placeholders(&path, &name)?;

            }

        }

        Ok(())
    }



}


fn parse_webhook_config(path: &String, name: &String) -> Result<WebhookConfigContents, errors::Error> {

    let mut webhook_url: String;
    let mut webhook_type: String;
    let mut webhook_url: CronDescriptor;



    Ok(WebhookConfigContents { 
        webhook_url, 
        webhook_type, 
        cron_descriptor, 
    })
}


fn parse_webhook_json_placeholders(path: &String, name: &String) -> Result<WebhookJsonPlaceholders, errors::Error> {

    let mut placeholders = Vec::new();

    Ok(WebhookJsonPlaceholders {
        placeholders,
    })
}

