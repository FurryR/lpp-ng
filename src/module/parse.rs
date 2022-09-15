use utf8_slice;
#[derive(PartialEq)]
pub enum QuoteStatus {
  None,
  Single,
  Double,
}
pub struct LppStatus {
  pub quote: QuoteStatus,
  pub splash: bool,
  pub brace: usize,
}
impl LppStatus {
  pub fn new() -> Self {
    LppStatus {
      quote: QuoteStatus::None,
      splash: false,
      brace: 0,
    }
  }
}
pub fn transfer(nowchar: char, status: &mut LppStatus) {
  if nowchar == '\\' {
    status.splash = !status.splash;
  } else if nowchar == '\'' && !status.splash {
    if status.quote == QuoteStatus::None || status.quote == QuoteStatus::Single {
      status.quote = if status.quote == QuoteStatus::None {
        QuoteStatus::Single
      } else {
        QuoteStatus::None
      };
    }
  } else if nowchar == '\"' && !status.splash {
    if status.quote == QuoteStatus::None || status.quote == QuoteStatus::Double {
      status.quote = if status.quote == QuoteStatus::None {
        QuoteStatus::Double
      } else {
        QuoteStatus::None
      };
    }
  } else {
    status.splash = false;
  }
  if (nowchar == '(' || nowchar == '{' || nowchar == '[') && status.quote == QuoteStatus::None {
    status.brace += 1;
  } else if (nowchar == ')' || nowchar == '}' || nowchar == ']')
    && status.quote == QuoteStatus::None
  {
    status.brace -= 1;
  }
}
pub fn transfer_rev(nowchar: char, lastchar: char, status: &mut LppStatus) {
  if lastchar == '\\' {
    status.splash = !status.splash;
  } else if nowchar == '\'' && !status.splash {
    if status.quote == QuoteStatus::None || status.quote == QuoteStatus::Single {
      status.quote = if status.quote == QuoteStatus::None {
        QuoteStatus::Single
      } else {
        QuoteStatus::None
      };
    }
  } else if nowchar == '\"' && !status.splash {
    if status.quote == QuoteStatus::None || status.quote == QuoteStatus::Double {
      status.quote = if status.quote == QuoteStatus::None {
        QuoteStatus::Double
      } else {
        QuoteStatus::None
      };
    }
  } else {
    status.splash = false;
  }
  if (nowchar == '(' || nowchar == '{' || nowchar == '[') && status.quote == QuoteStatus::None {
    status.brace -= 1;
  } else if (nowchar == ')' || nowchar == '}' || nowchar == ']')
    && status.quote == QuoteStatus::None
  {
    status.brace += 1;
  }
}
#[derive(Debug)]
pub struct Lpp {
  pub name: String,
  pub args: String,
}
impl Lpp {
  pub fn new() -> Self {
    Lpp {
      name: String::new(),
      args: String::new(),
    }
  }
}
impl From<(String, String)> for Lpp {
  fn from(val: (String, String)) -> Self {
    Lpp {
      name: val.0,
      args: val.1,
    }
  }
}
impl ToString for Lpp {
  fn to_string(&self) -> String {
    format!(
      "{}{}{}",
      self.name,
      if (utf8_slice::len(self.args.as_str()) == 0 && self.name != "")
        || (utf8_slice::len(self.args.as_str()) != 0 && self.args.starts_with('('))
        || utf8_slice::len(self.name.as_str()) == 0
      {
        ""
      } else {
        " "
      },
      self.args
    )
  }
}
impl Lpp {
  pub fn parse(str: &str) -> Self {
    let mut status = LppStatus::new();
    for (i, item) in str.chars().enumerate() {
      transfer(item, &mut status);
      if item == '\n' || item == '\t' {
        continue;
      }
      if item == '(' && status.quote == QuoteStatus::None && status.brace == 1 {
        break;
      }
      if item == ' ' && status.quote == QuoteStatus::None && status.brace == 0 {
        return Lpp::from((
          utf8_slice::slice(str, 0, i).to_string(),
          utf8_slice::slice(str, i + 1, utf8_slice::len(str)).to_string(),
        ));
      }
    }
    let mut lastchar = '\0';
    status = LppStatus::new();
    for (i, item) in str.chars().rev().enumerate() {
      transfer_rev(item, lastchar, &mut status);
      if item == '\n' || item == '\t' {
        continue;
      }
      if (item == '{' || item == '(') && status.quote == QuoteStatus::None && status.brace == 0 {
        if item != '{' || lastchar != ')' {
          return Lpp::from((
            utf8_slice::slice(str, 0, utf8_slice::len(str) - i - 1).to_string(),
            utf8_slice::slice(str, utf8_slice::len(str) - i - 1, utf8_slice::len(str)).to_string(),
          ));
        }
      }
      lastchar = item;
    }
    Lpp::from((str.to_string(), String::new()))
  }
}
