use super::error::Error;
use super::parse::{transfer, Lpp, LppStatus, QuoteStatus};
use super::var::{covered_with, ExprValue, FuncValue, ValueType, Var};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::ptr;
use std::rc::Rc;
#[derive(Clone)]
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
  now: Rc<RefCell<Scope>>,
  global: Option<Rc<RefCell<Scope>>>,
  this: Option<Rc<RefCell<Var>>>,
}
impl From<Rc<RefCell<Scope>>> for Context {
  fn from(now: Rc<RefCell<Scope>>) -> Self {
    Context {
      now,
      global: None,
      this: None,
    }
  }
}
impl From<(Rc<RefCell<Scope>>, Rc<RefCell<Scope>>, Rc<RefCell<Var>>)> for Context {
  fn from(val: (Rc<RefCell<Scope>>, Rc<RefCell<Scope>>, Rc<RefCell<Var>>)) -> Self {
    Context {
      now: val.0,
      global: Some(val.1),
      this: Some(val.2),
    }
  }
}
impl Context {
  pub fn now(&self) -> Rc<RefCell<Scope>> {
    self.now.clone()
  }
  pub fn global(&self) -> Rc<RefCell<Scope>> {
    if let Some(val) = &self.global {
      val.clone()
    } else {
      self.now.clone()
    }
  }
  pub fn this(&self) -> Rc<RefCell<Var>> {
    if let Some(val) = &self.this {
      val.clone()
    } else {
      self.now.borrow().raw()
    }
  }
}
pub enum RetVal {
  CalcValue(Var),
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
pub struct NativeFunc {
  pub use_type: BTreeSet<ValueType>,
  pub func: FuncValue,
  pub isval: bool,
}
pub type Parser = Lpp;
pub type Native = BTreeMap<String, NativeFunc>;
pub type TableType = BTreeMap<String, fn(parser: &Parser) -> Result<RetVal, Error>>;
pub struct Handler {
  pub context: Context,
  pub cmd: TableType,
  pub next: NextVal,
  pub native: Native,
}
pub enum LazyRef {
  Value(Rc<RefCell<Var>>),
  Array((Rc<RefCell<Var>>, usize)),
  Object((Rc<RefCell<Var>>, String)),
  ScopeVar((Rc<RefCell<Scope>>, String)),
  Scope(Rc<RefCell<Scope>>),
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
  pub fn get(&self) -> Option<Rc<RefCell<Var>>> {
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
  pr: Option<RefObj>,
}
impl ResultObj {
  pub fn val(&self) -> &RefObj {
    return &self.val;
  }
  pub fn pr(&self) -> &RefObj {
    if let Some(ref val) = self.pr {
      val
    } else {
      &self.val
    }
  }
}
impl Handler {
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
    self.is_keyword(st.name.as_str())
      || (!ExprValue::isexp(st.to_string().as_str()) && covered_with(st.args.as_str(), '(', ')'))
  }
}
impl Handler {
  pub fn exec(&mut self, value: &Parser) -> Result<RetVal, Error> {
    let retval: RetVal;
    if self.is_keyword(value.name.as_str()) {
      if self.next.cmd != value.name && self.next.limit {
        return Err(Error::from(String::from("Invalid statement")));
      }
      if self.next.cmd != value.name {
        self.next = NextVal::new();
      }
      retval = self
        .cmd
        .get(&value.name)
        .expect("Keyword implement not found")(value)?;
    } else if self.cmd.contains_key("") {
      if self.next.cmd != "" && self.next.limit {
        return Err(Error::from(String::from("Invalid statement")));
      }
      if self.next.cmd != value.name {
        self.next = NextVal::new();
      }
      retval = self
        .cmd
        .get(&String::from(""))
        .expect("Default implement not found")(value)?;
    } else {
      return Err(Error::from(String::from("Invalid statement")));
    }
    Ok(retval)
  }
  pub fn get_member(&self, mut obj: RefObj, index: &Var) -> Result<RefObj, LppError> {
    let find_str = if let Var::String(str) = index {
      str.clone()
    } else {
      index.to_string()
    };
    if find_str == "this" {
      return i;
    } else if let Some((index, item)) = self.native.iter().find(|(index, item)| {
      if index == find_str.as_str() {
        Some(item)
      } else {
        None
      }
    }) {
      let scope: Scope = Scope::new();
      if let RefObj::Ref(obj) = i {
        if item.use_type.is_empty()
          || item.use_type.contains(if let Some(tmp) = obj.get() {
          } else {
          })
        {}
      }
    }
  }
  fn update_scope(now_scope: &Scope, temp_scope: &Scope) -> Scope {
    let mut ret = now_scope.clone();
    if let Var::Object(now) = &*now_scope.raw().borrow() {
      for key in now.keys() {
        if let (Some(v), c) = temp_scope.get(key) {
          ret.set(key.clone(), (v.borrow().clone(), c))
        } else {
          ret.remove(key);
        }
      }
    } else {
      panic!("Scope.raw() must be Var::Object")
    }
    ret
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
  fn var_index(&mut self, access: &str, mut start: ResultObj) -> Result<ResultObj, Error> {
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
    Err(Error::from(String::new()))
  }
  fn get_object(&mut self, str: &str) -> Result<ResultObj, Error> {
    let first_name = Self::firstname(str);
    if first_name == "" {
      return Err(Error::from(String::from("Syntax error")));
    }
    if first_name == "this" {
      return self.var_index(
        utf8_slice::slice(
          str,
          utf8_slice::len(first_name.as_str()),
          utf8_slice::len(str),
        ),
        ResultObj {
          val: RefObj::Ref(LazyRef::Value(self.context.this())),
          pr: None,
        },
      );
    }
    Err(Error::from(String::from("Syntax error")))
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
impl From<(Context, TableType, NextVal, Native)> for Handler {
  fn from(val: (Context, TableType, NextVal, Native)) -> Self {
    Handler {
      context: val.0,
      cmd: val.1,
      next: val.2,
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
