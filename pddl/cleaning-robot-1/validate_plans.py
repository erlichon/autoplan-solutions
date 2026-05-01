#!/usr/bin/env python3
"""
Validate plans against both domain models (with and without 'locked' predicate)
to determine which model matches the reference.
"""

import re
from copy import deepcopy


ROOMS = {"a", "b", "c", "d"}
CONNECTIONS = {("a", "b"), ("b", "a"), ("a", "c"), ("c", "a"),
               ("b", "d"), ("d", "b"), ("c", "d"), ("d", "c")}


def make_init_state_v1():
    """Model v1: with 'locked' predicate preventing double-opens."""
    return {
        "robot_at": "a",
        "unlocked": set(),
        "locked": {("a", "b"), ("b", "a"), ("a", "c"), ("c", "a"),
                   ("b", "d"), ("d", "b"), ("c", "d"), ("d", "c")},
        "clean": set(),
    }


def make_init_state_v2():
    """Model v2: without 'locked' predicate (idempotent open)."""
    return {
        "robot_at": "a",
        "unlocked": set(),
        "clean": set(),
    }


def apply_action_v1(state, action_name, args):
    s = deepcopy(state)
    if action_name == "open":
        x, y = args
        if s["robot_at"] != x:
            return None
        if (x, y) not in CONNECTIONS:
            return None
        if (x, y) not in s["locked"]:
            return None
        s["unlocked"].add((x, y))
        s["unlocked"].add((y, x))
        s["locked"].discard((x, y))
        s["locked"].discard((y, x))
        return s
    elif action_name == "drive":
        x, y = args
        if s["robot_at"] != x:
            return None
        if (x, y) not in CONNECTIONS:
            return None
        if (x, y) not in s["unlocked"]:
            return None
        s["robot_at"] = y
        return s
    elif action_name == "clean":
        (x,) = args
        if s["robot_at"] != x:
            return None
        s["clean"].add(x)
        return s
    return None


def apply_action_v2(state, action_name, args):
    s = deepcopy(state)
    if action_name == "open":
        x, y = args
        if s["robot_at"] != x:
            return None
        if (x, y) not in CONNECTIONS:
            return None
        s["unlocked"].add((x, y))
        s["unlocked"].add((y, x))
        return s
    elif action_name == "drive":
        x, y = args
        if s["robot_at"] != x:
            return None
        if (x, y) not in CONNECTIONS:
            return None
        if (x, y) not in s["unlocked"]:
            return None
        s["robot_at"] = y
        return s
    elif action_name == "clean":
        (x,) = args
        if s["robot_at"] != x:
            return None
        s["clean"].add(x)
        return s
    return None


def check_goal(state):
    return "b" in state["clean"] and "c" in state["clean"]


def parse_plan(plan_str):
    actions = re.findall(r'\(([^)]+)\)', plan_str)
    result = []
    for a in actions:
        parts = a.strip().split()
        name = parts[0]
        args = tuple(parts[1:])
        result.append((name, args))
    return result


def validate_plan(init_fn, apply_fn, plan):
    state = init_fn()
    for i, (action_name, args) in enumerate(plan):
        new_state = apply_fn(state, action_name, args)
        if new_state is None:
            return False, i, f"{action_name}({', '.join(args)}) not applicable at step {i}"
        state = new_state
    if check_goal(state):
        return True, -1, "Goal reached"
    return False, -1, "Goal not reached"


