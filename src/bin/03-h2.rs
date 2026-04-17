/// week 03: 03-h2 -- the h^2 (Haslum pair-regression) heuristic.
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

    match problem.h2(&state) {
        Some(h) => println!("{}", h),
        None => println!("infinity"),
    }

    Ok(())
}
