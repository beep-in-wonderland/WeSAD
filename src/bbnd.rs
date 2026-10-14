use crate::tokio;
use crate::dbus;
use crate::daemonize;

use crate::discord;
use crate::smtp;
use crate::database;

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