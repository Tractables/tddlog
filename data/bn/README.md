# Bayesian networks

Discrete Bayesian networks from the bnlearn repository
(<https://www.bnlearn.com/bnrepository/>) in Hugin `.net` format, the
source of the `bn` and `bnev` benchmark families: tddlog reads each
itself, and `tddlog --program` writes it for the systems that read
ProbLog as a propositional program with one annotated disjunction per
conditional probability table row, querying the marginals of the leaf
variables (`scripts/gen_bn.py` did before tddlog's reader replaced it,
and writes the other systems' files). The networks with the largest tables
(barley, diabetes, mildew, 1 to 5 MB each) and the four MUNIN
subnetworks munin1 to munin4 are the `large` suite, gzipped in
`data/bn-large`.

The networks are not in the repository. bnlearn's pages are under CC BY-SA
3.0, but no license names the network files, and each network comes from
the publication its page cites, so no license for redistributing them is
established. `data/fetch.txt` lists each file, where it is downloaded and
its SHA-256: the copies distributed with ProbLog 2
(`conversions/hugin_samples`, at the revision the URLs name), and munin1
to munin4 from bnlearn (`munin4/munin{1,2,3,4}.net.gz` on its "very
large" page). On the baselines branch, `python3 scripts/bench.py fetch`
downloads them into this directory and `data/bn-large` and checks each
SHA-256; `bench.py gen` stops, naming that command, while one is missing.
munin.net is a different version of the full MUNIN network.

asia and cancer are `examples/asia.net` and `examples/cancer.net`,
written out from their publications (the same networks, the same
programs), which the README, the tests and the baselines branch's `bench.py` read.
