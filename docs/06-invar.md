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

## Verification

- All 5 invar-easy samples pass with exact string match.
- The 1 invar-hard sample passes with exact string match.
- The 21 existing h^2 tests still pass after the refactoring.

## Running Tests Locally

```bash
cargo test --test 06-invar
```

## Submission

```bash
scripts/prepare.sh 06-invar
```
