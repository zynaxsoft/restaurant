use std::{fmt::Display, str::FromStr};

use anyhow::{anyhow, bail, Result};

pub type MenuName = String;
type Quantity = i64;
type TableId = usize;

trait MySplit<'a> {
    fn my_split<'b>(&'a self, p: &'b str) -> (Option<&'a str>, Option<&'a str>);
}

impl<'a> MySplit<'a> for &'a str {
    fn my_split<'b>(&'a self, p: &'b str) -> (Option<&'a str>, Option<&'a str>) {
        let mut iter = self.splitn(2, p);
        (iter.next(), iter.next())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    New,
    Cancel,
    Check,
    Yeet,
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Command::*;
        match s.trim() {
            "new order" => Ok(New),
            "cancel" => Ok(Cancel),
            "check" => Ok(Check),
            "yeet" => Ok(Yeet),
            c => Err(anyhow!("Unknown command: {}", c)),
        }
    }
}

pub enum Param {
    MenuQuantities(Vec<(MenuName, Quantity)>),
    Menu(Vec<MenuName>),
}

fn get_menu_quant(s: &str) -> Result<Param> {
    let menu_quant: Option<Vec<_>> = s
        .trim()
        .split(',')
        // [.., .., ..]
        .map(|e| e.trim())
        // [Some(m, q), Some(m, q), ...] or [None, None, ...]
        .map(|e| e.split_once('*'))
        .collect();

    if let Some(menu_quant) = menu_quant {
        let menu_quant: Result<Vec<_>> = menu_quant
            .into_iter()
            .map(|(m, q)| match q.trim().parse() {
                Ok(q) => Ok((m.trim().into(), q)),
                Err(_) => Err(anyhow!("error parsing number {}", q)),
            })
            .collect();
        return menu_quant.map(|e| Param::MenuQuantities(e));
    }
    bail!("Inconsistent parameters: some parameter doesn't form a menu * quantity pair.");
}

fn get_menu(s: &str) -> Result<Param> {
    let menu: Vec<_> = s.trim().split(',').map(|e| e.trim().into()).collect();
    Ok(Param::Menu(menu))
}

impl FromStr for Param {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            bail!("Parameter must not be an empty string.");
        }
        // Input: menu, menu, menu...
        // Input: menu * quant, menu * quant, menu * quant,...
        if s.contains('*') {
            get_menu_quant(s)
        } else {
            get_menu(s)
        }
    }
}

impl Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Param::MenuQuantities(v) => {
                let mut iter = v.iter().peekable();
                while let Some((m, q)) = iter.next() {
                    write!(f, "{} * {}", m, q)?;
                    if !iter.peek().is_none() {
                        write!(f, ", ")?;
                    }
                }
                Ok(())
            }
            Param::Menu(v) => {
                let mut iter = v.iter().peekable();
                while let Some(m) = iter.next() {
                    write!(f, "{}", m)?;
                    if !iter.peek().is_none() {
                        write!(f, ", ")?;
                    }
                }
                Ok(())
            }
        }
    }
}

pub struct Toro {
    pub command: Command,
    pub table_id: Option<TableId>,
    pub param: Option<Param>,
}

// Parse left side of ':' which can contain command or table id
fn parse_left(s: Option<&str>) -> Result<(Command, Option<TableId>)> {
    // Input: command for table id
    let s = s.ok_or(anyhow!("Left side must not be empty."))?;
    let s = s.trim();
    let (command, table_id) = s.my_split("for table");
    let command = command.ok_or(anyhow!("empty command"))?.trim().parse()?;
    let table_id = table_id.map(|t| t.trim().parse()).transpose()?;
    Ok((command, table_id))
}

// Parse right side of ':' which can only be parameters
fn parse_right(s: Option<&str>) -> Result<Option<Param>> {
    s.map(|inner| inner.parse()).transpose()
}

impl Toro {
    fn integrity_check(&self) -> Result<()> {
        use Command::*;
        match self.command {
            New => {
                self.table_id.ok_or(anyhow!("new order command needs table id"))?;
                self.param
                    .as_ref()
                    .ok_or(anyhow!("new order command needs parameters"))?;
            }
            Cancel => {
                self.table_id.ok_or(anyhow!("cancel command needs table id"))?;
                self.param
                    .as_ref()
                    .ok_or(anyhow!("cancel command needs parameters"))?;
            }
            Check => {
                self.table_id.ok_or(anyhow!("check command needs table id"))?;
            }
            Yeet => {
                if self.table_id.is_some() || self.param.is_some() {
                    bail!("yeet needs nothing. Just only yeet.");
                }
            }
        };
        Ok(())
    }
    pub fn from_toro_string(input: &str) -> Result<Self> {
        // Input: command for table id: params
        let input = input.trim();
        let (left, right) = input.my_split(":");
        let (command, table_id) = parse_left(left)?;
        let param = parse_right(right)?;
        let toro = Self {
            command,
            table_id,
            param,
        };
        toro.integrity_check()?;
        Ok(toro)
    }

