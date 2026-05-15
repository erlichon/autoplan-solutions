#!/usr/bin/env python3
"""Reference h^2 invariant extractor for cross-checking the Rust implementation."""
import sys
import re

INF = float('inf')

def parse_sas(text):
    lines = text.split('\n')
    idx = 0
    def next_line():
        nonlocal idx
        while idx < len(lines) and lines[idx].strip() == '':
            idx += 1
        if idx < len(lines):
            l = lines[idx]
            idx += 1
            return l.strip()
        return None

    assert next_line() == 'begin_version'
    next_line()  # version
    assert next_line() == 'end_version'
    assert next_line() == 'begin_metric'
    metric = int(next_line())
    assert next_line() == 'end_metric'

    num_vars = int(next_line())
    variables = []
    for _ in range(num_vars):
        assert next_line() == 'begin_variable'
        name = next_line()
        axiom = int(next_line())
        domain_size = int(next_line())
        values = [next_line() for _ in range(domain_size)]
        assert next_line() == 'end_variable'
        variables.append({'name': name, 'axiom': axiom, 'domain': domain_size, 'values': values})

    num_mutex = int(next_line())
    for _ in range(num_mutex):
        assert next_line() == 'begin_mutex_group'
        n = int(next_line())
        for _ in range(n):
            next_line()
        assert next_line() == 'end_mutex_group'

    assert next_line() == 'begin_state'
    init = [int(next_line()) for _ in range(num_vars)]
    assert next_line() == 'end_state'

    assert next_line() == 'begin_goal'
    num_goal = int(next_line())
    goal = []
    for _ in range(num_goal):
        parts = next_line().split()
        goal.append((int(parts[0]), int(parts[1])))
    assert next_line() == 'end_goal'

    num_ops = int(next_line())
    operators = []
    for _ in range(num_ops):
        assert next_line() == 'begin_operator'
        op_name = next_line()
        num_prevail = int(next_line())
        prevail = []
        for _ in range(num_prevail):
            parts = next_line().split()
            prevail.append((int(parts[0]), int(parts[1])))
        num_effects = int(next_line())
        effects = []
        for _ in range(num_effects):
            nums = list(map(int, next_line().split()))
            nc = nums[0]
            conds = [(nums[1+2*i], nums[2+2*i]) for i in range(nc)]
            base = 1 + 2*nc
            var = nums[base]
            pre = nums[base+1]
            post = nums[base+2]
            effects.append({'conds': conds, 'var': var, 'pre': pre, 'post': post})
        cost = int(next_line())
        if metric == 0:
            cost = 1
        assert next_line() == 'end_operator'
        operators.append({'name': op_name, 'prevail': prevail, 'effects': effects, 'cost': cost})

    num_axioms = int(next_line())
    for _ in range(num_axioms):
        assert next_line() == 'begin_rule'
        nc = int(next_line())
        for _ in range(nc):
            next_line()
        next_line()  # effect
        assert next_line() == 'end_rule'

    return variables, init, goal, operators


