# 07-LM-Cut: LM-Cut Heuristic

**Problem IDs:** lm-cut-easy (F), lm-cut-hard (F)

## Problem Statement

Given an SAS+ problem, compute the LM-cut heuristic value by iteratively
extracting disjunctive action landmarks from the delete-relaxed problem.

- Input: SAS+ file on standard input.
- Output: a sequence of blocks, one per LM-cut round. Each block has:
  - One line: current hmax value of the goal.
  - One line: N (landmark size) followed by N operator indices.
  - One line: the cost of this landmark.
  - The final block is a single `0` (hmax reached zero).

## Chain of Thought

### Step 1 -- Understand the LM-cut algorithm

LM-cut is an admissible heuristic that iteratively finds "landmark cuts"
in the delete-relaxed planning graph. Each round:

1. Compute hmax (h^1) under current operator costs.
2. If hmax(goal) = 0, stop.
3. Build a justification graph: for each operator, select the precondition
   with maximum hmax cost as the "supporter". Edges go from supporter to
   each effect, labeled with the operator cost.
4. Find V0 (the 0-cost goal zone): facts from which the goal is reachable
   using only 0-cost arcs in the justification graph.
5. The landmark cut L+ is the set of operators whose supporter is outside
   V0 but have an effect inside V0.
6. Subtract the minimum cost of L+ from all operators in L+.
7. Accumulate the minimum cost; repeat.

### Step 2 -- Unary operator representation

Each (operator, effect) pair becomes a "unary operator" with:
- Preconditions: prevail conditions + all effects' pre_values + this
  effect's conditional effect conditions.
- Effect: this effect's post_value.
- Cost: the operator's cost.

This representation enables per-effect hmax computation and supporter
selection. Precondition facts are sorted by ID and deduplicated for
deterministic tie-breaking.

### Step 3 -- Multi-goal handling via artificial goal operator

The algorithm assumes a single goal fact. For multi-goal problems, we
model an artificial goal operator whose preconditions are all goal facts
and whose effect is an artificial goal fact g_art. The supporter of this
artificial operator is the goal fact with maximum hmax cost (first in
goal list wins ties). V0 starts from this single fact and expands backward
through 0-cost arcs.

### Step 4 -- Virtual initial fact

Operators with no preconditions use a "virtual initial fact" as their
supporter. This fact is included in the zero-cost reverse graph so that
V0 correctly includes it when a 0-cost no-precondition operator has an
effect in V0.

### Step 5 -- Supporter tie-breaking

When multiple preconditions share the maximum hmax value, the smallest
fact ID wins (since pre_facts are sorted). This matches the reference
implementation's deterministic output.

## Algorithm

```
uops = build_unary_ops(operators)  // (op_idx, effect_fid, sorted pre_facts)
costs = [op.cost for op in operators]

loop:
    hmax = compute_hmax(state, costs, uops)
    h = max(hmax[g] for g in goal_facts)
    if h == 0: print 0; break

    supporters = for each uo: argmax_{p in uo.pre_facts} hmax[p]

    zero_rev = reverse edges of 0-cost arcs (effect -> supporter)

    start = first goal fact with hmax = h
    V0 = backward BFS from start through zero_rev

    landmark = {uo.op_idx : supporters[uo] not in V0  and  uo.effect in V0}
    lm_cost = min(costs[a] for a in landmark)

    print h, landmark, lm_cost
    for a in landmark: costs[a] -= lm_cost
```

## Verification

- All 8 LM-cut-easy samples pass with exact match.
- All 10 LM-cut-hard samples pass with exact match.

## Running Tests Locally

```bash
cargo test --test 06-lm-cut
```

## Submission

```bash
scripts/prepare.sh 06-lm-cut
```
