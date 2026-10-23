% Non-exact program (Bogaerts & Van den Broeck 2015, Example 4.15): a and b
% are undefined in every world, so no probability exists.
% `tddlog examples/unsound.pl` stops with exit status 1 and names an
% undefined atom.
0.5::e.
a :- \+b.
b :- \+a.
c :- \+b.
c :- e.
d :- a, \+c.
query(a).
query(d).
