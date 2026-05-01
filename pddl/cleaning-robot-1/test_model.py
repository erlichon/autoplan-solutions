#!/usr/bin/env python3
"""
Comprehensive test for the v5 PDDL model:
  - Bidirectional open: open ?x ?y sets both (unlocked ?x ?y) and (unlocked ?y ?x)
  - Non-idempotent clean: requires (not (clean ?x))
"""

from collections import deque
from copy import deepcopy

ROOMS = ["a", "b", "c", "d"]
CONNECTIONS = [("a", "b"), ("b", "a"), ("a", "c"), ("c", "a"),
               ("b", "d"), ("d", "b"), ("c", "d"), ("d", "c")]
CONN_SET = set(CONNECTIONS)


def make_state():
    return {"robot_at": "a", "unlocked": set(), "clean": set()}


def is_goal(state):
    return "b" in state["clean"] and "c" in state["clean"]


def applicable(state, action_name, args):
    robot = state["robot_at"]
    if action_name == "open":
        x, y = args
        return robot == x and (x, y) in CONN_SET and (x, y) not in state["unlocked"]
    elif action_name == "drive":
        x, y = args
        return robot == x and (x, y) in CONN_SET and (x, y) in state["unlocked"]
    elif action_name == "clean":
        return robot == args[0] and args[0] not in state["clean"]
    return False


def apply_action(state, action_name, args):
    s = deepcopy(state)
    if action_name == "open":
        x, y = args
        s["unlocked"].add((x, y))
        s["unlocked"].add((y, x))
    elif action_name == "drive":
        s["robot_at"] = args[1]
    elif action_name == "clean":
        s["clean"].add(args[0])
    return s


def parse_plan(plan_str):
    actions = []
    i = 0
    while i < len(plan_str):
        if plan_str[i] == '(':
            j = plan_str.index(')', i)
            parts = plan_str[i+1:j].split()
            actions.append((parts[0], tuple(parts[1:])))
            i = j + 1
        else:
            i += 1
    return actions


def validate_plan(plan_str):
    actions = parse_plan(plan_str)
    state = make_state()
    for i, (name, args) in enumerate(actions):
        if not applicable(state, name, args):
            return False, f"Action {i}: ({name} {' '.join(args)}) not applicable"
        state = apply_action(state, name, args)
    if not is_goal(state):
        return False, "Goal not reached"
    return True, "Valid"


