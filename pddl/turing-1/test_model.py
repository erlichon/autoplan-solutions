"""Verify the PDDL Turing Machine model against the reference plan set."""

import re
import sys

POSITIONS = ["p0", "p1", "p2", "p3", "p4", "p5", "p6", "p7"]
POS_IDX = {p: i for i, p in enumerate(POSITIONS)}
STATES = ["s0", "s1", "s2", "s3"]
SYMBOLS = ["zero", "one", "blank"]
ACCEPTING = {"s3"}

NEXT_RIGHT = {POSITIONS[i]: POSITIONS[i + 1] for i in range(7)}
NEXT_LEFT = {POSITIONS[i + 1]: POSITIONS[i] for i in range(7)}

DELTA = {
    ("s0", "zero", "s0", "zero", "right"),
    ("s0", "one",  "s0", "one",  "right"),
    ("s0", "one",  "s1", "zero", "right"),
    ("s0", "zero", "s2", "one",  "right"),
    ("s1", "zero", "s1", "zero", "right"),
    ("s1", "one",  "s1", "one",  "right"),
    ("s1", "zero", "s3", "one",  "left"),
    ("s2", "zero", "s2", "zero", "right"),
    ("s2", "one",  "s2", "one",  "right"),
    ("s2", "one",  "s3", "zero", "left"),
    ("s3", "zero", "s3", "zero", "left"),
    ("s3", "one",  "s3", "one",  "left"),
    ("s3", "blank","s0", "blank","right"),
}

INIT_TAPE = ["blank", "zero", "one", "zero", "one", "one", "zero", "blank"]
INIT_HEAD = 1
INIT_STATE = "s0"


def validate_plan(plan_str):
    """Check if a plan is valid in our model. Returns (ok, reason)."""
    head = INIT_HEAD
    st = INIT_STATE
    tape = list(INIT_TAPE)
    halted = False

    actions = re.findall(r"\((\w+)\s+([^)]+)\)", plan_str)
    if not actions:
        return False, "empty plan"

    for i, (name, args_str) in enumerate(actions):
        args = args_str.split()
        if halted:
            return False, f"step {i}: action after halt"

        if name == "transition":
            if len(args) != 7:
                return False, f"step {i}: wrong arity {len(args)}"
            p, p2, s, s2, x, x2, d = args
            if POSITIONS[head] != p:
                return False, f"step {i}: head at {POSITIONS[head]} != {p}"
            if st != s:
                return False, f"step {i}: state {st} != {s}"
            if tape[head] != x:
                return False, f"step {i}: tape[{head}]={tape[head]} != {x}"
            if (s, x, s2, x2, d) not in DELTA:
                return False, f"step {i}: no delta({s},{x},{s2},{x2},{d})"
            nxt = NEXT_RIGHT if d == "right" else NEXT_LEFT
            if p not in nxt or nxt[p] != p2:
                return False, f"step {i}: bad move {p}->{p2} via {d}"
            tape[head] = x2
            head = POS_IDX[p2]
            st = s2

        elif name == "halt":
            if len(args) != 3:
                return False, f"step {i}: wrong arity {len(args)}"
            p, s, x = args
            if POSITIONS[head] != p:
                return False, f"step {i}: head at {POSITIONS[head]} != {p}"
            if st != s:
                return False, f"step {i}: state {st} != {s}"
            if tape[head] != x:
                return False, f"step {i}: tape[{head}]={tape[head]} != {x}"
            if s not in ACCEPTING:
                return False, f"step {i}: {s} not accepting"
            if x != "blank":
                return False, f"step {i}: {x} not blank"
            halted = True
        else:
            return False, f"step {i}: unknown action {name}"

    if not halted:
        return False, "plan does not end with halt"
    return True, "ok"


def enumerate_short_plans(max_depth=10):
    """DFS enumerate all plans up to max_depth to check for spurious plans."""
    plans = []
    trace = []

    delta_list = list(DELTA)

    def dfs(head, st, tape, depth):
        if depth > max_depth:
            return
        cur_sym = tape[head]

        if st in ACCEPTING and cur_sym == "blank":
            action = f"(halt {POSITIONS[head]} {st} {cur_sym})"
            trace.append(action)
            plans.append("".join(trace))
            trace.pop()

        for s, x, s2, x2, d in delta_list:
            if s != st or x != cur_sym:
                continue
            nxt = NEXT_RIGHT if d == "right" else NEXT_LEFT
            p_name = POSITIONS[head]
            if p_name not in nxt:
                continue
            p2_name = nxt[p_name]
            new_tape = tape[:head] + [x2] + tape[head + 1:]
            action = f"(transition {p_name} {p2_name} {s} {s2} {x} {x2} {d})"
            trace.append(action)
            dfs(POS_IDX[p2_name], s2, new_tape, depth + 1)
            trace.pop()

    dfs(INIT_HEAD, INIT_STATE, list(INIT_TAPE), 0)
    return plans


def load_reference(path="../../tests/samples/04-Turing/1.ans"):
    with open(path) as f:
        return set(line.strip() for line in f if line.strip())


def main():
    ref_set = load_reference()
    print(f"Reference has {len(ref_set)} plans")

    # 1) Validate every reference plan against our model
    fail = 0
    for plan in ref_set:
        ok, reason = validate_plan(plan)
        if not ok:
            fail += 1
            if fail <= 5:
                short = plan[:120] + "..." if len(plan) > 120 else plan
                print(f"  FAIL: {reason}")
                print(f"    {short}")
    print(f"Validation: {len(ref_set) - fail}/{len(ref_set)} pass, {fail} fail")

    # 2) Enumerate short plans and verify they're all in the reference
    print("\nEnumerating short plans (depth <= 10)...")
    short_plans = enumerate_short_plans(max_depth=10)
    print(f"  Found {len(short_plans)} short plans")
    spurious = [p for p in short_plans if p not in ref_set]
    if spurious:
        print(f"  {len(spurious)} SPURIOUS (in model, not in reference):")
        for p in spurious[:5]:
            print(f"    {p}")
    else:
        print("  All short plans are in the reference set.")

    if fail == 0 and not spurious:
        print("\nPERFECT MATCH (within tested depth)")
    else:
        print(f"\nISSUES: {fail} invalid reference plans, {len(spurious)} spurious plans")
        sys.exit(1)


if __name__ == "__main__":
    main()
