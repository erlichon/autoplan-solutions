# 06-invar: h^2 Binary Invariants

**Problem IDs:** h2-invar-easy (L), h2-invar-hard (M)

## Problem Statement

Given an SAS+ file, extract all binary invariants. A binary invariant is
a clause `L1 \/ L2` over two literals that holds in every state reachable
from the initial state.

- Input: SAS+ file on stdin.
- Output: one invariant per line, lexicographically sorted as strings.

Each literal is `sign var val` where sign is `+` (positive) or `-`
(negated). For each invariant the first literal's fact (v1, x1) is
lexicographically smaller than the second's (v2, x2).

## Chain of Thought

### Step 1 -- What are "h^2 invariants"?

Despite the name, "h^2" here means *binary* (two-literal) disjunctive
invariants, not invariants derived from the Haslum-Geffner h^2 cost
table. The algorithm taught in lecture 12 is the **Rintanen (1998)
filter-refinement** approach.

### Step 2 -- Literal representation

FDR facts `(v, d)` map to two literals each: positive `(v=d)` and
negative `¬(v=d)`. With F total facts we have 2F literals. We encode
them as integers: positive = `2*fact_id`, negative = `2*fact_id + 1`,
so negation is just `lit ^ 1`.

### Step 3 -- Algorithm (Rintanen filter-refinement)

1. **Initialise** with every binary disjunction `l1 ∨ l2` that is true
   in the initial state (at least one of the two literals holds).

2. **Filter** — for each action `a = (pre, eff)`:
   - Compute the set of known literals from `pre` (including derived
     negatives from the FDR single-value constraint).
   - **Unit-propagate** using the current invariant set: if `¬l` is
     known and `l ∨ l'` is an invariant, then `l'` is known. Repeat
     to fixpoint. If a contradiction arises, the action can never fire
     from a state satisfying the invariants — skip it.
   - Compute **U** (literals guaranteed true after the action):
     `U = (known \ {¬e | e ∈ eff}) ∪ eff`.
   - For each invariant `l1 ∨ l2`: if the action makes `l1` false
     (`¬l1 ∈ eff`) then `l2` must be in U; symmetrically for `l2`.
     Remove invariants that fail this check.

3. **Repeat** step 2 until no invariant is removed (fixpoint).

The self-reinforcing unit propagation (using I ∪ pre) is what
distinguishes this from a plain h^2 cost-table approach — the invariant
set itself helps prove that other invariants are preserved.

### Step 4 -- Conditional effects

For each conditional effect we check whether its conditions are entailed
by unit propagation (definitely fires), contradicted (definitely does
not fire), or unknown (uncertain). Uncertain effects are handled
conservatively: their deletes mark literals as affected, but their adds
are not counted as guaranteed in U.

### Step 5 -- Output

Collect surviving invariants for each ordered fact pair `(v1,x1) <
(v2,x2)` in all four sign combinations, format as strings, sort
lexicographically.

## Previous Approach and Why It Failed

The first implementation extracted invariants from the **Haslum-Geffner
h^2 cost table**: a pair with infinite cost was declared unreachable and
its negation emitted as an invariant. This passed all local samples but
received `wrong-answer` on DOMjudge's hidden tests.

The h^2 cost table tracks forward reachability of positive fact *pairs*
only. The Rintanen algorithm works on *literals* (positive and negative)
and uses **self-reinforcing reasoning** — the current invariant set
feeds back into the unit propagation that determines which invariants
survive each action. This lets it discover invariants that h^2's
pair-level abstraction misses.

During the h^2 approach we also fixed two real bugs in the cost table
(conditional-effect handling in the `assigned` vector, and skipping the
effect's own variable in the frame loop). Those fixes remain in `h2.rs`
for the 03-h2 heuristic problem.

## Running Tests Locally

```bash
cargo test --test 06-invar
```

## Submission

```bash
scripts/prepare.sh 06-invar
```
