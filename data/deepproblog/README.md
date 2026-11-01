# DeepProbLog's handwritten formulas

The logic program of the handwritten-formula task (HWF) of DeepProbLog,
which `scripts/gen_hwf.py` reads (`bench.py --suite lit`, family `hwf`).

## Source and license

Robin Manhaeve, Sebastijan Dumančić, Angelika Kimmig, Thomas Demeester
and Luc De Raedt, "Neural probabilistic logic programming in
DeepProbLog", Artificial Intelligence, 2021
(<https://www.sciencedirect.com/science/article/abs/pii/S0004370221000552>,
as the repository's README cites it). The file is from the
repository ML-KULeuven/deepproblog on GitHub at commit
f64181f66a3a789a96a47d7c2d3fae4d56178990, under the Apache License 2.0;
`LICENSE` is the repository's license file.

## Files

- `hwf_model.pl`: a copy of `src/deepproblog/examples/HWF/model.pl`. Its
  two neural predicates classify each image of a formula as a digit or
  an operator; `parse/2` evaluates the formula's symbols and
  `almost_equal/2` compares the value with the label, or, called with
  the label unbound, computes it.
