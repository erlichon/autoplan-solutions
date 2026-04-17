# 03-hff: The h^FF (Fast-Forward) Heuristic

**Problem ID:** Programming E -- h^FF

## Problem Statement

Read an SAS+ file from standard input and output:

1. One line with two integers `H L`, where `H` is the value of the h^FF
   heuristic for the initial state and `L` is the length of the extracted
   delete-relaxed plan.
2. `L` lines, each containing one 0-indexed operator id, listing the
   relaxed plan that h^FF found.

The grader accepts "any plan that can reasonably be produced by the h^FF
heuristic".

## Chain of Thought

### Step 1 -- Understand what h^FF is

The classical FF heuristic (Hoffmann & Nebel, 2001) is a two-phase procedure:

1. **Reachability analysis** -- build a delete-relaxed planning graph
   (RPG) from the initial state. Under delete relaxation, a fact that ever
   becomes true stays true forever, so we can compute the earliest layer
   at which each fact is reachable.
2. **Relaxed plan extraction** -- starting from the goal facts, walk the
   layered graph backwards, picking an achiever for every unmet subgoal
   and thereby accumulating the preconditions of those achievers as new
   subgoals. The final set of chosen operators is the relaxed plan.

The heuristic value h^FF is defined as the **cost of the extracted
relaxed plan** -- the sum of operator costs. Under unit-cost problems
this coincides with the plan length `L`, but when operator costs are
non-unit the grader re-computes the cost from the listed operator ids
and rejects the submission if our reported `H` does not match that sum.
An earlier version of this solution reported `H = L` and was rejected
on several test cases with diagnostic messages of the form
"given and calculated cost of plan do not match 32 != 622", which
pinned down the correct definition.

### Step 2 -- Map RPG to SAS+

In STRIPS the RPG is a sequence of fact sets `F_0 ⊆ F_1 ⊆ ...`. In SAS+,
facts are `(variable, value)` pairs and operator applicability is more
nuanced:

- An operator fires when **all** its prevail conditions hold **and** every
  effect with `pre_value >= 0` has that pre-value in the current state.
  This is the same applicability rule already used in `Operator::is_applicable`
  (`src/sasplus.rs:42`).
- A conditional effect `e` (one with non-empty `e.conditions`) fires only
  when the operator is applicable **and** its conditional conditions hold.
- Under delete relaxation, `pre_value` is still a **precondition** (the
  operator must see that value), but no fact is ever unset -- the
  "post_value becomes true" semantics is monotonic.

So for h^FF, the precondition of "effect `e` of operator `o` achieves fact
`(e.variable, e.post_value)`" is

- every `(v, d)` in `o.prevail`,
- for **every** effect `e'` of `o` with `e'.pre_value >= 0`, the pair
  `(e'.variable, e'.pre_value)` -- the operator's full applicability, not
  just `e`'s own pre-value,
- every `(v, d)` in `e.conditions`.

This matches the precondition used by `h1` in `src/sasplus.rs:119`. I got
this wrong on the first pass (I only included `eff.pre_value`), and the
sample's expected output `3 3 / 3 / 0 / 1` immediately flagged it: my
broken version found a spurious length-1 relaxed plan because the "depart"
operator has a `served` effect with `pre_value = -1` (no pre on that
effect) but a `boarded` effect with `pre_value = 0` (needs boarded), and
only the latter prevents single-step "teleport to served".

### Step 3 -- Compute layer levels and record achievers

Fixed-point iteration over a 2-D table `level[v][d]`:

- Initialise `level[v][d] = 0` if `state[v] == d`, else `infinity`.
- For every operator `o` and effect `e`, compute the precondition level
  (max of the levels of all facts described above). If that is finite,
  set `new_level = precond_level + 1`. If it is smaller than the current
  `level[e.variable][e.post_value]`, lower it and remember `(op_idx, eff_idx)`
  as the achiever.
- Repeat until nothing changes.

This is `O(iters * ops * effects)` where `iters` is bounded by the number
of reachable facts. Each iteration either lowers some level or terminates.

### Step 4 -- Extract the relaxed plan (FF-style backward pass)

Process subgoals layer by layer, highest first:

1. Seed `subgoals[level[g]]` with every goal fact `g`. Goal facts already
   true in the initial state (`level == 0`) need no action.
