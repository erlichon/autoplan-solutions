# 01-BFS-Simple: Shortest Plan Length via BFS

**Problem ID:** BFS 1

## Problem Statement

Given an SAS+ file, find the **shortest plan** (counting number of actions, ignoring costs) and output its length.

## Key Concept: Planning as Graph Search

A planning problem defines a **state space graph**:
- **Nodes** = states (each is a complete assignment of values to all variables)
- **Edges** = actions (an edge from state S to state S' exists if some operator is applicable in S and applying it produces S')
- **Start node** = the initial state from the SAS+ file
- **Goal nodes** = any state satisfying all goal conditions

Finding the shortest plan is equivalent to finding the shortest path from the start node to any goal node. Since all edges have uniform weight (we ignore costs), **Breadth-First Search (BFS)** is optimal.

## Algorithm

```
BFS-Plan(problem, initial_state):
    if initial_state satisfies goal:
        return 0

    visited = {initial_state}
    queue = [(initial_state, 0)]

    while queue is not empty:
        (state, depth) = queue.pop_front()

        for each operator in problem:
            if operator is applicable in state:
                new_state = apply(operator, state)

                if new_state satisfies goal:
                    return depth + 1

                if new_state not in visited:
                    visited.add(new_state)
                    queue.push_back(new_state, depth + 1)

    return no solution
```

### Why BFS Works

BFS explores states in order of increasing depth (number of actions from the start). The first time we reach a goal state, we're guaranteed it's at the minimum depth. This is because:
- All states at depth `d` are explored before any state at depth `d+1`
- So the first goal found is at the shallowest possible depth

### Critical Implementation Details

1. **Visited set**: Without tracking visited states, BFS would loop forever (e.g., `up` then `down` returns to the same state). We use a `HashSet<Vec<usize>>` for O(1) lookups.

2. **Early goal check**: We check for the goal when **generating** a successor (before enqueuing), not when **expanding** (after dequeuing). This saves exploring an entire extra level of the search tree.

3. **Initial state goal check**: The initial state might already satisfy the goal (test case 2, answer = 0). We handle this as a special case before starting BFS.

4. **Insert-and-check idiom**: `visited.insert(x)` returns `true` if `x` was newly inserted. This combines the "is it visited?" check and the "mark as visited" step into one operation.

## Walkthrough: Test Case 1 (Elevator)

**Initial state**: `[0, 1, 1]` — lift at f0, person not boarded, not served.
**Goal**: var2 = 0 (person served).

| Depth | State       | Action           | New State   |
|-------|-------------|------------------|-------------|
| 0     | [0, 1, 1]   | up f0→f1         | [1, 1, 1]   |
| 1     | [1, 1, 1]   | board f1 p0      | [1, 0, 1]   |
| 1     | [1, 1, 1]   | down f1→f0       | [0, 1, 1] (visited) |
| 2     | [1, 0, 1]   | down f1→f0       | [0, 0, 1]   |
| 3     | [0, 0, 1]   | depart f0 p0     | **[0, 1, 0]** — goal! |

**Answer: 4**

The plan is: up → board → down → depart.

## Complexity

- **Time**: O(|S| × |O|) where |S| is the number of reachable states and |O| is the number of operators. Each state is expanded at most once, and for each we try all operators.
- **Space**: O(|S|) for the visited set and queue. The state space can be exponential in the number of variables (product of all domain sizes), but BFS with duplicate detection ensures we never store more than the reachable state space.

## Implementation

The BFS in `src/bfs.rs`:

```rust
pub fn bfs_shortest_length(&self, start_state: State) -> Option<usize> {
    if self.is_goal(&start_state) {
        return Some(0);
    }

    let mut visited: HashSet<Vec<usize>> = HashSet::new();
    visited.insert(start_state.values.clone());

    let mut queue: VecDeque<(State, usize)> = VecDeque::new();
    queue.push_back((start_state, 0));

    while let Some((state, depth)) = queue.pop_front() {
        for op in &self.operators {
            if op.is_applicable(&state) {
                let new_state = op.apply(&state);
                if self.is_goal(&new_state) {
                    return Some(depth + 1);
                }
                if visited.insert(new_state.values.clone()) {
                    queue.push_back((new_state, depth + 1));
                }
            }
        }
    }

    None
}
```

## Running

```bash
# Run tests
cargo test --test 01-bfs-simple

# Run on a specific input
cargo run --bin 01-bfs-simple < tests/samples/01-bfs-simple/1.in
```
