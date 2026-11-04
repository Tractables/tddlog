;; A two-bit counter: its register counts up where the enable, the
;; conjunction of an input and a one, is false and holds where it is true,
;; and the output's high bit is the register's two bits' conjunction. Wire
;; 11 is a gate no output reads, outside Circulette's slice, and no fault.
;; Written for tddlog's tests in the netlist format of Circulette.
(def-netlist counter
  (hash 0 1)
  (hash 1 2)
  (hash 2 2)
  (3 (ws 2 (list 0)))
  (4 (ws 2 (list 1)))
  (5 (w~ 3))
  (6 (w^ 4 3))
  (7 (wc (list 6 5)))
  (10 (<w= (bv-const 1 1)))
  (12 (w& 0 10))
  (8 (wx 12 7 2))
  (2 (<<= 8))
  (9 (w& 3 4))
  (11 (w& 10 3))
  (1 (<<= (wc (list 9 (bv-const 0 1))))))

(define p-clean 0.99)
(define p-faulty 0.01)
(define expected-output (list #f #f #f #f))
(define internal-signals (hash 3 4 5))
(sketch-reroll counter (loops ))
