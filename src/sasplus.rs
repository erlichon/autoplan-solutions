use std::fmt::{Display, Formatter};

mod parse;
mod query;

#[derive(Clone)]
pub struct Variable {
    pub name: String,
    pub axiom_layer: i32,
    pub values: Vec<String>,
}

#[derive(Clone)]
pub struct Effect {
    pub conditions: Vec<(usize, usize)>,
    pub variable: usize,
    pub pre_value: i32,
    pub post_value: usize,
}

#[derive(Clone)]
pub struct Operator {
    pub name: String,
    pub prevail: Vec<(usize, usize)>,
    pub effects: Vec<Effect>,
    pub cost: usize,
}

#[derive(Clone)]
pub struct SASPlus {
    pub variables: Vec<Variable>,
    pub goal: Vec<(usize, usize)>,
    pub operators: Vec<Operator>,
}

#[derive(Clone)]
pub struct State {
    pub values: Vec<usize>,
}

impl Operator {
    pub fn is_applicable(&self, state: &State) -> bool {
        for &(var, val) in &self.prevail {
            if state.values[var] != val {
                return false;
            }
        }
        for eff in &self.effects {
            if eff.pre_value >= 0 && state.values[eff.variable] != eff.pre_value as usize {
                return false;
            }
        }
        true
    }

    pub fn apply(&self, state: &State) -> State {
        let mut new_state = state.clone();
        for eff in &self.effects {
            let conditions_met = eff
                .conditions
                .iter()
                .all(|&(var, val)| state.values[var] == val);
            if conditions_met {
                new_state.values[eff.variable] = eff.post_value;
            }
        }
        new_state
    }
}

impl SASPlus {
    pub fn is_goal(&self, state: &State) -> bool {
        self.goal.iter().all(|&(var, val)| state.values[var] == val)
    }

    pub fn h1(&self, state: &State) -> Option<usize> {
        let mut cost: Vec<Vec<usize>> = self
            .variables
            .iter()
            .enumerate()
            .map(|(i, var)| {
                (0..var.values.len())
                    .map(|v| if state.values[i] == v { 0 } else { usize::MAX })
                    .collect()
            })
            .collect();

        loop {
            let mut changed = false;
            for op in &self.operators {
                for eff in &op.effects {
                    let precond_cost = Self::operator_effect_precond_cost(op, eff, &cost);
                    if precond_cost == usize::MAX {
                        continue;
                    }
                    let new_cost = op.cost.saturating_add(precond_cost);
                    if new_cost < cost[eff.variable][eff.post_value] {
                        cost[eff.variable][eff.post_value] = new_cost;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }

        let mut h = 0usize;
        for &(var, val) in &self.goal {
            let c = cost[var][val];
            if c == usize::MAX {
                return None;
            }
            h = h.max(c);
        }
        Some(h)
    }

    fn operator_effect_precond_cost(op: &Operator, eff: &Effect, cost: &[Vec<usize>]) -> usize {
        let mut max_cost = 0usize;
        for &(var, val) in &op.prevail {
            let c = cost[var][val];
            if c == usize::MAX {
                return usize::MAX;
            }
            max_cost = max_cost.max(c);
        }
        for e in &op.effects {
            if e.pre_value >= 0 {
                let c = cost[e.variable][e.pre_value as usize];
                if c == usize::MAX {
                    return usize::MAX;
                }
                max_cost = max_cost.max(c);
            }
        }
        for &(var, val) in &eff.conditions {
            let c = cost[var][val];
            if c == usize::MAX {
                return usize::MAX;
            }
            max_cost = max_cost.max(c);
        }
        max_cost
    }
}

impl Display for SASPlus {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        writeln!(f, "Variables: {}", self.variables.len())?;
        for (i, var) in self.variables.iter().enumerate() {
            write!(f, "  var{} ({}): ", i, var.name)?;
            for (j, val) in var.values.iter().enumerate() {
                if j > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", val)?;
            }
            writeln!(f)?;
        }
        writeln!(f, "Operators: {}", self.operators.len())?;
        writeln!(f, "Goal: {} conditions", self.goal.len())
    }
}
