use super::*;
use crate::json::object;
use crate::axum::http::StatusCode;
use crate::thiserror::Error;

pub use crate::service::listener::TagValue;

pub struct Client {
    endpoint : String,
}

#[derive(Debug,Error)]
pub enum ClientErr {
    #[error("HTTP error ")]
    NotFound(#[from] reqwest::Error)
}


impl Client {

    pub fn new() -> Result<Self, ClientErr> 
    {
        let endpoint = format!("{}:{}", io::DEFAULT_ADDR, io::DEFAULT_PORT);

        Client::validate_wesad_listener(&endpoint)?;

        Ok(Client { 
            endpoint,
        })
    }

    pub fn new_from_ip<T>(endpoint: &T) -> Result<Self, ClientErr>
    where T: Display + Clone
    {

        Client::validate_wesad_listener(&endpoint)?;

        Ok(Client { 
            endpoint: endpoint.to_string().clone(),
        })
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


    fn validate_wesad_listener<T>(addr: &T) -> Result<StatusCode, ClientErr>
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

