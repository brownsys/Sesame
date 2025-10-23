use std::env::VarError;

#[derive(Debug)]
pub enum Error {
    VarError(VarError),
    IoError(std::io::Error),
    ManifestError(cargo_toml::Error)
}
impl From<VarError> for Error {
    fn from(e: VarError) -> Self {
        Error::VarError(e)
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}
impl From<cargo_toml::Error> for Error {
    fn from(e: cargo_toml::Error) -> Self {
        Error::ManifestError(e)
    }
}