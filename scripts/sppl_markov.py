#!/usr/bin/env python3
"""SPPL (Saad, Rinard and Mansinghka, PLDI 2021) on its own hierarchical
Markov switching model, as examples/hierarchcial-markov-switching.ipynb of
github.com/probsys/sppl at e537b0c writes it, for N steps, run as the
notebook runs it: the program is compiled, the observations are constrained
(`constrain`), and the posterior probabilities of `separated` and of the last
state come out of `prob`. The observations are those of
data/alea/sppl/markov_switching.jl (its linear congruential generator, seeded
with 2021, and the quantiles of the normal and Poisson distributions; the
bit-blasted program rounds the normal ones to its fixed-point grid, SPPL's
program takes them as they are, and normal emissions are densities there).

    sppl_markov.py N

prints `sppl_q q1 P` (P(separated)), `sppl_q q2 P` (P(last state is 1)), and
the seconds of the compile, the constrain and the two probabilities
(`sppl_seconds`). Run it with the Python that has sppl installed.
"""
import sys
import time

P_TRANSITION = [0.2, 0.8]
MU_X = [[5, 7], [5, 15]]
MU_Y = [[5, 8], [3, 8]]
MASK = (1 << 64) - 1


def observations(n):
    """(xs, ys) of the first n steps of markov_switching.jl's DATA (s = 1)."""
    import scipy.stats   # only the run needs it, not program() (the pages' capture)
    state = 2021

    def u():
        nonlocal state
        state = (state * 0x5851f42d4c957f2d + 0x14057b7ef767814f) & MASK
        return ((state >> 11) + 0.5) * 2.0 ** -53
    xs, ys, z = [], [], u() < 0.5
    for i in range(n):
        if i > 0:
            z = u() < P_TRANSITION[int(z)]
        xs.append(float(scipy.stats.norm.ppf(u(), MU_X[0][int(z)], 1.0)))
        ys.append(min(int(scipy.stats.poisson.ppf(u(), MU_Y[0][int(z)])), 31))
    return xs, ys


def program(n):
    return f'''n = {n}
p_transition = [.2, .8]
mu_x = [[5, 7], [5, 15]]
mu_y = [[5, 8], [3, 8]]

X = array(n)
Y = array(n)
Z = array(n)

separated ~= bernoulli(p=.4)
switch (separated) cases (s in [0,1]):
    Z[0] ~= bernoulli(p=.5)
    switch (Z[0]) cases (z in [0, 1]):
        X[0] ~= norm(loc=mu_x[s][z])
        Y[0] ~= poisson(mu=mu_y[s][z])
    for i in range(1, n):
        switch (Z[i-1]) cases (z in [0, 1]):
            Z[i] ~= bernoulli(p=p_transition[z])
            switch (Z[i]) cases (z in [0, 1]):
                X[i] ~= norm(loc=mu_x[s][z])
                Y[i] ~= poisson(mu=mu_y[s][z])
'''


def main():
    if len(sys.argv) != 2 or not sys.argv[1].isdigit():
        sys.exit(__doc__)
    n = int(sys.argv[1])
    from sppl.compilers.sppl_to_python import SPPL_Compiler
    xs, ys = observations(n)
    start = time.time()
    namespace = SPPL_Compiler(program(n)).execute_module()
    model = namespace.model
    observation = {**{namespace.Y[i]: ys[i] for i in range(n)}, **{namespace.X[i]: xs[i] for i in range(n)}}
    posterior = model.constrain(observation)
    p_separated = posterior.prob(namespace.separated << {1})
    p_last = posterior.prob(namespace.Z[n - 1] << {1})
    print('sppl_q q1', repr(float(p_separated)))
    print('sppl_q q2', repr(float(p_last)))
    print('sppl_seconds', time.time() - start)


if __name__ == '__main__':
    main()
