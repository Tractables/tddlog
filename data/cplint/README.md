# Programs from cplint

Copies of example and dataset files from Fabrizio Riguzzi's cplint
(<https://github.com/friguzzi/cplint>, `prolog/examples/`, commit cb4add72329fc9b287d7da36f7610e53087f2041)
and cplint_datasets (<https://github.com/friguzzi/cplint_datasets>,
`prolog/coralearn.pl`, commit 168a98599ad4cd45216f8dc5ed1fea2e8775544f), both
under the Artistic License 2.0. They are the sources of the `uwcse`, `cora`,
`cplint`, `dnahmm`, `plcg` and `hmmpos` benchmark families: tddlog reads an
LPAD itself, and `tddlog --program` converts it to ProbLog for the other
systems (`scripts/lpad2problog.py` did before tddlog's reader replaced it).

- `bloodtype.pl`: Mendelian inheritance of blood types (Meert, Struyf,
  Blockeel, ILP 2009).
- `uwcse3s.pl`: the UW-CSE university domain as a CP-logic theory (same
  source), one course, one professor, three students.
- `cora_m1.pl`: from cplint_datasets' `coralearn.pl`, the Cora
  citation-matching learning problem, its initial theory (`in([...])`)
  and fold m1 of its five folds of facts and labels, input of
  `scripts/gen_cora.py`.
- `hmm.pl`: a hidden Markov model of DNA sequences (Christiansen and
  Gallagher, ICLP 2009), each choice of a state and of a letter
  indexed by the states before it, input of `scripts/gen_dnahmm.py`.
- `plcg.pl`: a probabilistic left-corner grammar for S -> S S | a | b
  (Sato, Kameya and Kurihara, AMAI 2008), each choice indexed by the
  derivation so far, input of `scripts/gen_plcg.py`.
- `hmmpos.pl`: a hidden Markov model for part-of-speech tagging (Torbjorn
  Lager's, adapted to MCINTYRE by Fabrizio Riguzzi), each choice of a tag
  indexed by the tags before it, input of `scripts/gen_hmmpos.py`.

The `cplintpita` benchmark family is cplint's examples written for PITA
(`:- use_module(library(pita))` with `?- prob(...)` queries), copied from
`prolog/examples/` at the same commit, each read as it is by tddlog's LPAD
reader and converted by `tddlog --program` for the other systems
(`scripts/gen_cplint.py`): alarm, bag_game, coin, coin2, dice, earthquake,
epidemic, eruption, event_calculus, flexprob, jail, light, markov_factor,
markov_network, mendel, mendelc, mln, monty, multiple_paths,
multiple_paths_simple, path, pathdb, pcfg, simpson, sneezing, testdb,
threesideddice, trigger and viral. Left out: bloodtype (the standard
suite's), uwcse3s, uwcse4s and uwcse_inf (the uwcse family's), hmm
(dnahmm's), the MAP and MPE versions coin_mpe, mendel_map and mendel_mpe,
eruption_tab (eruption with tabling, the same program), and those no
system answers as the benchmark harness gives them: coinmsw and
coinmsw_memo (PRISM's msw/2), meta and metacomb (prob/2 inside the
program), markov_chaindb, pcfglrdb and var_objdb (which no system nor
ProbLog's reference answers in time), and persons (whose LPAD tddlog's reader
refuses: a probability computed in the body, `P is F+M`).
