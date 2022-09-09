pub mod module;
use module::var;
//use module::var::NewVar;
fn main() {
  let a: var::Var = var::Var::from("1.1e3").unwrap();
  if let var::Var::Number(val) = a {
    println!("{}", val);
  }
}