    pub fn to_toro_string(&self) -> String {
        fn inner(toro: &Toro) -> Result<String> {
            use Command::*;
            let result = match toro.command {
                New => format!(
                    "new order for table {}: {}",
                    toro.table_id.ok_or(anyhow!("table id must exist"))?,
                    toro.param.as_ref().ok_or(anyhow!("param must exist"))?
                ),
                Cancel => format!(
                    "cancel for table {}: {}",
                    toro.table_id.ok_or(anyhow!("table id must exist"))?,
                    toro.param.as_ref().ok_or(anyhow!("param must exist"))?
                ),
                Check => {
                    let table_id = toro.table_id.ok_or(anyhow!("table id must exist"))?;
                    match &toro.param {
                        Some(param) => format!("check for table {}: {}", table_id, param),
                        None => format!("check for table {}", table_id),
                    }
                }
                Yeet => "yeet".into(),
            };
            Ok(result)
        }
        inner(self).expect("how did you messed this up?")
    }
}

impl From<Toro> for String {
    fn from(toro: Toro) -> Self {
        toro.to_toro_string()
    }
}

impl FromStr for Toro {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Toro::from_toro_string(s)
    }
}

#[cfg(test)]
mod tests {
    use super::Command::*;
    use super::Param::*;
    use super::*;

    const NEW_ORDER: &str = "new order for table 1: a * 1, b * 2, c c c * 3";
    const NEW_ORDER2: &str = "  new order  for table 1  :a* 1, b*2, c c c* 3";
    const CANCEL: &str = "cancel for table 1: a * 1, b * 2";
    const CHECK: &str = "check for table 1: a, b, c";
    const CHECK_ALL: &str = "check for table 1";
    const YEET: &str = "yeet";
    const KRANGLED: &str = "what is this for table something: oh a semicolon;";

    #[test]
    fn test_bad_string() {
        assert!(Toro::from_toro_string(KRANGLED).is_err());
        assert!(Toro::from_toro_string("asdfasdfjklasjdflkjaskldjfkljasjdkf").is_err());
        assert!(Toro::from_toro_string("").is_err());
    }

    #[test]
    fn test_command_tokens() {
        assert!(matches!(
            Toro::from_toro_string(NEW_ORDER).unwrap().command,
            New
        ));
        assert!(matches!(
            Toro::from_toro_string(NEW_ORDER2).unwrap().command,
            New
        ));
        assert!(matches!(
            Toro::from_toro_string(CANCEL).unwrap().command,
            Cancel
        ));
        assert!(matches!(
            Toro::from_toro_string(CHECK).unwrap().command,
            Check
        ));
        assert!(matches!(
            Toro::from_toro_string(CHECK_ALL).unwrap().command,
            Check
        ));
        assert!(matches!(
            Toro::from_toro_string(YEET).unwrap().command,
            Yeet
        ));
    }

    #[test]
    fn test_param_tokens() {
        let _expected = MenuQuantities(vec![("a".into(), 1), ("b".into(), 2), ("c c c".into(), 3)]);
        assert!(matches!(
            Toro::from_toro_string(NEW_ORDER).unwrap().param.unwrap(),
            _expected
        ));
        assert!(matches!(
            Toro::from_toro_string(NEW_ORDER2).unwrap().param.unwrap(),
            _expected
        ));
        let _expected = Menu(vec!["a".into(), "b".into(), "c".into()]);
        assert!(matches!(
            Toro::from_toro_string(CHECK).unwrap().param.unwrap(),
            _expected
        ));
        assert!(Toro::from_toro_string(CHECK_ALL).unwrap().param.is_none());
        assert!(Toro::from_toro_string(YEET).unwrap().param.is_none());
    }

    #[test]
    fn test_serde() {
        let de_str = Toro::from_toro_string(NEW_ORDER).unwrap().to_toro_string();
        assert_eq!(NEW_ORDER, de_str);
        let de_str = Toro::from_toro_string(CANCEL).unwrap().to_toro_string();
        assert_eq!(CANCEL, de_str);
        let de_str = Toro::from_toro_string(CHECK).unwrap().to_toro_string();
        assert_eq!(CHECK, de_str);
        let de_str = Toro::from_toro_string(CHECK_ALL).unwrap().to_toro_string();
        assert_eq!(CHECK_ALL, de_str);
        let de_str = Toro::from_toro_string(YEET).unwrap().to_toro_string();
        assert_eq!(YEET, de_str);
    }

    #[test]
    fn test_integrity_check() {
        assert!(Toro::from_toro_string("check: name").is_err());
        assert!(Toro::from_toro_string("new order: name").is_err());
        assert!(Toro::from_toro_string("cancel: name").is_err());
        assert!(Toro::from_toro_string("yeet for table 1: name").is_err());
    }
}
