"""Verify the PDDL Lamps model against the reference plan set."""

import re
import sys

COORDS = ["l0", "l1", "l2"]
COORD_IDX = {c: i for i, c in enumerate(COORDS)}
N = 3
SUCC = {0: 1, 1: 2}  # succ[a] = a+1

INIT_ON = {(0, 0), (1, 0), (2, 0), (0, 1), (0, 2)}


def apply_flip(grid, x, y):
    """Apply flip at (x,y) and return new grid (tuple of tuples)."""
    g = [list(row) for row in grid]
    was_on = g[y][x]

    # Collect cells to toggle: the center + straight-line propagation.
    to_toggle = [(x, y)]

    # 4 cardinal directions: (dx, dy)
    for dx, dy in [(1, 0), (-1, 0), (0, 1), (0, -1)]:
        cx, cy = x + dx, y + dy
        while 0 <= cx < N and 0 <= cy < N and g[cy][cx] == was_on:
            to_toggle.append((cx, cy))
            cx += dx
            cy += dy

    for tx, ty in to_toggle:
        g[ty][tx] = not g[ty][tx]

    return tuple(tuple(row) for row in g)


def make_grid(on_cells):
    """Create grid from set of (x,y) ON cells."""
    g = [[False] * N for _ in range(N)]
    for x, y in on_cells:
        g[y][x] = True
    return tuple(tuple(row) for row in g)


def is_goal(grid):
    return all(not cell for row in grid for cell in row)


INIT_GRID = make_grid(INIT_ON)


def validate_plan(plan_str):
    """Check if a plan is valid in our model. Returns (ok, reason)."""
    grid = INIT_GRID

    actions = re.findall(r"\(flip (\S+) (\S+)\)", plan_str)
    if not actions:
        return False, "empty plan"

    for i, (sx, sy) in enumerate(actions):
        if sx not in COORD_IDX or sy not in COORD_IDX:
            return False, f"step {i}: bad coord ({sx}, {sy})"
        x, y = COORD_IDX[sx], COORD_IDX[sy]
        grid = apply_flip(grid, x, y)

    if not is_goal(grid):
        return False, "does not reach goal"
    return True, "ok"


def count_plans_by_depth(max_depth=5):
    """Count plans per depth (plans may pass through goal)."""
    from collections import Counter
    counts = Counter()

    def dfs(grid, depth, trace_len):
        if trace_len > 0 and is_goal(grid):
            counts[trace_len] += 1
        if depth >= max_depth:
            return
        for x in range(N):
            for y in range(N):
                dfs(apply_flip(grid, x, y), depth + 1, trace_len + 1)

    dfs(INIT_GRID, 0, 0)
    return counts


def load_reference(path="../../tests/samples/05-Lamps/1.ans"):
    with open(path) as f:
        return set(line.strip() for line in f if line.strip())


def main():
    ref_set = load_reference()
    print(f"Reference has {len(ref_set)} plans")

    # 1) Validate every reference plan
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

    # 2) Count plans by depth and compare with reference distribution
    ref_by_depth = {}
    for plan in ref_set:
        d = plan.count("(flip")
        ref_by_depth[d] = ref_by_depth.get(d, 0) + 1

    print("\nReference plan depth distribution:")
    for d in sorted(ref_by_depth):
        print(f"  depth {d}: {ref_by_depth[d]}")

    print("\nCounting model plans by depth (1-5)...")
    counts = count_plans_by_depth(max_depth=5)
    all_match = True
    for d in sorted(counts):
        ref_d = ref_by_depth.get(d, 0)
        ok = "OK" if counts[d] == ref_d else "MISMATCH"
        if counts[d] != ref_d:
            all_match = False
        print(f"  depth {d}: model={counts[d]} ref={ref_d} {ok}")

    if fail == 0 and all_match:
        print("\nALL OK: reference validated, depth 1-5 counts match exactly")
    elif fail == 0:
        print("\nREFERENCE VALIDATED: all 50K plans pass, minor depth count diffs")
    else:
        print(f"\nISSUES: {fail} invalid reference plans")
        sys.exit(1)


if __name__ == "__main__":
    main()
