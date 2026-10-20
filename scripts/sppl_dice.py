#!/usr/bin/env python3
"""Run an SPPL program that dice2sppl.py wrote: compile it with SPPL's
compiler (`SPPL_Compiler(...).execute_module()`, as SPPL's notebooks do)
and ask `prob` for each value of each component of the Dice program's
value, one marginal at a time, as named by its `# component K TYPE NAME
VALUES` lines (`# component K TYPE const V` for a constant).
With a `# joint` line the components are Booleans asked jointly: it asks
the probability of each assignment of them, printed as component 0, the
int whose bits they are (the first the most significant).

    sppl_dice.py PROGRAM.sppl

prints `sppl_c K VALUE P` for each value of each component (VALUE `true`
or `false` for a Boolean) and `sppl_seconds S`, the compile and the
queries. Run it with the Python that has sppl installed (`-I`: it imports
nothing of tddlog's).
"""
import sys
import threading
import time


def marginals(model, comps, Id):
    """(K, VALUE, P) for each value of each component."""
    for c in comps:
        k, ty = c[0], c[1]
        width = None if ty == 'bool' else int(c[2])
        rest = c[2:] if width is None else c[3:]
        values = [0, 1] if width is None else list(range(2 ** width))
        if rest[0] == 'const':
            probs = {v: float(v == int(rest[1])) for v in values}
        elif width is None:   # a Boolean: its one query
            p = float(model.prob(Id(rest[0]) << {1}))
            probs = {0: 1 - p, 1: p}
        else:
            name, support = Id(rest[0]), {int(v) for v in rest[1].split(',')}
            probs = {v: float(model.prob(name << {v})) if v in support else 0.0 for v in values}
        for v, p in probs.items():
            yield k, ('true' if v else 'false') if width is None else str(v), p


def joint(model, comps, Id):
    """(0, U, P) for each assignment of the Boolean components, U the int
    whose bits they are, the first the most significant."""
    n = len(comps)
    for u in range(2 ** n):
        event, possible = None, True
        for i, c in enumerate(comps):
            bit = (u >> (n - 1 - i)) & 1
            if c[2] == 'const':
                possible &= int(c[3]) == bit
            else:
                event = Id(c[2]) << {bit} if event is None else event & (Id(c[2]) << {bit})
        yield '0', str(u), float(possible) if event is None or not possible else float(model.prob(event))


def run(path):
    from sppl.compilers.sppl_to_python import SPPL_Compiler
    from sppl.transforms import Id
    text = open(path).read()
    comps, asked = [], marginals
    for line in text.splitlines():
        f = line.split()
        if f[:2] == ['#', 'component']:
            comps.append(f[2:])
        elif f == ['#', 'joint']:
            asked = joint
    start = time.time()
    model = SPPL_Compiler(text).execute_module().model
    for k, value, p in asked(model, comps, Id):
        print('sppl_c', k, value, repr(p))
    print('sppl_seconds', time.time() - start)


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    # SPPL's compile and its sum-product expressions recurse as deep as
    # the program nests and sequences its branches
    sys.setrecursionlimit(1_000_000)
    threading.stack_size(1 << 29)
    failed = []

    def target():
        try:
            run(sys.argv[1])
        except BaseException as err:
            failed.append(err)
    t = threading.Thread(target=target)
    t.start()
    t.join()
    if failed:
        raise failed[0]


if __name__ == '__main__':
    main()
