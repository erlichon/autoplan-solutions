#!/usr/bin/env python3
"""
Semantic tests and plan enumeration for the magnets-1 PDDL model.

State representation (mirroring the PDDL):
  - at: dict magnet -> (x, y)
  - attached: set of frozensets of magnet pairs (symmetric)

Actions:
  - move(m, x, y, xnew, ynew)
    pre: at[m] == (x,y); adjacency; destination not forbidden (l2,l2)
    eff: at[m] = (xnew, ynew); for every n attached to m, at[n] = (xnew, ynew)

  - attach(m, n, x, y)
    pre: at[m] == at[n] == (x, y); m != n; not already attached
    eff: attached(m, n)
"""

from collections import deque
from copy import deepcopy
from itertools import product


COORDS = ["l0", "l1", "l2"]
MAGNETS = ["m1", "m2"]
FORBIDDEN = {("l2", "l2")}


def adjacent(x1, y1, x2, y2):
    """Manhattan distance exactly 1 on the 3x3 grid."""
    ix1, iy1 = COORDS.index(x1), COORDS.index(y1)
    ix2, iy2 = COORDS.index(x2), COORDS.index(y2)
    return abs(ix1 - ix2) + abs(iy1 - iy2) == 1


ADJ_PAIRS = [
    (x1, y1, x2, y2)
    for x1, y1, x2, y2 in product(COORDS, COORDS, COORDS, COORDS)
    if adjacent(x1, y1, x2, y2)
]


def make_state():
    return {
        "at": {"m1": ("l0", "l0"), "m2": ("l1", "l1")},
        "attached": set(),
    }


def is_goal(state):
    return state["at"]["m1"] == ("l0", "l2") and state["at"]["m2"] == ("l0", "l2")


def applicable(state, name, args):
    if name == "move":
        m, x, y, xn, yn = args
        if state["at"][m] != (x, y):
            return False
        if not adjacent(x, y, xn, yn):
            return False
        if (xn, yn) in FORBIDDEN:
            return False
        return True
    if name == "attach":
        m, n, x, y = args
        if m == n:
            return False
        if state["at"][m] != (x, y) or state["at"][n] != (x, y):
            return False
        if frozenset((m, n)) in state["attached"]:
            return False
        return True
    return False


def apply_action(state, name, args):
    s = deepcopy(state)
    if name == "move":
        m, x, y, xn, yn = args
        s["at"][m] = (xn, yn)
        for n in MAGNETS:
            if n == m:
                continue
            if frozenset((m, n)) in s["attached"]:
                s["at"][n] = (xn, yn)
    elif name == "attach":
        m, n, x, y = args
        s["attached"].add(frozenset((m, n)))
    return s


def enumerate_applicable(state):
    out = []
    for m in MAGNETS:
        x, y = state["at"][m]
        for _, _, xn, yn in [p for p in ADJ_PAIRS if p[0] == x and p[1] == y]:
            if (xn, yn) in FORBIDDEN:
                continue
            out.append(("move", (m, x, y, xn, yn)))
    for m in MAGNETS:
        for n in MAGNETS:
            if m == n:
                continue
            if state["at"][m] == state["at"][n]:
                if frozenset((m, n)) not in state["attached"]:
                    x, y = state["at"][m]
                    out.append(("attach", (m, n, x, y)))
    return out


def fmt(plan):
    return "".join(f"({n} {' '.join(a)})" for n, a in plan)


def parse_plan(s):
    out = []
    i = 0
    while i < len(s):
        if s[i] == "(":
            j = s.index(")", i)
            parts = s[i + 1:j].split()
            out.append((parts[0], tuple(parts[1:])))
            i = j + 1
        else:
            i += 1
    return out


def validate_plan(plan_str):
    state = make_state()
    for idx, (name, args) in enumerate(parse_plan(plan_str)):
        if not applicable(state, name, args):
            return False, f"step {idx}: ({name} {' '.join(args)}) not applicable"
        state = apply_action(state, name, args)
    return (True, "valid") if is_goal(state) else (False, "goal not reached")


