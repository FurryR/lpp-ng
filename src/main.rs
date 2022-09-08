pub mod module;
use module::var;
//use module::var::NewVar;
fn main() {
  let a: var::Var = var::Var::from("func(){awa}").unwrap();
  if let var::RawValue::Function(val) = a.value {
    println!("{}", val.value.value);
  }
}
