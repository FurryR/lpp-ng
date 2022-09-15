pub mod module;
use module::{lpp, var::Var};
use std::cell::RefCell;
//use module::var::NewVar;
fn main() {
  let mut a = lpp::Scope::new();
  a.set(String::from("awa"), (Var::Boolean(true), true));
  let f = RefCell::new(a);
  let mut b = lpp::Context::from(&f);
  //let c = lpp::Handler::from(b);
  if let lpp::RefObj::Ref(mut a) = b.test().unwrap() {
    println!("{}", a.get().unwrap().to_string());
    *a.get_mut().unwrap() = Var::new();
    if let lpp::RefObj::Ref(c) = b.test().unwrap() {
      println!("{}", c.get().unwrap().to_string());
      println!("{}", a.get().unwrap().to_string())
    }
  }
}
