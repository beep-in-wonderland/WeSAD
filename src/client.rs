use super::*;
use crate::json::object;

pub use crate::daemon::listener::PlaceholderValue;

pub struct Client {
    addr : String,
}


impl Client {

    pub fn new() -> Self 
    {
        Client { addr: "http://127.0.0.1:22000/status".to_string() }
    }

    pub fn new_from_ip<T>(addr: &T) -> Self 
    where T: Display + Clone
    {
        Client { 
            addr: addr.to_string().clone(),
        }
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

        let resp = reqwest::blocking::Client::new()
            .post(&self.addr)
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


    fn validate_daemon(&self) 
    {
        // TODO send a GET to the daemon to check for health
    }
}

