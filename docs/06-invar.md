# 06-invar: h^2 Binary Invariants

**Problem IDs:** h2-invar-easy (L), h2-invar-hard (M)

## Problem Statement

Given an SAS+ file, extract all binary invariants derivable from the h^2
cost table. A binary invariant is a clause `L1 \/ L2` over two literals
that holds in every state reachable from the initial state.

- Input: SAS+ file on stdin.
- Output: one invariant per line, lexicographically sorted as strings.

Each literal is `sign var val` where sign is `+` (positive) or `-`
(negated). For each invariant the first literal's fact (v1, x1) is
lexicographically smaller than the second's (v2, x2).

## Chain of Thought

### Step 1 -- Connection to h^2

We already have a full h^2 implementation from exercise 03. The cost
table `cost[f1][f2]` records the estimated cost to simultaneously reach
facts f1 and f2. Entries at infinity mean the pair is provably
unreachable under the h^2 approximation.

An invariant `L1 \/ L2` holds iff the conjunction `not-L1 /\ not-L2` is
unreachable. Since h^2 is an admissible relaxation, any pair it declares
unreachable is genuinely unreachable -- so invariants extracted this way
are *sound* (though not complete).

### Step 2 -- Four sign combinations

For an ordered fact pair (v1,x1) < (v2,x2), a literal is positive
`(v=x)` or negative `not(v=x)`. We check all four sign pairs:

| Signs | Invariant meaning | Check |
|-------|-------------------|-------|
| `- -` | not both facts hold | `cost[f1][f2] = inf` |
| `+ -` | if v2=x2 then v1=x1 | for all a != x1: `cost[(v1,a)][f2] = inf` |
| `- +` | if v1=x1 then v2=x2 | for all b != x2: `cost[f1][(v2,b)] = inf` |
| `+ +` | at least one holds | for all a != x1, b != x2: `cost[(v1,a)][(v2,b)] = inf` |

### Step 3 -- Reuse h^2 infrastructure

Rather than duplicating the h^2 fixed-point, I refactored the existing
`h2()` method:

- New `h2_cost_table()` returns the full cost matrix and fact-ID offset
  array.
- `h2()` now calls `h2_cost_table()` and reads the goal value from it.
- `h2_invariants()` calls `h2_cost_table()` and scans all ordered fact
  pairs against the four sign checks.

This avoids any code duplication and reuses the already-optimised h^2
(precomputed operator data, O(k) case-B inner loop, etc.).

### Step 4 -- Sorting

The problem requires output sorted lexicographically as strings. Since
the format is `"sign var val sign var val"`, string sorting gives the
right order. We collect all valid invariant strings into a Vec, sort it,
and print.

## Algorithm

```
table = h2_cost_table(initial_state)

for each ordered fact pair (v1,x1) < (v2,x2):
  f1 = offset[v1] + x1
  f2 = offset[v2] + x2

  if cost[f1][f2] = inf:            emit "- v1 x1 - v2 x2"
  if all a!=x1: cost[(v1,a)][f2] = inf:  emit "+ v1 x1 - v2 x2"
  if all b!=x2: cost[f1][(v2,b)] = inf:  emit "- v1 x1 + v2 x2"
  if all a!=x1, b!=x2: cost[(v1,a)][(v2,b)] = inf:
                                     emit "+ v1 x1 + v2 x2"

sort output lines lexicographically
```

Complexity: O(h^2_fixpoint + F^2 * D_max^2) where F is the total number
of facts and D_max the largest domain size. The h^2 fixed-point dominates.

## Bug: Conditional Effects and the `assigned` Vector

### Symptom

All local samples passed (5 easy, 1 hard), and DOMjudge's sample runs
showed "correct", yet the overall submission received `wrong-answer`.

### Investigation

1. **Cross-validation with Python reference.** Wrote an independent
   Python h^2 implementation mirroring the Rust code. Both agreed on all
   samples and 200 random SAS+ files -- so the bug was not in the
   invariant extraction logic.

2. **BFS ground-truth checker.** Built a script that performs BFS on the
   full state space to discover every truly reachable fact pair, then
   compares against h^2's reachability claims. Over 300 random problems
   it found cases where h^2 declared a pair unreachable (`cost = INF`)
   even though BFS reached it. This violates admissibility and produces
   spurious invariants.

3. **Tracing a minimal counterexample (seed 3).** An operator had three
   effects: two unconditional (on var0 and var2) and one conditional (on
   var1, conditioned on var2=0). When var2 != 0, the conditional effect
   does not fire and var1's value is preserved. The path
   `(0,1,2) -> op2 -> (0,2,2) -> op7 -> (1,2,2)` reaches (var0=1,
   var1=2), but h^2 claimed this pair was unreachable.

### Root Cause

Two issues in `precompute_op` in `h2.rs`:

1. **`assigned[v]` was set for ALL effects, including conditional.**
   The h^2 regression skips "assigned" variables in the frame loop
   (they are handled by PairPc instead). But when a conditional effect's
   conditions aren't met, the variable's value *is preserved* -- it
   behaves as a frame variable. Marking it assigned prevented h^2 from
   considering this case, causing it to miss reachable pairs.

2. **`op_pre` excluded conditional effects' `pre_value`s.** An earlier
   fix had moved conditional effects' `pre_value`s out of `op_pre` and
   into per-effect `pre_base`. But in standard SAS+ semantics, ALL
   effects' `pre_value`s are global operator preconditions (the operator
   is only applicable when they hold, regardless of whether conditions
   are met).

### Fix (two parts)

**Part 1 -- frame loop and assigned vector:**

```rust
// op_pre: include ALL effects' pre_values (standard SAS+ semantics)
for e in &op.effects {
    if e.pre_value >= 0 {
        op_pre.push(fid(e.variable, e.pre_value as usize));
    }
}

// assigned: only variables with UNCONDITIONAL effects
for e in &op.effects {
    if e.conditions.is_empty() {
        assigned[e.variable] = true;
    }
}
```

This correctly handles both scenarios for a conditional effect on
variable v:
- **Effect fires** (conditions met): captured by PairPc between effects.
- **Effect does not fire** (conditions not met): v is a frame variable,
  captured by the frame loop since `assigned[v] = false`.

**Part 2 -- skip effect's own variable in frame loop:**

The Part 1 fix alone introduced a subtle secondary bug: when a
conditional effect on variable v has `assigned[v] = false`, the frame
loop iterates over v itself. This pairs `fid(v, post)` with
`fid(v, pre)` -- a *same-variable* pair that must always be INF (a
variable can't hold two values). Setting it finite causes missed `--`
invariants.

The effect's own variable must always be excluded from the frame loop
regardless of the `assigned` vector, because the effect firing (producing
`p1 = post`) and the variable being preserved (frame case) are
contradictory for the same variable.

```rust
// In the frame loop for each EffPc:
for v in 0..n {
    if v == ed.var || opc.assigned[v] { continue; }
    // ...
}
```

### Verification After Fix

- All 6 local samples pass.
- All 21 h^2 heuristic tests pass (no regression).
- 500-seed stress test: 0 same-variable pair violations, 0 Rust-vs-Python
  mismatches.

## Running Tests Locally

```bash
cargo test --test 06-invar
```

## Submission

```bash
scripts/prepare.sh 06-invar
```
