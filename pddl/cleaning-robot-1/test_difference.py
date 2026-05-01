#!/usr/bin/env python3
"""
Test specific plans that should differ between v1 and v2.
Plans with redundant 'open' on already-unlocked connections should be:
- INVALID in v1 (locked prevents double-open)
- VALID in v2 (open is idempotent)
"""

import re
from copy import deepcopy


CONNECTIONS = {("a", "b"), ("b", "a"), ("a", "c"), ("c", "a"),
               ("b", "d"), ("d", "b"), ("c", "d"), ("d", "c")}


def make_init_v1():
    return {
        "robot_at": "a",
        "unlocked": set(),
        "locked": {("a", "b"), ("b", "a"), ("a", "c"), ("c", "a"),
                   ("b", "d"), ("d", "b"), ("c", "d"), ("d", "c")},
        "clean": set(),
    }


def make_init_v2():
    return {"robot_at": "a", "unlocked": set(), "clean": set()}


def apply_v1(state, name, args):
    s = deepcopy(state)
    if name == "open":
        x, y = args
        if s["robot_at"] != x or (x, y) not in CONNECTIONS or (x, y) not in s["locked"]:
            return None
        s["unlocked"].add((x, y)); s["unlocked"].add((y, x))
        s["locked"].discard((x, y)); s["locked"].discard((y, x))
        return s
    elif name == "drive":
        x, y = args
        if s["robot_at"] != x or (x, y) not in CONNECTIONS or (x, y) not in s["unlocked"]:
            return None
        s["robot_at"] = y
        return s
    elif name == "clean":
        if s["robot_at"] != args[0]:
            return None
        s["clean"].add(args[0])
        return s
    return None


def apply_v2(state, name, args):
    s = deepcopy(state)
    if name == "open":
        x, y = args
        if s["robot_at"] != x or (x, y) not in CONNECTIONS:
            return None
        s["unlocked"].add((x, y)); s["unlocked"].add((y, x))
        return s
    elif name == "drive":
        x, y = args
        if s["robot_at"] != x or (x, y) not in CONNECTIONS or (x, y) not in s["unlocked"]:
            return None
        s["robot_at"] = y
        return s
    elif name == "clean":
        if s["robot_at"] != args[0]:
            return None
        s["clean"].add(args[0])
        return s
    return None


def check_goal(state):
    return "b" in state["clean"] and "c" in state["clean"]


def parse_plan(s):
    return [(parts[0], tuple(parts[1:])) for parts in
            [a.strip().split() for a in re.findall(r'\(([^)]+)\)', s)]]


def validate(init_fn, apply_fn, plan):
    state = init_fn()
    for i, (name, args) in enumerate(plan):
        state = apply_fn(state, name, args)
        if state is None:
            return False, f"Step {i}: {name}({','.join(args)}) not applicable"
    if check_goal(state):
        return True, "Goal reached"
    return False, "Goal not reached"


# Plans with REDUNDANT opens (same connection opened twice)
test_plans = [
    # Double open same direction
    "(open a b)(open a b)(drive a b)(clean b)(drive b a)(open a c)(drive a c)(clean c)",
    # Open both directions of same connection
    "(open a b)(drive a b)(open b a)(clean b)(drive b a)(open a c)(drive a c)(clean c)",
    # Open a-c twice (once before and once after)
    "(open a c)(open a b)(drive a b)(clean b)(drive b a)(open a c)(drive a c)(clean c)",
    # Open a-b, then from b side open b-a (same connection)
    "(open a b)(drive a b)(open b a)(drive b a)(open a c)(drive a c)(clean c)(drive c a)(drive a b)(clean b)",
    # Valid in BOTH (no redundant opens) - control
    "(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(clean c)",
    # Open connection not adjacent (should fail both)
    "(open a d)(open a b)(drive a b)(clean b)(drive b a)(open a c)(drive a c)(clean c)",
]

print("Testing plans that should differentiate v1 and v2:")
print("=" * 70)
for plan_str in test_plans:
    plan = parse_plan(plan_str)
    ok1, r1 = validate(make_init_v1, apply_v1, plan)
    ok2, r2 = validate(make_init_v2, apply_v2, plan)
    marker = " *** DIFFERS ***" if ok1 != ok2 else ""
    print(f"\nPlan: {plan_str}")
    print(f"  v1 (locked): {'PASS' if ok1 else 'FAIL'} - {r1}")
    print(f"  v2 (no locked): {'PASS' if ok2 else 'FAIL'} - {r2}")
    if marker:
        print(f"  {marker}")
