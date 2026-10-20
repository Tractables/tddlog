#!/usr/bin/env python3
"""Generate a probabilistic-graph program from the Biomine network.

usage: gen_biomine.py GRAPH K

The workload of ProbLog's first evaluation (De Raedt, Kimmig, Toivonen,
IJCAI 2007; Kimmig et al., TPLP 2011) and of the Tp-compilation paper's
Alzheimer domain (Vlasselaer et al., IJAR 2016): the probability that
genes known to be related to Alzheimer's disease are connected to the
disease in Biomine, a network of biological entities whose links carry the
probability that the relation they record exists (Sevon, Eronen, Hintsanen,
Kulovesi, Toivonen, DILS 2006). GRAPH is a Biomine graph in the format of
https://biomine.ijs.si/downloads/ (`node1 node2 type goodness` per line,
plain or zipped): the human graph `human_3_oct_2018.zip`, licensed for
academic and non-commercial use only, so it stays out of the repository.

The program is the union of the K most probable simple paths between the
source, Alzheimer disease (OMIM 104300), and each of the four genes of
those evaluations (APP, APBB2, BCHE, PCSK2; HGNC 620, 582, 983, 8744; by
their Entrez ids, which the 2018 graph uses), the subgraph that Biomine
and the reliable-subgraph work on it extract (Hintsanen and Toivonen, DMKD
2008). Links are undirected, as in ProbLog's Biomine programs; two links
between the same nodes (of different types) are one edge holding with
1 - (1 - p1)(1 - p2). Paths are ranked by the product of their
probabilities (Yen's algorithm on -log p, each spur path found by A*
toward the gene, ties broken by the nodes' names); K grows the subgraph
from the best paths to many, denser, ones.
Each gene is queried, `path(source, gene)`.
"""
import argparse
import heapq
import io
import math
import re
import sys
import zipfile

SOURCE = 'Phenotype_MIM:104300'
GENES = {'app': 'Gene_EntrezGene:351', 'apbb2': 'Gene_EntrezGene:323', 'bche': 'Gene_EntrezGene:590',
         'pcsk2': 'Gene_EntrezGene:5126'}


def lines(path):
    if path.endswith('.zip'):
        with zipfile.ZipFile(path) as z:
            names = [n for n in z.namelist() if n.endswith('.bmg')]
            if len(names) != 1:
                sys.exit(f'{path}: not one .bmg file in the archive')
            with z.open(names[0]) as f:
                yield from io.TextIOWrapper(f, encoding='utf-8')
    else:
        with open(path, encoding='utf-8') as f:
            yield from f


def load(path):
    """The graph as node names, and for each node number {neighbour: -log p}
    with the links between two nodes combined."""
    number, names, adj = {}, [], []

    def node(name):
        i = number.get(name)
        if i is None:
            i = number[name] = len(names)
            names.append(name)
            adj.append({})
        return i
    for line in lines(path):
        if line.startswith('#'):
            continue
        f = line.split()
        if len(f) != 4:
            continue
        a, b, p = node(f[0]), node(f[1]), float(f[3])
        if a == b or p <= 0.0:
            continue
        # Kept as the probability that no link between a and b holds.
        q = adj[a].get(b, 1.0) * (1.0 - min(p, 1.0))
        adj[a][b] = adj[b][a] = q
    for nb in adj:
        for b, q in nb.items():
            nb[b] = math.inf if q >= 1.0 else -math.log1p(-q)
    return number, names, adj


def dijkstra(adj, s, names):
    """The distance from s of every node that s reaches, ties broken by
    the nodes' names."""
    dist = {s: 0.0}
    heap = [(0.0, names[s], s)]
    done = set()
    while heap:
        d, _, v = heapq.heappop(heap)
        if v in done:
            continue
        done.add(v)
        for w, c in adj[v].items():
            nd = d + c
            if nd < dist.get(w, math.inf):
                dist[w] = nd
                heapq.heappush(heap, (nd, names[w], w))
    return dist


