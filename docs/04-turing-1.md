# 04-Turing-1: PDDL Modelling

**Problem ID:** Modelling 2 -- Turing Machine

## Problem Statement

Model the execution of a specific Turing Machine inside a PDDL planning
problem. The domain should encode generic TM mechanics; only the problem file
should encode the specific machine.

- 4 states: `s0` (initial), `s1`, `s2`, `s3` (accepting).
- 3 tape symbols: `zero`, `one`, `blank`.
- 8-cell tape: `p0`..`p7`. Head starts at `p1`.
- Initial tape: `blank zero one zero one one zero blank`.
- The machine is nondeterministic (multiple transitions may apply from one
  configuration).
- Halting: the machine may halt **only** in an accepting state with `blank`
  under the head.
- Two actions with fixed signatures:
  - `transition(?p, ?p2, ?s, ?s2, ?x, ?x2, ?d)` -- apply a delta rule.
  - `halt(?p, ?s, ?x)` -- final action; nothing may follow.

## Chain of Thought

### Step 1 -- Understand the TM execution model

A Turing Machine configuration is `(head-position, state, tape-contents)`.
Each step either applies a transition (read symbol, write replacement, move
head, change state) or halts. After halting, no further actions are allowed.

The planner explores **all** nondeterministic choices, so valid plans are all
possible execution traces that reach the halt condition.

### Step 2 -- Identify what belongs in the domain vs. the problem

The domain encodes the **mechanics** of any TM:

- A `transition` action that looks up the delta function, updates the tape,
  moves the head, and changes state.
- A `halt` action that checks the accepting/blank conditions and locks the
  machine.

The problem file encodes the **specific** machine:

- Objects (positions, states, symbols, directions).
- The delta function as a static predicate `(delta ?s ?x ?s2 ?x2 ?d)`.
- Tape adjacency as a static predicate `(next ?p ?p2 ?d)`.
- Initial tape contents, head position, and state.
- Which states are accepting and which symbol is blank.

### Step 3 -- Read the transition diagram

From the state diagram (and validated against the reference plan set):

| State | Read  | Write | Move  | Next |
|-------|-------|-------|-------|------|
| s0    | zero  | zero  | right | s0   |
| s0    | one   | one   | right | s0   |
| s0    | one   | zero  | right | s1   |
| s0    | zero  | one   | right | s2   |
| s1    | zero  | zero  | right | s1   |
| s1    | one   | one   | right | s1   |
| s1    | zero  | one   | left  | s3   |
| s2    | zero  | zero  | right | s2   |
| s2    | one   | one   | right | s2   |
| s2    | one   | zero  | left  | s3   |
| s3    | zero  | zero  | left  | s3   |
| s3    | one   | one   | left  | s3   |
| s3    | blank | blank | right | s0   |

13 transitions total. States `s0`, `s1`, `s2` move right (scanning); `s3`
moves left (rewinding). The `s3 → s0` transition on blank restarts a scan,
creating a loop that generates the large set of valid plans (50,000 in the
reference sample).

### Step 4 -- Design the predicates

| Predicate                          | Meaning                              | Dynamic? |
|------------------------------------|--------------------------------------|----------|
| `(head ?p - position)`             | head is at `?p`                      | yes      |
| `(current-state ?s - state)`       | machine is in state `?s`             | yes      |
| `(tape ?p - position ?x - symbol)` | cell `?p` contains `?x`             | yes      |
| `(halted)`                         | machine has halted                   | yes      |
| `(delta ?s ?x ?s2 ?x2 ?d)`        | transition rule lookup               | static   |
| `(next ?p ?p2 ?d)`                 | `?p2` is the neighbour of `?p` in direction `?d` | static |
| `(accepting ?s - state)`           | `?s` is an accepting state           | static   |
| `(is-blank ?x - symbol)`           | `?x` is the blank symbol             | static   |

### Step 5 -- Design the actions

**`transition(?p, ?p2, ?s, ?s2, ?x, ?x2, ?d)`**

- Precondition: `(head ?p)`, `(current-state ?s)`, `(tape ?p ?x)`,
  `(delta ?s ?x ?s2 ?x2 ?d)`, `(next ?p ?p2 ?d)`, `(not (halted))`.
- Effect: move head from `?p` to `?p2`, change state from `?s` to `?s2`,
  overwrite `tape[?p]` from `?x` to `?x2`.

When `?x == ?x2` (or `?s == ?s2`), STRIPS semantics applies deletes before
adds, so the fact is correctly preserved.

**`halt(?p, ?s, ?x)`**

- Precondition: `(head ?p)`, `(current-state ?s)`, `(tape ?p ?x)`,
  `(accepting ?s)`, `(is-blank ?x)`, `(not (halted))`.
- Effect: `(halted)`.

The `(not (halted))` precondition on both actions ensures nothing fires after
halt. The goal `(halted)` forces every valid plan to end with `halt`.

### Step 6 -- Tape boundaries

The `(next ...)` predicate only lists adjacent pairs within `p0..p7`.
There is no `(next p7 ? right)` or `(next p0 ? left)`, so transitions that
would move the head off the tape simply have no applicable grounding -- no
extra boundary checks needed.

### Requirements

```
(:requirements :strips :typing :negative-preconditions)
```

Only `:negative-preconditions` beyond basic STRIPS, for `(not (halted))`.

## Verification

A Python test in `pddl/turing-1/test_model.py` mirrors the PDDL semantics:

1. **Reference validation**: Every reference plan (50,000 plans, lengths 4--44)
   is replayed step-by-step against the model's transition rules. All 50,000
   pass.
2. **Spurious-plan check**: All plans generated by DFS up to depth 20 (143
   plans) are verified to be present in the reference set. Zero spurious.

## Submission

Submit two files to DOMjudge (language: PDDL):

- `pddl/turing-1/domain.pddl`
- `pddl/turing-1/problem.pddl`

## Running Tests Locally

```bash
cd pddl/turing-1 && python3 test_model.py
```

Expected output: `PERFECT MATCH (within tested depth)`.
