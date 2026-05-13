/// week 05: 05-lmh -- canonical landmark heuristic.
use autoplan::{canonical_lm_heuristic, parse_landmarks, SASPlus};
use std::{
    error::Error,
    io::{Read as _, stdin},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut data = String::new();
    stdin().read_to_string(&mut data)?;

    let (rest, (problem, _state)) = SASPlus::parse(&data).expect("could not parse SAS+");

    let landmarks = parse_landmarks(rest);
    let h = canonical_lm_heuristic(&problem, &landmarks);

    println!("{}", h);
    Ok(())
}