def compute_h2(variables, init, operators):
    n = len(variables)
    offset = [0] * (n + 1)
    for i in range(n):
        offset[i+1] = offset[i] + variables[i]['domain']
    nf = offset[n]

    fid = lambda v, d: offset[v] + d
    fvar = [0] * nf
    fval = [0] * nf
    for v in range(n):
        for d in range(variables[v]['domain']):
            fvar[offset[v]+d] = v
            fval[offset[v]+d] = d

    cost = [[INF]*nf for _ in range(nf)]
    init_facts = [fid(v, init[v]) for v in range(n)]
    for f1 in init_facts:
        for f2 in init_facts:
            cost[f1][f2] = 0

    def facts_consistent(facts):
        seen = {}
        for f in facts:
            v, d = fvar[f], fval[f]
            if v in seen and seen[v] != d:
                return False
            seen[v] = d
        return True

    def h2_of_set(s):
        m = 0
        for p in s:
            for q in s:
                c = cost[p][q]
                if c == INF:
                    return INF
                m = max(m, c)
        return m

    # Precompute operator data
    for iteration in range(10000):
        changed = False
        for op in operators:
            op_pre = []
            for v, d in op['prevail']:
                op_pre.append(fid(v, d))
            for e in op['effects']:
                if e['pre'] >= 0:
                    op_pre.append(fid(e['var'], e['pre']))
            if not facts_consistent(op_pre):
                continue

            assigned = [False] * n
            for e in op['effects']:
                assigned[e['var']] = True

            # Case A: pairs of effects
            effs = op['effects']
            for i in range(len(effs)):
                for j in range(i, len(effs)):
                    e1, e2 = effs[i], effs[j]
                    if i != j and e1['var'] == e2['var']:
                        continue
                    p1 = fid(e1['var'], e1['post'])
                    p2 = fid(e2['var'], e2['post']) if i != j else p1
                    pre_set = list(op_pre)
                    for cv, cd in e1['conds']:
                        pre_set.append(fid(cv, cd))
                    if i != j:
                        for cv, cd in e2['conds']:
                            pre_set.append(fid(cv, cd))
                    if not facts_consistent(pre_set):
                        continue
                    h = h2_of_set(pre_set)
                    if h == INF:
                        continue
                    new = op['cost'] + h
                    if i == j:
                        if new < cost[p1][p1]:
                            cost[p1][p1] = new
                            changed = True
                    else:
                        if new < cost[p1][p2]:
                            cost[p1][p2] = new
                            cost[p2][p1] = new
                            changed = True

            # Case B: one effect + one persisting fact
            for e in effs:
                p1 = fid(e['var'], e['post'])
                pre_base = list(op_pre)
                for cv, cd in e['conds']:
                    pre_base.append(fid(cv, cd))
                if not facts_consistent(pre_base):
                    continue
                base_h = h2_of_set(pre_base)
                if base_h == INF:
                    continue

                for v in range(n):
                    if assigned[v]:
                        continue
                    for d in range(variables[v]['domain']):
                        p2 = fid(v, d)
                        if cost[p2][p2] == INF:
                            continue
                        extended = pre_base + [p2]
                        if not facts_consistent(extended):
                            continue
                        h = h2_of_set(extended)
                        if h == INF:
                            continue
                        new = op['cost'] + h
                        if new < cost[p1][p2]:
                            cost[p1][p2] = new
                            cost[p2][p1] = new
                            changed = True

        if not changed:
            break

    return cost, offset


def extract_invariants(variables, cost, offset):
    n = len(variables)
    out = []
    for v1 in range(n):
        d1 = variables[v1]['domain']
        for x1 in range(d1):
            for v2 in range(v1, n):
                d2 = variables[v2]['domain']
                x2_start = x1 + 1 if v2 == v1 else 0
                for x2 in range(x2_start, d2):
                    f1 = offset[v1] + x1
                    f2 = offset[v2] + x2

                    # --
                    if cost[f1][f2] == INF:
                        out.append(f"- {v1} {x1} - {v2} {x2}")

                    # +-
                    if all(a == x1 or cost[offset[v1]+a][f2] == INF for a in range(d1)):
                        out.append(f"+ {v1} {x1} - {v2} {x2}")

                    # -+
                    if all(b == x2 or cost[f1][offset[v2]+b] == INF for b in range(d2)):
                        out.append(f"- {v1} {x1} + {v2} {x2}")

                    # ++
                    if all(a == x1 or all(b == x2 or cost[offset[v1]+a][offset[v2]+b] == INF for b in range(d2)) for a in range(d1)):
                        out.append(f"+ {v1} {x1} + {v2} {x2}")

    out.sort()
    return out


def main():
    text = sys.stdin.read()
    variables, init, goal, operators = parse_sas(text)
    cost, offset = compute_h2(variables, init, operators)
    invariants = extract_invariants(variables, cost, offset)
    for inv in invariants:
        print(inv)


if __name__ == '__main__':
    main()
