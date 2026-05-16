use autoplan::SASPlus;

test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/06-lm-cut-easy" as p06_lm_cut_easy => test_lm_cut }
test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/06-lm-cut-hard" as p06_lm_cut_hard => test_lm_cut }

fn test_lm_cut([input, expected]: [&'static str; 2]) {
    let (_, (problem, state)) = SASPlus::parse(input).expect("parse error");
    let rounds = problem.lm_cut(&state);

    let mut lines = Vec::new();
    for round in &rounds {
        lines.push(format!("{}", round.hmax));
        if !round.landmark.is_empty() {
            let ops: Vec<String> = round.landmark.iter().map(|o| o.to_string()).collect();
            lines.push(format!("{} {}", round.landmark.len(), ops.join(" ")));
            lines.push(format!("{}", round.cost));
        }
    }
    let actual = lines.join("\n");
    let expected = expected.trim_end();
    assert_eq!(actual, expected, "lm-cut output mismatch");
}
