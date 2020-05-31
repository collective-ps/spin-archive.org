use thiserror::Error;

#[derive(Error, Debug)]
pub enum IngestorError {
  #[error("Invalid URL was provided")]
  InvalidUrl,

  #[error("No media found at the given URL")]
  NoMediaFound,
}