def test_semantic_properties():
    print("=== Semantic property tests ===")

    s = make_state()
    assert not is_goal(s)
    print("  PASS: initial state is not the goal")

    assert not applicable(s, "move", ("m1", "l0", "l0", "l0", "l0")), \
        "staying still must be forbidden"
    assert not applicable(s, "move", ("m1", "l0", "l0", "l1", "l1")), \
        "diagonal move must be forbidden"
    print("  PASS: non-unit and diagonal moves rejected")

    assert not applicable(s, "attach", ("m1", "m1", "l0", "l0")), \
        "self-attach must be forbidden"
    assert not applicable(s, "attach", ("m1", "m2", "l0", "l0")), \
        "attach requires co-location"
    print("  PASS: self-attach and non-colocated attach rejected")

    s2 = apply_action(s, "move", ("m2", "l1", "l1", "l1", "l2"))
    assert not applicable(s2, "move", ("m2", "l1", "l2", "l2", "l2")), \
        "cannot move into the forbidden tile (l2,l2)"
    print("  PASS: forbidden tile (l2,l2) cannot be entered")

    s3 = apply_action(s, "move", ("m1", "l0", "l0", "l0", "l1"))
    s3 = apply_action(s3, "move", ("m2", "l1", "l1", "l0", "l1"))
    assert applicable(s3, "attach", ("m1", "m2", "l0", "l1"))
    s4 = apply_action(s3, "attach", ("m1", "m2", "l0", "l1"))
    assert not applicable(s4, "attach", ("m1", "m2", "l0", "l1"))
    assert not applicable(s4, "attach", ("m2", "m1", "l0", "l1")), \
        "attach is symmetric: cannot be reissued in any order"
    print("  PASS: attach is one-shot and symmetric")

    s5 = apply_action(s4, "move", ("m1", "l0", "l1", "l0", "l2"))
    assert s5["at"]["m1"] == ("l0", "l2")
    assert s5["at"]["m2"] == ("l0", "l2"), \
        "moving an attached magnet must drag the partner"
    print("  PASS: attached magnets move together")

    shortest = (
        "(move m1 l0 l0 l0 l1)"
        "(move m1 l0 l1 l0 l2)"
        "(move m2 l1 l1 l0 l1)"
        "(move m2 l0 l1 l0 l2)"
    )
    ok, why = validate_plan(shortest)
    assert ok, why
    print("  PASS: a 4-step non-attach plan is valid")

    attach_plan = (
        "(move m1 l0 l0 l0 l1)"
        "(move m2 l1 l1 l0 l1)"
        "(attach m1 m2 l0 l1)"
        "(move m1 l0 l1 l0 l2)"
    )
    ok, why = validate_plan(attach_plan)
    assert ok, why
    print("  PASS: a 4-step attach plan is valid")

    bad = (
        "(move m2 l1 l1 l1 l2)"
        "(move m2 l1 l2 l2 l2)"
    )
    ok, why = validate_plan(bad)
    assert not ok, "plan entering (l2,l2) should be rejected"
    print(f"  PASS: plan entering (l2,l2) correctly rejected ({why})")

    print()


def enumerate_plans(max_depth):
    """BFS-style enumeration of all plans up to max_depth ending in the goal."""
    print(f"=== Plan enumeration up to depth {max_depth} ===")
    queue = deque()
    queue.append((make_state(), []))
    by_depth = {}
    seen = set()

    while queue:
        state, plan = queue.popleft()
        d = len(plan)
        if d > max_depth:
            continue
        if is_goal(state):
            p = fmt(plan)
            if p not in seen:
                seen.add(p)
                by_depth.setdefault(d, []).append(p)
            if d < max_depth:
                for name, args in enumerate_applicable(state):
                    queue.append((apply_action(state, name, args),
                                  plan + [(name, args)]))
            continue
        for name, args in enumerate_applicable(state):
            queue.append((apply_action(state, name, args),
                          plan + [(name, args)]))

    print(f"  total plans <= depth {max_depth}: {len(seen)}")
    for d in sorted(by_depth):
        print(f"    depth {d}: {len(by_depth[d])} plans")

    if by_depth:
        d_opt = min(by_depth)
        print(f"\n  Optimal depth: {d_opt}")
        print(f"  Number of optimal plans: {len(by_depth[d_opt])}")
        for p in sorted(by_depth[d_opt])[:20]:
            print(f"    {p}")
        if len(by_depth[d_opt]) > 20:
            print(f"    ... ({len(by_depth[d_opt]) - 20} more)")

    return by_depth


if __name__ == "__main__":
    test_semantic_properties()
    enumerate_plans(max_depth=5)
    print("\nAll tests passed.")
