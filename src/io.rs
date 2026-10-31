use super::*;

use crate::logging::*;
use crate::json::{
    self,
    JsonValue,
};
use crate::File;

use crate::fancy_regex::Regex;
use crate::errors::Error::*;

pub const ETC_PATH_DEV: &str = "dev/etc/";
pub const LOG_PATH_DEV: &str =  "dev/log/";
pub const DATA_PATH_DEV: &str =  "dev/lib/";
pub const ETC_PATH_WEBHOOKS: &str = "webhooks.d";

pub const DEFAULT_ADDR: &str = "0.0.0.0";
pub const DEFAULT_PORT: &str = "22000";

/// Regex capture which will split a fully qualified file path with extension into three non-overlapping captures:
/// example: ``/this/is/my/file.conf``
/// `dir = /this/is/my/`
/// `name = file`
/// `ext = .conf`
pub(super) const REGEX_CONF: &str = "(?<dir>^[(?:\\/|\\\\)0-9a-zA-Z_\\-\\. ]+(?:\\.d)?(?:\\/|\\\\))(?<name>[0-9a-zA-Z_\\-. ]+)(?<ext>\\.conf)$";
pub(super) const REGEX_PLACEHOLDERS: &str = "(\\$\\$(?<flag>[\\w]+)\\$\\$)";


pub(super) struct WebhookConfigContents {
    pub(super) name : String,
    pub(super) url : String,
    pub(super) webhook_type: webhook::Type,
    pub(super) interval: String,
    pub(super) period: i64,
    pub(super) msg_id: Option<String>,
}


pub(super) struct RuntimeConfig {
    pub(super) listener_addr : String,
    pub(super) listener_port : String,
    pub(super) smtp_user: String,
    pub(super) smtp_target: String,
    pub(super) smtp_host: String,
    pub(super) smtp_port: String,
}


impl RuntimeConfig {
    pub(super) fn new(paths: &io::ProgramPaths) -> Self {

        let json_str = io::open_as_string(
            &paths.get_etc().to_str().unwrap().to_string(),
            &"bbnd".to_string(),
            &".conf".to_string()
        ).unwrap_or(
            // If the file wasn't found, generate an empty json str so defaults get populated
            "{\"never gonna\" : \"give you up\"}".to_string() 
        );

        let config = json::parse(json_str.as_str()).expect("Either config was found or blank was generated");

        let result = RuntimeConfig { 
            listener_addr: RuntimeConfig::has_key_or_default(
                &config, vec!["listener_addr"]), 
            listener_port: RuntimeConfig::has_key_or_default(
                &config, vec!["listener_port"]), 
            smtp_user: RuntimeConfig::has_key_or_default(
                &config, vec!["smtp_config", "smtp_user"]), 
            smtp_target: RuntimeConfig::has_key_or_default(
                &config, vec!["smtp_config", "smtp_target"]), 
            smtp_host: RuntimeConfig::has_key_or_default(
                &config, vec!["smtp_config", "smtp_host"]), 
            smtp_port: RuntimeConfig::has_key_or_default(
                &config, vec!["smtp_config", "smtp_port"]), 
        };

        debug!("{}", result);
        result
    }


    fn has_key_or_default(jsonval: &JsonValue, keys: Vec<&str>) -> String {

        let mut target: JsonValue = jsonval.clone();

        for key in keys {
            if target.has_key(key) {
                target = target[key].clone();
            } else {
                return match key {
                    "listener_addr" => DEFAULT_ADDR,
                    "listener_port" => DEFAULT_PORT,
                    _               => ""
                }.to_string()
            }
        }
        return target.to_string()

    }
}

impl Display for RuntimeConfig {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RuntimeConfig:
        listener-ip: {}:{}
        smtp user: {}
        smtp target: {}
        smtp host: {}:{}", 
        self.listener_addr, self.listener_port, 
        self.smtp_user, self.smtp_target,
        self.smtp_host, self.smtp_port)
    }
}


// Relevant paths for folders 
pub struct ProgramPaths {
    log: PathBuf,
    persist: PathBuf,
    etc: PathBuf,
}

impl ProgramPaths {

