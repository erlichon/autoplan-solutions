# 02-Cleaning-Robot-1: PDDL Modelling

**Problem ID:** Modelling 1 - Cleaning Robot 1

## Problem Statement

Model a cleaning robot domain in PDDL. Four rooms (A, B, C, D) in a grid layout, all connections initially locked. The robot starts in A, has keys for all connections, and must clean rooms B and C.

## Room Layout

```
A --- B
|     |
C --- D
```

Connections: A-B, A-C, B-D, C-D. No diagonal connections (no A-D, no B-C).

## Final (Correct) Domain Design -- v5

### Requirements

```
(:requirements :strips :typing :negative-preconditions)
```

`:negative-preconditions` is required because `open` uses `(not (unlocked ?x ?y))` and `clean` uses `(not (clean ?x))`.

### Types

A single type `room`.

### Predicates

| Predicate | Meaning |
|-----------|---------|
| `(robot-at ?x)` | Robot is in room ?x |
| `(connected ?x ?y)` | Rooms ?x and ?y are directly connected |
| `(unlocked ?x ?y)` | The path from ?x to ?y is unlocked |
| `(clean ?x)` | Room ?x has been cleaned |

Only four predicates. No `locked` predicate.

### Actions

#### `open(?x, ?y)`
- **Precondition**: Robot at ?x, ?x-?y connected, `(not (unlocked ?x ?y))`
- **Effect**: `(unlocked ?x ?y)` AND `(unlocked ?y ?x)` -- **BOTH directions**
- Since the effect sets both directions, after `open a b`, `open b a` is automatically blocked (its precondition `(not (unlocked b a))` is already false)
- This correctly enforces "you cannot unlock the same connection twice"

#### `drive(?x, ?y)`
- **Precondition**: Robot at ?x, ?x-?y connected, ?x->?y unlocked
- **Effect**: Robot moves to ?y, no longer at ?x

#### `clean(?x)`
- **Precondition**: Robot at ?x, `(not (clean ?x))`
- **Effect**: Room ?x is clean
- **Non-idempotent**: cannot clean a room that is already clean

## Shortest Plans (7 steps)

With bidirectional open and non-idempotent clean, there are exactly 6 optimal 7-step plans:

```
(open a b)(drive a b)(clean b)(drive b a)(open a c)(drive a c)(clean c)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(clean c)
(open a b)(open a c)(drive a c)(clean c)(drive c a)(drive a b)(clean b)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a b)(clean b)
(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(clean c)
(open a c)(open a b)(drive a c)(clean c)(drive c a)(drive a b)(clean b)
```

All patterns go through room A, opening connections from there and visiting B and C.

---

## What Went Wrong: Five Failed Approaches

This problem took 5 attempts. Understanding the failures is instructive.

### Two Independent Bugs

The model had TWO bugs that needed to be fixed simultaneously:

1. **Open effect directionality** (affects optimal plan length)
2. **Clean action idempotency** (affects total plan count)

Because both needed to be correct at the same time, fixing only one still produced wrong-answer.

### Bug 1: Unidirectional `open` Effect (v4)

In attempt 4, the `open` action only set ONE direction:
```pddl
:effect (unlocked ?x ?y)   ;; WRONG -- only one direction
```

This treated A->B and B->A as separate "locks." The robot needed `open a b` to go A->B AND a separate `open b a` to go B->A. This produces 8-step shortest plans instead of 7.

**The correct interpretation:** The problem says connections are "bidirectional" and "you cannot unlock the same connection twice." A-B is ONE connection. Opening it from either side unlocks both directions.

### Bug 2: Idempotent `clean` Action (v1--v4)

In all attempts 1--4, the `clean` action had NO check on whether the room was already clean:
```pddl
:precondition (robot-at ?x)   ;; WRONG -- allows re-cleaning
```

The problem says "a room that is clean can still be cleaned again." This was interpreted as "the clean action IS applicable to clean rooms." However, the reference model requires `(not (clean ?x))` -- the clean action can only be applied to a room that is NOT already clean.

**Why this matters:** With idempotent clean, the robot can insert `(clean b)` after b is already clean, generating thousands of extra plans. These extra plans don't exist in the reference model.

**Why this is reasonable:** Cleaning an already-clean room is a no-op. In planning, no-op actions don't contribute to the goal and are typically prevented by preconditions to keep the plan space manageable.

### Attempt History

| Version | Open Effect | Clean Precondition | Optimal Plans | Result |
|---------|-------------|-------------------|---------------|--------|
| v1 | bidir (`locked`+`unlocked` predicates) | idempotent | 7-step | wrong-answer (19 SAS+ vars instead of 11) |
| v2 | bidir (no re-open check) | idempotent | 7-step | wrong-answer (extra double-open plans) |
| v3 | bidir (`not unlocked`) | idempotent | 7-step | wrong-answer (extra re-cleaning plans) |
| v4 | **unidir** (`not unlocked`) | idempotent | **8-step** | wrong-answer (wrong plan length + extra re-cleaning) |
| v5 | bidir (`not unlocked`) | **non-idempotent** | 7-step | **expected correct** |

### Key Takeaway

When DOMjudge says wrong-answer, the issue could be:
- **Wrong optimal plan length** (different action semantics, like uni- vs bidirectional open)
- **Extra plans at non-optimal lengths** (actions that are too permissive, like idempotent clean)
- Both at the same time!

Always check the Diff output on DOMjudge to see which plans differ.

---

## Verification

- **Fast Downward translation**: 11 variables, 18 operators
- **Semantic tests**: bidirectional open confirmed, non-idempotent clean confirmed
- **Cross-variant analysis**: 6 model variants tested, plan sets compared pairwise
- **Plan enumeration**: 6 optimal plans at depth 7, 4770 total plans up to depth 11

## Submission

Submit two files to DOMjudge (language: PDDL):
- `domain.pddl`
- `problem.pddl`

## Running Tests Locally

```bash
# Translate PDDL to SAS+ with Fast Downward
cd /tmp/downward
python3 fast-downward.py --translate \
  path/to/domain.pddl path/to/problem.pddl

# Run semantic tests + plan enumeration
python3 pddl/cleaning-robot-1/test_model.py

# Compare all model variants (A through F)
python3 pddl/cleaning-robot-1/compare_variants.py
```
