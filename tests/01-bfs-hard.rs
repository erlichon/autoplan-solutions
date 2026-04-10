/// week 03: 01-BFS-hard
use autoplan::SASPlus;

test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/01-BFS-hard" as p01_bfs_hard => test }

fn test([input, expected]: [&'static str; 2]) {
    let (_, (problem, state)) = SASPlus::parse(input).expect("parse error");

    let result = problem.bfs_shortest_length(state);
    let observed = match result {
        Some(len) => format!("{}\n", len),
        None => "no solution\n".to_string(),
    };

    assert_eq!(expected, observed);
}
