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
filter-refinement** approach (slide 26-28).

### Step 2 -- FDR-to-STRIPS conversion

The lecture defines the algorithm over STRIPS actions `a = (pre, add,
del)`. FDR facts `(v, d)` map to STRIPS facts. Each FDR fact gives two
literals: positive `(v=d)` and negative `¬(v=d)`. With F total facts we
have 2F literals.

For each FDR effect `(v, pre → post)`:

- `add = {(v=post)}`
- `del = {(v=pre)}` if `pre >= 0`, else `{(v=d) | d ≠ post}` (don't-care)
- `eff = add ∪ {¬f | f ∈ del}` (literal set of the action's effects)

The precondition includes all prevail conditions and all effects'
`pre_value`s (standard SAS+ global-precondition semantics).

### Step 3 -- Literal encoding

We encode literals as integers: positive = `2*fact_id`,
negative = `2*fact_id + 1`, so negation is just `lit ^ 1`. This makes
the adjacent-bit-swap trick possible for computing the negation mapping
over entire u64 words at once (see Step 7).

### Step 4 -- Algorithm (Rintanen filter-refinement)

1. **Initialise** with every binary disjunction `l1 ∨ l2` that is true
   in the initial state (at least one of the two literals holds).

2. **Filter** — for each action `a = (pre, eff)`:
   - Compute the set of known literals from `pre` (including derived
     negatives from the FDR single-value constraint: if `v=d` is a
     precondition, then `¬(v=d')` for every `d' ≠ d`).
   - **Unit-propagate** using the current invariant set: if `¬l` is
     known and `l ∨ l'` is an invariant, then `l'` is known. Repeat
     to fixpoint. If a contradiction arises (both `l` and `¬l` known),
     the action can never fire from a state satisfying the
     invariants — skip it.
   - Compute **U** (literals guaranteed true after the action):
     `U = (known \ {¬e | e ∈ eff}) ∪ eff`.
   - For each invariant `l1 ∨ l2`: if the action makes `l1` false
     (`¬l1 ∈ eff`) then `l2` must be in U; symmetrically for `l2`.
     Remove invariants that fail this check.

3. **Repeat** step 2 until no invariant is removed (fixpoint).

The self-reinforcing unit propagation (using I ∪ pre) is what
distinguishes this from a plain h^2 cost-table approach — the invariant
set itself helps prove that other invariants are preserved.

### Step 5 -- Conditional effects

For each conditional effect we check whether its conditions are entailed
by unit propagation (definitely fires), contradicted (definitely does
not fire), or unknown (uncertain). Uncertain effects are handled
conservatively: their deletes mark literals as affected (so the filter
checks invariants involving them), but their adds are not counted as
guaranteed in U (since the effect might not fire).

### Step 6 -- Output

Collect surviving invariants for each ordered fact pair `(v1,x1) <
(v2,x2)` in all four sign combinations, format as strings, sort
lexicographically.

## Algorithm

```
nl   = 2 * sum of domain sizes       (total literals)
inv  = nl x nl symmetric bit matrix

# Initialise: l1 ∨ l2 is candidate iff at least one holds in I
for each literal l:
  if l is true in I:  inv[l] = all literals
  else:               inv[l] = { l' : l' is true in I }

repeat until no change:
  for each operator o:
    known = literals from preconditions of o (including ¬(v=d') derivations)

    # Unit propagation
    queue = known
    while queue not empty:
      k = dequeue
      for each l' in inv[¬k]:
        if l' not known:
          known += l'
          enqueue l'
      if ¬k also known: contradiction → skip operator

    # Compute effects and U
    eff       = { (v=post) } ∪ { ¬(v=pre) }  for each firing effect
    affected  = { l : ¬l ∈ eff }
    U         = (known \ { ¬e : e ∈ eff }) ∪ eff

    # Filter
    for each affected literal l:
      for each l' in inv[l]:
        if l' ∉ U:
          remove l ∨ l'  (clear both inv[l][l'] and inv[l'][l])

# Output
for each (v1,x1) < (v2,x2), each sign combination:
  if invariant survives: emit line
sort lines lexicographically
```

## Performance: clearing the h2-hard timelimit

The naive implementation using `Vec<Vec<bool>>` for the invariant matrix
was O(nl) per literal per operation. For large inputs (nl in the
thousands, thousands of operators) this hit the time limit.

The fix: represent every row `inv[l]` and every working array (`known`,
`u`, `affected`, etc.) as `Vec<u64>` bitsets. This gives three speedups:

1. **Unit propagation**: `inv[¬k] AND NOT known` finds all new
   derivations in one word op per 64 literals. `trailing_zeros()` then
   enumerates only the set bits.

2. **U computation**: the negation mapping `l → l^1` swaps adjacent
   bits. Within a u64 word this is a single shift-and-mask:
   `neg = ((w >> 1) & 0x5555…) | ((w & 0x5555…) << 1)`. Then
   `U[wi] = (known[wi] & !neg) | def_eff[wi]`.

3. **Filter**: `inv[l] AND NOT u` identifies all invariants to remove
   in one word op per 64 literals. Symmetric removal iterates only the
   removed bits via `trailing_zeros()`.

Overall ~64× speedup on all inner loops. Memory for the matrix is
`nl * ceil(nl/64) * 8` bytes (e.g. 2 MB for nl = 4000).

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

## Verification

- All 5 invar-easy samples pass with exact string match.
- The 1 invar-hard sample passes with exact string match.
- All 21 h^2 heuristic tests still pass (no regression from the
  refactoring that separated `h2.rs` from `invar.rs`).

## Running Tests Locally

```bash
cargo test --test 06-invar
```

## Submission

```bash
scripts/prepare.sh 06-invar
```
