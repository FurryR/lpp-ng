use super::error::Error;
use super::parse::{transfer, LppStatus, QuoteStatus};
use super::var::{covered_with, ExprValue, FuncValue, ValueType, Var};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::ptr;
use std::rc::{Rc, Weak};
pub struct Scope {
  val: Rc<RefCell<Var>>,
  constant: BTreeMap<String, bool>,
}
impl Scope {
  pub fn raw(&self) -> Rc<RefCell<Var>> {
    self.val.clone()
  }
  pub fn get(&self, key: &String) -> (Option<Rc<RefCell<Var>>>, bool) {
    if let Var::Object(ref val) = *self.val.borrow() {
      if let Some(value) = val.get(key) {
        if let Some(constant) = self.constant.get(key) {
          (Some(value.clone()), *constant)
        } else {
          (Some(value.clone()), false)
        }
      } else {
        (None, false)
      }
    } else {
      panic!("self.val must be Var::Object");
    }
  }
  pub fn set(&mut self, key: String, value: (Var, bool)) {
    if let Var::Object(ref mut val) = *self.val.borrow_mut() {
      val.insert(key.clone(), Rc::new(RefCell::new(value.0)));
      self.constant.insert(key, value.1);
    } else {
      panic!("self.val must be Var::Object");
    }
  }
  pub fn remove(&mut self, key: &String) -> (Option<Rc<RefCell<Var>>>, bool) {
    if let Var::Object(ref mut val) = *self.val.borrow_mut() {
      (
        val.remove(key),
        if let Some(item) = self.constant.remove(key) {
          item
        } else {
          false
        },
      )
    } else {
      panic!("self.val must be Var::Object");
    }
  }
}
impl From<(BTreeMap<String, Rc<RefCell<Var>>>, BTreeMap<String, bool>)> for Scope {
  fn from(val: (BTreeMap<String, Rc<RefCell<Var>>>, BTreeMap<String, bool>)) -> Self {
    Scope {
      val: Rc::new(RefCell::new(Var::Object(val.0))),
      constant: val.1,
    }
  }
}
impl Scope {
  pub fn new() -> Self {
    Scope {
      val: Rc::new(RefCell::new(Var::Object(BTreeMap::new()))),
      constant: BTreeMap::new(),
    }
  }
}
pub struct Context {
  pub now: Rc<RefCell<Scope>>,
  pub global: Rc<RefCell<Scope>>,
  pub this: Weak<RefCell<Var>>,
}
impl From<Rc<RefCell<Scope>>> for Context {
  fn from(now: Rc<RefCell<Scope>>) -> Self {
    Context {
      now,
      global: now.clone(),
      this: now.borrow().raw(),
    }
  }
}
impl From<(Rc<RefCell<Scope>>, Rc<RefCell<Scope>>, Rc<RefCell<Var>>)> for Context {
  fn from(val: (Rc<RefCell<Scope>>, Rc<RefCell<Scope>>, Rc<RefCell<Var>>)) -> Self {
    Context {
      now: val.0,
      global: val.1,
      this: val.2,
    }
  }
}
pub enum RetVal {
  RetValue(Var),
  ThrowValue(Var),
}
pub struct NextVal {
  pub cmd: String,
  pub limit: bool,
  pub value: Var,
}
pub enum LppError {
  UnexpectedReturn(RetVal),
  Error(Error),
}
impl NextVal {
  pub fn new() -> Self {
    NextVal {
      cmd: String::new(),
      limit: false,
      value: Var::Null(()),
    }
  }
}
#[derive(Clone)]
pub struct NativeFunc {
  pub use_type: BTreeSet<ValueType>,
  pub func: FuncValue,
  pub isval: bool,
}
pub trait ParserInterface {
  fn name(&self) -> &String;
  fn args(&self) -> &String;
  fn new() -> Self;
  fn to_string(&self) -> String;
  fn parse(str: &str) -> Self;
}
pub trait CodeSplitInterface {
  fn code_split(str: &str) -> Vec<String>;
}
pub struct Handler<Parser> {
  pub context: Context,
  pub cmd: BTreeMap<String, fn(parser: &Parser) -> Result<Var, LppError>>,
  pub next: RefCell<NextVal>,
  pub native: BTreeMap<String, NativeFunc>,
}
pub enum LazyRef {
  Value(Weak<RefCell<Var>>),
  Array((Weak<RefCell<Var>>, usize)),
  Object((Weak<RefCell<Var>>, String)),
  ScopeVar((Weak<RefCell<Scope>>, String)),
  Scope(Weak<RefCell<Scope>>),
}
impl LazyRef {
  pub fn create(&self) {
    match self {
      LazyRef::Value(_) => (),
      LazyRef::Array((val, index)) => {
        if let Var::Array(arr) = &*val.borrow() {
          if *index >= arr.len() {
            if let Var::Array(ref mut arr) = *val.borrow_mut() {
              arr.resize(*index + 1, Rc::new(RefCell::new(Var::new())));
            }
          }
        } else {
          panic!("Cannot create in a non-Array object");
        }
      }
      LazyRef::Object((val, index)) => {
        if let Var::Object(obj) = &*val.borrow() {
          if !obj.contains_key(index) {
            if let Var::Object(ref mut obj) = *val.borrow_mut() {
              obj.insert(index.clone(), Rc::new(RefCell::new(Var::new())));
            }
          }
        } else {
          panic!("Cannot create in a non-Object object");
        }
      }
      LazyRef::ScopeVar((val, index)) => {
        if let None = val.borrow().get(index).0 {
          val.borrow_mut().set(index.clone(), (Var::new(), false));
        }
      }
      LazyRef::Scope(_) => (),
    }
  }
  pub fn get(&self) -> Weak<RefCell<Var>> {
    match self {
      LazyRef::Value(val) => Some(val.clone()),
      LazyRef::Array((val, index)) => {
        if let Var::Array(arr) = &*val.borrow() {
          if let Some(rc) = arr.get(*index) {
            Some(rc.clone())
          } else {
            None
          }
        } else {
          panic!("Cannot get in a non-Array object");
        }
      }
      LazyRef::Object((val, index)) => {
        if let Var::Object(obj) = &*val.borrow() {
          if let Some(rc) = obj.get(index) {
            Some(rc.clone())
          } else {
            None
          }
        } else {
          panic!("Cannot get in a non-Object object");
        }
      }
      LazyRef::ScopeVar((val, index)) => val.borrow().get(index).0,
      LazyRef::Scope(val) => None,
    }
  }
}
impl From<Rc<RefCell<Var>>> for LazyRef {
  fn from(val: Rc<RefCell<Var>>) -> Self {
    LazyRef::Value(val)
  }
}
impl From<(Rc<RefCell<Var>>, usize)> for LazyRef {
  fn from(val: (Rc<RefCell<Var>>, usize)) -> Self {
    LazyRef::Array(val)
  }
}
impl From<(Rc<RefCell<Var>>, String)> for LazyRef {
  fn from(val: (Rc<RefCell<Var>>, String)) -> Self {
    LazyRef::Object(val)
  }
}
impl From<Rc<RefCell<Scope>>> for LazyRef {
  fn from(val: Rc<RefCell<Scope>>) -> Self {
    LazyRef::Scope(val)
  }
}
impl From<(Rc<RefCell<Scope>>, String)> for LazyRef {
  fn from(val: (Rc<RefCell<Scope>>, String)) -> Self {
    LazyRef::ScopeVar(val)
  }
}
pub enum RefObj {
  Value(Var),
  Ref(LazyRef),
  Overloaded((Var, LazyRef)),
}
pub struct ResultObj {
  val: RefObj,
  pr: RefObj,
}
impl ResultObj {
  pub fn val(&self) -> &RefObj {
    &self.val
  }
  pub fn pr(&self) -> &RefObj {
    &self.pr
  }
}
impl<Parser: ParserInterface> Handler<Parser>
where
  Handler<Parser>: CodeSplitInterface,
{
  pub fn is_keyword(&self, str: &str) -> bool {
    str != "" && self.cmd.contains_key(&str.to_string())
  }
  pub fn is_identifier(&self, str: &str) -> bool {
    if utf8_slice::len(str) == 0 || self.is_keyword(str) {
      false
    } else {
      let mut flag = false;
      for (index, item) in str.chars().enumerate() {
        if index == 1 {
          flag = true;
        }
        if item >= '0' && item <= '9' {
          if !flag {
            return false;
          }
        } else if !((item >= 'a' && item <= 'z')
          || (item >= 'A' && item <= 'Z')
          || item == '_'
          || item == '$')
        {
          return false;
        }
      }
      true
    }
  }
  pub fn is_statement(&self, st: &Parser) -> bool {
    self.is_keyword(st.name().as_str())
      || (!ExprValue::isexp(st.to_string().as_str()) && covered_with(st.args().as_str(), '(', ')'))
  }
}
impl<Parser: ParserInterface> Handler<Parser>
where
  Handler<Parser>: CodeSplitInterface,
{
  pub fn exec(&self, value: &Parser) -> Result<Var, LppError> {
    let retval: Var;
    if self.is_keyword(value.name().as_str()) {
      if self.next.borrow().cmd != *value.name() && self.next.borrow().limit {
        return Err(LppError::Error(Error::from("Invalid statement")));
      }
      if self.next.borrow().cmd != *value.name() {
        *self.next.borrow_mut() = NextVal::new();
      }
      retval = self
        .cmd
        .get(value.name())
        .expect("Keyword implement not found")(value)?;
    } else if self.cmd.contains_key("") {
      if self.next.borrow().cmd != "" && self.next.borrow().limit {
        return Err(LppError::Error(Error::from("Invalid statement")));
      }
      if self.next.borrow().cmd != *value.name() {
        *self.next.borrow_mut() = NextVal::new();
      }
      retval = self
        .cmd
        .get(&String::from(""))
        .expect("Default implement not found")(value)?;
    } else {
      return Err(LppError::Error(Error::from("Invalid statement")));
    }
    Ok(retval)
  }
  pub fn runfunc(&self, func: &FuncValue, args: Vec<Var>) -> Result<Var, LppError> {
    let mut scope = Scope::new();
    let mut arguments: Vec<Rc<RefCell<Var>>> = vec![];
    let code = Self::code_split(func.value.value.as_str());
    for (index, item) in func.args.iter().enumerate() {
      if args.len() > index {
        //let v = self.expr(Var::parse(item.value.as_str()))?;
        arguments.push(Rc::new(RefCell::new(args[index].clone())));
        scope.set(item.name.clone(), (args[index].clone(), false));
      } else {
        if item.value == "" {
          return Err(LppError::Error(Error::from("Too few arguments given")));
        }
        let v = self.expr(Var::parse(item.value.as_str()))?;
        arguments.push(Rc::new(RefCell::new(v.clone())));
        scope.set(item.name.clone(), (v.clone(), false));
      }
    }
    scope.set(String::from("arguments"), (Var::Array(arguments), false))
    //a.set(, value)
  }
  pub fn get_member(&self, mut obj: RefObj, index: &Var) -> Result<RefObj, LppError> {
    let find_str = if let Var::String(str) = index {
      str.clone()
    } else {
      index.to_string()
    };
    if find_str == "this" {
      Ok(obj)
    } else if let Some((index, item)) = self.native.iter().find(|(index, _)| **index == find_str) {
      let scope: Scope = Scope::new();
      if let RefObj::Ref(obj) = obj {
        if item.use_type.is_empty()
          || if let Some(tmp) = obj.get() {
            item.use_type.contains(&tmp.borrow().tp())
          } else {
            false
          }
        {
          if item.isval {
            let a = Handler::<Parser>::from((
              Context::from((self.context.now, self.context.global)),
              self.cmd.clone(),
              NextVal::new(),
              self.native.clone(),
            ));
          }
        }
      }
    }
  }
  fn update_scope(now: Scope, temp: &Scope) -> Scope {
    if let Var::Object(obj) = &*now.raw().borrow() {
      for item in obj.keys().cloned().collect::<Vec<String>>() {
        if let (Some(v), c) = temp.get(&item) {
          now.set(item, (v.borrow().clone(), c))
        } else {
          now.remove(&item);
        }
      }
      now
    } else {
      panic!("Scope.raw() must be Var::Object")
    }
  }
  fn firstname(str: &str) -> String {
    let mut temp = String::new();
    let mut status = LppStatus::new();
    let mut lastchar = '\0';
    for item in str.chars() {
      transfer(item, &mut status);
      if item == '.' && status.quote == QuoteStatus::None && status.brace == 0 {
        break;
      } else if item == '[' && status.quote == QuoteStatus::None && status.brace == 1 {
        if temp == "" && (lastchar != ']') {
          temp.push(item);
        } else {
          break;
        }
      } else {
        temp.push(item);
      }
      lastchar = item;
    }
    temp
  }
  fn var_index(&self, access: &str, mut start: ResultObj) -> Result<ResultObj, LppError> {
    let visit = Self::name_split(access);
    let this_keep = if let RefObj::Value(_) = start.val() {
      false
    } else {
      if let RefObj::Ref(v) = start.val() {
        if let Some(s) = v.get() {
          ptr::eq(s.as_ptr(), self.context.this().as_ptr())
        } else {
          false
        }
      } else if let RefObj::Overloaded(v) = start.val {
        if let Some(s) = v.1.get() {
          std::ptr::eq(s.as_ptr(), self.context.this().as_ptr())
        } else {
          false
        }
      } else {
        false
      }
    };
    Err(LppError::Error(Error::from("")))
  }
  fn get_object(&self, str: &str) -> Result<ResultObj, LppError> {
    let first_name = Self::firstname(str);
    if first_name == "" {
      return Err(LppError::Error(Error::from("Syntax error")));
    }
    if first_name == "this" {
      self.var_index(
        utf8_slice::slice(
          str,
          utf8_slice::len(first_name.as_str()),
          utf8_slice::len(str),
        ),
        ResultObj {
          val: RefObj::Ref(LazyRef::Value(self.context.this())),
          pr: RefObj::Ref(LazyRef::Value(self.context.this())),
        },
      );
    }
    Err(LppError::Error(Error::from("Syntax error")))
  }
  fn name_split(str: &str) -> Vec<String> {
    let mut ret: Vec<String> = vec![];
    let mut temp = String::new();
    let mut skip: usize = 0;
    let mut status = LppStatus::new();
    for (index, item) in str.chars().enumerate() {
      if skip > 0 {
        skip -= 1;
        continue;
      }
      transfer(item, &mut status);
      if (item == '[' || item == ']')
        && status.quote == QuoteStatus::None
        && status.brace == if item == '[' { 1 } else { 0 }
      {
        if temp != "" {
          ret.push(temp);
          temp = String::new();
        }
      } else if item == '.' && status.quote == QuoteStatus::None && status.brace == 0 {
        skip += 1;
        let slice = utf8_slice::slice(str, index + skip, utf8_slice::len(str));
        for subitem in slice.chars() {
          temp.push(subitem);
          skip += 1;
        }
        ret.push(format!("\"{}\"", temp));
        temp.clear();
        skip -= 1;
      } else {
        temp.push(item);
      }
    }
    if temp != "" {
      ret.push(temp);
    }
    return ret;
  }
}
impl<Parser: ParserInterface>
  From<(
    Context,
    BTreeMap<String, fn(parser: &Parser) -> Result<Var, LppError>>,
    NextVal,
    BTreeMap<String, NativeFunc>,
  )> for Handler<Parser>
{
  fn from(
    val: (
      Context,
      BTreeMap<String, fn(parser: &Parser) -> Result<Var, LppError>>,
      NextVal,
      BTreeMap<String, NativeFunc>,
    ),
  ) -> Self {
    Handler {
      context: val.0,
      cmd: val.1,
      next: RefCell::new(val.2),
      native: val.3,
    }
  }
}
// impl Context {
//   pub fn test(&mut self) -> Option<RefObj> {
//     let a = self.global().borrow().get_mut(&String::from("awa"));
//     if let Some(val) = a.0 {
//       Some(RefObj::Ref(LazyRef::Value(val)))
//     } else {
//       None
//     }
//   }
// }
