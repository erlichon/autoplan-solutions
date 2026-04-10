use nom::bytes::complete::tag;
use nom::character::complete::{line_ending, not_line_ending};
use nom::multi::count;
use nom::sequence::terminated;
use nom::{IResult, Parser};

use super::{Effect, Operator, SASPlus, State, Variable};

fn line(s: &str) -> IResult<&str, &str> {
    terminated(not_line_ending, line_ending).parse(s)
}

fn int_line(s: &str) -> IResult<&str, i64> {
    let (s, l) = line(s)?;
    let n: i64 = l
        .trim()
        .parse()
        .map_err(|_| nom::Err::Failure(nom::error::Error::new(s, nom::error::ErrorKind::Digit)))?;
    Ok((s, n))
}

fn parse_variable(s: &str) -> IResult<&str, Variable> {
    let (s, _) = terminated(tag("begin_variable"), line_ending).parse(s)?;
    let (s, name) = line(s)?;
    let name = name.to_string();
    let (s, axiom_layer) = int_line(s)?;
    let (s, range) = int_line(s)?;
    let (s, values) = count(line, range as usize).parse(s)?;
    let values: Vec<String> = values.into_iter().map(|v| v.to_string()).collect();
    let (s, _) = terminated(tag("end_variable"), line_ending).parse(s)?;
    Ok((
        s,
        Variable {
            name,
            axiom_layer: axiom_layer as i32,
            values,
        },
    ))
}

fn parse_var_val_pair(s: &str) -> IResult<&str, (usize, usize)> {
    let (s, l) = line(s)?;
    let mut parts = l.trim().split_whitespace();
    let var: usize = parts
        .next()
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| nom::Err::Failure(nom::error::Error::new(s, nom::error::ErrorKind::Digit)))?;
    let val: usize = parts
        .next()
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| nom::Err::Failure(nom::error::Error::new(s, nom::error::ErrorKind::Digit)))?;
    Ok((s, (var, val)))
}

fn parse_effect_line(s: &str) -> IResult<&str, Effect> {
    let (s, l) = line(s)?;
    let nums: Vec<i64> = l
        .trim()
        .split_whitespace()
        .map(|n| n.parse::<i64>().unwrap())
        .collect();

    let num_conditions = nums[0] as usize;
    let mut conditions = Vec::with_capacity(num_conditions);
    let mut idx = 1;
    for _ in 0..num_conditions {
        conditions.push((nums[idx] as usize, nums[idx + 1] as usize));
        idx += 2;
    }
    let variable = nums[idx] as usize;
    let pre_value = nums[idx + 1] as i32;
    let post_value = nums[idx + 2] as usize;

    Ok((
        s,
        Effect {
            conditions,
            variable,
            pre_value,
            post_value,
        },
    ))
}

fn parse_operator(s: &str) -> IResult<&str, Operator> {
    let (s, _) = terminated(tag("begin_operator"), line_ending).parse(s)?;
    let (s, name) = line(s)?;
    let name = name.to_string();
    let (s, num_prevail) = int_line(s)?;
    let (s, prevail) = count(parse_var_val_pair, num_prevail as usize).parse(s)?;
    let (s, num_effects) = int_line(s)?;
    let (s, effects) = count(parse_effect_line, num_effects as usize).parse(s)?;
    let (s, cost) = int_line(s)?;
    let (s, _) = terminated(tag("end_operator"), line_ending).parse(s)?;
    Ok((
        s,
        Operator {
            name,
            prevail,
            effects,
            cost: cost as usize,
        },
    ))
}

fn skip_axiom(s: &str) -> IResult<&str, ()> {
    let (s, _) = terminated(tag("begin_rule"), line_ending).parse(s)?;
    let (s, num_conditions) = int_line(s)?;
    let (s, _) = count(line, num_conditions as usize).parse(s)?;
    let (s, _) = line(s)?;
    let (s, _) = terminated(tag("end_rule"), line_ending).parse(s)?;
    Ok((s, ()))
}

fn skip_mutex_group(s: &str) -> IResult<&str, ()> {
    let (s, _) = terminated(tag("begin_mutex_group"), line_ending).parse(s)?;
    let (s, num_facts) = int_line(s)?;
    let (s, _) = count(line, num_facts as usize).parse(s)?;
    let (s, _) = terminated(tag("end_mutex_group"), line_ending).parse(s)?;
    Ok((s, ()))
}

impl SASPlus {
    pub fn parse(s: &str) -> IResult<&str, (SASPlus, State)> {
        // Version section
        let (s, _) = terminated(tag("begin_version"), line_ending).parse(s)?;
        let (s, _) = int_line(s)?;
        let (s, _) = terminated(tag("end_version"), line_ending).parse(s)?;

        // Metric section
        let (s, _) = terminated(tag("begin_metric"), line_ending).parse(s)?;
        let (s, _) = int_line(s)?;
        let (s, _) = terminated(tag("end_metric"), line_ending).parse(s)?;

        // Variables section
        let (s, num_variables) = int_line(s)?;
        let (s, variables) = count(parse_variable, num_variables as usize).parse(s)?;

        // Mutex section
        let (s, num_mutex) = int_line(s)?;
        let (s, _) = count(skip_mutex_group, num_mutex as usize).parse(s)?;

        // Initial state section
        let (s, _) = terminated(tag("begin_state"), line_ending).parse(s)?;
        let (s, state_values) = count(int_line, num_variables as usize).parse(s)?;
        let state = State {
            values: state_values.into_iter().map(|v| v as usize).collect(),
        };
        let (s, _) = terminated(tag("end_state"), line_ending).parse(s)?;

        // Goal section
        let (s, _) = terminated(tag("begin_goal"), line_ending).parse(s)?;
        let (s, num_goal) = int_line(s)?;
        let (s, goal) = count(parse_var_val_pair, num_goal as usize).parse(s)?;
        let (s, _) = terminated(tag("end_goal"), line_ending).parse(s)?;

        // Operator section
        let (s, num_operators) = int_line(s)?;
        let (s, operators) = count(parse_operator, num_operators as usize).parse(s)?;

        // Axiom section
        let (s, num_axioms) = int_line(s)?;
        let (s, _) = count(skip_axiom, num_axioms as usize).parse(s)?;

        let problem = SASPlus {
            variables,
            goal,
            operators,
        };

        Ok((s, (problem, state)))
    }
}
