% Evidence and no query, as in bn_mildew_ev and bn_survey_ev: nothing to
% answer, and the evidence consistent. ProbLog 2.3.0 prints nothing.
0.2::x; 0.5::y.
0.4::w.
z :- x.
z :- y, \+w.
evidence(z, true).
