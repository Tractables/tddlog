;; A two-bit ripple-carry adder, s = a + b, with the carry out as the sum's
;; third bit. Written for tddlog's tests in the netlist format of
;; Circulette, with its tables as hash literals.
#| The wires: 3 to 6 the operands' bits, 7 and 10 the sum's bits,
   8 the first carry and 13 the carry out. |#
(def-netlist adder2
  #hash((0 . 2) (1 . 2))
  #hash((2 . 3))
  #hash()
  (3 (ws 0 (list 0)))
  (4 (ws 0 (list 1)))
  (5 (ws 1 (list 0)))
  (6 (ws 1 (list 1)))
  (7 (w^ 3 5))
  (8 (w& 3 5))
  (9 (w^ 4 6))
  (10 (w^ 9 8))
  (11 (w& 4 6))
  (12 (w& 9 8))
  (13 (w|| 11 12))
  (2 (wc (list 7 10 13))))

(define p-clean 0.99)
(define expected-output '(#f #f #f))
(define internal-signals (hash 7 8 9 10 11 12 13))
