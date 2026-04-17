/// week 03: 03-hff -- validates that the h^FF extracted plan is a correct
/// delete-relaxed plan (i.e., achieves the goal under delete relaxation) and
/// that the reported H equals the summed operator cost of the plan.
use autoplan::SASPlus;

test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/01-bfs-simple" as p03_hff_bfs_simple => test }
test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/01-BFS-medium" as p03_hff_bfs_medium => test }
test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/01-BFS-hard" as p03_hff_bfs_hard => test }

// Official sample packs for the FF problem. The grader accepts any
// reasonable relaxed plan, so we reuse the same plan-legality +
// H == cost-sum validator rather than comparing strings against .ans.
test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/03-FF-easy-1" as p03_hff_official_easy_1 => test }
test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/03-FF-easy-2" as p03_hff_official_easy_2 => test }
test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/03-FF-medium-1" as p03_hff_official_medium_1 => test }
test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/03-FF-medium-2" as p03_hff_official_medium_2 => test }
test_each_file::test_each_file! { for ["in", "ans"] in "./tests/samples/03-FF-hard" as p03_hff_official_hard => test }

fn test([input, _expected_bfs_len]: [&'static str; 2]) {
    let (_, (problem, state)) = SASPlus::parse(input).expect("parse error");

    let hff = problem.h_ff(&state).expect("goal unreachable");

    // H must equal the sum of the operator costs in the extracted plan --
    // this is what the grader re-computes from our plan output and
    // compares against the H value on the header line.
    let plan_cost: usize = hff
        .plan
        .iter()
        .map(|&op_idx| problem.operators[op_idx].cost)
        .sum();
    assert_eq!(
        hff.heuristic, plan_cost,
        "h^FF heuristic value must equal the summed operator cost of the extracted plan"
    );

    // Simulate the plan under DELETE RELAXATION.
    //
    // In the relaxed semantics every reached (var, val) stays reachable
    // forever, so we track a set of reached facts rather than a single
    // SAS+ state value per variable.
    let mut reached: Vec<Vec<bool>> = problem
        .variables
        .iter()
        .map(|var| vec![false; var.values.len()])
        .collect();
    for (i, &v) in state.values.iter().enumerate() {
        reached[i][v] = true;
    }

    // A used-up set of operator indices, to detect duplicates in the plan.
    let mut used = vec![false; problem.operators.len()];

    for (step, &op_idx) in hff.plan.iter().enumerate() {
        assert!(
            op_idx < problem.operators.len(),
            "step {step}: operator index {op_idx} out of range"
        );
        assert!(
            !used[op_idx],
            "step {step}: operator {op_idx} appears twice in the relaxed plan"
        );
        used[op_idx] = true;

        let op = &problem.operators[op_idx];

        // Operator applicability under delete relaxation: all prevail and
        // all effect pre-values (>=0) must be reached at least once.
        for &(v, d) in &op.prevail {
            assert!(
                reached[v][d],
                "step {step}: operator {op_idx} ({}) fires before prevail \
                 ({}, {}) is reached",
                op.name,
                problem.variables[v].name,
                problem.variables[v].values[d]
            );
        }
        for e in &op.effects {
            if e.pre_value >= 0 {
                let d = e.pre_value as usize;
                assert!(
                    reached[e.variable][d],
                    "step {step}: operator {op_idx} ({}) fires before \
                     pre-value ({}, {}) is reached",
                    op.name,
                    problem.variables[e.variable].name,
                    problem.variables[e.variable].values[d]
                );
            }
        }

        // Apply the add-effects. Under delete relaxation, conditional
        // effects fire whenever their conditions are reached.
        for e in &op.effects {
            let cond_ok = e.conditions.iter().all(|&(cv, cd)| reached[cv][cd]);
            if cond_ok {
                reached[e.variable][e.post_value] = true;
            }
        }
    }

    // The goal must be reached under delete relaxation.
    for &(v, d) in &problem.goal {
        assert!(
            reached[v][d],
            "relaxed plan does not reach goal fact ({}, {})",
            problem.variables[v].name,
            problem.variables[v].values[d]
        );
    }
}
