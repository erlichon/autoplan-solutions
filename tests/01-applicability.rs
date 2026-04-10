/// week 02: 01-applicability
use autoplan::SASPlus;

test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/01-applicability" as p01_applicability => test }

fn test([input, expected]: [&'static str; 2]) {
    let (_, (problem, state)) = SASPlus::parse(input).expect("parse error");

    let mut observed = String::new();
    for op in &problem.operators {
        if op.is_applicable(&state) {
            observed.push_str("applicable\n");
        } else {
            observed.push_str("not applicable\n");
        }
    }

    assert_eq!(expected, observed);
}
