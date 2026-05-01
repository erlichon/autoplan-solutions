#!/usr/bin/env python3
"""
Compare all plausible model variants for the Cleaning Robot 1 problem.

Variants:
  A: bidirectional open + idempotent clean (v3)
  B: bidirectional open + non-idempotent clean
  C: unidirectional open + idempotent clean (v4)
  D: unidirectional open + non-idempotent clean
  E: bidirectional open + symmetric precondition + idempotent clean
  F: bidirectional open + symmetric precondition + non-idempotent clean
"""

from collections import deque
from copy import deepcopy

ROOMS = ["a", "b", "c", "d"]
CONNECTIONS = [("a", "b"), ("b", "a"), ("a", "c"), ("c", "a"),
               ("b", "d"), ("d", "b"), ("c", "d"), ("d", "c")]
CONN_SET = set(CONNECTIONS)


def make_init():
    return {"robot_at": "a", "unlocked": set(), "clean": set()}


def is_goal(state):
    return "b" in state["clean"] and "c" in state["clean"]


def format_plan(actions):
    return "".join(f"({name} {' '.join(args)})" for name, args in actions)


def get_actions(state, *, bidir_open, idem_clean, sym_open_pre=False):
    robot = state["robot_at"]
    actions = []

    for (x, y) in CONNECTIONS:
        if robot == x and (x, y) in CONN_SET and (x, y) not in state["unlocked"]:
            if sym_open_pre:
                if (y, x) not in state["unlocked"]:
                    actions.append(("open", (x, y)))
            else:
                actions.append(("open", (x, y)))

    for (x, y) in CONNECTIONS:
        if robot == x and (x, y) in CONN_SET and (x, y) in state["unlocked"]:
            actions.append(("drive", (x, y)))

    if idem_clean:
        actions.append(("clean", (robot,)))
    else:
        if robot not in state["clean"]:
            actions.append(("clean", (robot,)))

    return actions


def apply_action(state, action, *, bidir_open):
    name, args = action
    s = deepcopy(state)
    if name == "open":
        x, y = args
        s["unlocked"].add((x, y))
        if bidir_open:
            s["unlocked"].add((y, x))
    elif name == "drive":
        s["robot_at"] = args[1]
    elif name == "clean":
        s["clean"].add(args[0])
    return s


def enumerate_plans(bidir_open, idem_clean, sym_open_pre, max_depth):
    queue = deque()
    init = make_init()
    queue.append((init, []))
    plans_by_depth = {}
    all_plans = set()

    while queue:
        state, plan = queue.popleft()
        depth = len(plan)
        if depth > max_depth:
            continue

        if is_goal(state):
            p = format_plan(plan)
            if p not in all_plans:
                all_plans.add(p)
                plans_by_depth.setdefault(depth, []).append(p)
            if depth < max_depth:
                for action in get_actions(state, bidir_open=bidir_open,
                                          idem_clean=idem_clean,
                                          sym_open_pre=sym_open_pre):
                    new_state = apply_action(state, action, bidir_open=bidir_open)
                    queue.append((new_state, plan + [action]))
            continue

        for action in get_actions(state, bidir_open=bidir_open,
                                  idem_clean=idem_clean,
                                  sym_open_pre=sym_open_pre):
            new_state = apply_action(state, action, bidir_open=bidir_open)
            queue.append((new_state, plan + [action]))

    return plans_by_depth, all_plans


MAX_DEPTH = 11

variants = [
    ("A: bidir_open + idem_clean", True, True, False),
    ("B: bidir_open + non-idem_clean", True, False, False),
    ("C: unidir_open + idem_clean", False, True, False),
    ("D: unidir_open + non-idem_clean", False, False, False),
    ("E: bidir_open + sym_pre + idem_clean", True, True, True),
    ("F: bidir_open + sym_pre + non-idem_clean", True, False, True),
]

results = {}
for label, bidir, idem, sym in variants:
    print(f"\n{'='*60}")
    print(f"Variant {label}")
    print(f"{'='*60}")
    plans_by_depth, all_plans = enumerate_plans(bidir, idem, sym, MAX_DEPTH)
    results[label] = all_plans

    total = len(all_plans)
    total_chars = sum(len(p) + 1 for p in all_plans)  # +1 for newline
    print(f"  Total unique plans up to depth {MAX_DEPTH}: {total}")
    print(f"  Total chars (plans + newlines): {total_chars}")

    for d in sorted(plans_by_depth.keys()):
        count = len(plans_by_depth[d])
        print(f"    Depth {d}: {count} plans")

    shortest = min(plans_by_depth.keys())
    print(f"\n  Shortest plans (depth {shortest}):")
    for p in sorted(plans_by_depth[shortest]):
        print(f"    {p}")

print(f"\n{'='*60}")
print("CROSS-COMPARISON")
print(f"{'='*60}")

labels = list(results.keys())
for i in range(len(labels)):
    for j in range(i + 1, len(labels)):
        l1, l2 = labels[i], labels[j]
        s1, s2 = results[l1], results[l2]
        only1 = s1 - s2
        only2 = s2 - s1
        common = s1 & s2
        if len(only1) == 0 and len(only2) == 0:
            print(f"\n  {l1} == {l2} (IDENTICAL)")
        else:
            print(f"\n  {l1} vs {l2}:")
            print(f"    Only in {l1[:1]}: {len(only1)}")
            print(f"    Only in {l2[:1]}: {len(only2)}")
            print(f"    Common: {len(common)}")
            if only1:
                samples = sorted(only1, key=lambda x: (x.count("("), x))[:3]
                for s in samples:
                    print(f"      Only in {l1[:1]}: {s}")
            if only2:
                samples = sorted(only2, key=lambda x: (x.count("("), x))[:3]
                for s in samples:
                    print(f"      Only in {l2[:1]}: {s}")