# ALL plans from DOMjudge output (the v1 submission)
DOMJUDGE_PLANS = """(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(clean c)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(clean c)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a b)(clean b)
(open a c)(open a b)(drive a c)(clean c)(drive c a)(drive a b)(clean b)
(open a b)(open a c)(drive a c)(clean c)(drive c a)(drive a b)(clean b)
(open a b)(drive a b)(clean b)(drive b a)(open a c)(drive a c)(clean c)
(open a c)(open a b)(drive a b)(clean b)(clean b)(drive b a)(drive a c)(clean c)
(open a b)(open a c)(drive a b)(clean b)(clean b)(drive b a)(drive a c)(clean c)
(open a c)(open a b)(drive a b)(open b d)(clean b)(drive b a)(drive a c)(clean c)
(open a b)(open a c)(drive a b)(open b d)(clean b)(drive b a)(drive a c)(clean c)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a b)(clean b)(clean b)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a b)(clean b)(open b d)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a b)(clean b)(drive b a)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a b)(open b d)(clean b)
(open a c)(open a b)(drive a c)(clean c)(open c d)(drive c a)(drive a b)(clean b)
(open a b)(open a c)(drive a c)(clean c)(open c d)(drive c a)(drive a b)(clean b)
(open a b)(drive a b)(clean b)(clean b)(drive b a)(open a c)(drive a c)(clean c)
(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(clean c)
(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(drive c a)
(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(open c d)
(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(open c d)(clean c)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(clean c)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(open c d)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(drive c a)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(open c d)(clean c)
(open a b)(drive a b)(open b d)(clean b)(drive b a)(open a c)(drive a c)(clean c)
(open a b)(drive a b)(clean b)(open b d)(drive b a)(open a c)(drive a c)(clean c)
(open a c)(drive a c)(open c d)(clean c)(drive c d)(open d b)(drive d b)(clean b)
(open a c)(drive a c)(clean c)(open c d)(drive c d)(open d b)(drive d b)(clean b)
(open a c)(open a b)(drive a b)(clean b)(open b d)(drive b a)(drive a c)(clean c)
(open a b)(open a c)(drive a b)(clean b)(open b d)(drive b a)(drive a c)(clean c)
(open a b)(drive a b)(open b d)(clean b)(drive b d)(open d c)(drive d c)(clean c)
(open a b)(drive a b)(clean b)(open b d)(drive b d)(open d c)(drive d c)(clean c)
(open a c)(open a b)(drive a c)(clean c)(clean c)(drive c a)(drive a b)(clean b)
(open a b)(open a c)(drive a c)(clean c)(clean c)(drive c a)(drive a b)(clean b)
(open a c)(open a b)(drive a c)(open c d)(clean c)(drive c a)(drive a b)(clean b)
(open a b)(open a c)(drive a c)(open c d)(clean c)(drive c a)(drive a b)(clean b)
(open a b)(drive a b)(clean b)(drive b a)(open a c)(drive a c)(clean c)(clean c)
(open a b)(drive a b)(clean b)(drive b a)(open a c)(drive a c)(clean c)(drive c a)
(open a b)(drive a b)(clean b)(drive b a)(open a c)(drive a c)(clean c)(open c d)
(open a b)(drive a b)(clean b)(drive b a)(open a c)(drive a c)(open c d)(clean c)
(open a c)(drive a c)(clean c)(clean c)(drive c a)(open a b)(drive a b)(clean b)
(open a c)(open a b)(drive a c)(clean c)(drive c a)(drive a b)(clean b)(clean b)
(open a c)(open a b)(drive a c)(clean c)(drive c a)(drive a b)(clean b)(drive b a)
(open a c)(open a b)(drive a c)(clean c)(drive c a)(drive a b)(clean b)(open b d)
(open a c)(open a b)(drive a c)(clean c)(drive c a)(drive a b)(open b d)(clean b)
(open a b)(open a c)(drive a c)(clean c)(drive c a)(drive a b)(clean b)(clean b)
(open a b)(open a c)(drive a c)(clean c)(drive c a)(drive a b)(clean b)(open b d)
(open a b)(open a c)(drive a c)(clean c)(drive c a)(drive a b)(clean b)(drive b a)
(open a b)(open a c)(drive a c)(clean c)(drive c a)(drive a b)(open b d)(clean b)
(open a c)(drive a c)(open c d)(clean c)(drive c a)(open a b)(drive a b)(clean b)
(open a c)(drive a c)(clean c)(open c d)(drive c a)(open a b)(drive a b)(clean b)
(open a b)(drive a b)(clean b)(clean b)(clean b)(drive b a)(open a c)(drive a c)(clean c)
(open a c)(open a b)(drive a b)(clean b)(clean b)(clean b)(drive b a)(drive a c)(clean c)
(open a c)(open a b)(drive a b)(clean b)(clean b)(drive b a)(drive a c)(clean c)(clean c)
(open a c)(open a b)(drive a b)(clean b)(clean b)(drive b a)(drive a c)(clean c)(drive c a)
(open a c)(open a b)(drive a b)(clean b)(clean b)(drive b a)(drive a c)(clean c)(open c d)
(open a c)(open a b)(drive a b)(clean b)(clean b)(drive b a)(drive a c)(open c d)(clean c)
(open a c)(open a b)(drive a b)(clean b)(clean b)(open b d)(drive b a)(drive a c)(clean c)
(open a b)(open a c)(drive a b)(clean b)(clean b)(clean b)(drive b a)(drive a c)(clean c)
(open a b)(open a c)(drive a b)(clean b)(clean b)(drive b a)(drive a c)(clean c)(clean c)
(open a b)(open a c)(drive a b)(clean b)(clean b)(drive b a)(drive a c)(clean c)(drive c a)
(open a b)(open a c)(drive a b)(clean b)(clean b)(drive b a)(drive a c)(clean c)(open c d)
(open a b)(open a c)(drive a b)(clean b)(clean b)(drive b a)(drive a c)(open c d)(clean c)
(open a b)(open a c)(drive a b)(clean b)(clean b)(open b d)(drive b a)(drive a c)(clean c)
(open a b)(drive a b)(open b d)(clean b)(clean b)(drive b a)(open a c)(drive a c)(clean c)
(open a b)(drive a b)(open b d)(clean b)(clean b)(drive b d)(open d c)(drive d c)(clean c)
(open a b)(drive a b)(drive b a)(drive a b)(clean b)(drive b a)(open a c)(drive a c)(clean c)
(open a b)(drive a b)(clean b)(open b d)(clean b)(drive b a)(open a c)(drive a c)(clean c)
(open a b)(drive a b)(clean b)(open b d)(clean b)(drive b d)(open d c)(drive d c)(clean c)
(open a c)(open a b)(drive a b)(open b d)(clean b)(clean b)(drive b a)(drive a c)(clean c)
(open a c)(open a b)(drive a b)(open b d)(clean b)(drive b a)(drive a c)(clean c)(clean c)
(open a c)(open a b)(drive a b)(open b d)(clean b)(drive b a)(drive a c)(clean c)(drive c a)
(open a c)(open a b)(drive a b)(open b d)(clean b)(drive b a)(drive a c)(clean c)(open c d)
(open a c)(open a b)(drive a b)(open b d)(clean b)(drive b a)(drive a c)(open c d)(clean c)
(open a c)(open a b)(drive a b)(open b d)(clean b)(drive b d)(open d c)(drive d c)(clean c)
(open a b)(open a c)(drive a b)(open b d)(clean b)(clean b)(drive b a)(drive a c)(clean c)
(open a b)(open a c)(drive a b)(open b d)(clean b)(drive b a)(drive a c)(clean c)(clean c)
(open a b)(open a c)(drive a b)(open b d)(clean b)(drive b a)(drive a c)(clean c)(drive c a)
(open a b)(open a c)(drive a b)(open b d)(clean b)(drive b a)(drive a c)(clean c)(open c d)
(open a b)(open a c)(drive a b)(open b d)(clean b)(drive b a)(drive a c)(open c d)(clean c)
(open a b)(open a c)(drive a b)(open b d)(clean b)(drive b d)(open d c)(drive d c)(clean c)
(open a b)(drive a b)(clean b)(drive b a)(drive a b)(drive b a)(open a c)(drive a c)(clean c)
(open a c)(open a b)(drive a b)(drive b a)(drive a b)(clean b)(drive b a)(drive a c)(clean c)
(open a b)(open a c)(drive a b)(drive b a)(drive a b)(clean b)(drive b a)(drive a c)(clean c)
(open a c)(open a b)(drive a c)(drive c a)(drive a b)(clean b)(drive b a)(drive a c)(clean c)
(open a b)(open a c)(drive a c)(drive c a)(drive a b)(clean b)(drive b a)(drive a c)(clean c)
(open a b)(drive a b)(drive b a)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(clean c)
(open a c)(drive a c)(drive c a)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(clean c)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a b)(clean b)(clean b)(clean b)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a b)(clean b)(clean b)(drive b a)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a b)(clean b)(clean b)(open b d)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a b)(clean b)(drive b a)(drive a b)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a b)(clean b)(drive b a)(drive a c)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a b)(clean b)(open b d)(clean b)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a b)(clean b)(open b d)(drive b a)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a b)(clean b)(open b d)(drive b d)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a b)(drive b a)(drive a b)(clean b)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a b)(open b d)(clean b)(clean b)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a b)(open b d)(clean b)(drive b a)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a b)(open b d)(clean b)(drive b d)
(open a c)(drive a c)(clean c)(drive c a)(open a b)(drive a c)(drive c a)(drive a b)(clean b)
(open a b)(drive a b)(open b d)(clean b)(drive b d)(open d c)(drive d c)(clean c)(clean c)
(open a b)(drive a b)(open b d)(clean b)(drive b d)(open d c)(drive d c)(clean c)(drive c d)
(open a b)(drive a b)(open b d)(clean b)(drive b d)(open d c)(drive d c)(clean c)(open c a)
(open a b)(drive a b)(open b d)(clean b)(drive b d)(open d c)(drive d c)(open c a)(clean c)
(open a b)(drive a b)(clean b)(open b d)(drive b d)(open d c)(drive d c)(clean c)(clean c)
(open a b)(drive a b)(clean b)(open b d)(drive b d)(open d c)(drive d c)(clean c)(open c a)
(open a b)(drive a b)(clean b)(open b d)(drive b d)(open d c)(drive d c)(clean c)(drive c d)
(open a b)(drive a b)(clean b)(open b d)(drive b d)(open d c)(drive d c)(open c a)(clean c)
(open a b)(drive a b)(clean b)(clean b)(drive b a)(open a c)(drive a c)(clean c)(clean c)
(open a b)(drive a b)(clean b)(clean b)(drive b a)(open a c)(drive a c)(clean c)(drive c a)
(open a b)(drive a b)(clean b)(clean b)(drive b a)(open a c)(drive a c)(clean c)(open c d)
(open a b)(drive a b)(clean b)(clean b)(drive b a)(open a c)(drive a c)(open c d)(clean c)
(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a b)(drive b a)(drive a c)(clean c)
(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(clean c)(clean c)
(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(clean c)(drive c a)
(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(clean c)(open c d)
(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(drive c a)(drive a b)
(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(drive c a)(drive a c)
(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(open c d)(clean c)
(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(open c d)(drive c a)
(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(open c d)(drive c d)
(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(drive c a)(drive a c)(clean c)
(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(open c d)(clean c)(clean c)
(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(open c d)(clean c)(drive c a)
(open a c)(open a b)(drive a b)(clean b)(drive b a)(drive a c)(open c d)(clean c)(drive c d)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a b)(drive b a)(drive a c)(clean c)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(clean c)(clean c)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(clean c)(drive c a)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(clean c)(open c d)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(drive c a)(drive a b)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(drive c a)(drive a c)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(open c d)(clean c)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(open c d)(drive c a)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(clean c)(open c d)(drive c d)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(open c d)(clean c)(clean c)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(open c d)(clean c)(drive c a)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(open c d)(clean c)(drive c d)
(open a b)(open a c)(drive a b)(clean b)(drive b a)(drive a c)(drive c a)(drive a c)(clean c)
(open a c)(drive a c)(clean c)(open c d)(drive c a)(open a b)(drive a b)(clean b)(clean b)
(open a c)(drive a c)(clean c)(open c d)(drive c a)(open a b)(drive a b)(clean b)(drive b a)
(open a c)(drive a c)(clean c)(open c d)(drive c a)(open a b)(drive a b)(clean b)(open b d)
(open a c)(drive a c)(clean c)(open c d)(drive c a)(open a b)(drive a b)(open b d)(clean b)
(open a c)(drive a c)(clean c)(clean c)(open c d)(drive c a)(open a b)(drive a b)(clean b)
(open a c)(drive a c)(clean c)(clean c)(open c d)(drive c d)(open d b)(drive d b)(clean b)
(open a c)(open a b)(drive a c)(clean c)(open c d)(clean c)(drive c a)(drive a b)(clean b)
(open a c)(open a b)(drive a c)(clean c)(open c d)(drive c a)(drive a b)(clean b)(clean b)
(open a c)(open a b)(drive a c)(clean c)(open c d)(drive c a)(drive a b)(clean b)(drive b a)
(open a c)(open a b)(drive a c)(clean c)(open c d)(drive c a)(drive a b)(clean b)(open b d)
(open a c)(open a b)(drive a c)(clean c)(open c d)(drive c a)(drive a b)(open b d)(clean b)
(open a c)(open a b)(drive a c)(clean c)(open c d)(drive c d)(open d b)(drive d b)(clean b)
(open a b)(open a c)(drive a c)(clean c)(open c d)(clean c)(drive c a)(drive a b)(clean b)
(open a b)(open a c)(drive a c)(clean c)(open c d)(drive c d)(open d b)(drive d b)(clean b)
(open a b)(open a c)(drive a c)(clean c)(open c d)(drive c a)(drive a b)(clean b)(clean b)
(open a b)(open a c)(drive a c)(clean c)(open c d)(drive c a)(drive a b)(clean b)(drive b a)
(open a b)(open a c)(drive a c)(clean c)(open c d)(drive c a)(drive a b)(clean b)(open b d)
(open a b)(open a c)(drive a c)(clean c)(open c d)(drive c a)(drive a b)(open b d)(clean b)
(open a b)(drive a b)(clean b)(clean b)(clean b)(clean b)(drive b a)(open a c)(drive a c)(clean c)
(open a c)(open a b)(drive a b)(clean b)(clean b)(clean b)(clean b)(drive b a)(drive a c)(clean c)
(open a c)(open a b)(drive a b)(clean b)(clean b)(clean b)(drive b a)(drive a c)(clean c)(clean c)
(open a c)(open a b)(drive a b)(clean b)(clean b)(clean b)(drive b a)(drive a c)(clean c)(drive c a)
(open a c)(open a b)(drive a b)(clean b)(clean b)(clean b)(drive b a)(drive a c)(clean c)(open c d)
(open a c)(open a b)(drive a b)(clean b)(clean b)(clean b)(drive b a)(drive a c)(open c d)(clean c)
(open a c)(open a b)(drive a b)(clean b)(clean b)(clean b)(open b d)(drive b a)(drive a c)(clean c)
(open a b)(open a c)(drive a b)(clean b)(clean b)(clean b)(clean b)(drive b a)(drive a c)(clean c)
(open a b)(open a c)(drive a b)(clean b)(clean b)(clean b)(open b d)(drive b a)(drive a c)(clean c)
(open a b)(open a c)(drive a b)(clean b)(clean b)(clean b)(drive b a)(drive a c)(clean c)(clean c)
(open a b)(open a c)(drive a b)(clean b)(clean b)(clean b)(drive b a)(drive a c)(clean c)(drive c a)
(open a b)(open a c)(drive a b)(clean b)(clean b)(clean b)(drive b a)(drive a c)(clean c)(open c d)
(open a b)(open a c)(drive a b)(clean b)(clean b)(clean b)(drive b a)(drive a c)(open c d)(clean c)
(open a b)(drive a b)(open b d)(clean b)(clean b)(clean b)(drive b a)(open a c)(drive a c)(clean c)
(open a b)(drive a b)(open b d)(clean b)(clean b)(clean b)(drive b d)(open d c)(drive d c)(clean c)
(open a b)(drive a b)(drive b a)(drive a b)(clean b)(clean b)(drive b a)(open a c)(drive a c)(clean c)
(open a b)(drive a b)(clean b)(open b d)(clean b)(clean b)(drive b a)(open a c)(drive a c)(clean c)
(open a b)(drive a b)(clean b)(open b d)(clean b)(clean b)(drive b d)(open d c)(drive d c)(clean c)
(open a c)(open a b)(drive a b)(open b d)(clean b)(clean b)(clean b)(drive b a)(drive a c)(clean c)
(open a c)(open a b)(drive a b)(open b d)(clean b)(clean b)(drive b a)(drive a c)(clean c)(clean c)
(open a c)(open a b)(drive a b)(open b d)(clean b)(clean b)(drive b a)(drive a c)(clean c)(drive c a)
(open a c)(open a b)(drive a b)(open b d)(clean b)(clean b)(drive b a)(drive a c)(clean c)(open c d)
(open a c)(open a b)(drive a b)(open b d)(clean b)(clean b)(drive b a)(drive a c)(open c d)(clean c)
(open a c)(open a b)(drive a b)(open b d)(clean b)(clean b)(drive b d)(open d c)(drive d c)(clean c)
(open a b)(open a c)(drive a b)(open b d)(clean b)(clean b)(clean b)(drive b a)(drive a c)(clean c)
(open a b)(open a c)(drive a b)(open b d)(clean b)(clean b)(drive b a)(drive a c)(clean c)(clean c)
(open a b)(open a c)(drive a b)(open b d)(clean b)(clean b)(drive b a)(drive a c)(clean c)(drive c a)
(open a b)(open a c)(drive a b)(open b d)(clean b)(clean b)(drive b a)(drive a c)(clean c)(open c d)
(open a b)(open a c)(drive a b)(open b d)(clean b)(clean b)(drive b a)(drive a c)(open c d)(clean c)
(open a b)(open a c)(drive a b)(open b d)(clean b)(clean b)(drive b d)(open d c)(drive d c)(clean c)
(open a c)(drive a c)(clean c)(clean c)(clean c)(clean c)(drive c a)(open a b)(drive a b)(clean b)
(open a c)(open a b)(drive a c)(clean c)(clean c)(clean c)(clean c)(drive c a)(drive a b)(clean b)
(open a c)(open a b)(drive a c)(clean c)(clean c)(clean c)(drive c a)(drive a b)(clean b)(clean b)
(open a c)(open a b)(drive a c)(clean c)(clean c)(clean c)(drive c a)(drive a b)(clean b)(drive b a)
(open a c)(open a b)(drive a c)(clean c)(clean c)(clean c)(drive c a)(drive a b)(clean b)(open b d)
(open a c)(open a b)(drive a c)(clean c)(clean c)(clean c)(drive c a)(drive a b)(open b d)(clean b)
(open a c)(open a b)(drive a c)(clean c)(clean c)(clean c)(open c d)(drive c a)(drive a b)(clean b)
(open a b)(open a c)(drive a c)(clean c)(clean c)(clean c)(clean c)(drive c a)(drive a b)(clean b)
(open a b)(open a c)(drive a c)(clean c)(clean c)(clean c)(drive c a)(drive a b)(clean b)(clean b)
(open a b)(open a c)(drive a c)(clean c)(clean c)(clean c)(drive c a)(drive a b)(clean b)(drive b a)
(open a b)(open a c)(drive a c)(clean c)(clean c)(clean c)(drive c a)(drive a b)(clean b)(open b d)
(open a b)(open a c)(drive a c)(clean c)(clean c)(clean c)(drive c a)(drive a b)(open b d)(clean b)
(open a b)(open a c)(drive a c)(clean c)(clean c)(clean c)(open c d)(drive c a)(drive a b)(clean b)
(open a c)(drive a c)(open c d)(clean c)(clean c)(clean c)(drive c a)(open a b)(drive a b)(clean b)
(open a c)(drive a c)(open c d)(clean c)(clean c)(clean c)(drive c d)(open d b)(drive d b)(clean b)
(open a c)(drive a c)(clean c)(open c d)(clean c)(clean c)(drive c a)(open a b)(drive a b)(clean b)
(open a c)(drive a c)(clean c)(open c d)(clean c)(clean c)(drive c d)(open d b)(drive d b)(clean b)
(open a c)(open a b)(drive a c)(clean c)(open c d)(clean c)(clean c)(drive c a)(drive a b)(clean b)
(open a c)(open a b)(drive a c)(clean c)(open c d)(clean c)(drive c a)(drive a b)(clean b)(clean b)
(open a c)(open a b)(drive a c)(clean c)(open c d)(clean c)(drive c a)(drive a b)(clean b)(drive b a)
(open a c)(open a b)(drive a c)(clean c)(open c d)(clean c)(drive c a)(drive a b)(clean b)(open b d)
(open a c)(open a b)(drive a c)(clean c)(open c d)(clean c)(drive c a)(drive a b)(open b d)(clean b)
(open a c)(open a b)(drive a c)(clean c)(open c d)(clean c)(drive c d)(open d b)(drive d b)(clean b)
(open a b)(open a c)(drive a c)(clean c)(open c d)(clean c)(clean c)(drive c a)(drive a b)(clean b)
(open a b)(open a c)(drive a c)(clean c)(open c d)(clean c)(drive c d)(open d b)(drive d b)(clean b)
(open a b)(open a c)(drive a c)(clean c)(open c d)(clean c)(drive c a)(drive a b)(clean b)(clean b)
(open a b)(open a c)(drive a c)(clean c)(open c d)(clean c)(drive c a)(drive a b)(clean b)(drive b a)
(open a b)(open a c)(drive a c)(clean c)(open c d)(clean c)(drive c a)(drive a b)(clean b)(open b d)
(open a b)(open a c)(drive a c)(clean c)(open c d)(clean c)(drive c a)(drive a b)(open b d)(clean b)"""


