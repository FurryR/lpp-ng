use super::error::Error;
use super::parse::{transfer, LppStatus, QuoteStatus};
use parse_int;
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};
use std::i32;
/// 语句块。
/// 你可以以以下方式定义一个语句块：
/// ```
/// let s = StmtValue::new();
/// assert_eq!(s.value, "");
/// let s2 = StmtValue::from(String::new("awa"));
/// assert_eq!(s2.value, "awa");
/// let s3 = StmtValue::parse("{awa}");
/// assert_eq!(s3.value, "awa");
/// ```
#[derive(PartialEq, Clone)]
pub struct StmtValue {
  /// 语句块的内容。
  /// 保存原始内容（含有空格，分隔符等），需要手动分割。
  pub value: String,
}
impl StmtValue {
  /// 新建一个语句块。
  /// 若需要指定语句块内容，请使用 `StmtValue::from` 而不是 `StmtValue::new`。
  /// ```
  /// let s = StmtValue::new();
  /// assert_eq!(s.value, "");
  /// ```
  pub fn new() -> Self {
    StmtValue {
      value: String::new(),
    }
  }
}
impl From<String> for StmtValue {
  /// 以指定内容新建语句块。
  /// `value`: 语句块的内容。
  /// ```
  /// let s = StmtValue::from(String::from("awa"));
  /// assert_eq!(s.value, "awa");
  /// ```
  fn from(value: String) -> Self {
    StmtValue { value }
  }
}
impl StmtValue {
  /// 对语句块进行反序列化。
  /// `str`: 序列化语句块。
  /// ```
  /// let a = StmtValue::parse("{awa}");
  /// assert_eq!(a.value, "awa");
  /// ```
  fn parse(str: &str) -> Self {
    StmtValue {
      value: utf8_slice::slice(str, 1, utf8_slice::len(str) - 1).to_string(),
    }
  }
}
impl ToString for StmtValue {
  /// 对语句块进行序列化。
  /// ```
  /// let a = StmtValue::parse("{awa}");
  /// assert_eq!(a.to_string(), "{awa}");
  /// ```
  fn to_string(&self) -> String {
    format!("{{{}}}", self.value)
  }
}
/// 单个参数。
/// 参数允许可选值。一旦一个参数为可选参数，则后面的参数都必须为可选参数。
/// ```
/// let a = ArgItem::parse("awa=1");
/// assert_eq!(a.name, "awa");
/// assert_eq!(a.value, "1");
/// ```
#[derive(PartialEq, Clone)]
pub struct ArgItem {
  /// 参数的名字。
  /// 此模块不会对命名进行检查。
  pub name: String,
  /// 参数的内容（可选）。
  /// 没有参数时，`value`应为空字符串。
  pub value: String,
}
impl ArgItem {
  /// 新建参数。
  /// 不推荐使用此方法。请换用 `ArgItem::from` 来新建参数。
  /// ```
  /// let a = ArgItem::new();
  /// assert_eq!(a.name, "");
  /// assert_eq!(a.value, "");
  /// ```
  pub fn new() -> Self {
    ArgItem {
      name: String::new(),
      value: String::new(),
    }
  }
}
impl ToString for ArgItem {
  /// 序列化参数。
  /// 序列化的参数将可以被 `ArgItem::parse` 解析。
  /// ```
  /// let a = ArgItem::from((String::from("awa"),String::from("1")));
  /// assert_eq!(a.to_string(), "awa=1");
  /// ```
  fn to_string(&self) -> String {
    if self.value != "" {
      format!("{}={}", self.name, self.value)
    } else {
      self.name.clone()
    }
  }
}
impl From<(String, String)> for ArgItem {
  /// 由给定的参数名和默认值（如果有）构建参数。
  /// `val.0`: 参数名。
  /// `val.1`: 默认值。
  /// ```
  /// let a = ArgItem::from((String::from("awa"),String::from("1")));
  /// assert_eq!(a.name, "awa");
  /// assert_eq!(a.value, "1");
  /// ```
  fn from(val: (String, String)) -> Self {
    ArgItem {
      name: val.0,
      value: val.1,
    }
  }
}
impl ArgItem {
  /// 反序列化参数。
  /// `str`: 序列化后的参数。
  /// ```
  /// let a = ArgItem::parse("awa=1");
  /// assert_eq!(a.name, "awa");
  /// assert_eq!(a.value, "1");
  /// ```
  pub fn parse(str: &str) -> Self {
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
    ArgItem {
      name: str.to_string(),
      value: String::new(),
    }
  }
}
/// 函数。
/// ```
/// let a = FuncValue::parse("func(){}").unwrap();
/// ```
#[derive(PartialEq, Clone)]
pub struct FuncValue {
  /// 参数列表。
  pub args: Vec<ArgItem>,
  /// 语句块。
  pub value: StmtValue,
}
impl FuncValue {
  /// 创建空的函数。
  /// ```
  /// let s = FuncValue::new();
  /// assert_eq!(s.value.value, "");
  /// ```
  pub fn new() -> Self {
    FuncValue {
      args: vec![],
      value: StmtValue::new(),
    }
  }
}
impl TryFrom<(Vec<ArgItem>, StmtValue)> for FuncValue {
  type Error = Error;
  /// 由参数列表和内容块构造函数。
  /// `val.0`: 参数列表。
  /// `val.1`: 函数体。
  fn try_from(val: (Vec<ArgItem>, StmtValue)) -> Result<Self, Self::Error> {
    let mut flag = false;
    for item in val.0.iter() {
      if item.value == "" && flag {
        return Err(Error::from(String::from("Syntax Error")));
      }
      if item.value != "" {
        flag = true;
      }
    }
    Ok(FuncValue {
      args: val.0,
      value: val.1,
    })
  }
}
impl ToString for FuncValue {
  fn to_string(&self) -> String {
    let mut tmp = String::from("func(");
    for (index, item) in self.args.iter().enumerate() {
      tmp += item.to_string().as_str();
      if index + 1 < self.args.len() {
        tmp.push(',')
      }
    }
    tmp.push(')');
    format!("{}{}", tmp, self.value.to_string())
  }
}
impl FuncValue {
  pub fn parse(str: &str) -> Result<Self, Error> {
    let brace = str.find('(');
    match brace {
      Some(idx) => {
        let mut arg: Vec<ArgItem> = vec![];
        let mut temp: String = String::new();
        let mut status: LppStatus = LppStatus::new();
        let mut nowindex: usize = idx;
        for item in utf8_slice::slice(str, idx + 1, utf8_slice::len(str)).chars() {
          nowindex += 1;
          if item == ')' && status.brace == 0 && status.quote == QuoteStatus::None {
            break;
          }
          transfer(item, &mut status);
          if item == ',' && status.brace == 0 && status.quote == QuoteStatus::None {
            arg.push(ArgItem::parse(temp.as_str()));
            temp.clear();
          } else {
            temp.push(item);
          }
        }
        if temp != "" {
          arg.push(ArgItem::parse(temp.as_str()));
        }
        nowindex += 1;
        if str.chars().nth(nowindex) != Some('{') {
          return Err(Error::from(String::from("Syntax error")));
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
        Ok(FuncValue::try_from((arg, StmtValue::parse(temp.as_str())))?)
      }
      None => Err(Error::from(String::from("Syntax error"))),
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
      return index == utf8_slice::len(str) - 1;
    }
  }
  false
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
  if tmp != "" {
    ret.push(tmp);
  }
  ret
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
#[derive(Clone)]
pub enum ExprValue {
  // val,l,r
  Expr((String, String, String)),
  Val(String),
}
impl ExprValue {
  pub fn new() -> Self {
    ExprValue::Val(String::new())
  }
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
  pub fn isexp(str: &str) -> bool {
    if covered_with(str, '(', ')') {
      return true;
    }
    match ExprValue::parse(clearnull(str).as_str()) {
      Ok(p) => {
        if let ExprValue::Expr(_) = p {
          true
        } else {
          false
        }
      }
      Err(_) => false,
    }
  }
}
impl ExprValue {
  pub fn parse(str: &str) -> Result<Self, Error> {
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
      return Err(Error::from(String::from("Invalid expression")));
    }
    if minpr == i32::MAX {
      return Ok(ExprValue::from(str.to_string()));
    }
    Ok(ExprValue::from((
      utf8_slice::slice(str, opindex, opend).to_string(),
      utf8_slice::slice(str, 0, opindex).to_string(),
      utf8_slice::slice(str, opend + 1, utf8_slice::len(str)).to_string(),
    )))
  }
}
impl From<(String, String, String)> for ExprValue {
  fn from(val: (String, String, String)) -> Self {
    ExprValue::Expr(val)
  }
}
impl From<String> for ExprValue {
  fn from(val: String) -> Self {
    ExprValue::Val(val)
  }
}
#[derive(Clone)]
pub enum Var {
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
pub enum ValueType {
  Null,
  Boolean,
  Number,
  String,
  Array,
  Object,
  Function,
  Statement,
  Expression,
}
// tp
impl Var {
  pub fn tp(&self) -> ValueType {
    match self {
      Var::Null(_) => ValueType::Null,
      Var::Boolean(_) => ValueType::Boolean,
      Var::Number(_) => ValueType::Number,
      Var::String(_) => ValueType::String,
      Var::Array(_) => ValueType::Array,
      Var::Object(_) => ValueType::Object,
      Var::Function(_) => ValueType::Object,
      Var::Statement(_) => ValueType::Statement,
      Var::Expression(_) => ValueType::Expression,
    }
  }
}
// convert
impl Var {
  pub fn convert(self, tp: ValueType) -> Result<Var, Error> {
    match tp {
      ValueType::Null => Ok(Var::Null(TryInto::<()>::try_into(self)?)),
      ValueType::Boolean => Ok(Var::Boolean(TryInto::<bool>::try_into(self)?)),
      ValueType::Number => Ok(Var::Number(TryInto::<f64>::try_into(self)?)),
      ValueType::String => Ok(Var::String(TryInto::<String>::try_into(self)?)),
      ValueType::Array => Ok(Var::Array(TryInto::<Vec<Var>>::try_into(self)?)),
      ValueType::Object => Ok(Var::Object(TryInto::<BTreeMap<String, Var>>::try_into(
        self,
      )?)),
      _ => Err(Error::from(String::from("Conversion failed"))),
    }
  }
}
impl TryFrom<Var> for () {
  type Error = Error;
  fn try_from(val: Var) -> Result<(), Self::Error> {
    match val {
      Var::Null(_) => Ok(()),
      _ => Err(Error::from(String::from("Conversion failed"))),
    }
  }
}
impl TryFrom<Var> for bool {
  type Error = Error;
  fn try_from(val: Var) -> Result<Self, Self::Error> {
    match val {
      Var::Boolean(val) => Ok(val),
      Var::Number(val) => Ok(val != 0.0),
      _ => Err(Error::from(String::from("Conversion failed"))),
    }
  }
}
impl TryFrom<Var> for f64 {
  type Error = Error;
  fn try_from(val: Var) -> Result<Self, Self::Error> {
    match val {
      Var::Number(val) => Ok(val),
      Var::Boolean(val) => Ok(if val { 1.0 } else { 0.0 }),
      _ => Err(Error::from(String::from("Conversion failed"))),
    }
  }
}
impl TryFrom<Var> for String {
  type Error = Error;
  fn try_from(val: Var) -> Result<Self, Self::Error> {
    match val {
      Var::String(val) => Ok(val),
      _ => Ok(val.to_string()),
    }
  }
}
impl TryFrom<Var> for Vec<Var> {
  type Error = Error;
  fn try_from(val: Var) -> Result<Self, Self::Error> {
    match val {
      Var::Array(val) => Ok(val),
      _ => Err(Error::from(String::from("Conversion failed"))),
    }
  }
}
impl TryFrom<Var> for BTreeMap<String, Var> {
  type Error = Error;
  fn try_from(val: Var) -> Result<Self, Self::Error> {
    match val {
      Var::Object(val) => Ok(val),
      _ => Err(Error::from(String::from("Conversion failed"))),
    }
  }
}
// opcall
impl Var {
  pub fn opcall_single(self, op: char) -> Result<Var, Error> {
    match op {
      '~' => Ok(Var::Number(f64::from(
        !((TryInto::<f64>::try_into(self)?) as i32),
      ))),
      '-' => Ok(Var::Number(-(TryInto::<f64>::try_into(self)?))),
      '+' => Ok(Var::Number(TryInto::<f64>::try_into(self)?)),
      '!' => Ok(Var::Boolean(!(TryInto::<bool>::try_into(self)?))),
      _ => Err(Error::from(String::from("Unknown operand"))),
    }
  }
  fn opcmp(&self, op: &str, val: &Var) -> Result<bool, Error> {
    match op {
      "===" | "==" => match self {
        Var::Null(_) => {
          if let Var::Null(_) = val {
            Ok(true)
          } else {
            Ok(false)
          }
        }
        Var::Boolean(left) => {
          if let Var::Boolean(right) = val {
            Ok(left == right)
          } else {
            Ok(false)
          }
        }
        Var::Number(left) => {
          if let Var::Number(right) = val {
            Ok(left == right)
          } else {
            Ok(false)
          }
        }
        Var::String(left) => {
          if let Var::String(right) = val {
            Ok(left == right)
          } else {
            Ok(false)
          }
        }
        Var::Array(left) => {
          if let Var::Array(right) = val {
            if left.len() == right.len() {
              Ok(left.iter().enumerate().all(|(index, item)| {
                if let Ok(val) = item.clone().opcall(op, &right[index]) {
                  if let Var::Boolean(val) = val {
                    val
                  } else {
                    false
                  }
                } else {
                  false
                }
              }))
            } else {
              Ok(false)
            }
          } else {
            Ok(false)
          }
        }
        Var::Object(left) => {
          if let Var::Object(right) = val {
            if left.len() == right.len() {
              Ok(left.iter().all(|(key, value)| {
                if let Some(r_val) = right.get(key) {
                  if let Ok(val) = value.clone().opcall(op, r_val) {
                    if let Var::Boolean(val) = val {
                      val
                    } else {
                      false
                    }
                  } else {
                    false
                  }
                } else {
                  false
                }
              }))
            } else {
              Ok(false)
            }
          } else {
            Ok(false)
          }
        }
        Var::Function(left) => {
          if let Var::Function(right) = val {
            Ok(left == right)
          } else {
            Ok(false)
          }
        }
        _ => Ok(false),
      },
      "!=" | "!==" => Ok(!(self.opcmp(op, val)?)),
      ">" => match self {
        Var::Number(left) => {
          if let Var::Number(right) = val {
            Ok(*left > *right)
          } else {
            Ok(false)
          }
        }
        Var::String(left) => {
          if let Var::String(right) = val {
            Ok(left > right)
          } else {
            Ok(false)
          }
        }
        _ => Ok(false),
      },
      "<" => match self {
        Var::Number(left) => {
          if let Var::Number(right) = val {
            Ok(*left < *right)
          } else {
            Ok(false)
          }
        }
        Var::String(left) => {
          if let Var::String(right) = val {
            Ok(left < right)
          } else {
            Ok(false)
          }
        }
        _ => Ok(false),
      },
      ">=" => Ok(!(self.opcmp("<", val)?)),
      "<=" => Ok(!(self.opcmp(">", val)?)),
      _ => Err(Error::from(String::from("Unknown operand"))),
    }
  }
  pub fn opcall(self, op: &str, val: &Var) -> Result<Var, Error> {
    match op {
      "==" | "!=" | ">=" | "<=" | ">" | "<" => {
        if let Ok(conv) = self.convert(val.tp()) {
          Ok(Var::Boolean(conv.opcmp(op, &val)?))
        } else {
          Ok(Var::Boolean(false))
        }
      }
      "===" | "!==" => Ok(Var::Boolean(self.opcmp(op, &val)?)),
      _ => {
        let conv = self.convert(val.tp())?;
        match op {
          "+" => match conv {
            Var::Number(left) => {
              if let Var::Number(right) = val {
                Ok(Var::Number(left + right))
              } else {
                Err(Error::from(String::from("Calculation failed")))
              }
            }
            Var::String(left) => {
              if let Var::String(right) = val {
                Ok(Var::String(left + right.as_str()))
              } else {
                Err(Error::from(String::from("Calculation failed")))
              }
            }
            Var::Array(left) => {
              if let Var::Array(right) = val {
                let mut s = left.clone();
                s.append(&mut right.clone());
                Ok(Var::Array(s))
              } else {
                Err(Error::from(String::from("Calculation failed")))
              }
            }
            _ => Err(Error::from(String::from("Calculation failed"))),
          },
          "-" => {
            if let Var::Number(left) = conv {
              if let Var::Number(right) = val {
                Ok(Var::Number(left - right))
              } else {
                Err(Error::from(String::from("Calculation failed")))
              }
            } else {
              Err(Error::from(String::from("Calculation failed")))
            }
          }
          "*" => {
            if let Var::Number(left) = conv {
              if let Var::Number(right) = val {
                Ok(Var::Number(left * right))
              } else {
                Err(Error::from(String::from("Calculation failed")))
              }
            } else {
              Err(Error::from(String::from("Calculation failed")))
            }
          }
          "/" => {
            if let Var::Number(left) = conv {
              if let Var::Number(right) = val {
                if *right != 0.0 {
                  Ok(Var::Number(left / right))
                } else {
                  Ok(Var::Number(f64::NAN))
                }
              } else {
                Err(Error::from(String::from("Calculation failed")))
              }
            } else {
              Err(Error::from(String::from("Calculation failed")))
            }
          }
          "%" => {
            if let Var::Number(left) = conv {
              if let Var::Number(right) = val {
                if *right != 0.0 {
                  Ok(Var::Number(left % right))
                } else {
                  Ok(Var::Number(f64::NAN))
                }
              } else {
                Err(Error::from(String::from("Calculation failed")))
              }
            } else {
              Err(Error::from(String::from("Calculation failed")))
            }
          }
          "&" => {
            if let Var::Number(left) = conv {
              if let Var::Number(right) = val {
                Ok(Var::Number(((left as i32) & (*right as i32)) as f64))
              } else {
                Err(Error::from(String::from("Calculation failed")))
              }
            } else {
              Err(Error::from(String::from("Calculation failed")))
            }
          }
          "|" => {
            if let Var::Number(left) = conv {
              if let Var::Number(right) = val {
                Ok(Var::Number(((left as i32) | (*right as i32)) as f64))
              } else {
                Err(Error::from(String::from("Calculation failed")))
              }
            } else {
              Err(Error::from(String::from("Calculation failed")))
            }
          }
          "^" => {
            if let Var::Number(left) = conv {
              if let Var::Number(right) = val {
                Ok(Var::Number(((left as i32) ^ (*right as i32)) as f64))
              } else {
                Err(Error::from(String::from("Calculation failed")))
              }
            } else {
              Err(Error::from(String::from("Calculation failed")))
            }
          }
          "<<" => {
            if let Var::Number(left) = conv {
              if let Var::Number(right) = val {
                Ok(Var::Number(((left as i32) << (*right as i32)) as f64))
              } else {
                Err(Error::from(String::from("Calculation failed")))
              }
            } else {
              Err(Error::from(String::from("Calculation failed")))
            }
          }
          ">>" => {
            if let Var::Number(left) = conv {
              if let Var::Number(right) = val {
                Ok(Var::Number(((left as i32) >> (*right as i32)) as f64))
              } else {
                Err(Error::from(String::from("Calculation failed")))
              }
            } else {
              Err(Error::from(String::from("Calculation failed")))
            }
          }
          "<<<" => {
            if let Var::Number(left) = conv {
              if let Var::Number(right) = val {
                Ok(Var::Number(((left as u32) << (*right as u32)) as f64))
              } else {
                Err(Error::from(String::from("Calculation failed")))
              }
            } else {
              Err(Error::from(String::from("Calculation failed")))
            }
          }
          ">>>" => {
            if let Var::Number(left) = conv {
              if let Var::Number(right) = val {
                Ok(Var::Number(((left as u32) >> (*right as u32)) as f64))
              } else {
                Err(Error::from(String::from("Calculation failed")))
              }
            } else {
              Err(Error::from(String::from("Calculation failed")))
            }
          }
          _ => Err(Error::from(String::from("Unknown operand"))),
        }
      }
    }
  }
}
// from
impl Var {
  pub fn new() -> Self {
    Var::Null(())
  }
  pub fn parse(str: &str) -> Result<Self, Error> {
    let raw = clearnull(str);
    let p = raw.as_str();
    if p == "" {
      return Ok(Var::new());
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
              opt = if let Ok(val) = parse_int::parse::<i32>(p) {
                Some(val as f64)
              } else {
                None
              };
            }
          }
          opt
        };
        if let Some(val) = res {
          return Ok(Var::Number(val));
        }
      }
      if p == "null" {
        return Ok(Var::new());
      } else if p == "true" || p == "false" {
        return Ok(Var::Boolean(p == "true"));
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
                      return Err(Error::from(String::from("Invalid unicode character")));
                    }
                  } else {
                    return Err(Error::from(String::from("Invalid unicode character")));
                  }
                }
                _ => {
                  ret.push(val);
                }
              }
            } else {
              return Err(Error::from(String::from("Unexpected end of string")));
            }
          } else {
            ret.push(item);
          }
        }
        return Ok(Var::String(ret));
      } else if p.starts_with("func") && p.chars().nth_back(0) == Some('}') {
        return Ok(Var::Function(FuncValue::parse(p)?));
      } else if covered_with(p, '[', ']') {
        let tmp = split_by(utf8_slice::slice(p, 1, utf8_slice::len(p) - 1), ',');
        let mut ret: Vec<Var> = vec![];
        for item in tmp.iter() {
          ret.push(Var::parse(item.as_str())?);
        }
        return Ok(Var::Array(ret));
      } else if covered_with(p, '{', '}') {
        let mut ret: BTreeMap<String, Var> = BTreeMap::new();
        let tmp = split_by(utf8_slice::slice(p, 1, utf8_slice::len(p) - 1), ',');
        for item in tmp.iter() {
          let pair = split_by(item.as_str(), ':');
          if pair.len() != 2 {
            return Ok(Var::Statement(StmtValue::parse(p)));
          }
          match Var::parse(pair[0].as_str()) {
            Ok(val) => {
              if let Var::String(str) = val {
                match Var::parse(pair[1].as_str()) {
                  Ok(val) => {
                    ret.insert(str, val);
                  }
                  Err(_) => {
                    return Ok(Var::Statement(StmtValue::parse(p)));
                  }
                }
              } else {
                return Ok(Var::Statement(StmtValue::parse(p)));
              }
            }
            Err(_) => {
              return Ok(Var::Statement(StmtValue::parse(p)));
            }
          }
        }
        return Ok(Var::Object(ret));
      }
    }
    let mut exp = p;
    while covered_with(exp, '(', ')') {
      exp = utf8_slice::slice(exp, 1, utf8_slice::len(exp) - 1);
    }
    Ok(Var::Expression(ExprValue::parse(exp)?))
  }
}
impl ToString for Var {
  fn to_string(&self) -> String {
    match self {
      Var::Null(_) => String::from("null"),
      Var::Boolean(val) => {
        if *val {
          String::from("true")
        } else {
          String::from("false")
        }
      }
      Var::Number(val) => val.to_string(),
      Var::String(val) => {
        let mut tmp = String::from("\"");
        for item in val.chars() {
          match item {
            '\r' => tmp += "\\r",
            '\t' => tmp += "\\t",
            '\0' => tmp += "\\0",
            '\n' => tmp += "\\n",
            '\\' | '\'' | '"' => tmp += format!("\\{}", item).as_str(),
            _ => tmp.push(item),
          };
        }
        tmp + "\""
      }
      Var::Array(val) => {
        let mut tmp = String::from("[");
        for (index, item) in val.iter().enumerate() {
          tmp += item.to_string().as_str();
          if index + 1 < val.len() {
            tmp.push(',');
          }
        }
        tmp + "]"
      }
      Var::Object(val) => {
        let mut tmp = String::from("{");
        for (index, (key, value)) in val.iter().enumerate() {
          tmp += format!(
            "{}:{}",
            Var::String(key.clone()).to_string(),
            value.to_string()
          )
          .as_str();
          if index + 1 < val.len() {
            tmp.push(',');
          }
        }
        tmp + "}"
      }
      Var::Function(val) => val.to_string(),
      _ => String::from("<error-type>"),
    }
  }
}
