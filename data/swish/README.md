# Programs from cplint on SWISH

Copies of example programs of cplint on SWISH, the web interface to
Fabrizio Riguzzi's cplint (<https://github.com/friguzzi/swish>, a fork of
SWISH, `examples/inference/`, commit
127bc19907a118402690dac040b3b10ee3670d10), under SWISH's Simplified BSD
License, whose text is `LICENSE` here. They join the `cplintpita` benchmark
family as `cplint_swish_<name>` (`scripts/gen_cplint.py NAME swish`): each
is read as it is by tddlog's LPAD reader and converted by `tddlog
--program` for the other systems, and ProbLog's answers, one query at a
time, are the reference.

- `admission.pl`: college admission by gender, department and
  qualification (Pearl, Causality, 2009, Section 4.5.3), asked whether a
  student is admitted given that she is female.
- `smoking.pl`: smoking, tar and cancer (Pearl, Causality, 2009, Section
  3.3.3), asked the probability of cancer given a smoker.
- `sneezing_simple.pl`: flu and hay fever as causes of sneezing, a
  noisy-or (Riguzzi and Swift, TPLP 2011).
- `viraltab.pl`: viral marketing over mutual trust, tabled, asked whether
  person 4 buys.

The folder's other programs written for PITA are left out: the copies of
the cplint pack's examples (the same model and queries, at most comments,
spelling or another task's examples differing: alarm, bloodtype, coin,
cora, earthquake, hmm, jail, light, markov_chaindb, mendel, mendelc,
monty, path_tabling, sneezing, and those copied byte for byte), which
`cplintpita` holds or leaves out for the reasons `data/cplint/README.md`
gives, the `_R` variants that add R plots to a program
(coinmsw_R and the rest), sneezing_simple_msw and sneezing_simple_pb
(sneezing_simple written with PRISM's msw/2 and with probabilistic facts),
viralpb (cplint's viral with its rule's probability written as a fact),
those that ask another task (MPE, abduction, eruption_vit, pitampe_hmm,
tiny_event_calculus) and the notebooks (`.swinb`). The programs written
for MCINTYRE, cplint's sampler, are not exact inference.