def main():
    plans = [line.strip() for line in DOMJUDGE_PLANS.strip().split("\n") if line.strip()]

    print(f"Total plans from DOMjudge: {len(plans)}")
    print()

    v1_pass = 0
    v1_fail = 0
    v1_failures = []
    v2_pass = 0
    v2_fail = 0
    v2_failures = []

    for i, plan_str in enumerate(plans):
        plan = parse_plan(plan_str)

        ok1, step1, reason1 = validate_plan(make_init_state_v1, apply_action_v1, plan)
        ok2, step2, reason2 = validate_plan(make_init_state_v2, apply_action_v2, plan)

        if ok1:
            v1_pass += 1
        else:
            v1_fail += 1
            v1_failures.append((i + 1, plan_str, reason1))

        if ok2:
            v2_pass += 1
        else:
            v2_fail += 1
            v2_failures.append((i + 1, plan_str, reason2))

    print("=" * 60)
    print("Model v1 (with 'locked' predicate - prevents double open)")
    print(f"  Pass: {v1_pass}, Fail: {v1_fail}")
    if v1_failures:
        print("  Failures:")
        for num, p, r in v1_failures[:10]:
            print(f"    Plan #{num}: {r}")
            print(f"      {p[:120]}...")

    print()
    print("=" * 60)
    print("Model v2 (without 'locked' - idempotent open)")
    print(f"  Pass: {v2_pass}, Fail: {v2_fail}")
    if v2_failures:
        print("  Failures:")
        for num, p, r in v2_failures[:10]:
            print(f"    Plan #{num}: {r}")
            print(f"      {p[:120]}...")

    print()
    print("=" * 60)
    discrepancies = []
    for i, plan_str in enumerate(plans):
        plan = parse_plan(plan_str)
        ok1, _, r1 = validate_plan(make_init_state_v1, apply_action_v1, plan)
        ok2, _, r2 = validate_plan(make_init_state_v2, apply_action_v2, plan)
        if ok1 != ok2:
            discrepancies.append((i + 1, plan_str, ok1, ok2, r1, r2))

    if discrepancies:
        print(f"Plans where models DISAGREE: {len(discrepancies)}")
        for num, p, ok1, ok2, r1, r2 in discrepancies[:20]:
            print(f"  Plan #{num}: v1={'PASS' if ok1 else 'FAIL'}, v2={'PASS' if ok2 else 'FAIL'}")
            print(f"    v1: {r1}")
            print(f"    v2: {r2}")
            print(f"    {p[:150]}")
    else:
        print("Both models AGREE on all plans from DOMjudge.")


if __name__ == "__main__":
    main()
