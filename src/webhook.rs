use super::*;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    STATUS,
    ALERT,
}


impl FromStr for Type {

    type Err = ();

    fn from_str(str: &str) -> Result<Type, Self::Err> {

        match str.to_lowercase().trim() {
            "status" => Ok(Type::STATUS),
            "alert"  => Ok(Type::ALERT),
            _        => Err(()),
        }
    }

}


impl Display for Type {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        match self {
            Type::ALERT => write!(f, "alert"),
            Type::STATUS => write!(f, "status"),
        }
    }
}


#[derive(Clone)]
pub struct Descriptor {
    pub name : String,
    pub url : String,
    pub webhook_type: Type,
    pub period: Period,
}


impl Display for Descriptor {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "
            Webhook {}
            url: {}
            type: {}
            period: {}
            ", 
            self.name,
            self.url,
            self.webhook_type,
            self.period.to_string(),
        )
    }
}


pub struct Status {
    pub name: String,
    pub url: String,
    pub period: webhook::Period,
    pub last: i64,
    pub msg_id: u64,
}


impl Status {
    pub fn should_update(&self, now: &i64) -> bool {
        *now > (self.last + self.period.get())
    }



}


#[derive(Clone,Copy,Debug)]
pub struct Period {
    value : i64,
}


impl Period {
    pub fn from_millis(ms: i64) -> Self {
        Self { value : ms}
    }

    pub fn from_secs(s: i64) -> Self {
        Self { value : s*1000}
    }

    pub fn from_mins(m: i64) -> Self {
        Self { value : m*1000*60}
    }

    pub fn from_hrs(h: i64) -> Self {
        Self { value : h*1000*60*60}
    }

    pub fn from_type_value(increment: &String, val: i64) -> Self {
        match increment.to_lowercase().trim() {
            "milliseconds" => Period::from_millis(val),
            "seconds" => Period::from_secs(val),
            "minutes" => Period::from_mins(val), // Convert minutes to seconds
            "hours" => Period::from_hrs(val), // Convert hours to seconds
            _ => Period::from_secs(0),
        }
    }

    pub fn get(&self) -> i64 {
        self.value
    }

}

impl Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<i64> for Period {
    fn from(value: i64) -> Self {
        Self {
            value: value
        }
    }
}