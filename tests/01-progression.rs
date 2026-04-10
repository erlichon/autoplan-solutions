/// week 02: 01-progression
use autoplan::SASPlus;

test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/01-progression" as p01_progression => test }

fn test([input, expected]: [&'static str; 2]) {
    let (_, (problem, state)) = SASPlus::parse(input).expect("parse error");

    let mut observed = String::new();
    for op in &problem.operators {
        if op.is_applicable(&state) {
            let new_state = op.apply(&state);
            for (i, &val) in new_state.values.iter().enumerate() {
                observed.push_str(&problem.variables[i].values[val]);
                observed.push('\n');
            }
        }
    }

    assert_eq!(expected, observed);
}
