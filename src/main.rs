pub mod module;
use module::{lpp, var::Var};
use std::cell::RefCell;
use std::rc::Rc;
//use module::var::NewVar;
fn main() {
  let mut a = lpp::Scope::new();
  a.set(String::from("awa"), (Var::Boolean(true), true));
  let f = RefCell::new(a);
  let b = lpp::Context::from(Rc::new(f));
  let c = lpp::Handler::from(b);
  c.
}
