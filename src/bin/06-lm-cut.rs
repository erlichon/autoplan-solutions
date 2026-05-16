/// week 6: 06-lm-cut -- the LM-cut heuristic.
use autoplan::SASPlus;
use std::{
    error::Error,
    io::{Read as _, Write as _, stdin, stdout},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut data = String::new();
    stdin().read_to_string(&mut data)?;
    let s: &str = data.as_ref();

    let (_, (problem, state)) = SASPlus::parse(s).expect("could not parse SAS+");

    let out = stdout();
    let mut out = out.lock();

    let rounds = problem.lm_cut(&state);
    for round in &rounds {
        writeln!(out, "{}", round.hmax)?;
        if !round.landmark.is_empty() {
            write!(out, "{}", round.landmark.len())?;
            for &op in &round.landmark {
                write!(out, " {}", op)?;
            }
            writeln!(out)?;
            writeln!(out, "{}", round.cost)?;
        }
    }

    Ok(())
}
