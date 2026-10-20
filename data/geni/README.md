# Geni's weather factory programs

The Dice programs of the weather factory benchmark in Geni's evaluation,
which `scripts/gen_geni.py` reads (`bench.py --suite geni`,
`geni_weather_n` and `geni_weather_h`). The grid networks of the same
evaluation are written by `gen_geni.py` itself.

## Source and license

Jianlin Li and Yizhou Zhang, "Compiling with Generating Functions",
ICFP 2025 (PACMPL 9, article 265). The files are from the paper's
artifact, the repository geni-icfp25-ae/geni-icfp25-ae on GitHub at
commit 9e5ffb25548258849ce58105feef395514c8df94, under the MIT License,
Copyright (c) 2025 Jianlin Li, Oghenevwogaga Ebresafe; `LICENSE` is the
artifact's license file.

## Files

- `weather_factory/weather-factory-N-10.dice`: copies of the artifact's
  `bench/ICFP/weather-factory/rubicon-dice/weather-factory-N-10.dice`
  for N = 2, 7 to 13 and 15 to 19, unchanged. The artifact's
  `bench.rubicon.sh` wrote them with Rubicon (Holtzen, Junges,
  Vazquez-Chanlatte, Millstein, Seshia and Van den Broeck, CAV 2021)
  from Rubicon's `examples/weatherfactory/weatherfactoryN.prism` and the
  property `P=? [F<=10 "allStrike"]`: the probability that all N
  factories are on strike within 10 days.

The artifact's programs for 10 factories at H = 1 to 9 and 100 to 1000
days differ from `weather-factory-10-10.dice` only in the number of
steps after `let _goal = false in`; `gen_geni.py weather 10 H --dice
PATH` writes each of them byte for byte from that file.
