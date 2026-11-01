use super::*;
use crate::io::{DEFAULT_ADDR, DEFAULT_PORT};
use crate::json::object;
use crate::axum::http::StatusCode;


pub use crate::service::listener::TagValue;

pub struct Client {
    endpoint : String,
}


impl Client {

    pub fn new() -> Result<Self, errors::Error> 
    {
        let endpoint = format!("http://{}:{}", io::DEFAULT_ADDR, io::DEFAULT_PORT);

        match Client::validate_wesad_listener(&endpoint) {
            Ok(_) => Ok(Client { 
                endpoint,
            }),
            Err(_) => {Err(
                errors::Error::ClientErr(format!("Could not validate WeSAD listener at http://{}:{}. Please ensure
                the service is running and the port is reachable.", DEFAULT_ADDR, DEFAULT_PORT)))
            }
        }
    }

    pub fn new_from_url<T>(endpoint: &T) -> Result<Self, errors::Error>
    where T: Display + Clone
    {

        match Client::validate_wesad_listener(&endpoint) {
            Ok(_) => Ok(Client { 
                endpoint : endpoint.clone().to_string(),
            }),
            Err(_) => {Err(
                errors::Error::ClientErr(format!("Could not validate WeSAD listener at http://{}:{}. Please ensure
                the service is running and the port is reachable.", DEFAULT_ADDR, DEFAULT_PORT)))
            }
        }
    }

    pub fn send_tags<T>(&self, webhook: T, tags: Vec<service::listener::TagValue>) 
    where T: Display + Clone
    {
        let mut json = Client::build_json_status_template(webhook.to_string());
        json["content"] = json::array![];

        for pl in tags {
            let tag = object!{tag : pl.tag, value : pl.value};
            json["content"].push(tag).unwrap();
        }
        println!("dump: {}", &json.dump());

        self.send_status_update(json.dump());
    }


    fn send_status_update(&self, payload: String) 
    {

        println!("{}", &payload);
        let mut url = self.endpoint.clone();
        url.push_str("/status");

        let resp = reqwest::blocking::Client::new()
            .post(url)
            .header("Content-Type", "application/json")
            .body(payload)
            .send().unwrap().text().unwrap();

        println!("{}", resp);
    }


    fn build_json_status_template(webhook: String) -> json::JsonValue 
    {
        let json = json::object!{
            webhook: webhook,
            update_type: "status"
        };

        return json
    }


    fn validate_wesad_listener<T>(addr: &T) -> Result<StatusCode, errors::Error>
    where T: Display + Clone
    {   
        let mut url = addr.to_string().clone();
        url.push_str("/health");

        let resp = reqwest::blocking::Client::new()
            .get(url)
            .send()?;

        Ok(resp.status())
    }
}

