// all crates for the project
use tokio; // task scheduling
use daemonize;
use dbus; // communication with external programs
use mail_send; // SMTP email sending
use sqlite; // managing persistent data
use json; // parsing config files


// separate custom modules
mod bbdb; // Database management module using SQLite3
mod bbnd;
mod bbconf;
mod discord;
mod smtp;


fn main() {
    println!("Hello, world!");

    // read, validate, and store configs (bbconf.rs)
    // init & check connection to persist db (bbdb.rs)
    //      validate data within persist db (bbdb.rs)
    // test connections to 
    //      SMTP (smtp.rs)
    //      Discord webhooks (discord.rs) 

    // initialize daemon with validated data
    //      daemon publishes to session D-Bus
    
    // external scripts use dbus method calls to send updates
    //      bbnd updates the stored data in the database
    // on set cron intervals as defined in bbnd.d/myfile.conf, the daemon
    //   will publish updates to the webhooks and/or send emails 


}