2. For each `t` from `max_layer` down to `1`, iterate through
   `subgoals[t]`:
   - Look up the achiever `(op_idx, eff_idx)` recorded in Phase 1.
   - If the operator is not yet in the plan, append it (`used_op` flag).
   - Enqueue the achiever's preconditions (same set as in Phase 1: the
     operator's prevail, every effect's pre-value, and this effect's
     conditional conditions) at their own layers.

Notes on subtleties:

- **Operators are deduplicated.** If two subgoals share the same achiever
  operator via different effects, we still only list the operator once
  in the plan -- which is what FF reports.
- **Preconditions of both effects are kept.** When an operator is reused
  for a second effect, we still enqueue *that* effect's preconditions (so
  conditional-effect conditions are not dropped).
- **We do not "free" other effects.** Some implementations mark all
  post-values of a chosen operator as "already achieved" so that they
  never enter the subgoal list. That is unsound with conditional effects,
  so we only mark the fact we explicitly committed to and rely on
  `is_subgoal` to avoid duplicate work.
- **Ordering.** We append to `plan` top-down (high layer first) and
  reverse at the end. The resulting order is applicable under delete
  relaxation by construction (every operator's preconditions sit at
  strictly lower layers).

### Step 5 -- What to report

- `H = sum of op.cost over the extracted plan`. Under unit costs this
  equals `L = plan.len()`; under non-unit costs they differ.
- `L = plan.len()`, the number of operators in the extracted plan.
- Operators, one per line, in the forward order produced by the reverse
  step above.
- If the goal is unreachable under delete relaxation, print `infinity`
  (mirroring `02-h1`).

## Algorithm Summary

```
Phase 1 (h^max layering):
  level[v][d] <- 0 if state[v]=d else +inf
  achiever[v][d] <- None
  repeat until no change:
    for each (op, eff):
      pre = max level of ( prevail(op) ∪ every effect pre-value of op
                                ∪ eff.conditions )
      if pre is finite and pre+1 < level[eff.var][eff.post]:
        level[eff.var][eff.post] <- pre + 1
        achiever[eff.var][eff.post] <- (op, eff)

Phase 2 (relaxed plan extraction):
  subgoals[t] <- { g in goal : level[g] = t }   for each t > 0
  plan <- []
  used_op <- {}
  for t = max_layer downto 1:
    for each (v, d) in subgoals[t]:
      (op, eff) = achiever[v][d]
      if op not in used_op:
        plan.append(op); used_op.add(op)
      for each precondition fact p of (op, eff):
        if level[p] > 0 and p not in subgoals:
          add p to subgoals[level[p]]
  reverse(plan)
  return (plan.len(), plan)
```

## Properties

- **Not admissible**: h^FF can exceed h*(s) because extraction may pick a
  longer achiever chain than strictly necessary. Our test data shows this
  in practice: on `01-BFS-hard/3.in`, BFS optimal is 15 but h^FF reports
  18.
- **Informative**: often a much tighter estimate than h^1 / h^max; on
  `01-BFS-hard/4.in` h^FF = 7 while h* = 127, showing that delete
  relaxation can be wildly optimistic -- typical for problems dominated by
  reversible motion.
- **Polynomial**: both phases are polynomial in the number of facts and
  operators.

## Verification

`tests/03-hff.rs` runs two checks on every SAS+ sample in
`tests/samples/01-bfs-simple`, `01-BFS-medium`, and `01-BFS-hard`:

1. `H == sum of op.cost over the plan` (same thing the grader re-computes).
2. The emitted operator sequence is a valid **delete-relaxed** plan:
   simulate it on a monotone fact-set (no deletes), require every
   operator's prevail and every effect pre-value to be already reached,
   then mark every (conditional-condition-satisfied) post-value as
   reached. Finally, assert that every goal fact is reached.

All 12 test cases pass.

On the problem's sample input (the miconic-style lift problem identical to
`tests/samples/01-bfs-simple/1.in`), our output matches the expected
sample exactly:

```
3 3
3
0
1
```

i.e. `up f0 f1`, `board f1 p0`, `depart f0 p0` -- a length-3 relaxed plan
that reaches `served(p0)`.

## Running

```bash
cargo run --release --bin 03-hff < input.sas
```

## Submission

```bash
scripts/prepare.sh 03-hff
```

Submit the resulting `.tar.xz` to DOMjudge as type `cargo`.
