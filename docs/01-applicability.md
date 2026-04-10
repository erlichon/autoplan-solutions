# 01-Applicability: Checking Action Application

**Problem ID:** 02a-test-apply

## Problem Statement

Given an SAS+ file, determine for every action (operator) whether it is **applicable** in the initial state. Output `applicable` or `not applicable` for each action, in the order they appear in the input.

## Key Concepts

### SAS+ States and Variables

In SAS+ planning, the world is described by a set of **variables**, each with a finite domain of values. A **state** assigns exactly one value to each variable. For example, with 4 variables:

```
state = [2, 0, 1, 0]
```

This means var0 has value 2, var1 has value 0, var2 has value 1, var3 has value 0.

### Operators (Actions)

An operator transforms one state into another, but it can only be applied if its **preconditions** are satisfied. Preconditions come from two places in the SAS+ format:

#### 1. Prevail Conditions (`op.prevail`)

These are `(variable, value)` pairs that must hold in the current state. The operator does **not** change these variables — they are "read-only requirements."

For example, if an operator has prevail condition `(1, 0)`, it requires `state[1] == 0`.

#### 2. Effect Pre-values (`op.effects[i].pre_value`)

Each effect describes how the operator changes a single variable. An effect has:

| Field        | Meaning                                                        |
|------------- |----------------------------------------------------------------|
| `variable`   | Which variable this effect modifies                            |
| `pre_value`  | Required current value of the variable (`-1` = don't care)     |
| `post_value` | The value the variable will have after applying the action     |
| `conditions` | Extra conditions for conditional effects (don't affect applicability of the action itself) |

If `pre_value >= 0`, the variable must currently hold exactly that value. If `pre_value == -1`, there is no requirement on the variable's current value.

### Applicability Rule

**An action is applicable if and only if ALL of the following hold:**

1. Every prevail condition `(var, val)` is satisfied: `state[var] == val`
2. For every effect where `pre_value >= 0`: `state[effect.variable] == pre_value`

If even a single condition fails, the action is **not applicable**.

## Walkthrough: Test Case 1

**Initial state:** `[2, 0, 1, 0]`

| Index | Variable | Value | Meaning         |
|-------|----------|-------|-----------------|
| 0     | var0     | 2     | on-table(b1)    |
| 1     | var1     | 0     | clear(b2)       |
| 2     | var2     | 1     | NOT clear(b1)   |
| 3     | var3     | 0     | on(b2, b1)      |

### Operator 0: "move-b-to-b b1 b2 b1" → not applicable

- No prevail conditions.
- Effect `0 2 0 1`: requires var2 = 0, but state has var2 = **1** → FAIL.

Since one effect precondition fails, the action is not applicable.

### Operator 1: "move-b-to-b b2 b1 b2" → applicable

- No prevail conditions.
- Effect `0 2 -1 0`: var2, pre = -1 → don't care, always OK.
- Effect `0 1 0 1`: requires var1 = 0, state has var1 = 0 → OK.
- Effect `0 3 0 1`: requires var3 = 0, state has var3 = 0 → OK.

All conditions satisfied → applicable.

### Operator 3: "move-b-to-t b2 b1" → applicable

- Prevail: `(1, 0)` → requires var1 = 0, state has var1 = 0 → OK.
- Effect `0 2 -1 0`: pre = -1 → don't care → OK.
- Effect `0 3 0 2`: requires var3 = 0, state has var3 = 0 → OK.

All conditions satisfied → applicable.

### Operator 4: "move-t-to-b b1 b1" → not applicable

- No prevail conditions.
- Effect `0 2 0 1`: requires var2 = 0, but state has var2 = **1** → FAIL.

## Implementation

The core logic is a method on `Operator` (`src/sasplus.rs`):

```rust
impl Operator {
    pub fn is_applicable(&self, state: &State) -> bool {
        for &(var, val) in &self.prevail {
            if state.values[var] != val {
                return false;
            }
        }
        for eff in &self.effects {
            if eff.pre_value >= 0 && state.values[eff.variable] != eff.pre_value as usize {
                return false;
            }
        }
        true
    }
}
```

The binary (`src/bin/01-applicability.rs`) parses the SAS+ file and iterates each operator:

```rust
let (_, (problem, state)) = SASPlus::parse(s).expect("could not parse SAS+");

for op in &problem.operators {
    if op.is_applicable(&state) {
        println!("applicable");
    } else {
        println!("not applicable");
    }
}
```

## Running

```bash
# Run tests
cargo test --test 01-applicability

# Run on a specific input
cargo run --bin 01-applicability < tests/samples/01-applicability/1.in
```
