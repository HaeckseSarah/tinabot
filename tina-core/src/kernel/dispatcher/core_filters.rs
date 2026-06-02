use super::plugin_filter::PluginFilterMatcher;
use mlua::{Table, Value};
use regex::Regex;
use std::sync::Arc;
use tina_plugin_api::FilterRegistry;

pub struct CoreFilterMatcher;

impl CoreFilterMatcher {
    pub fn eval(
        filters: &Table,
        payload: &Table,
        registry: &Arc<FilterRegistry>,
    ) -> mlua::Result<bool> {
        for pair in filters.pairs::<usize, Table>() {
            let (_, rule) = pair?;

            let field: String = rule.get(1)?;
            let operator: String = rule.get(2)?;
            let filter_val: Value = rule.get(3)?;

            if !payload.contains_key(&*field)? {
                return Ok(false);
            }
            let payload_val: Value = payload.get(&*field)?;

            if operator.contains('.') {
                let plugin_matched =
                    PluginFilterMatcher::eval_rule(&operator, payload_val, filter_val, registry);

                if !plugin_matched {
                    return Ok(false);
                }
                continue;
            }

            if operator.contains('.') {
                continue;
            }

            let matched = match operator.as_str() {
                "=" | "==" | "is" => Self::compare_eq_insensitive(&payload_val, &filter_val)?,
                "===" => Self::compare_eq(&payload_val, &filter_val)?,

                "!=" | "is_not" => !Self::compare_eq_insensitive(&payload_val, &filter_val)?,
                "!==" => !Self::compare_eq(&payload_val, &filter_val)?,

                "^=" | "starts_with" => {
                    if let (Value::String(p), Value::String(f)) = (&payload_val, &filter_val) {
                        p.to_string_lossy()
                            .to_lowercase()
                            .starts_with(&f.to_string_lossy().to_lowercase())
                    } else {
                        false
                    }
                }
                "^==" => {
                    if let (Value::String(p), Value::String(f)) = (&payload_val, &filter_val) {
                        p.to_string_lossy().starts_with(&f.to_string_lossy())
                    } else {
                        false
                    }
                }
                "*=" | "contains" => {
                    if let (Value::String(p), Value::String(f)) = (&payload_val, &filter_val) {
                        p.to_string_lossy()
                            .to_lowercase()
                            .contains(&f.to_string_lossy().to_lowercase())
                    } else {
                        false
                    }
                }
                "*==" => {
                    if let (Value::String(p), Value::String(f)) = (&payload_val, &filter_val) {
                        p.to_string_lossy().contains(&f.to_string_lossy())
                    } else {
                        false
                    }
                }
                "//" | "regex" => {
                    if let (Value::String(p), Value::String(f)) = (&payload_val, &filter_val) {
                        let p_res = p.to_str()?;
                        let f_res = f.to_str()?;
                        // FIX: Deref via &* erzwingen
                        if let Ok(re) = Regex::new(&*f_res) {
                            re.is_match(&*p_res)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }

                ">" | "gt" => Self::compare_num(&payload_val, &filter_val, |p, f| p > f),
                ">=" | "gte" => Self::compare_num(&payload_val, &filter_val, |p, f| p >= f),
                "<" | "lt" => Self::compare_num(&payload_val, &filter_val, |p, f| p < f),
                "<=" | "lte" => Self::compare_num(&payload_val, &filter_val, |p, f| p <= f),

                _ => false,
            };
            println!("matched: {}", matched);
            if !matched {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn compare_eq(a: &Value, b: &Value) -> mlua::Result<bool> {
        match (a, b) {
            (Value::String(s1), Value::String(s2)) => Ok(s1.to_str()? == s2.to_str()?),
            (Value::Integer(i1), Value::Integer(i2)) => Ok(i1 == i2),
            (Value::Boolean(b1), Value::Boolean(b2)) => Ok(b1 == b2),
            _ => Ok(false),
        }
    }

    fn compare_eq_insensitive(a: &Value, b: &Value) -> mlua::Result<bool> {
        match (a, b) {
            (Value::String(s1), Value::String(s2)) => {
                Ok(s1.to_str()?.to_lowercase() == s2.to_str()?.to_lowercase())
            }
            _ => Self::compare_eq(a, b),
        }
    }

    fn compare_num<F>(a: &Value, b: &Value, op: F) -> bool
    where
        F: Fn(i64, i64) -> bool,
    {
        if let (Value::Integer(p), Value::Integer(f)) = (a, b) {
            op(*p, *f)
        } else {
            false
        }
    }
}
