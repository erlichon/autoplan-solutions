# 02-Magnets-1: PDDL Modelling

**Problem ID:** Modelling 2 -- Magnets 1

## Problem Statement

Model the movement of two magnets `m1`, `m2` on a 3x3 grid in PDDL.

- Locations are coordinates `l0`, `l1`, `l2`; a cell is a pair `(li, lj)`.
- Initial positions: `m1 = (l0, l0)`, `m2 = (l1, l1)`.
- Goal: both magnets at `(l0, l2)`.
- Constraint: **no magnet may ever have been at `(l2, l2)`** at any point during the plan.
- Two actions with fixed signatures:
  - `move(?m, ?x, ?y, ?xnew, ?ynew)` -- 1-tile horizontal/vertical move
    (Manhattan distance `= 1`).
  - `attach(?m, ?n, ?x, ?y)` -- only if both magnets are at `(?x, ?y)`,
    `?m != ?n`, and the pair has not been attached before. Once attached,
    moving one magnet also moves the other.

The grader compares our plan set against a reference model's plan set, so
arity, action names, object names, and semantics must match.

## Chain of Thought

### Step 1 -- Read the spec carefully

Five fixed external commitments:

1. Action names `move`/`attach` with arities 5 and 4 respectively.
2. Parameter typing: `?m ?n` are one type (magnets), `?x ?y ?xnew ?ynew` are
   another (coordinates).
3. Object names: `m1`, `m2`, and `l0`, `l1`, `l2` (from the problem text).
4. The forbidden tile `(l2, l2)` is a *throughout-plan* invariant, not just a
   goal condition -- so it must be encoded as an action precondition, not as a
   goal state check.
