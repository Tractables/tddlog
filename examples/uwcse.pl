% UW-CSE (cplint uwcse3s example, Meert, Struyf, Blockeel ILP 2009): a course, a
% professor and two students; annotated disjunctions and negation, acyclic.
% The formula of taught_by aggregates over the students, so it grows with
% them: with four (tests/it/examples.rs, `uwcse_four_students`) the
% program has 120 facts and the query's formula under `closed` is about
% 36 k pairs. Run with `tddlog examples/uwcse.pl`.
course(c1).
student(s1).
student(s2).
professor(p1).
0.10708782742681::advised_by(A,B) :- student(A),
    professor(B),
    position(B,faculty).
0.0278422273781903::advised_by(A,B) :- student(A),
    professor(B),
    \+position(B,faculty).
0.0666666666666667::course_level(A,level_300); 0.318518518518519::course_level(A,level_400); 0.614814814814815::course_level(A,level_500) :- course(A).
0.935483870967742::phase(A,post_Generals); 0.032258064516129::phase(A,post_Quals); 0.032258064516129::phase(A,pre_Quals) :- student(A),
    years_in_program(A,year_6_or_more).
0.04::phase(A,post_Generals); 0.04::phase(A,post_Quals); 0.92::phase(A,pre_Quals) :- student(A),
    \+years_in_program(A,year_6_or_more),
    years_in_program(A,year_1).
0.566666666666667::phase(A,post_Generals); 0.366666666666667::phase(A,post_Quals); 0.0666666666666667::phase(A,pre_Quals) :- student(A),
    \+years_in_program(A,year_6_or_more),
    \+years_in_program(A,year_1),
    years_in_program(A,year_5).
0.222222222222222::phase(A,post_Generals); 0.666666666666667::phase(A,post_Quals); 0.111111111111111::phase(A,pre_Quals) :- student(A),
    \+years_in_program(A,year_6_or_more),
    \+years_in_program(A,year_1),
    \+years_in_program(A,year_5),
    years_in_program(A,year_4).
0.0476190476190476::phase(A,post_Generals); 0.547619047619048::phase(A,post_Quals); 0.404761904761905::phase(A,pre_Quals) :- student(A),
    \+years_in_program(A,year_6_or_more),
    \+years_in_program(A,year_1),
    \+years_in_program(A,year_5),
    \+years_in_program(A,year_4).
0.732142857142857::position(A,faculty); 0.125::position(A,faculty_adjunct); 0.0714285714285714::position(A,faculty_affiliate); 0.0714285714285714::position(A,faculty_emeritus) :- professor(A).
aux1_prof_nb_publications(A) :- advised_by(B,A),
    student_nb_publications(B,three_or_more).
0.0416666666666667::prof_nb_publications(A,none); 0.416666666666667::prof_nb_publications(A,one_to_nine); 0.541666666666667::prof_nb_publications(A,ten_or_more) :- professor(A),
    aux1_prof_nb_publications(A).
0.5::prof_nb_publications(A,none); 0.294117647058824::prof_nb_publications(A,one_to_nine); 0.205882352941176::prof_nb_publications(A,ten_or_more) :- professor(A),
    \+aux1_prof_nb_publications(A).
0.00101010101010101::shared_publication(A,B) :- professor(A),
    student(B),
    student_nb_publications(B,none).
0.956521739130435::shared_publication(A,B) :- professor(A),
    student(B),
    \+student_nb_publications(B,none),
    advised_by(B,A),
    years_in_program(B,year_6_or_more).
0.125::shared_publication(A,B) :- professor(A),
    student(B),
    \+student_nb_publications(B,none),
    advised_by(B,A),
    \+years_in_program(B,year_6_or_more),
    prof_nb_publications(A,none).
0.631578947368421::shared_publication(A,B) :- professor(A),
    student(B),
    \+student_nb_publications(B,none),
    advised_by(B,A),
    \+years_in_program(B,year_6_or_more),
    \+prof_nb_publications(A,none).
0.00515463917525773::shared_publication(A,B) :- professor(A),
    student(B),
    \+student_nb_publications(B,none),
    \+advised_by(B,A),
    prof_nb_publications(A,none).
0.099236641221374::shared_publication(A,B) :- professor(A),
    student(B),
    \+student_nb_publications(B,none),
    \+advised_by(B,A),
    \+prof_nb_publications(A,none),
    prof_nb_publications(A,one_to_nine),
    student_nb_publications(B,one_or_two).
0.0152671755725191::shared_publication(A,B) :- professor(A),
    student(B),
    \+student_nb_publications(B,none),
    \+advised_by(B,A),
    \+prof_nb_publications(A,none),
    prof_nb_publications(A,one_to_nine),
    \+student_nb_publications(B,one_or_two).
0.146017699115044::shared_publication(A,B) :- professor(A),
    student(B),
    \+student_nb_publications(B,none),
    \+advised_by(B,A),
    \+prof_nb_publications(A,none),
    \+prof_nb_publications(A,one_to_nine).
0.594405594405594::student_nb_publications(A,none); 0.195804195804196::student_nb_publications(A,one_or_two); 0.20979020979021::student_nb_publications(A,three_or_more) :- student(A).
aux1_taught_by(A,B) :- teaching_assistant(A,C),
    advised_by(C,B).
aux2_taught_by(A,B) :- teaching_assistant(A,C),
    advised_by(C,B),
    shared_publication(B,C).
0.777777777777778::taught_by(A,B) :- course(A),
    professor(B),
    aux1_taught_by(A,B),
    aux2_taught_by(A,B).
0.333333333333333::taught_by(A,B) :- course(A),
    professor(B),
    aux1_taught_by(A,B),
    \+aux2_taught_by(A,B).
0.266666666666667::taught_by(A,B) :- course(A),
    professor(B),
    \+aux1_taught_by(A,B),
    position(B,faculty),
    course_level(A,level_300).
0.102941176470588::taught_by(A,B) :- course(A),
    professor(B),
    \+aux1_taught_by(A,B),
    position(B,faculty),
    \+course_level(A,level_300).
0.029673590504451::taught_by(A,B) :- course(A),
    professor(B),
    \+aux1_taught_by(A,B),
    \+position(B,faculty).
0.0333333333333333::teaching_assistant(A,B) :- course(A),
    student(B),
    course_level(A,level_300),
    student_nb_publications(B,three_or_more).
0.158227848101266::teaching_assistant(A,B) :- course(A),
    student(B),
    course_level(A,level_300),
    \+student_nb_publications(B,three_or_more).
0.0337519623233909::teaching_assistant(A,B) :- course(A),
    student(B),
    \+course_level(A,level_300),
    course_level(A,level_400).
0.0120999219359875::teaching_assistant(A,B) :- course(A),
    student(B),
    \+course_level(A,level_300),
    \+course_level(A,level_400).
0.244444444444444::years_in_program(A,year_1); 0.155555555555556::years_in_program(A,year_2); 0.233333333333333::years_in_program(A,year_3); 0.144444444444444::years_in_program(A,year_4); 0.111111111111111::years_in_program(A,year_5); 0.111111111111111::years_in_program(A,year_6_or_more) :- student(A),
    student_nb_publications(A,none).
0.032258064516129::years_in_program(A,year_1); 0.0645161290322581::years_in_program(A,year_2); 0.0645161290322581::years_in_program(A,year_3); 0.209677419354839::years_in_program(A,year_4); 0.306451612903226::years_in_program(A,year_5); 0.32258064516129::years_in_program(A,year_6_or_more) :- student(A),
    \+student_nb_publications(A,none).
query(taught_by(c1,p1)).
