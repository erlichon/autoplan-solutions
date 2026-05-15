/// week 06: 06-invar -- validates h^2 invariant extraction against .ans files.
use autoplan::SASPlus;

test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/06-invar-easy" as p06_invar_easy => test_invar }
test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/06-invar-hard" as p06_invar_hard => test_invar }

fn test_invar([input, expected]: [&'static str; 2]) {
    let (_, (problem, state)) = SASPlus::parse(input).expect("parse error");
    let invariants = problem.h2_invariants(&state);
    let actual = invariants.join("\n");
    let expected = expected.trim_end();
    assert_eq!(actual, expected, "invariant output mismatch");
}