5. Attach is symmetric ("the pair was attached") and one-shot ("not attached
   again").

### Step 2 -- Decide what is modellable with plain STRIPS vs. extensions

Three pieces of semantics are not trivial in plain STRIPS:

| Semantic requirement                          | How we encode it                                                                            |
|-----------------------------------------------|---------------------------------------------------------------------------------------------|
| "Manhattan distance exactly 1"                | A static 4-ary `(adj ?x1 ?y1 ?x2 ?y2)` predicate listing all 24 grid edges.                 |
| "`?m != ?n`" in `attach`                      | A static `(different ?m ?n)` predicate (`m1 m2` and `m2 m1`). Avoids needing `:equality`.   |
| "Moving an attached magnet drags the partner" | A conditional universal effect: `(forall (?n) (when (attached ?m ?n) ...))`.                |
| "`(l2, l2)` never visited"                    | A static `(forbidden ?x ?y)` predicate + `(not (forbidden ?xnew ?ynew))` precondition.      |

Using 4-ary adjacency and static "different" / "forbidden" predicates keeps
the requirements minimal -- only `:conditional-effects` and
`:negative-preconditions` beyond basic STRIPS -- and keeps preconditions free
of disjunctions and equality.

### Step 3 -- Watch the attach-symmetry trap

If `attach(?m, ?n)` only set `(attached ?m ?n)`, then after `(attach m1 m2)`
the planner could still fire `(attach m2 m1)` -- because `(attached m2 m1)`
has not been asserted. That would admit plans the reference model forbids
("attached pair cannot be attached again").

Fix: `attach` sets **both directions** symmetrically:

```pddl
:effect (and (attached ?m ?n) (attached ?n ?m))
```

This mirrors the `open` bug/fix in `02-cleaning-robot-1.md`: whenever a
relation is semantically symmetric, set both directions on the effect.

### Step 4 -- Watch the forbidden-tile trap

We must prevent **ever being at `(l2, l2)`**, not just being there in the
goal. Two observations make this easy:

- Both initial positions are not `(l2, l2)`.
- The only way to become located at `(l2, l2)` is via a `move` action.

So adding `(not (forbidden ?xnew ?ynew))` as a precondition of `move` is
enough. And because attached magnets are always co-located (attach requires
same cell; move moves both synchronously), blocking `?m` from entering
`(l2, l2)` also blocks the dragged partner from entering it.

### Step 5 -- Watch the "dragged partner" semantics

```pddl
(forall (?n - magnet)
  (when (attached ?m ?n)
    (and (not (at ?n ?x ?y)) (at ?n ?xnew ?ynew))))
```

This iterates over every magnet `?n`. For `?n = ?m`, `(attached ?m ?m)` is
false (we never assert self-attachment), so the body is skipped -- no
interference with the primary effect on `?m`. For the attached partner, the
body fires and drags it.

## Final Model

### Requirements

```
(:requirements :strips :typing :negative-preconditions :conditional-effects)
```

- `:negative-preconditions` for `(not (attached ...))` and `(not (forbidden ...))`.
- `:conditional-effects` for `(forall ... (when ... ...))` in `move`'s effect.

### Types

Two types: `magnet`, `coord`.

### Predicates

| Predicate                           | Meaning                                          | Dynamic? |
|-------------------------------------|--------------------------------------------------|----------|
| `(at ?m - magnet ?x ?y - coord)`    | magnet `?m` is at `(?x, ?y)`                     | yes      |
| `(attached ?m ?n - magnet)`         | `?m` and `?n` are attached (stored symmetrically)| yes      |
| `(adj ?x1 ?y1 ?x2 ?y2 - coord)`     | 4-ary grid adjacency, Manhattan = 1              | static   |
| `(different ?m ?n - magnet)`        | `?m != ?n`, listed as `(m1,m2)` and `(m2,m1)`    | static   |
| `(forbidden ?x ?y - coord)`         | true only for `(l2, l2)`                         | static   |

### Actions

`move(?m, ?x, ?y, ?xnew, ?ynew)`

- **Precondition**: `(at ?m ?x ?y)`, `(adj ?x ?y ?xnew ?ynew)`,
  `(not (forbidden ?xnew ?ynew))`.
- **Effect**: update `at` for `?m`, and for every `?n` with `(attached ?m ?n)`
  update `?n` too.

`attach(?m, ?n, ?x, ?y)`

- **Precondition**: `(at ?m ?x ?y)`, `(at ?n ?x ?y)`, `(different ?m ?n)`,
  `(not (attached ?m ?n))`.
- **Effect**: `(attached ?m ?n)` and `(attached ?n ?m)`.

### Problem file

- Objects: `m1 m2 - magnet`, `l0 l1 l2 - coord`.
- Init: `(at m1 l0 l0)`, `(at m2 l1 l1)`, `(different m1 m2)`,
  `(different m2 m1)`, `(forbidden l2 l2)`, plus all 24 `(adj ...)` edges of
  the 3x3 grid.
- Goal: `(at m1 l0 l2) and (at m2 l0 l2)`.

## Verification

A Python semantic simulator in `pddl/magnets-1/test_model.py` mirrors the
PDDL semantics and performs BFS plan enumeration.

### Semantic tests (all pass)

- Initial state is not the goal.
- Staying still and diagonal moves rejected.
- Self-attach rejected. Non-colocated attach rejected.
- Moving into `(l2, l2)` rejected from any adjacent cell.
- `attach` is one-shot: after `(attach m1 m2 l0 l1)`, neither
  `(attach m1 m2 l0 l1)` nor `(attach m2 m1 l0 l1)` is applicable.
- Moving an attached magnet drags the partner to the new cell.
- A 4-step no-attach plan and a 4-step attach plan both reach the goal.

### Plan enumeration (depth 4)

The model admits exactly **20 optimal plans of length 4**. They partition
into three families:

1. **Parallel walks, no attach** -- each magnet takes its own 2-step path to
   `(l0, l2)`. Both magnets have two shortest paths (`m1` via `(l0, l1)`;
   `m2` via `(l0, l1)` or `(l1, l2)`), and the four moves can be interleaved
   in any order that respects each magnet's per-magnet ordering.
2. **Attach then drag** -- meet at `(l0, l1)`, attach, then one single move
   drags both to `(l0, l2)`. Because the move and attach actions each admit
   two orderings of the two magnets, four attach-variants exist for each
   pre-attach move ordering.
3. **Mixed** -- a variation where the two magnets cross through `(l0, l1)`
   without attaching, with various interleavings.

The count of 20 is consistent with the reference model's symmetries:
`attach(m1, m2)` and `attach(m2, m1)` each produce an applicable, distinct
action (same effect on state, different action tuple in the plan), so both
appear in the plan set -- which matches the grader's expectation of treating
`attach` as a parameterised action name.

## Submission

Submit two files to DOMjudge (language: PDDL):

- `pddl/magnets-1/domain.pddl`
- `pddl/magnets-1/problem.pddl`

## Running Tests Locally

```bash
python3 pddl/magnets-1/test_model.py
```

Expected output ends with `All tests passed.` and lists 20 optimal plans at
depth 4.
