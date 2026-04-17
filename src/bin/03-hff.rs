/// week 03: 03-hff -- the FF (delete-relaxation) heuristic.
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

    match problem.h_ff(&state) {
        Some(result) => {
            writeln!(out, "{} {}", result.heuristic, result.plan.len())?;
            for op in &result.plan {
                writeln!(out, "{}", op)?;
            }
        }
        None => {
            writeln!(out, "infinity")?;
        }
    }

    Ok(())
}
