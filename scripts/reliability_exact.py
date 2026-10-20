#!/usr/bin/env python3
"""Exact two-terminal reliability: the reference for the undirected graph
programs (gen_biomine.py, gen_graph.py --undirected), where ProbLog does not
finish.

usage: reliability_exact.py program.pl [--precision N]

Reads the edges `p::E(a,b).` of the program, each usable in both
directions (the rules `conn(X,Y) :- E(X,Y). conn(X,Y) :- E(Y,X).` and
`path/2` over `conn/2`, as those generators write them), and answers each
query `path(s,t)` with the probability that s and t are connected when
every edge is present independently with its probability. Edges listed
twice between the same two nodes are independent: the pair is connected
with 1 - (1 - p1)(1 - p2).

The method shares nothing with tddlog's: the classical reductions of
network reliability (Satyanarayana and Wood, 1985), which keep the
answer, then a frontier-based dynamic program over the edges (the method
of Hardy, Lucet and Limnios, IEEE Trans. Reliability 2007). Reductions:
a node other than s and t with one edge is dropped; one with two edges is
replaced by an edge between its neighbours with the product of the two
probabilities (series); parallel edges become one (parallel). The dynamic
program takes the edges in an order of the nodes chosen greedily to keep
the frontier (the nodes with edges on both sides of the current one) small,
the greedy order started from each node and the one with the smallest
largest frontier kept, and keeps for each state, the partition of the
frontier into the components the edges so far join and which of them
hold s and t, its probability. Its states grow with the Bell number of the frontier, which
bounds the graphs it answers.

Prints `q:\tp` per query, in the order of the query facts, as tddlog does.
"""
import argparse
import re
import sys

EDGE = re.compile(r'^([0-9.eE+-]+)::(\w+)\(([^,()]+),([^,()]+)\)\.\s*(?:%.*)?$')


def parse(text):
    """The edges as {frozenset({a, b}): p}, parallel edges combined, and the
    queries (s, t) in order."""
    edges = {}
    for line in text.splitlines():
        m = EDGE.match(line)
        if m:
            a, b = m.group(3).strip(), m.group(4).strip()
            if a == b:
                continue  # a loop joins nothing
            key = frozenset((a, b))
            edges[key] = 1.0 - (1.0 - edges.get(key, 0.0)) * (1.0 - float(m.group(1)))
    qs = []
    for m in re.finditer(r'^query\((path\(\s*([^,()]+?)\s*,\s*([^,()]+?)\s*\))\)\.\s*$', text, re.M):
        qs.append((m.group(1).replace(' ', ''), m.group(2), m.group(3)))
    return edges, qs


def reduce(edges, s, t):
    """The graph with the reductions applied until none applies: a dict
    {node: {neighbour: p}} that holds the component of s."""
    adj = {}
    for key, p in edges.items():
        a, b = tuple(key)
        adj.setdefault(a, {})[b] = p
        adj.setdefault(b, {})[a] = p
    # The component of s: nothing outside it can connect s and t.
    seen, todo = {s}, [s]
    while todo:
        for w in adj.get(todo.pop(), {}):
            if w not in seen:
                seen.add(w)
                todo.append(w)
    adj = {v: {w: p for w, p in adj.get(v, {}).items()} for v in seen}
    todo = [v for v in adj if v not in (s, t)]
    while todo:
        v = todo.pop()
        if v not in adj or v in (s, t):
            continue
        nb = adj[v]
        if len(nb) > 2:
            continue
        del adj[v]
        for w in nb:
            del adj[w][v]
            todo.append(w)
        if len(nb) == 2:
            (a, pa), (b, pb) = nb.items()
            p = pa * pb
            q = adj[a].get(b, 0.0)
            adj[a][b] = adj[b][a] = 1.0 - (1.0 - q) * (1.0 - p)
    return adj


