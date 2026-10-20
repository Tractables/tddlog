# Ported from experiments/dicejl/caesar_large.jl of Tractables/Alea.jl,
# branch arithmetic, commit 7e9ff23517086eb4518845e50218bf18b4d9f2a5
# (Apache License 2.0). Mechanical port: Dice -> Alea, @dice -> @alea, the
# BenchmarkTools harness dropped, the query moved to QUERY.
# Cao, Garg, Holtzen, Van den Broeck, Millstein (UAI 2023), Table 1: caesar-large.
using Alea

const QUERY = :pr
const IGNORE_ERRORS = true

model() = @alea begin
    char_freqs = [0.082, 0.018, 0.024, 0.04, 0.123, 0.022, 0.022, 0.066, 0.069, 0.001, 0.008, 0.045, 0.024, 0.069, 0.073, 0.018, 0.002, 0.055, 0.067, 0.092, 0.028, 0.009, 0.023, 0.001, 0.018, 0.001]
    key = uniform(DistUInt{7}, 0, 26)

    function sendChar(key::DistUInt{7}, c::DistUInt{7})
        gen = discrete(DistUInt{7}, char_freqs)
        enc = key + gen
        enc = ifelse(enc > DistUInt{7}(25), enc-DistUInt{7}(26), enc)
        observe(prob_equals(enc, c))
    end

    text = [9, 11, 8, 21, 20, 21, 2, 5, 2, 12, 13, 2, 22, 9, 11, 8, 0, 11, 20, 6, 6, 2, 7, 0, 5, 20, 7, 0, 14, 20, 0, 24, 12, 9, 9, 5, 12, 20, 11, 24, 20, 7, 24, 17, 9, 11, 24, 12, 12, 2, 15, 24, 6, 24, 20, 7, 12, 8, 25, 11, 24, 9, 11, 24, 12, 24, 7, 13, 2, 7, 0, 20, 7, 23, 11, 24, 20, 12, 8, 7, 2, 7, 0, 20, 21, 8, 14, 13, 9, 11, 8, 21, 20, 21, 2, 5, 2, 12, 13, 2]
    for char in text
        sendChar(key, DistUInt{7}(char))
    end
    key
end
