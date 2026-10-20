#!/usr/bin/env python3
"""Genfer programs (SGCL, Zaiser, Murawski and Ong, NeurIPS 2023) of the
suites' instances that are Genfer's own.

    gen_genfer.py population STEPS

writes the population model of Winner and Sheldon (NeurIPS 2016), as
benchmarks/neurips2023/approx/population/population.sgcl of
github.com/fzaiser/genfer at d132c76 has it, for STEPS seasons: the four
seasons of that file (arrival rates 232.6, 420.8, 300.8, 85.6; counts 45, 98,
73, 38) repeated in turn. STEPS = 4 is the file, up to its comments. The
program's counts are not truncated: Genfer's answer is the posterior mean of
the untruncated model, which data/alea/genfer/population.jl truncates at 1023
(a probability below 1e-60).
"""
import sys

ARRIVALS = [232.6, 420.8, 300.8, 85.6]
COUNTS = [45, 98, 73, 38]


def population(steps):
    lines = ['population ~ Poisson(51.4);', '']
    for k in range(steps):
        lines += ['population ~ Binomial(population, 0.2636);',
                  f'population +~ Poisson({ARRIVALS[k % 4]});',
                  f'observe {COUNTS[k % 4]} ~ Binomial(population, 0.2);', '']
    return '\n'.join(lines + ['return population', ''])


if __name__ == '__main__':
    if len(sys.argv) != 3 or sys.argv[1] != 'population' or not sys.argv[2].isdigit():
        sys.exit(__doc__)
    sys.stdout.write(population(int(sys.argv[2])))
