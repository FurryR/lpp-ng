use super::error::Error;
use super::parse::{transfer, Lpp, LppStatus, QuoteStatus};
use super::var::{covered_with, ExprValue, FuncValue, ValueType, Var};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::ptr;
#[derive(Clone)]
pub struct Scope {
  val: RefCell<Var>,
  constant: BTreeMap<String, bool>,
}
impl Scope {
  pub fn raw(&self) -> &RefCell<Var> {
    &self.val
  }
  // pub fn raw_mut(&mut self) -> &mut Var {
  //   &mut self.val
  // }
  pub fn get(&self, key: &String) -> (Option<Var>, bool) {
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
  // pub fn get_mut(&self, key: &String) -> (Option<&mut Var>, bool) {
  //   if let Var::Object(ref mut val) = *self.val.borrow_mut() {
  //     if let Some(value) = val.get_mut(key) {
  //       if let Some(constant) = self.constant.get(key) {
  //         (Some(value), *constant)
  //       } else {
  //         (Some(value), false)
  //       }
  //     } else {
  //       (None, false)
  //     }
  //   } else {
  //     panic!("self.val must be Var::Object");
  //   }
  // }
  pub fn set(&mut self, key: String, value: (Var, bool)) {
    if let Var::Object(ref mut val) = *self.val.borrow_mut() {
      val.insert(key.clone(), value.0);
      self.constant.insert(key, value.1);
    } else {
      panic!("self.val must be Var::Object");
    }
  }
  pub fn remove(&mut self, key: &String) -> (Option<Var>, bool) {
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
impl From<(BTreeMap<String, Var>, BTreeMap<String, bool>)> for Scope {
  fn from(val: (BTreeMap<String, Var>, BTreeMap<String, bool>)) -> Self {
    Scope {
      val: RefCell::new(Var::Object(val.0)),
      constant: val.1,
    }
  }
}
impl Scope {
  pub fn new() -> Self {
    Scope {
      val: RefCell::new(Var::Object(BTreeMap::new())),
      constant: BTreeMap::new(),
    }
  }
}
pub struct Context<'a> {
  now: &'a RefCell<Scope>,
  global: Option<&'a RefCell<Scope>>,
  this: Option<&'a RefCell<Var>>,
}
impl<'a> From<&'a RefCell<Scope>> for Context<'a> {
  fn from(now: &'a RefCell<Scope>) -> Self {
    Context {
      now,
      global: None,
      this: None,
    }
  }
}
impl<'a> From<(&'a RefCell<Scope>, &'a RefCell<Scope>, &'a RefCell<Var>)> for Context<'a> {
  fn from(val: (&'a RefCell<Scope>, &'a RefCell<Scope>, &'a RefCell<Var>)) -> Self {
    Context {
      now: val.0,
      global: Some(val.1),
      this: Some(val.2),
    }
  }
}
impl<'a> Context<'a> {
  pub fn now(&self) -> &RefCell<Scope> {
    &self.now
  }
  pub fn global(&self) -> &RefCell<Scope> {
    if let Some(val) = self.global {
      val
    } else {
      &self.now
    }
  }
  pub fn this(&self) -> &RefCell<Var> {
    if let Some(val) = self.this {
      val
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
pub struct Handler<'a> {
  pub context: Context<'a>,
  pub cmd: TableType,
  pub next: NextVal,
  pub native: Native,
}
pub enum LazyRef<'a> {
  Value(&'a mut Var),
  Array((&'a mut Vec<Var>, usize)),
  Object((&'a mut BTreeMap<String, Var>, String)),
  ScopeVar((&'a RefCell<Scope>, String)),
  Scope(&'a RefCell<Scope>),
}
impl LazyRef<'_> {
  pub fn create(&mut self) {
    match self {
      LazyRef::Value(_) => (),
      LazyRef::Array((val, index)) => {
        if *index >= val.len() {
          (*val).resize(*index + 1, Var::new());
        }
      }
      LazyRef::Object((val, index)) => {
        if !val.contains_key(index) {
          val.insert(index.clone(), Var::new());
        }
      }
      LazyRef::ScopeVar((val, index)) => {
        if let None = val.borrow().get_mut(index).0 {
          val.borrow_mut().set(index.clone(), (Var::new(), false));
        }
      }
      LazyRef::Scope(_) => (),
    }
  }
  pub fn get_mut(&mut self) -> Option<&mut Var> {
    match self {
      LazyRef::Value(val) => Some(val),
      LazyRef::Array((val, index)) => val.get_mut(*index),
      LazyRef::Object((val, index)) => val.get_mut(index),
      LazyRef::ScopeVar((val, index)) => val.borrow().get_mut(index).0,
      LazyRef::Scope(val) => Some(val.borrow().raw().get_mut()),
    }
  }
  pub fn get(&self) -> Option<&Var> {
    match self {
      LazyRef::Value(val) => Some(val),
      LazyRef::Array((val, index)) => val.get(*index),
      LazyRef::Object((val, index)) => val.get(index),
      LazyRef::ScopeVar((val, index)) => val.borrow().get(index).0,
      LazyRef::Scope(val) => Some(&val.borrow().raw().borrow()),
    }
  }
}
impl<'a> From<&'a mut Var> for LazyRef<'a> {
  fn from(val: &'a mut Var) -> Self {
    LazyRef::Value(val)
  }
}
impl<'a> From<(&'a mut Vec<Var>, usize)> for LazyRef<'a> {
  fn from(val: (&'a mut Vec<Var>, usize)) -> Self {
    LazyRef::Array(val)
  }
}
impl<'a> From<(&'a mut BTreeMap<String, Var>, String)> for LazyRef<'a> {
  fn from(val: (&'a mut BTreeMap<String, Var>, String)) -> Self {
    LazyRef::Object(val)
  }
}
impl<'a> From<&'a RefCell<Scope>> for LazyRef<'a> {
  fn from(val: &'a RefCell<Scope>) -> Self {
    LazyRef::Scope(val)
  }
}
impl<'a> From<(&'a RefCell<Scope>, String)> for LazyRef<'a> {
  fn from(val: (&'a RefCell<Scope>, String)) -> Self {
    LazyRef::ScopeVar(val)
  }
}
pub enum RefObj<'a> {
  Value(Var),
  Ref(LazyRef<'a>),
  Overloaded((Var, LazyRef<'a>)),
}
pub struct ResultObj<'a> {
  val: RefObj<'a>,
  pr: Option<RefObj<'a>>,
}
impl ResultObj<'_> {
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
impl Handler<'_> {
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
impl Handler<'_> {
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
  fn update_scope(now_scope: &Scope, temp_scope: &Scope) -> Scope {
    let mut ret = now_scope.clone();
    if let Var::Object(now) = *now_scope.raw().borrow() {
      for key in now.keys() {
        if let (Some(v), c) = temp_scope.get(key) {
          ret.set(key.clone(), (v.clone(), c))
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
          ptr::eq(
            s as *const Var,
            &*self.context.this().borrow() as *const Var,
          )
        } else {
          false
        }
      } else if let RefObj::Overloaded(v) = start.val {
        if let Some(s) = v.1.get() {
          std::ptr::eq(
            s as *const Var,
            &*self.context.this().borrow() as *const Var,
          )
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
          val: RefObj::Ref(LazyRef::Value(self.context.this().get_mut())),
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
impl<'a> From<(Context<'a>, TableType, NextVal, Native)> for Handler<'a> {
  fn from(val: (Context<'a>, TableType, NextVal, Native)) -> Self {
    Handler {
      context: val.0,
      cmd: val.1,
      next: val.2,
      native: val.3,
    }
  }
}
impl Context<'_> {
  pub fn test(&mut self) -> Option<RefObj> {
    let a = self.global().borrow().get_mut(&String::from("awa"));
    if let Some(val) = a.0 {
      Some(RefObj::Ref(LazyRef::Value(val)))
    } else {
      None
    }
  }
}
