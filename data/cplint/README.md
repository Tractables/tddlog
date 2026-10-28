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
