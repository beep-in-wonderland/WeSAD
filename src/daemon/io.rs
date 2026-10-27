use super::*;

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
        let log: PathBuf;
        let persist: PathBuf;
        let etc: PathBuf;

        match is_dev_build {
            true => {
                let root_dir = current_dir().unwrap();

                log = root_dir.join("dev/log/");
                persist = root_dir.join("dev/persist/");
                etc = root_dir.join("dev/etc/")
            }

            false => {
                log = PathBuf::from("/var/log/bbnd/");
                persist = PathBuf::from("/var/lib/bbnd/");
                etc = PathBuf::from("/etc/")
            }
        }

        debug!("Log path: {0}", log.display());
        debug!("Persist path: {0}", persist.display());
        debug!("Etc path: {0}", etc.display());
        

        Self { 
            log, 
            persist, 
            etc, 
        }
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