def order(adj, s):
    """The nodes in an order that keeps the frontier small: from s, each next
    node the one that leaves the fewest nodes with edges still to take, ties
    to the one with most taken neighbours, then by name."""
    placed, out = set(), []
    left = {v: len(nb) for v, nb in adj.items()}   # edges not yet taken
    frontier = set()
    nxt = s
    while True:
        placed.add(nxt)
        out.append(nxt)
        for w in adj[nxt]:
            if w in placed:
                left[w] -= 1
                left[nxt] -= 1
        frontier = {v for v in frontier | {nxt} if left[v] > 0}
        cands = {w for v in frontier for w in adj[v] if w not in placed}
        if not cands:
            rest = sorted(v for v in adj if v not in placed)
            if not rest:
                return out
            cands = {rest[0]}

        def cost(w):
            taken = sum(1 for u in adj[w] if u in placed)
            closes = sum(1 for u in adj[w] if u in frontier and left[u] == 1)
            return (len(frontier) + (len(adj[w]) > taken) - closes, -taken, w)
        nxt = min(cands, key=cost)


def edge_order(adj, nodes):
    """The edges in the order the dynamic program takes them for a node
    order, each `(a, b, p)`, and the largest frontier it meets."""
    pos = {v: i for i, v in enumerate(nodes)}
    edges = [(a, b, p) for _, _, a, b, p in sorted((max(pos[a], pos[b]), min(pos[a], pos[b]), a, b, p)
                                                   for a in adj for b, p in adj[a].items() if pos[a] < pos[b])]
    last = {}
    for k, (a, b, _) in enumerate(edges):
        last[a] = last[b] = k
    front, width = set(), 0
    for k, (a, b, _) in enumerate(edges):
        front |= {a, b}
        width = max(width, len(front))
        front = {v for v in front if last[v] > k}
    return edges, width


def connected(adj, s, t):
    """P(s and t connected) by the frontier dynamic program, over the
    greedy node order (`order`) with the smallest frontier among those
    started from each node, s first and then by name on ties."""
    if s == t:
        return 1.0
    if t not in adj:
        return 0.0
    best = None
    for start in [s] + sorted(v for v in adj if v != s):
        edges, width = edge_order(adj, order(adj, start))
        if best is None or width < best[1]:
            best = edges, width
    edges = best[0]
    last = {}
    for k, (a, b, _) in enumerate(edges):
        last[a] = last[b] = k
    front = []           # the frontier's nodes, in the order of the labels
    # A state: (labels of the frontier's nodes, label of s's part or -1
    # before s enters, label of t's part or -1) -> probability.
    states = {((), -1, -1): 1.0}
    done = 0.0
    for k, (a, b, p) in enumerate(edges):
        for v in (a, b):
            if v not in front:
                front.append(v)
                grown = {}
                for (labels, sc, tc), w in states.items():
                    new = max(labels, default=-1) + 1
                    key = (labels + (new,), new if v == s else sc, new if v == t else tc)
                    grown[key] = grown.get(key, 0.0) + w
                states = grown
        ia, ib = front.index(a), front.index(b)
        gone = {i for i, v in enumerate(front) if last[v] == k}
        nxt = {}

        def add(labels, sc, tc, w):
            key = canonical(tuple(l for i, l in enumerate(labels) if i not in gone), sc, tc)
            if key is not None:
                nxt[key] = nxt.get(key, 0.0) + w
        for (labels, sc, tc), w in states.items():
            if p < 1.0:
                add(labels, sc, tc, w * (1.0 - p))
            if p > 0.0:
                x, y = labels[ia], labels[ib]
                if x != y:
                    labels = tuple(x if l == y else l for l in labels)
                    sc, tc = (x if sc == y else sc), (x if tc == y else tc)
                if sc != -1 and sc == tc:
                    done += w * p
                else:
                    add(labels, sc, tc, w * p)
        states = nxt
        front = [v for i, v in enumerate(front) if i not in gone]
    return done


def canonical(labels, sc, tc):
    """The state with labels renumbered by first occurrence, or None where
    the part of s or of t has left the frontier: s and t can no longer be
    joined."""
    for x in (sc, tc):
        if x != -1 and x not in labels:
            return None
    rename = {}
    for l in labels:
        rename.setdefault(l, len(rename))
    return (tuple(rename[l] for l in labels), rename.get(sc, -1), rename.get(tc, -1))


def answers(text):
    edges, qs = parse(text)
    out = {}
    for q, s, t in qs:
        out[q] = connected(reduce(edges, s, t), s, t)
    return out, [q for q, _, _ in qs]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('program')
    ap.add_argument('--precision', type=int, default=6)
    a = ap.parse_args()
    res, qs = answers(open(a.program).read())
    for q in qs:
        print(f'{q}:\t{res[q]:.{a.precision}f}')


if __name__ == '__main__':
    main()
