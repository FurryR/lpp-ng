pub struct Error {
  err: String,
}
impl Error {
  pub fn new(error: String) -> Error {
    Error { err: error }
  }
  pub fn what(&self) -> &String {
    &self.err
  }
}
