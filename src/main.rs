pub mod module;
use module::var;
use module::var::NewVar;
fn main() {
  let a: var::Var = var::Var::new(1.0);
  if let var::RawValue::Number(val) = a.value {
    println!("{}", val);
  }
}
