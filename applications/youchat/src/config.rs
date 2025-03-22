use std::fs;
use std::io::{Error, ErrorKind, Read};
use toml;
use alohomora::AlohomoraType;

#[derive(AlohomoraType, Debug, Clone)]
#[alohomora_out_type(to_derive = [Debug, Clone])]
pub struct Config {
    /// user for the mySQL database
    pub db_user : String,
    /// password for the mySQL database
    pub db_password : String,
    /// custom directory for templates
    pub template_dir : String,
    //whether or not to initialize the mySQL database
    pub prime: bool,
}

pub(crate) fn parse(path: &str) -> Result<Config, Error>{
    let mut f = fs::File::open(path)?;
    let mut buf = String::new();

    f.read_to_string(&mut buf)?;

    //can update to newer version of toml (i think parser might be outdated)
    let value = match toml::Parser::new(&buf).parse() {
        None => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "failed to parse config!",
            ))
        }
        Some(v) => v,
    };

    Ok(Config {
        db_user: value.get("db_user").unwrap().as_str().unwrap().into(),
        db_password: value.get("db_password").unwrap().as_str().unwrap().into(),
        template_dir: value.get("template_dir").unwrap().as_str().unwrap().into(),
        prime: value.get("prime").unwrap().as_bool().unwrap().into(),
    })
}