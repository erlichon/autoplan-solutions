#!/usr/bin/env python3
"""
Enumerate plans for different model variants to find which matches DOMjudge reference.
"""

from collections import deque
from copy import deepcopy


ROOMS = ["a", "b", "c", "d"]
CONNECTIONS = [("a", "b"), ("b", "a"), ("a", "c"), ("c", "a"),
               ("b", "d"), ("d", "b"), ("c", "d"), ("d", "c")]


def get_actions_v3(state):
    """v3: negative precondition, BOTH directions in effect."""
    robot = state["robot_at"]
    actions = []
    for (x, y) in CONNECTIONS:
        if robot == x and (x, y) not in state["unlocked"]:
            actions.append(("open", (x, y)))
    for (x, y) in CONNECTIONS:
        if robot == x and (x, y) in state["unlocked"]:
            actions.append(("drive", (x, y)))
    actions.append(("clean", (robot,)))
    return actions


def apply_v3(state, action):
    name, args = action
    s = deepcopy(state)
    if name == "open":
        x, y = args
        s["unlocked"].add((x, y))
        s["unlocked"].add((y, x))
    elif name == "drive":
        s["robot_at"] = args[1]
    elif name == "clean":
        s["clean"].add(args[0])
    return s


def get_actions_v4(state):
    """v4: negative precondition, ONE direction only in effect."""
    robot = state["robot_at"]
    actions = []
    for (x, y) in CONNECTIONS:
        if robot == x and (x, y) not in state["unlocked"]:
            actions.append(("open", (x, y)))
    for (x, y) in CONNECTIONS:
        if robot == x and (x, y) in state["unlocked"]:
            actions.append(("drive", (x, y)))
    actions.append(("clean", (robot,)))
    return actions


def apply_v4(state, action):
    name, args = action
    s = deepcopy(state)
    if name == "open":
        x, y = args
        s["unlocked"].add((x, y))  # Only one direction!
    elif name == "drive":
        s["robot_at"] = args[1]
    elif name == "clean":
        s["clean"].add(args[0])
    return s


def is_goal(state):
    return "b" in state["clean"] and "c" in state["clean"]


def format_plan(actions):
    return "".join(f"({name} {' '.join(args)})" for name, args in actions)


def make_init():
    return {"robot_at": "a", "unlocked": set(), "clean": set()}


def count_plans_bfs(init_state, get_actions, apply_fn, max_depth):
    queue = deque()
    queue.append((init_state, [], 0))
    seen_plans = set()
    counts = {}

    while queue:
        state, plan, depth = queue.popleft()
        if depth > max_depth:
            continue
        if is_goal(state):
            plan_str = format_plan(plan)
            if plan_str not in seen_plans:
                seen_plans.add(plan_str)
                counts[depth] = counts.get(depth, 0) + 1
            if depth < max_depth:
                for action in get_actions(state):
                    new_state = apply_fn(state, action)
                    queue.append((new_state, plan + [action], depth + 1))
            continue
        for action in get_actions(state):
            new_state = apply_fn(state, action)
            queue.append((new_state, plan + [action], depth + 1))

    return counts, seen_plans


MAX_DEPTH = 10

for label, get_actions, apply_fn in [
    ("v3 (both directions)", get_actions_v3, apply_v3),
    ("v4 (one direction only)", get_actions_v4, apply_v4),
]:
    print(f"\nModel {label}:")
    counts, plans = count_plans_bfs(make_init(), get_actions, apply_fn, MAX_DEPTH)
    print(f"  Total unique plans up to depth {MAX_DEPTH}: {len(plans)}")
    for d in sorted(counts.keys()):
        print(f"    Depth {d}: {counts[d]} plans")

    if "v4" in label:
        print(f"\n  Sample v4 shortest plans:")
        shortest_depth = min(counts.keys())
        for p in sorted(plans):
            if p.count("(") == shortest_depth:
                print(f"    {p}")

    if "v3" in label:
        v3_plans = plans
    else:
        v4_plans = plans

print(f"\n--- Comparison ---")
only_v3 = v3_plans - v4_plans
only_v4 = v4_plans - v3_plans
common = v3_plans & v4_plans
print(f"Plans only in v3 (both dirs): {len(only_v3)}")
print(f"Plans only in v4 (one dir):   {len(only_v4)}")
print(f"Plans in both:                {len(common)}")

print(f"\nSample plans ONLY in v3 (shortest first):")
for p in sorted(only_v3, key=lambda x: x.count("("))[:10]:
    print(f"  {p}")

print(f"\nSample plans ONLY in v4 (shortest first):")
for p in sorted(only_v4, key=lambda x: x.count("("))[:10]:
    print(f"  {p}")
