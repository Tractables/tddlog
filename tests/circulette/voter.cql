;; A majority voter of three bits: out = (a & b) | (a & c) | (b & c).
;; Written for tddlog's tests in the netlist format of Circulette.
(def-netlist voter
  (hash 0 1 1 1 2 1)
  (hash 3 1)
  (hash)
  (4 (w& 0 1))
  (5 (w& 0 2))
  (6 (w& 1 2))
  (7 (w|| 4 5))
  (8 (w|| 7 6))
  (3 (<w= 8)))

(define p-clean 0.99)
(define p-faulty 0.01)
(define expected-output (list #f))
(sketch-reroll voter (loops ))
