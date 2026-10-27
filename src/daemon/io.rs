use super::*;

const ETC_PATH_DEV: &str = "dev/etc/";
const LOG_PATH_DEV: &str =  "dev/log/";
const DATA_PATH_DEV: &str =  "dev/lib/";


// Relevant paths for folders 
pub struct ProgramPaths {
    log: PathBuf,
    persist: PathBuf,
    etc: PathBuf,
}


impl ProgramPaths {

    // Initializes the paths for the program files based on whether this executable
    // is running from a dev build folder or on a real machine.
    pub fn get_paths(is_dev_build: bool) -> Self {
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
            .expect("Failed to find ENV directories");

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




pub fn open_as_string(path: &String, name: &String, ext: &str) -> Result<String, errors::Error> {

    // Combine the path and name into a single PathBuf
    let path_full: &PathBuf = &PathBuf::from(path).join(format!("{}{}",name, ext));

    // Open the config file and prep a string buffer to receive it's contents
    let mut file = File::open(path_full)?;
    let mut str_buf: String = String::new();

    // Write the contents of the conf file to the string buffer and parse it to a json object
    file.read_to_string(&mut str_buf)?;
    Ok(str_buf)
}


pub(super) fn parse_webhook_config(path: &String, name: &String) -> Result<WebhookConfigContents, errors::Error> {

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
    })
}


pub(super) fn parse_webhook_json_placeholders(path: &String, name: &String) -> Result<Vec<String>, errors::Error> {

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


