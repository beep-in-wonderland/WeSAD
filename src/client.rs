use super::*;
use crate::json::object;
use crate::axum::http::StatusCode;
use crate::thiserror::Error;

pub use crate::daemon::listener::PlaceholderValue;

pub struct Client {
    addr : String,
}

#[derive(Debug,Error)]
pub enum ClientErr {
    #[error("HTTP error ")]
    NotFound(#[from] reqwest::Error)
}


impl Client {

    pub fn new() -> Result<Self, ClientErr> 
    {
        let addr = "http://0.0.0.0:22000";

        Client::validate_daemon(&addr)?;

        Ok(Client { 
            addr: addr.to_string().clone(),
        })
    }

    pub fn new_from_ip<T>(addr: &T) -> Result<Self, ClientErr>
    where T: Display + Clone
    {

        Client::validate_daemon(&addr)?;

        Ok(Client { 
            addr: addr.to_string().clone(),
        })
    }

    pub fn send_placeholders<T>(&self, webhook: T, placeholders: Vec<daemon::listener::PlaceholderValue>) 
    where T: Display + Clone
    {

        let mut json = Client::build_json_status_template(webhook.to_string());
        json["content"] = json::array![];

        for pl in placeholders {
            let placeholder = object!{placeholder : pl.placeholder, value : pl.value};
            json["content"].push(placeholder).unwrap();
        }
        println!("dump: {}", &json.dump());

        self.send_status_update(json.dump());
    }


    fn send_status_update(&self, payload: String) 
    {

        println!("{}", &payload);
        let mut url = self.addr.clone();
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


    fn validate_daemon<T>(addr: &T) -> Result<StatusCode, ClientErr>
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

