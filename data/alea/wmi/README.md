# Weighted model integration

`path.jl` and `star.jl` are the path- and star-shaped problems of the
papers on weighted model integration with tree-shaped primal graphs:
Zhe Zeng, Guy Van den Broeck, "Efficient Search-Based Weighted Model
Integration", UAI 2019, and Zhe Zeng, Paolo Morettin, Fanqi Yan, Antonio
Vergari, Andrea Passerini, Guy Van den Broeck, "Scaling up Hybrid
Probabilistic Inference with Logical and Arithmetic Constraints via
Message Passing", ICML 2020. Real variables uniform on [0, 1], neighbours
constrained by x_i + x_j <= 1, and the query x_1 <= 1/2 given the
constraints; each variable is a uniform fixed-point number. The
continuous answers: on the path of n variables the constraints have
volume E_n / n! (E_n the Euler zigzag number); on the star of n leaves the
answer is (n+1)/(2n) (1 - 2^-n) + 2^-(n+1). `bench.py --suite bitblast`
runs them as `bb_wmi_path` and `bb_wmi_star`, laddered by n at 8 bits
after the binary point.

Reference: Alea.jl's answer on the discretized program; tddlog's answers
on the circuit are checked against it in every run. At 8 bits the star's
is 0.53253 for n = 16 (continuous 0.53125) and 0.50956 for n = 64
(continuous 0.50781).
