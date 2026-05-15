/// week 06: 06-invar -- h^2 binary invariant extraction.
use autoplan::SASPlus;
use std::{
    error::Error,
    io::{Read as _, stdin},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut data = String::new();
    stdin().read_to_string(&mut data)?;
    let s: &str = data.as_ref();

    let (_, (problem, state)) = SASPlus::parse(s).expect("could not parse SAS+");
    let invariants = problem.h2_invariants(&state);
    for inv in &invariants {
        println!("{}", inv);
    }

    Ok(())
}
