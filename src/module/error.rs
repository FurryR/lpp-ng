#[derive(Debug)]
pub struct Error {
  pub err: String,
}
impl Error {
  pub fn new(err: String) -> Self {
    Error { err }
  }
}
impl From<&str> for Error {
  fn from(err: &str) -> Self {
    Error {
      err: err.to_string(),
    }
  }
}
