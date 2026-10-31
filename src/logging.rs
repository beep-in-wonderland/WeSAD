use super::*;

// This use statement deliberately pulls all the macros into scope so
// that other modules can simply `use crate::logging::*;` to do the same.
pub use log::{info,warn,debug,error,Level};


pub fn init_logger(log_level: &Level, log_path: &PathBuf) {

    let log_path: &PathBuf = &log_path.join("bbnd.log");
    if !exists(log_path).unwrap() {
        let _ = File::create_new(log_path);
    };

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log_level.to_level_filter())
        .chain(std::io::stdout())
        .chain(fern::log_file(log_path).unwrap())
        .apply()
        .unwrap();

    debug!("Logger initialized")
}