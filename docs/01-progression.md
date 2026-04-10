# 01-Progression: Applying Actions

**Problem ID:** 02b-apply

## Problem Statement

Given an SAS+ file, for every **applicable** action, apply it individually to the initial state and output the resulting state. Each action is applied to the **initial** state independently -- you do not chain actions (i.e., the result of one action is not fed into the next).

Output variable values as their names, in variable declaration order, with no separators between consecutive states.

## Key Concept: Action Progression

"Progression" means computing the successor state that results from applying an action to a state. In notation: given initial state `I` and action `a`, compute `I[[a]]`.

### How to Apply an Action

Given that an action is applicable (all prevail and effect preconditions are satisfied), applying it means:

1. **Clone** the current state.
2. For each effect in the operator:
   - Check if the effect's **conditions** are satisfied in the **original** state (important: not the new state being built). These are the conditional-effect conditions stored in `eff.conditions`.
   - If conditions are met (or there are none), set `new_state[effect.variable] = effect.post_value`.
3. The resulting state is the successor.

### Why Check Conditions Against the Original State?

In SAS+, all effects of an action are applied **simultaneously**. This means:
- Effect conditions are evaluated against the state **before** any effects take place.
- If effect A changes var0 and effect B has a condition on var0, effect B checks the **original** value of var0, not the value set by effect A.

Cloning the state first and then modifying the clone achieves this correctly.

### Conditional Effects

Some effects have additional conditions (`eff.conditions`). These are `(variable, value)` pairs. The effect only fires if **all** its conditions are satisfied in the original state. This is different from `pre_value`, which is a precondition on the effect's own target variable.

- `pre_value`: Determines whether the **action** is applicable (checked during applicability)
- `eff.conditions`: Determines whether this specific **effect fires** when the action is applied (checked during progression)

## Walkthrough: Test Case 1

**Initial state:** `[2, 0, 1, 0]` = on-table(b1), clear(b2), NOT-clear(b1), on(b2,b1)

Only operators 1 and 3 are applicable (from the applicability exercise).

### Operator 1: "move-b-to-b b2 b1 b2"

Effects (all have 0 conditions, so all fire):

| Effect | Variable | Post-value | Change |
|--------|----------|------------|--------|
| var2 → 0 | var2 | 0 | NOT-clear(b1) → clear(b1) |
| var1 → 1 | var1 | 1 | clear(b2) → NOT-clear(b2) |
| var3 → 1 | var3 | 1 | on(b2,b1) → on(b2,b2) |

Result: `[2, 1, 0, 1]` → on-table(b1), NOT-clear(b2), clear(b1), on(b2,b2)

### Operator 3: "move-b-to-t b2 b1"

Applied to the **initial** state (not operator 1's result):

| Effect | Variable | Post-value | Change |
|--------|----------|------------|--------|
| var2 → 0 | var2 | 0 | NOT-clear(b1) → clear(b1) |
| var3 → 2 | var3 | 2 | on(b2,b1) → on-table(b2) |

Result: `[2, 0, 0, 2]` → on-table(b1), clear(b2), clear(b1), on-table(b2)

## Implementation

The `apply` method on `Operator` (`src/sasplus.rs`):

```rust
pub fn apply(&self, state: &State) -> State {
    let mut new_state = state.clone();
    for eff in &self.effects {
        let conditions_met = eff
            .conditions
            .iter()
            .all(|&(var, val)| state.values[var] == val);
        if conditions_met {
            new_state.values[eff.variable] = eff.post_value;
        }
    }
    new_state
}
```

The binary (`src/bin/01-progression.rs`) iterates applicable operators:

```rust
for op in &problem.operators {
    if op.is_applicable(&state) {
        let new_state = op.apply(&state);
        for (i, &val) in new_state.values.iter().enumerate() {
            println!("{}", problem.variables[i].values[val]);
        }
    }
}
```

## Running

```bash
# Run tests
cargo test --test 01-progression

# Run on a specific input
cargo run --bin 01-progression < tests/samples/01-progression/1.in
```
