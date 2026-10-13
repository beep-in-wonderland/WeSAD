// use declarations

// separate modules
mod bbdb; // Database management module using SQLite3
mod bbnd;
mod bbconf;
mod discord;
mod smtp;

fn main() {
    println!("Hello, world!");

    // read, validate, and store configs (bbconf.rs)
    // init & check connection to persist db (bbdb.rs)
    // validate data within persist db (bbdb.rs)
    // test connections to SMTP (smtp.rs) and discord (discord.rs) webhooks 
    // initialize daemon with validated data

}
