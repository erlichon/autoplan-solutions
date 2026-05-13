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

### Step 1 -- Understand disjunctive action landmarks

A disjunctive action landmark L = {a_1, ..., a_k} says "every valid plan
must contain at least one of these actions." The cheapest way to satisfy
L costs min_{a in L} cost(a).

### Step 2 -- Additivity and the canonical heuristic

Two landmarks are *additive* if they share no action -- satisfying one
cannot help satisfy the other, so their costs can be summed safely. The
canonical landmark heuristic h^C finds the maximum-weight set of pairwise
additive landmarks:

```
h^C = max over all pairwise-additive subsets S of landmarks:
        sum_{L_i in S}  min_{a in L_i} cost(a)
```

This is equivalent to Maximum Weight Independent Set (MWIS) on the
*conflict graph*, where two landmarks are connected iff they share an
action.

### Step 3 -- Why not LP?

An earlier attempt used LP-based optimal cost partitioning, which
computes an upper bound >= h^C. The LP can yield non-integer or strictly
higher values (e.g. three pairwise-conflicting unit-cost landmarks give
LP = 1.5 but h^C = 1). The problem asks for h^C specifically.

### Step 4 -- Efficient MWIS via connected components

MWIS is NP-hard in general, but the conflict graph typically decomposes
into small connected components (landmarks from different parts of the
planning task rarely share actions). The algorithm:

1. Build the conflict graph.
2. Find connected components (BFS).
3. For each component independently, solve MWIS:
   - k <= 30: bitmask DP in O(2^k) time.
   - k > 30: branch-and-bound with pruning (upper bound = sum of all
     remaining non-excluded values).
4. Sum the per-component MWIS values.

### Step 5 -- Metric handling

SAS+ files with `metric=0` use unit cost (all operators cost 1),
regardless of the raw cost field in each operator. The parser normalises
costs to 1 when `metric=0`, so heuristic code always uses `op.cost`
directly.

## Algorithm

```
values[i] = min cost of any action in landmark i

conflict[i][j] = true iff landmarks i and j share at least one action

components = connected components of the conflict graph

for each component C:
  if |C| == 1: total += values[C[0]]
  else:
    remap to local indices [0..k)
    if k <= 30: bitmask DP
      dp[mask] = max( dp[mask without v],
                      values[v] + dp[mask without v and without neighbors(v)] )
      where v = lowest set bit of mask
    else: branch-and-bound with sum-of-remaining pruning

return total
```

## Verification

- All 11 LMH-easy samples pass with exact integer match.
- The 1 LMH-hard sample passes with exact integer match.
- Synthetic test cases confirm h^C (MWIS) != LP for conflicting
  landmarks (e.g. three pairwise-conflicting unit-cost landmarks:
  h^C = 1, LP = 1.5).

## Running Tests Locally

```bash
cargo test --test 05-lmh
```

## Submission

```bash
scripts/prepare.sh 05-lmh
```
