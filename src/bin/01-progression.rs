/// week 02: 02b-apply (progression)
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
            let new_state = op.apply(&state);
            for (i, &val) in new_state.values.iter().enumerate() {
                println!("{}", problem.variables[i].values[val]);
            }
        }
    }

    Ok(())
}
