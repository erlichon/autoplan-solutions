# 01-BFS-Medium / 01-BFS-Hard: Shortest Plan Length via BFS

**Problem ID:** 03b-BFS-hard

## Problem Statement

Same as BFS-simple: given an SAS+ file, find the **shortest plan** (counting number of actions, ignoring costs) and output its length.

The medium and hard variants have larger state spaces (more variables, more operators, longer optimal plans -- up to 127 actions in test case 4).

## Why No Code Changes Were Needed

The BFS implementation from `01-bfs-simple` already handles these cases efficiently because:

1. **BFS with visited-set deduplication** is the correct algorithm for unweighted shortest-path. It explores each reachable state at most once, so runtime is proportional to the reachable state space, not the total possible state space.

2. **HashSet lookup is O(1) amortized**, so the per-state cost is dominated by generating successors (trying each operator), not by checking for duplicates.

3. **Rust's standard library** (`HashSet`, `VecDeque`) provides efficient implementations out of the box.

Even the hardest test case (17 variables, ~300 operators, optimal plan length 127) completes in under 0.25 seconds in debug mode.

## When Would You Need to Optimize?

For much larger problems, potential optimizations include:

- **Compact state representation**: Pack variable values into a single `u64` or `u128` (if the total bits needed fit) instead of `Vec<usize>`. This makes hashing and comparison much faster and reduces memory per state.
- **Avoid cloning**: Use indices into a state storage vector instead of cloning states into the queue.
- **Bidirectional BFS**: Search forward from the initial state and backward from goal states simultaneously, meeting in the middle. This can dramatically reduce the search space.

None of these were needed for the provided test cases.

## Running

```bash
# Run medium tests
cargo test --test 01-bfs-medium

# Run hard tests
cargo test --test 01-bfs-hard

# Run a specific input
cargo run --bin 01-bfs-simple < tests/samples/01-BFS-hard/4.in
```
