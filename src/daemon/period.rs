use std::fmt::write;

use super::*;


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