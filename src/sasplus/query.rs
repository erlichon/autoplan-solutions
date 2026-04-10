use nom::character::complete::{line_ending, not_line_ending};
use nom::combinator::opt;
use nom::{IResult, Parser};

use super::{SASPlus, State};

/// Parse a line, tolerating a missing trailing newline (for the last line of input).
fn flex_line(s: &str) -> IResult<&str, &str> {
    let (s, content) = not_line_ending(s)?;
    let (s, _) = opt(line_ending).parse(s)?;
    Ok((s, content))
}

fn flex_int_line(s: &str) -> IResult<&str, i64> {
    let (s, l) = flex_line(s)?;
    let n: i64 = l
        .trim()
        .parse()
        .map_err(|_| nom::Err::Failure(nom::error::Error::new(s, nom::error::ErrorKind::Digit)))?;
    Ok((s, n))
}

fn format_var_val_pairs(pairs: &mut Vec<(usize, usize)>, vars: &[super::Variable]) -> String {
    pairs.sort_by_key(|&(var, _)| var);
    let mut out = String::new();
    for &(var, val) in pairs.iter() {
        out.push_str(&format!("{} {}\n", var, vars[var].values[val]));
    }
    out
}

impl SASPlus {
    pub fn query<'a>(&self, state: &State, s: &'a str) -> IResult<&'a str, String> {
        let (s, command) = flex_line(s)?;

        match command.trim() {
            "variable" => {
                let (s, args) = flex_line(s)?;
                let mut parts = args.trim().split_whitespace();
                let var: usize = parts.next().unwrap().parse().unwrap();
                let val: usize = parts.next().unwrap().parse().unwrap();
                Ok((s, format!("{}\n", self.variables[var].values[val])))
            }
            "state" => {
                let mut out = String::new();
                for (i, &val) in state.values.iter().enumerate() {
                    out.push_str(&self.variables[i].values[val]);
                    out.push('\n');
                }
                Ok((s, out))
            }
            "goal" => {
                let mut pairs: Vec<(usize, usize)> = self.goal.clone();
                Ok((s, format_var_val_pairs(&mut pairs, &self.variables)))
            }
            "actionname" => {
                let (s, action) = flex_int_line(s)?;
                Ok((s, format!("{}\n", self.operators[action as usize].name)))
            }
            "actioncost" => {
                let (s, action) = flex_int_line(s)?;
                Ok((s, format!("{}\n", self.operators[action as usize].cost)))
            }
            "precondition" => {
                let (s, action) = flex_int_line(s)?;
                let op = &self.operators[action as usize];
                let mut pairs: Vec<(usize, usize)> = op.prevail.clone();
                for eff in &op.effects {
                    if eff.pre_value >= 0 {
                        pairs.push((eff.variable, eff.pre_value as usize));
                    }
                }
                Ok((s, format_var_val_pairs(&mut pairs, &self.variables)))
            }
            "effect" => {
                let (s, action) = flex_int_line(s)?;
                let op = &self.operators[action as usize];
                let mut pairs: Vec<(usize, usize)> = op
                    .effects
                    .iter()
                    .map(|eff| (eff.variable, eff.post_value))
                    .collect();
                Ok((s, format_var_val_pairs(&mut pairs, &self.variables)))
            }
            _ => Err(nom::Err::Failure(nom::error::Error::new(
                s,
                nom::error::ErrorKind::Tag,
            ))),
        }
    }
}
