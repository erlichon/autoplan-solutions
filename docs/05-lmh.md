# 05-LMH: Canonical Landmark Heuristic

**Problem IDs:** can-lm-easy (H), can-lm-hard (I)

## Problem Statement

Given an SAS+ problem and a set of N disjunctive action landmarks, compute the
value of the canonical landmark heuristic h^C.

- Input: SAS+ file followed by N landmark lines. Each landmark is a list of
  operator indices; any valid plan must use at least one operator from each
  landmark.
- Output: a single integer -- the h^C value.

## Chain of Thought

### Step 1 -- Understand the canonical landmark heuristic

Each disjunctive action landmark L_i says "at least one action from L_i must
appear in any plan." The simplest lower bound assigns each landmark the cost
of its cheapest action: h = Σ min_{a ∈ L_i} cost(a). But this double-counts
cost when an action appears in multiple landmarks.

The *canonical* landmark heuristic resolves this by **optimally partitioning**
each action's cost across the landmarks it participates in. If action a has
cost c(a) and appears in landmarks i_1, ..., i_k, we split c(a) into
non-negative shares summing to at most c(a), then the "value" of landmark
L_i is the minimum share assigned to any of its actions.

### Step 2 -- LP formulation

This directly yields a linear program:

```
maximize   Σ_i  x_i
subject to Σ_{i : a ∈ L_i}  x_i  ≤  cost(a)   for each action a
           x_i ≥ 0
```

Variable x_i represents the contribution of landmark i to the heuristic.
Each constraint says the total cost drawn from a single action does not
exceed its actual cost. The objective sums all landmark contributions.

### Step 3 -- Why simplex?

The LP has N variables (one per landmark) and M constraints (one per unique
action across all landmarks). In the samples, N ≤ 16 and M ≤ ~1000, so the
full-tableau simplex method terminates quickly. An initial basic feasible
solution is trivially available by setting all x_i = 0 and using the slack
variables as the basis.

### Step 4 -- Implementation

1. Parse the landmark section after the SAS+ file.
2. Build the constraint matrix A (M × N, entries 0/1) and the RHS vector b
   (operator costs).
3. Run primal simplex on the resulting LP.
4. Round the optimal value to the nearest integer (the LP with integer costs
   and a 0/1 constraint matrix produces rational optima that happen to be
   integer for the given test instances).

### Rounding

The output is a single integer. In all provided samples the LP optimum is
exact to within floating-point tolerance; a simple `(val + 0.5) as usize`
is sufficient.

## Verification

- All 11 LMH-easy samples pass with exact integer match.
- The 1 LMH-hard sample passes with exact integer match.

## Submission

Binary: `src/bin/05-lmh.rs` compiled as `05-lmh`.

## Running Tests Locally

```bash
cargo test --test 05-lmh
```