    // Initializes the paths for the program files based on whether this executable
    // is running from a dev build folder or on a real machine.
    pub fn build_paths(is_dev_build: bool) -> Self {
        if is_dev_build {
            ProgramPaths::init_dev_directories()
        } else {
            ProgramPaths::init_env_directories()
        }.unwrap_or_else(|e| {
            panic!("Could not initialize directories: {e}")
        })
    }

    pub fn get_log(self: &ProgramPaths) -> &PathBuf {
        &self.log
    }

    pub fn get_etc(self: &ProgramPaths) -> &PathBuf {
        &self.etc
    }

    pub fn get_persist(self: &ProgramPaths) -> &PathBuf {
        &self.persist
    }

    fn init_dev_directories() -> Result<Self, errors::Error> {

        let root = current_dir()?;
        let etc = root.join(ETC_PATH_DEV);
        let log = root.join(LOG_PATH_DEV);
        let persist = root.join(DATA_PATH_DEV);

        Ok(Self {
            etc,
            log,
            persist,
        })
    }

    fn init_env_directories() -> Result<Self, errors::Error> {
        let dirs = directories::ProjectDirs::from(
            "com", "beepdodev", "bbnd")
            .expect("Failed to build ProjectDirs directories");

        let etc = dirs.config_dir().to_path_buf();
        let log = dirs.state_dir().unwrap().to_path_buf();
        let persist = dirs.data_dir().to_path_buf();

        std::fs::create_dir_all(etc.join("webhooks.d"))?;
        std::fs::create_dir_all(&log)?;
        std::fs::create_dir_all(&persist)?;

        Ok(Self { 
            log, 
            persist, 
            etc,
        })
    }
}


pub(super) fn open_as_string(path: &String, name: &String, ext: &str) -> Result<String, errors::Error> {

    // Combine the path and name into a single PathBuf
    let path_full: &PathBuf = &PathBuf::from(path).join(format!("{}{}",name, ext));

    // Open the config file and prep a string buffer to receive it's contents
    let mut file = File::open(path_full)?;
    let mut str_buf: String = String::new();

    // Write the contents of the conf file to the string buffer and parse it to a json object
    file.read_to_string(&mut str_buf)?;
    Ok(str_buf)
}


pub(super) fn read_webhook_config(path: &String, name: &String) -> Result<WebhookConfigContents, errors::Error> {

    // Open the config file
    let conf_str: &String = &open_as_string(path, name, ".conf")?;

    let conf_json = json::parse(conf_str)?;

    validate_config_lines(&conf_json, &name)?;

    Ok(WebhookConfigContents { 
        name: name.clone(),
        url : conf_json["url"].to_string(), 
        webhook_type : webhook::Type::from_str(conf_json["type"].as_str().unwrap()).unwrap(), 
        interval : conf_json["interval"].to_string(),
        period : conf_json["period"].as_i64().unwrap(),
        msg_id : if conf_json["msg_id"].is_string() {Some(conf_json["msg_id"].to_string())} else {None}
    })
}


pub(super) fn read_webhook_json_placeholders(path: &String, name: &String) -> Result<Vec<String>, errors::Error> {

    let mut placeholders = Vec::new();

    // Open the json template embed file
    let json_str: &String = &open_as_string(path, name, ".json")?;

    let re = Regex::new(REGEX_PLACEHOLDERS)?;
    let matches = re.captures_iter(json_str);

    for regmatch in matches {
        placeholders.push(regmatch.unwrap().get(2).unwrap().as_str().to_string());
    }

    Ok(placeholders)
}


fn validate_config_lines(json_data: &JsonValue, name: &String) -> Result<(), errors::Error> {

    if json_data["url"].is_string() { 
    } else {
        ConfigErr(format!("{}.conf: invalid webhook url.", &name));
    }

    if json_data["type"].is_string() { 
    } else {
        ConfigErr(format!("{}.conf: invalid webhook type. Choose \"status\" or \"alert\".", &name));
    }

    if json_data["interval"].is_string() { 
    } else {
        ConfigErr(format!("{}.conf: invalid webhook interval type. Choose \"seconds\", \"minutes\", or \"hours\".", &name));
    }

    if json_data["period"].is_number() { 
    } else {
        ConfigErr(format!("{}.conf: invalid webhook interval value. Choose an integer.", &name));
    }

    Ok(())
}