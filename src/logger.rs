use std::fs;

use crate::config::Config;

pub struct Logger {
    path_name: String,
}

impl Logger {
    pub fn new(filename: &str) -> Self {
        let conf = Config::new().unwrap();
        let logs_dir = conf.get_logs_dir();
        let path_name = format!("{}/{}", logs_dir, filename);
        Logger { path_name }
    }

    pub fn read_log_file(&self) -> Result<String, std::io::Error> {
        fs::read_to_string(&self.path_name)
    }

    pub fn info(&self, message: &str) {
        println!("INFO: {}", message);
    }

    pub fn error(&self, message: &str) {
        println!("ERROR: {}", message);
    }

    pub fn warning(&self, message: &str) {
        println!("WARNING: {}", message);
    }

    pub fn debug(&self, message: &str) {
        println!("DEBUG: {}", message);
    }

    pub fn exception(&self, message: &str) {
        println!("EXCEPTION: {}", message);
    }
}
