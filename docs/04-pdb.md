# 04-PDB: Pattern Database Heuristic

**Problem IDs:** pdb-easy (F), pdb-hard (G)

## Problem Statement

Build a pattern database (PDB) for a given SAS+ problem and pattern (subset of
variables). For every abstract state that is forward-reachable from the initial
state, output its variable values, its perfect hash, and its optimal goal
distance in the abstract space.

- Input: SAS+ file followed by a line `s v_0 v_1 ... v_{s-1}` specifying the
  pattern variables (sorted ascending).
- Output: one line per reachable abstract state, in lexicographic order of the
  value tuple. Each line: `a_0 a_1 ... a_{s-1} hash dist` (or `inf` if the
  goal is unreachable from that abstract state).

## Chain of Thought

### Step 1 -- Understand the PDB construction

A PDB precomputes the exact cost of reaching the goal in a **projected**
(abstracted) state space. The projection keeps only the pattern variables; all
other variables are dropped. Because the projection removes preconditions on
non-pattern variables, abstract operators are more permissive -- meaning the
PDB heuristic is admissible (never overestimates).

Three components:
1. **Abstract transition system** -- operators projected onto the pattern.
2. **Forward reachability** -- BFS from the abstract initial state.
3. **Goal distances** -- backward Dijkstra from abstract goal states.

### Step 2 -- Operator projection

For each SAS+ operator, keep only:
- Prevail conditions on pattern variables.
- Effects that change a pattern variable, with their conditions restricted to
  pattern variables.
- Pre-values of effects on pattern variables (these become additional
  preconditions in the abstract space).

Operators with no effects on pattern variables are discarded (they're abstract
self-loops).

Conditions/pre-values on non-pattern variables are dropped. This makes
abstract operators more permissive, preserving admissibility.

### Step 3 -- The perfect hash function

The hash is a little-endian mixed-radix encoding:

```
hash(a_0, ..., a_{s-1}) = a_0 + a_1 * d_0 + a_2 * d_0 * d_1 + ...
```

where `d_i` is the domain size of the i-th pattern variable. This maps each
abstract state to a unique integer in `[0, product(d_i))`.

This was reverse-engineered from the sample outputs:
- Sample 1: pattern {var1, var2}, both domain 2.
  `(0,0)→0, (1,0)→1, (0,1)→2, (1,1)→3` confirms `hash = a_0 + a_1 * 2`.
- Sample 6: pattern {var0, var12, var20}, domains 20, 2, 2.
  `(0,0,1)→40, (0,1,0)→20, (1,0,0)→1` confirms `hash = a_0 + a_1*20 + a_2*40`.

### Step 4 -- Forward BFS

Standard BFS from the abstract initial state. Marks all reachable abstract
states. Only reachable states appear in the output.

### Step 5 -- Backward Dijkstra

Goal states: all abstract states satisfying the goal conditions on pattern
variables. For pattern variables not mentioned in the goal, any value qualifies.

Build forward transition edges over the entire abstract space, then reverse
them. Run Dijkstra on the reversed graph from all goal states (distance 0) to
compute the minimum-cost path from each state to the nearest goal state.

States that are forward-reachable but have `dist == usize::MAX` are dead ends
-- they output `inf`.

### Step 6 -- Output ordering

States are listed in lexicographic order of the value tuple `(a_0, ...,
a_{s-1})`. Since the hash is little-endian, hash order does not match
lexicographic order -- so we decode each hash, sort by the tuple, then output.

## Implementation Notes

- The abstract space is enumerated explicitly (`total_states = prod(d_i)`).
- Forward edges are built by iterating over all abstract states and all
  projected operators, checking preconditions, and computing successors.
- The reversed adjacency list is built simultaneously and used for the
  backward Dijkstra.
- Both easy and hard problems use the same implementation. The abstract space
  sizes in the test samples are at most a few thousand entries, so there's no
  need for specialized optimizations.

## Verification

All 17 official samples pass with exact string matching:
- 11 PDB-easy samples.
- 6 PDB-hard samples.

## Submission

Submit two binaries to DOMjudge (one for each problem ID):

- **04-pdb** for both `pdb-easy` and `pdb-hard` (same binary).

Prepare with: `scripts/prepare.sh 04-pdb`

## Running Tests Locally

```bash
cargo test --test 04-pdb
```

Expected: 17 tests pass.
