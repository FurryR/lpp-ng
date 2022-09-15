#[derive(Debug)]
pub struct Error {
  pub err: String,
}
impl Error {
  pub fn new() -> Self {
    Error { err: String::new() }
  }
}
impl From<String> for Error {
  fn from(err: String) -> Self {
    Error { err }
  }
}
