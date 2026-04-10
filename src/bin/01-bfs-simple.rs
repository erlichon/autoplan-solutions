/// week 03: BFS simple (shortest plan length)
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

    match problem.bfs_shortest_length(state) {
        Some(len) => println!("{}", len),
        None => println!("no solution"),
    }

    Ok(())
}