def test_semantic_properties():
    print("=== Semantic Property Tests ===")

    # 1. open a b sets BOTH unlocked(a,b) AND unlocked(b,a)
    s = make_state()
    s = apply_action(s, "open", ("a", "b"))
    assert ("a", "b") in s["unlocked"], "open a b should set unlocked(a,b)"
    assert ("b", "a") in s["unlocked"], "open a b should ALSO set unlocked(b,a)"
    print("  PASS: open a b sets both directions (bidirectional)")

    # 2. After open a b, drive works in BOTH directions
    s2 = deepcopy(s)
    s2["robot_at"] = "a"
    assert applicable(s2, "drive", ("a", "b")), "drive a b should work"
    s3 = apply_action(s2, "drive", ("a", "b"))
    assert applicable(s3, "drive", ("b", "a")), "drive b a should also work (bidirectional)"
    print("  PASS: drive works both ways after bidirectional open")

    # 3. After open a b, open b a is NOT applicable (same connection)
    s4 = deepcopy(s)
    s4["robot_at"] = "b"
    assert not applicable(s4, "open", ("b", "a")), "open b a should NOT work (already unlocked)"
    print("  PASS: cannot unlock same connection twice")

    # 4. clean requires room NOT already clean
    s5 = make_state()
    assert applicable(s5, "clean", ("a",)), "can clean dirty room"
    s5 = apply_action(s5, "clean", ("a",))
    assert "a" in s5["clean"]
    assert not applicable(s5, "clean", ("a",)), "cannot clean already-clean room"
    print("  PASS: clean requires room to not be already clean")

    # 5. 7-step plan is valid
    plan7 = "(open a b)(drive a b)(clean b)(drive b a)(open a c)(drive a c)(clean c)"
    ok, msg = validate_plan(plan7)
    assert ok, f"7-step plan should be valid: {msg}"
    print(f"  PASS: 7-step plan is valid")

    # 6. v4-style plan with open b a is INVALID (connection already unlocked)
    plan_v4 = "(open a b)(drive a b)(open b a)(clean b)(drive b a)(open a c)(drive a c)(clean c)"
    ok, msg = validate_plan(plan_v4)
    assert not ok, f"v4-style plan with open b a should be INVALID"
    assert "open b a" in msg
    print(f"  PASS: v4-style plan with redundant 'open b a' correctly rejected ({msg})")

    # 7. Plan with redundant cleaning is INVALID
    plan_reclean = "(open a b)(drive a b)(clean b)(clean b)(drive b a)(open a c)(drive a c)(clean c)"
    ok, msg = validate_plan(plan_reclean)
    assert not ok, f"Plan with redundant clean b should be INVALID"
    assert "clean b" in msg
    print(f"  PASS: plan with redundant cleaning correctly rejected ({msg})")

    # 8. Route through D works
    planD = "(open a b)(drive a b)(clean b)(open b d)(drive b d)(drive d c)(open d c)(clean c)"
    ok, msg = validate_plan(planD)
    # Wait, need to open d c before driving d c. Let me fix the plan.
    planD = "(open a b)(drive a b)(clean b)(open b d)(drive b d)(open d c)(drive d c)(clean c)"
    ok, msg = validate_plan(planD)
    assert ok, f"Route A->B->D->C should be valid: {msg}"
    print(f"  PASS: route through D is valid (8 steps)")

    print("  All semantic tests PASSED\n")


def test_plan_counts():
    print("=== Plan Enumeration ===")

    def get_actions(state):
        robot = state["robot_at"]
        actions = []
        for (x, y) in CONNECTIONS:
            if robot == x and (x, y) not in state["unlocked"]:
                actions.append(("open", (x, y)))
        for (x, y) in CONNECTIONS:
            if robot == x and (x, y) in state["unlocked"]:
                actions.append(("drive", (x, y)))
        if robot not in state["clean"]:
            actions.append(("clean", (robot,)))
        return actions

    def fmt(actions):
        return "".join(f"({n} {' '.join(a)})" for n, a in actions)

    queue = deque()
    queue.append((make_state(), []))
    plans_by_depth = {}
    all_plans = set()
    MAX_DEPTH = 11

    while queue:
        state, plan = queue.popleft()
        depth = len(plan)
        if depth > MAX_DEPTH:
            continue
        if is_goal(state):
            p = fmt(plan)
            if p not in all_plans:
                all_plans.add(p)
                plans_by_depth.setdefault(depth, []).append(p)
            if depth < MAX_DEPTH:
                for action in get_actions(state):
                    name, args = action
                    new_state = apply_action(state, name, args)
                    queue.append((new_state, plan + [action]))
            continue
        for action in get_actions(state):
            name, args = action
            new_state = apply_action(state, name, args)
            queue.append((new_state, plan + [action]))

    total = len(all_plans)
    print(f"  Total plans up to depth {MAX_DEPTH}: {total}")
    for d in sorted(plans_by_depth.keys()):
        print(f"    Depth {d}: {len(plans_by_depth[d])} plans")

    print(f"\n  All shortest (depth 7) plans:")
    for p in sorted(plans_by_depth.get(7, [])):
        print(f"    {p}")

    assert 7 in plans_by_depth, "Shortest plans should be depth 7"
    assert len(plans_by_depth[7]) == 6, f"Expected 6 optimal plans, got {len(plans_by_depth[7])}"
    print(f"\n  PASS: 6 optimal plans at depth 7")

    return all_plans


if __name__ == "__main__":
    test_semantic_properties()
    all_plans = test_plan_counts()
    print("\nAll tests passed!")
