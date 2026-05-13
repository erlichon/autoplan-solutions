# 05-Lamps: PDDL Modelling

**Problem ID:** Modelling 4 -- Lamp Game

## Problem Statement

Model a 3x3 grid of lamps. Each lamp is either on or off. The single
user-facing action is `flip(?x, ?y)`, which toggles the lamp at that cell
**and** propagates the toggle along straight horizontal and vertical lines
through consecutive lamps that share the same pre-flip state.

- Grid coordinates: `l0`, `l1`, `l2`. A cell is a pair `(li, lj)`.
- Initially ON: `(l0,l0)`, `(l1,l0)`, `(l2,l0)`, `(l0,l1)`, `(l0,l2)`.
- Goal: every lamp OFF.

## Chain of Thought

### Step 1 -- Understand the propagation rule

The propagation is **straight-line**, not flood fill. When flipping a cell
C that was ON, extend in each of the 4 cardinal directions as long as
consecutive cells are also ON; those cells all turn OFF together. Same
logic (mirrored) for an OFF cell.

I verified this reading against the 5x5 example in the problem sheet:
flipping the center of that grid toggles a cross-shaped region along the
rows and columns, stopping where it hits a cell of different state. Flood
fill would turn OFF many more cells (including ones reachable by
L-shaped paths), which contradicts the expected 5x5 output.

### Step 2 -- Choose a modelling approach

For N=3 the maximum propagation distance in any direction is 2 cells.
This means all chain conditions can be baked into **conditional effects**
of a single `flip` action using `forall` quantifiers -- no technical
actions are needed.

Benefits:
- The planner sees branching factor 9 (one `flip` per cell), which is
  small enough for plan enumeration to work efficiently.
- No risk of the planner being slowed by chains of technical actions
  with branching factor > 1.

### Step 3 -- Design the predicates

| Predicate               | Meaning                  | Dynamic? |
|--------------------------|--------------------------|----------|
| `(on ?x ?y - coord)`    | lamp at `(?x,?y)` is on  | yes      |
| `(succ ?a ?b - coord)`  | `?b = ?a + 1`            | static   |

`succ` is used to express adjacency in each direction.

### Step 4 -- Design the action

`flip(?x, ?y)` has **no precondition** and a list of conditional effects:

1. **Toggle center**: two `when` clauses (one for ON→OFF, one for OFF→ON).
2. **Per direction** (+x, −x, +y, −y), two chain lengths:
   - **Distance 1**: if the 1st neighbour shares the center's pre-state,
     toggle it.
   - **Distance 2**: if both the 1st and 2nd neighbours share the
     center's pre-state, toggle the 2nd.

Each chain-length has two `when` clauses (ON case and OFF case), giving
16 forall-conditional effects for propagation plus 2 for the center
toggle (18 total).

All conditions evaluate the **pre-state** (standard PDDL conditional-effect
semantics), which is exactly what the problem requires -- propagation is
determined before any toggles are applied.

### Step 5 -- Correctness argument

- No cell appears in two propagation directions from the same center, so
  there are no conflicting add/delete effects.
- `(succ ?x ?x)` is never true, so the forall never accidentally touches
  the center cell.
- The action has no precondition; any cell can be flipped at any time.

### Requirements

```
(:requirements :strips :typing :negative-preconditions :conditional-effects)
```

`:conditional-effects` for the `when`/`forall` effects. `:negative-preconditions`
for the `(not (on ...))` conditions in the OFF-propagation clauses.

## Verification

A Python test in `pddl/lamps-1/test_model.py` mirrors the PDDL semantics:

1. **Reference validation**: all 50,000 reference plans (lengths 1--7) are
   replayed step-by-step. All 50,000 pass.
2. **Depth-count comparison**: the model's plan counts at depths 1--5 match
   the reference exactly (1, 8, 22, 390, 2480).

## Submission

Submit two files to DOMjudge (language: PDDL):

- `pddl/lamps-1/domain.pddl`
- `pddl/lamps-1/problem.pddl`

## Running Tests Locally

```bash
cd pddl/lamps-1 && python3 test_model.py
```

Expected output: `ALL OK: reference validated, depth 1-5 counts match exactly`.
