/// week 02: 02-applicability
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

    for op in &problem.operators {
        if op.is_applicable(&state) {
            println!("applicable");
        } else {
            println!("not applicable");
        }
    }

    Ok(())
}
