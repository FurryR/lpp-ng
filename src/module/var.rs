use super::error::Error;
use super::parse::{transfer, LppStatus, QuoteStatus};
use parse_int;
use std::collections::BTreeMap;
use std::i32;
pub enum RawValue {
  Null(()),
  Boolean(bool),
  Number(f64),
  String(String),
  Array(Vec<Var>),
  Object(BTreeMap<String, Var>),
  Function(FuncValue),
  Statement(StmtValue),
  Expression(ExprValue),
}
#[derive(PartialEq)]
pub struct StmtValue {
  pub value: String,
}
impl StmtValue {
  pub fn new(value: String) -> StmtValue {
    StmtValue { value }
  }
  pub fn from(str: &str) -> StmtValue {
    StmtValue {
      value: utf8_slice::slice(str, 1, utf8_slice::len(str) - 1).to_string(),
    }
  }
  pub fn to_string(&self) -> String {
    format!("{{{}}}", self.value)
  }
}
#[derive(PartialEq)]
pub struct ArgItem {
  name: String,
  value: String,
}
impl ArgItem {
  pub fn new(name: String, value: String) -> ArgItem {
    ArgItem { name, value }
  }
  pub fn from(str: &str) -> ArgItem {
    let mut status: LppStatus = LppStatus::new();
    for (index, item) in str.chars().enumerate() {
      transfer(item, &mut status);
      if item == '=' && status.brace == 0 && status.quote == QuoteStatus::None {
        return ArgItem {
          name: utf8_slice::slice(str, 0, index).to_string(),
          value: utf8_slice::slice(str, 0, utf8_slice::len(str)).to_string(),
        };
      }
    }
    return ArgItem {
      name: str.to_string(),
      value: String::new(),
    };
  }
  pub fn to_string(&self) -> String {
    if self.value != "" {
      format!("{}={}", self.name, self.value)
    } else {
      self.name.clone()
    }
  }
}
#[derive(PartialEq)]
pub struct FuncValue {
  pub args: Vec<ArgItem>,
  pub value: StmtValue,
}
impl FuncValue {
  pub fn new(args: Vec<ArgItem>, value: StmtValue) -> FuncValue {
    FuncValue { args, value }
  }
  pub fn from(str: &str) -> Result<FuncValue, Error> {
    let brace = str.find('(');
    match brace {
      Some(idx) => {
        let mut arg: Vec<ArgItem> = vec![];
        let mut temp: String = String::new();
        let mut status: LppStatus = LppStatus::new();
        let mut nowindex: usize = 0;
        for (index, item) in utf8_slice::slice(str, idx + 1, utf8_slice::len(str))
          .chars()
          .enumerate()
        {
          nowindex = idx + 1 + index;
          if item == ')' && status.brace == 0 && status.quote == QuoteStatus::None {
            break;
          }
          transfer(item, &mut status);
          if item == ',' && status.brace == 0 && status.quote == QuoteStatus::None {
            arg.push(ArgItem::from(temp.as_str()));
            temp.clear();
          } else {
            temp.push(item);
          }
        }
        if temp != "" {
          arg.push(ArgItem::from(temp.as_str()));
        }
        nowindex += 1;
        if str.chars().nth(nowindex) != Some('{') {
          return Err(Error::new(String::from("Syntax error")));
        }
        temp.clear();
        status = LppStatus::new();
        for item in utf8_slice::slice(str, nowindex, utf8_slice::len(str)).chars() {
          transfer(item, &mut status);
          temp.push(item);
          if item == '}' && status.brace == 0 && status.quote == QuoteStatus::None {
            break;
          }
        }
        return Ok(FuncValue {
          args: arg,
          value: StmtValue::from(temp.as_str()),
        });
      }
      None => Err(Error::new(String::from("Syntax error"))),
    }
  }
}
pub fn covered_with(str: &str, left: char, right: char) -> bool {
  if utf8_slice::len(str) < 2
    || str.chars().nth(0) != Some(left)
    || str.chars().nth_back(0) != Some(right)
  {
    return false;
  }
  let mut status = LppStatus::new();
  for (index, item) in str.chars().enumerate() {
    transfer(item, &mut status);
    if item == right && status.quote == QuoteStatus::None && status.brace == 0 {
      return if index != utf8_slice::len(str) - 1 {
        false
      } else {
        true
      };
    }
  }
  return false;
}
pub fn split_by(str: &str, delim: char) -> Vec<String> {
  let mut ret: Vec<String> = vec![];
  let mut tmp = String::new();
  let mut status = LppStatus::new();
  for item in str.chars() {
    transfer(item, &mut status);
    if item == delim && status.quote == QuoteStatus::None && status.brace == 0 {
      ret.push(tmp);
      tmp = String::new();
    } else {
      tmp.push(item);
    }
  }
  return ret;
}
pub fn clearnull(str: &str) -> String {
  let mut tmp = String::new();
  let mut status = LppStatus::new();
  for (index, item) in str.chars().enumerate() {
    transfer(item, &mut status);
    if item == '\r' {
      continue;
    }
    if item == '\t' && status.quote == QuoteStatus::None {
      continue;
    }
    if item == '\n' && status.quote == QuoteStatus::None && status.brace == 0 {
      continue;
    }
    if item == '\n' && status.quote == QuoteStatus::None && status.brace == 0 {
      let next = str.chars().nth(index + 1);
      match next {
        Some(val) => {
          if val == '{' || val == '[' || val == '(' || val == ' ' {
            continue;
          }
        }
        None => {
          continue;
        }
      }
    }
    tmp.push(item);
  }
  return tmp;
}
pub enum NodeValue {
  // val,l,r
  Expr((String, String, String)),
  Val(String),
}
pub struct ExprValue {
  pub val: NodeValue,
}
impl ExprValue {
  pub fn getprio(op: &str, front: bool) -> i32 {
    match op {
      "," => 0,
      "=" => 1,
      "+=" => 1,
      "*=" => 1,
      "/=" => 1,
      "%=" => 1,
      "|=" => 1,
      "&=" => 1,
      "^=" => 1,
      ">>=" => 1,
      ">>>==" => 1,
      "<<=" => 1,
      ":" => 2,
      "?" => 2,
      "||" => 3,
      "&&" => 4,
      "|" => 5,
      "^" => 6,
      "&" => 7,
      "==" => 8,
      "!=" => 8,
      "===" => 8,
      "!==" => 8,
      "<" => 9,
      "<=" => 9,
      ">" => 9,
      ">=" => 9,
      "<<" => 10,
      ">>" => 10,
      ">>>" => 10,
      "+" if !front => 11,
      "-" if !front => 11,
      "*" => 12,
      "/" => 12,
      "%" => 12,
      "~" => 13,
      "!" => 13,
      "++" if front => 13,
      "--" if front => 13,
      "+" if front => 13,
      "-" if front => 13,
      "++" if !front => 14,
      "--" if !front => 14,
      _ => -1,
    }
  }
  pub fn from(str: &str) -> Result<ExprValue, Error> {
    let mut status = LppStatus::new();
    let mut opindex: usize = 0;
    let mut opend: usize = 0;
    let mut temp = String::new();
    let mut front = true;
    let mut minpr = i32::MAX;
    for (index, item) in str.chars().enumerate() {
      if status.brace == 0 && status.quote == QuoteStatus::None {
        if ExprValue::getprio(format!("{}{}", temp, item).as_str(), front) == -1 {
          let c = ExprValue::getprio(temp.as_str(), front);
          if c != -1 {
            if c < minpr {
              minpr = c;
              opend = index;
              opindex = index - utf8_slice::len(temp.as_str());
            }
            temp.clear()
          }
        }
        if ExprValue::getprio(format!("{}{}", temp, item).as_str(), front) != -1 {
          temp.push(item);
        } else {
          front = false;
        }
      }
      transfer(item, &mut status);
    }
    let c = ExprValue::getprio(temp.as_str(), front);
    if c != -1 && c < minpr {
      minpr = c;
      opend = utf8_slice::len(str);
      opindex = utf8_slice::len(str) - utf8_slice::len(temp.as_str());
    }
    if status.brace != 0 || status.quote != QuoteStatus::None {
      return Err(Error::new(String::from("Invalid expression")));
    }
    if minpr == i32::MAX {
      return Ok(ExprValue::new(str.to_string()));
    }
    return Ok(ExprValue::new((
      utf8_slice::slice(str, opindex, opend).to_string(),
      utf8_slice::slice(str, 0, opindex).to_string(),
      utf8_slice::slice(str, opend + 1, utf8_slice::len(str)).to_string(),
    )));
  }
  pub fn isexp(str: &str) -> bool {
    //TODO:isexp
    if covered_with(str, '(', ')') {
      return true;
    }
    return match ExprValue::from(clearnull(str).as_str()) {
      Ok(p) => {
        if let NodeValue::Expr(_) = p.val {
          true
        } else {
          false
        }
      }
      Err(_) => false,
    };
  }
}
pub trait NewExpr<T> {
  fn new(val: T) -> ExprValue;
}
impl NewExpr<(String, String, String)> for ExprValue {
  fn new(val: (String, String, String)) -> ExprValue {
    ExprValue {
      val: NodeValue::Expr(val),
    }
  }
}
impl NewExpr<String> for ExprValue {
  fn new(val: String) -> ExprValue {
    ExprValue {
      val: NodeValue::Val(val),
    }
  }
}
pub struct Var {
  pub value: RawValue,
}
impl Var {
  pub fn from(str: &str) -> Result<Var, Error> {
    let raw = clearnull(str);
    let p = raw.as_str();
    if p == "" {
      return Ok(Var::new(()));
    } else if !ExprValue::isexp(p) {
      {
        let res = {
          let mut opt: Option<f64> = None;
          let mut flag: bool = true;
          for item in p.chars() {
            if !((item >= '0' && item <= '9')
              || (item >= 'a' && item <= 'f')
              || item == 'o'
              || item == 'b'
              || item == 'x'
              || item == '.'
              || item == 'e')
            {
              flag = false;
              break;
            }
          }
          if flag {
            if p.contains('.') || p.contains('e') {
              opt = if let Ok(val) = parse_int::parse::<f64>(p) {
                Some(val)
              } else {
                None
              };
            } else {
              opt = if let Ok(val) = parse_int::parse::<i64>(p) {
                Some(val as f64)
              } else {
                None
              };
            }
          }
          opt
        };
        if let Some(val) = res {
          return Ok(Var::new(val));
        }
      }
      if p == "null" {
        return Ok(Var::new(()));
      } else if p == "true" || p == "false" {
        return Ok(Var::new(p == "true"));
      } else if covered_with(p, '\'', '\'') || covered_with(p, '"', '"') {
        let tmp = utf8_slice::slice(p, 1, utf8_slice::len(p) - 1);
        let mut ret = String::new();
        let mut skip: usize = 0;
        for (index, item) in tmp.chars().enumerate() {
          if skip > 0 {
            skip -= 1;
          } else if item == '\n' || item == '\r' {
            ()
          } else if item == '\\' {
            if let Some(val) = tmp.chars().nth(index + 1) {
              match val {
                '\n' | '\r' => (),
                '\"' | '\\' | '\'' => ret.push(val),
                't' => ret.push('\t'),
                'r' => ret.push('\r'),
                'n' => ret.push('\n'),
                '0' => ret.push('\0'),
                'u' => {
                  if let Ok(val) =
                    u32::from_str_radix(utf8_slice::slice(tmp, index + 2, index + 6), 16)
                  {
                    if let Some(val) = char::from_u32(val) {
                      ret.push(val);
                      skip = 4;
                    } else {
                      return Err(Error::new(String::from("Invalid unicode character")));
                    }
                  } else {
                    return Err(Error::new(String::from("Invalid unicode character")));
                  }
                }
                _ => {
                  ret.push(val);
                }
              }
            } else {
              return Err(Error::new(String::from("Unexpected end of string")));
            }
          } else {
            ret.push(item);
          }
        }
        return Ok(Var::new(ret));
      } else if p.starts_with("func") && p.chars().nth_back(0) == Some('}') {
        return match FuncValue::from(p) {
          Ok(val) => Ok(Var::new(val)),
          Err(err) => Err(err),
        };
      } else if covered_with(p, '[', ']') {
        let tmp = split_by(utf8_slice::slice(p, 1, utf8_slice::len(p) - 1), ',');
        let mut ret: Vec<Var> = vec![];
        for item in tmp.iter() {
          match Var::from(item.as_str()) {
            Ok(val) => ret.push(val),
            Err(err) => return Err(err),
          }
        }
        return Ok(Var::new(ret));
      } else if covered_with(p, '{', '}') {
        let mut ret: BTreeMap<String, Var> = BTreeMap::new();
        let tmp = split_by(utf8_slice::slice(p, 1, utf8_slice::len(p) - 1), ',');
        for item in tmp.iter() {
          let pair = split_by(item.as_str(), ':');
          if pair.len() != 2 {
            return Ok(Var::new(StmtValue::from(p)));
          }
          match Var::from(pair[0].as_str()) {
            Ok(val) => {
              if let RawValue::String(str) = val.value {
                match Var::from(pair[1].as_str()) {
                  Ok(val) => {
                    ret.insert(str, val);
                  }
                  Err(_) => {
                    return Ok(Var::new(StmtValue::from(p)));
                  }
                }
              } else {
                return Ok(Var::new(StmtValue::from(p)));
              }
            }
            Err(_) => {
              return Ok(Var::new(StmtValue::from(p)));
            }
          }
        }
        return Ok(Var::new(ret));
      }
    }
    let mut exp = p;
    while covered_with(exp, '(', ')') {
      exp = utf8_slice::slice(exp, 1, utf8_slice::len(exp) - 1);
    }
    return match ExprValue::from(exp) {
      Ok(val) => Ok(Var::new(val)),
      Err(err) => Err(err),
    };
  }
}
pub trait NewVar<T> {
  fn new(val: T) -> Var;
}
impl NewVar<()> for Var {
  fn new(_: ()) -> Var {
    return Var {
      value: RawValue::Null(()),
    };
  }
}
impl NewVar<bool> for Var {
  fn new(val: bool) -> Var {
    return Var {
      value: RawValue::Boolean(val),
    };
  }
}
impl NewVar<f64> for Var {
  fn new(val: f64) -> Var {
    return Var {
      value: RawValue::Number(val),
    };
  }
}
impl NewVar<String> for Var {
  fn new(val: String) -> Var {
    return Var {
      value: RawValue::String(val),
    };
  }
}
impl NewVar<Vec<Var>> for Var {
  fn new(val: Vec<Var>) -> Var {
    return Var {
      value: RawValue::Array(val),
    };
  }
}
impl NewVar<BTreeMap<String, Var>> for Var {
  fn new(val: BTreeMap<String, Var>) -> Var {
    return Var {
      value: RawValue::Object(val),
    };
  }
}
impl NewVar<FuncValue> for Var {
  fn new(val: FuncValue) -> Var {
    return Var {
      value: RawValue::Function(val),
    };
  }
}
impl NewVar<StmtValue> for Var {
  fn new(val: StmtValue) -> Var {
    return Var {
      value: RawValue::Statement(val),
    };
  }
}
impl NewVar<ExprValue> for Var {
  fn new(val: ExprValue) -> Var {
    return Var {
      value: RawValue::Expression(val),
    };
  }
}