def shortest(adj, s, t, h, names, banned_nodes=(), banned_edges=()):
    """The shortest path from s to t that avoids the banned nodes and
    edges, or None: A* with h, the distances to t in the whole graph, a
    consistent lower bound, so that the search stays near the paths to t."""
    if s not in h:
        return None
    best, pred = {s: 0.0}, {s: None}
    heap = [(h[s], names[s], s)]
    done = set()
    while heap:
        _, _, v = heapq.heappop(heap)
        if v in done:
            continue
        if v == t:
            out = []
            while v is not None:
                out.append(v)
                v = pred[v]
            return out[::-1]
        done.add(v)
        for w, c in adj[v].items():
            if w in done or w not in h or w in banned_nodes or (v, w) in banned_edges:
                continue
            nd = best[v] + c
            if nd < best.get(w, math.inf):
                best[w], pred[w] = nd, v
                heapq.heappush(heap, (nd + h[w], names[w], w))
    return None


def cost(adj, path):
    return sum(adj[a][b] for a, b in zip(path, path[1:]))


def best_paths(adj, s, t, k, names):
    """The k shortest simple paths from s to t (Yen 1971), each spur path
    found by A* toward t; ties broken by the nodes' names."""
    h = dijkstra(adj, t, names)
    first = shortest(adj, s, t, h, names)
    if first is None:
        return []
    found, pending, queued = [first], [], {tuple(first)}
    while len(found) < k:
        prev = found[-1]
        for i in range(len(prev) - 1):
            spur, root = prev[i], prev[:i + 1]
            banned_edges = set()
            for p in found:
                if p[:i + 1] == root:
                    banned_edges |= {(p[i], p[i + 1]), (p[i + 1], p[i])}
            tail = shortest(adj, spur, t, h, names, set(root[:-1]), banned_edges)
            if tail is not None:
                path = root[:-1] + tail
                if tuple(path) not in queued:
                    queued.add(tuple(path))
                    heapq.heappush(pending, (cost(adj, path), [names[v] for v in path], path))
        if not pending:
            break
        found.append(heapq.heappop(pending)[2])
    return found


def ident(name):
    """A node's Biomine id as a ProbLog constant: lower case, every other
    character an underscore."""
    return re.sub(r'[^a-z0-9]', '_', name.lower())


def program(label, graph, k):
    """The program of the K best paths, as lines, for the graph `load`
    returned; `label` names the graph in the header."""
    number, names, adj = graph
    for name in [SOURCE, *GENES.values()]:
        if name not in number:
            sys.exit(f'{label}: no node {name}')
    s = number[SOURCE]
    edges = set()
    counts = {}
    for gene, name in GENES.items():
        paths = best_paths(adj, s, number[name], k, names)
        counts[gene] = len(paths)
        for p in paths:
            edges |= {(min(u, v), max(u, v)) for u, v in zip(p, p[1:])}
    nodes = {v for e in edges for v in e}
    idents = {v: ident(names[v]) for v in nodes}
    if len(set(idents.values())) != len(idents):
        sys.exit('two Biomine ids give the same constant')
    out = [f'% Biomine ({label}): the {k} most probable paths between '
           f'{SOURCE} (Alzheimer disease) and each gene, {len(nodes)} nodes, {len(edges)} edges']
    for gene, name in GENES.items():
        out.append(f'% {gene} {name} {idents[number[name]]}: {counts[gene]} paths')
    for u, v in sorted(edges, key=lambda e: tuple(sorted((idents[e[0]], idents[e[1]])))):
        a, b = sorted((idents[u], idents[v]))
        p = math.exp(-adj[u][v])
        out.append(f'{p:.9f}'.rstrip('0').rstrip('.') + f'::edge({a},{b}).')
    out += ['conn(X,Y) :- edge(X,Y).', 'conn(X,Y) :- edge(Y,X).', 'path(X,Y) :- conn(X,Y).',
            'path(X,Y) :- conn(X,Z), path(Z,Y).']
    out += [f'query(path({idents[s]},{idents[number[name]]})).' for name in GENES.values()]
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('graph')
    ap.add_argument('k', type=int)
    a = ap.parse_args()
    if a.k < 1:
        ap.error('K must be at least 1')
    print('\n'.join(program(a.graph.rsplit('/', 1)[-1], load(a.graph), a.k)))


if __name__ == '__main__':
    main()
