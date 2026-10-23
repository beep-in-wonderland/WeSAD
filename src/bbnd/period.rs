use std::fmt::write;

use super::*;


#[derive(Clone,Copy,Debug)]
pub struct Period {
    value : f64,
}


impl Period {
    pub fn from_secs(s: f64) -> Self {
        Self { value : s}
    }

    pub fn from_mins(m: f64) -> Self {
        Self { value : m*60.0}
    }

    pub fn from_hrs(h: f64) -> Self {
        Self { value : h*60.0*60.0}
    }

    pub fn from_type_value(increment: &String, val: f64) -> Self {
        Self {
            value : match increment.to_lowercase().trim() {
                    "seconds" => val,
                    "minutes" => val*60.0, // Convert minutes to seconds
                    "hours" => val*60.0*60.0, // Convert hours to seconds
                    _ => 0.0
                }
        }
    }

    pub fn get(&self) -> f64 {
        self.value
    }

}

impl Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<f64> for Period {
    fn from(value: f64) -> Self {
        Self {
            value: value
        }
    }
}