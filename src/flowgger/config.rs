use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::path::Path;
use toml;

#[derive(Clone)]
pub struct Config {
    config: toml::Value,
}

impl Config {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Config, Error> {
        let mut fd = File::open(path)?;
        let mut toml = String::new();
        fd.read_to_string(&mut toml)?;
        Config::from_string(&toml)
    }

    pub fn from_string(toml: &str) -> Result<Config, Error> {
        let config = match toml.parse() {
            Ok(config) => config,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Syntax error - config file is not valid TOML",
                ))
            }
        };
        Ok(Config { config })
    }

    pub fn lookup<'a>(&'a self, path: &'a str) -> Option<&'a toml::Value> {
        self.config.lookup(path)
    }
}
