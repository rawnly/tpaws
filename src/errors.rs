use thiserror::Error;

#[derive(Error, Debug)]
pub enum GenericError {
    #[error("{0} is not in your path")]
    NotInstalled(&'static str),
}
