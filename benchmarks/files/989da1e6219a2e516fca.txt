% Cora citation matching: the initial theory (the in/1 list) and fold m1 of
% the facts and labels, extracted verbatim from cplint_datasets
% prolog/coralearn.pl (commit 168a98599ad4cd45216f8dc5ed1fea2e8775544f,
% Artistic License 2.0). Input of scripts/gen_cora.py.
in([
(samebib(A,B):0.3 :- samebib(A,C), samebib(C,B)),

/*(sameauthor(A,B):0.3 :- sameauthor(A,C), sameauthor(C,B)),

(sametitle(A,B):0.3 :- sametitle(A,C), sametitle(C,B)),

(samevenue(A,B):0.3 :- samevenue(A,C), samevenue(C,B)),
*/

(samebib(B,C):0.3 :-
        author(B,D),author(C,E),sameauthor(D,E)),

(samebib(B,C):0.3 :-
        title(B,D),title(C,E),sametitle(D,E)),

(samebib(B,C):0.3 :-
  venue(B,D),venue(C,E),samevenue(D,E)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_06),
  haswordvenue(C,word_06)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_1),
  haswordvenue(C,word_1)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_10),
  haswordvenue(C,word_10)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_103),
  haswordvenue(C,word_103)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_11th),
  haswordvenue(C,word_11th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_12),
  haswordvenue(C,word_12)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_1208),
  haswordvenue(C,word_1208)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_121),
  haswordvenue(C,word_121)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_13),
  haswordvenue(C,word_13)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_13th),
  haswordvenue(C,word_13th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_14),
  haswordvenue(C,word_14)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_14th),
  haswordvenue(C,word_14th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_17),
  haswordvenue(C,word_17)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_19),
  haswordvenue(C,word_19)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_1987),
  haswordvenue(C,word_1987)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_1988),
  haswordvenue(C,word_1988)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_1989),
  haswordvenue(C,word_1989)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_1994),
  haswordvenue(C,word_1994)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_1996),
  haswordvenue(C,word_1996)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_19th),
  haswordvenue(C,word_19th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_1st),
  haswordvenue(C,word_1st)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_2),
  haswordvenue(C,word_2)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_20),
  haswordvenue(C,word_20)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_20th),
  haswordvenue(C,word_20th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_21),
  haswordvenue(C,word_21)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_21st),
  haswordvenue(C,word_21st)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_21th),
  haswordvenue(C,word_21th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_24th),
  haswordvenue(C,word_24th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_25),
  haswordvenue(C,word_25)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_26),
  haswordvenue(C,word_26)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_26th),
  haswordvenue(C,word_26th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_27),
  haswordvenue(C,word_27)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_28),
  haswordvenue(C,word_28)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_285),
  haswordvenue(C,word_285)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_28th),
  haswordvenue(C,word_28th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_295),
  haswordvenue(C,word_295)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_29th),
  haswordvenue(C,word_29th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_2nd),
  haswordvenue(C,word_2nd)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_3),
  haswordvenue(C,word_3)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_30th),
  haswordvenue(C,word_30th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_31st),
  haswordvenue(C,word_31st)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_33),
  haswordvenue(C,word_33)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_34),
  haswordvenue(C,word_34)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_341),
  haswordvenue(C,word_341)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_352),
  haswordvenue(C,word_352)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_36),
  haswordvenue(C,word_36)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_36th),
  haswordvenue(C,word_36th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_3rd),
  haswordvenue(C,word_3rd)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_4),
  haswordvenue(C,word_4)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_41),
  haswordvenue(C,word_41)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_413),
  haswordvenue(C,word_413)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_42),
  haswordvenue(C,word_42)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_43),
  haswordvenue(C,word_43)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_431),
  haswordvenue(C,word_431)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_433),
  haswordvenue(C,word_433)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_444),
  haswordvenue(C,word_444)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_48),
  haswordvenue(C,word_48)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_493),
  haswordvenue(C,word_493)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_4th),
  haswordvenue(C,word_4th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_5),
  haswordvenue(C,word_5)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_50),
  haswordvenue(C,word_50)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_53),
  haswordvenue(C,word_53)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_55),
  haswordvenue(C,word_55)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_57),
  haswordvenue(C,word_57)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_5rd),
  haswordvenue(C,word_5rd)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_5th),
  haswordvenue(C,word_5th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_61),
  haswordvenue(C,word_61)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_62),
  haswordvenue(C,word_62)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_6th),
  haswordvenue(C,word_6th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_7),
  haswordvenue(C,word_7)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_74),
  haswordvenue(C,word_74)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_7th),
  haswordvenue(C,word_7th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_8),
  haswordvenue(C,word_8)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_82),
  haswordvenue(C,word_82)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_87),
  haswordvenue(C,word_87)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_88),
  haswordvenue(C,word_88)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_89),
  haswordvenue(C,word_89)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_8th),
  haswordvenue(C,word_8th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_90),
  haswordvenue(C,word_90)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_91),
  haswordvenue(C,word_91)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_912919),
  haswordvenue(C,word_912919)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_92),
  haswordvenue(C,word_92)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_93),
  haswordvenue(C,word_93)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_94),
  haswordvenue(C,word_94)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_95),
  haswordvenue(C,word_95)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_96),
  haswordvenue(C,word_96)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_97),
  haswordvenue(C,word_97)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_9th),
  haswordvenue(C,word_9th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_a),
  haswordvenue(C,word_a)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_aaai),
  haswordvenue(C,word_aaai)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_an),
  haswordvenue(C,word_an)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_annals),
  haswordvenue(C,word_annals)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_art),
  haswordvenue(C,word_art)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_artificial),
  haswordvenue(C,word_artificial)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_by),
  haswordvenue(C,word_by)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_c),
  haswordvenue(C,word_c)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_chicago),
  haswordvenue(C,word_chicago)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_chp),
  haswordvenue(C,word_chp)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_colt),
  haswordvenue(C,word_colt)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_control),
  haswordvenue(C,word_control)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_crl),
  haswordvenue(C,word_crl)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_crypto),
  haswordvenue(C,word_crypto)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_denver),
  haswordvenue(C,word_denver)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_development),
  haswordvenue(C,word_development)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_dissertation),
  haswordvenue(C,word_dissertation)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_doctoral),
  haswordvenue(C,word_doctoral)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_e),
  haswordvenue(C,word_e)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_ecml),
  haswordvenue(C,word_ecml)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_eighth),
  haswordvenue(C,word_eighth)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_eleventh),
  haswordvenue(C,word_eleventh)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_ence),
  haswordvenue(C,word_ence)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_extended),
  haswordvenue(C,word_extended)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_fifth),
  haswordvenue(C,word_fifth)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_first),
  haswordvenue(C,word_first)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_focs),
  haswordvenue(C,word_focs)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_forthcoming),
  haswordvenue(C,word_forthcoming)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_fourteenth),
  haswordvenue(C,word_fourteenth)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_francisco),
  haswordvenue(C,word_francisco)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_from),
  haswordvenue(C,word_from)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_hanson),
  haswordvenue(C,word_hanson)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_he),
  haswordvenue(C,word_he)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_i),
  haswordvenue(C,word_i)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_icml),
  haswordvenue(C,word_icml)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_ieee),
  haswordvenue(C,word_ieee)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_ietf),
  haswordvenue(C,word_ietf)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_ima),
  haswordvenue(C,word_ima)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_inform),
  haswordvenue(C,word_inform)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_institute),
  haswordvenue(C,word_institute)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_its),
  haswordvenue(C,word_its)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_iv),
  haswordvenue(C,word_iv)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_j),
  haswordvenue(C,word_j)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_jcss),
  haswordvenue(C,word_jcss)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_kaufmann),
  haswordvenue(C,word_kaufmann)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_langley),
  haswordvenue(C,word_langley)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_lcs),
  haswordvenue(C,word_lcs)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_lecture),
  haswordvenue(C,word_lecture)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_lippmann),
  haswordvenue(C,word_lippmann)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_m),
  haswordvenue(C,word_m)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_macro),
  haswordvenue(C,word_macro)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_manuscript),
  haswordvenue(C,word_manuscript)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_mar),
  haswordvenue(C,word_mar)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_masters),
  haswordvenue(C,word_masters)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_mit),
  haswordvenue(C,word_mit)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_ml),
  haswordvenue(C,word_ml)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_moody),
  haswordvenue(C,word_moody)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_morgan),
  haswordvenue(C,word_morgan)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_national),
  haswordvenue(C,word_national)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_neural),
  haswordvenue(C,word_neural)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_new),
  haswordvenue(C,word_new)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_nineteenth),
  haswordvenue(C,word_nineteenth)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_nips),
  haswordvenue(C,word_nips)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_nips92),
  haswordvenue(C,word_nips92)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_notes),
  haswordvenue(C,word_notes)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_number),
  haswordvenue(C,word_number)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_october),
  haswordvenue(C,word_october)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_p),
  haswordvenue(C,word_p)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_pages),
  haswordvenue(C,word_pages)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_pattern),
  haswordvenue(C,word_pattern)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_phd),
  haswordvenue(C,word_phd)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_pos),
  haswordvenue(C,word_pos)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_pp),
  haswordvenue(C,word_pp)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_pre),
  haswordvenue(C,word_pre)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_preprint),
  haswordvenue(C,word_preprint)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_press),
  haswordvenue(C,word_press)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_progress),
  haswordvenue(C,word_progress)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_puting),
  haswordvenue(C,word_puting)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_r),
  haswordvenue(C,word_r)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_rep),
  haswordvenue(C,word_rep)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_retrieval),
  haswordvenue(C,word_retrieval)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_s),
  haswordvenue(C,word_s)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_san),
  haswordvenue(C,word_san)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_seventh),
  haswordvenue(C,word_seventh)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_siam),
  haswordvenue(C,word_siam)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_sigir),
  haswordvenue(C,word_sigir)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_simulation),
  haswordvenue(C,word_simulation)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_sixth),
  haswordvenue(C,word_sixth)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_snowbird),
  haswordvenue(C,word_snowbird)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_st),
  haswordvenue(C,word_st)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_statistics),
  haswordvenue(C,word_statistics)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_stoc),
  haswordvenue(C,word_stoc)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_stpc),
  haswordvenue(C,word_stpc)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_systems),
  haswordvenue(C,word_systems)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_t),
  haswordvenue(C,word_t)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_technical),
  haswordvenue(C,word_technical)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_tenth),
  haswordvenue(C,word_tenth)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_test),
  haswordvenue(C,word_test)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_th),
  haswordvenue(C,word_th)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_third),
  haswordvenue(C,word_third)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_thirtieth),
  haswordvenue(C,word_thirtieth)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_tions),
  haswordvenue(C,word_tions)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_tm),
  haswordvenue(C,word_tm)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_to),
  haswordvenue(C,word_to)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_tr),
  haswordvenue(C,word_tr)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_twentieth),
  haswordvenue(C,word_twentieth)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_ucsc),
  haswordvenue(C,word_ucsc)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_vol),
  haswordvenue(C,word_vol)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_volume),
  haswordvenue(C,word_volume)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_winter),
  haswordvenue(C,word_winter)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_work),
  haswordvenue(C,word_work)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_works),
  haswordvenue(C,word_works)),

(samevenue(B,C):0.3 :-
  haswordvenue(B,word_workshop),
  haswordvenue(C,word_workshop)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_a),
  haswordauthor(C,word_a)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_andrzej),
  haswordauthor(C,word_andrzej)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_auer),
  haswordauthor(C,word_auer)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_avrim),
  haswordauthor(C,word_avrim)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_bauer),
  haswordauthor(C,word_bauer)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_bianchi),
  haswordauthor(C,word_bianchi)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_blum),
  haswordauthor(C,word_blum)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_cesa),
  haswordauthor(C,word_cesa)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_cohen),
  haswordauthor(C,word_cohen)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_d),
  haswordauthor(C,word_d)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_david),
  haswordauthor(C,word_david)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_dietterich),
  haswordauthor(C,word_dietterich)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_druker),
  haswordauthor(C,word_druker)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_e),
  haswordauthor(C,word_e)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_ehrenfeucht),
  haswordauthor(C,word_ehrenfeucht)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_eric),
  haswordauthor(C,word_eric)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_feder),
  haswordauthor(C,word_feder)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_freund),
  haswordauthor(C,word_freund)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_fyoav),
  haswordauthor(C,word_fyoav)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_g),
  haswordauthor(C,word_g)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_goldman),
  haswordauthor(C,word_goldman)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_h),
  haswordauthor(C,word_h)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_harris),
  haswordauthor(C,word_harris)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_haussler),
  haswordauthor(C,word_haussler)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_helmbold),
  haswordauthor(C,word_helmbold)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_henry),
  haswordauthor(C,word_henry)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_j),
  haswordauthor(C,word_j)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_kautz),
  haswordauthor(C,word_kautz)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_kearns),
  haswordauthor(C,word_kearns)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_l),
  haswordauthor(C,word_l)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_lewis),
  haswordauthor(C,word_lewis)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_littlestone),
  haswordauthor(C,word_littlestone)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_m),
  haswordauthor(C,word_m)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_micahel),
  haswordauthor(C,word_micahel)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_michael),
  haswordauthor(C,word_michael)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_n),
  haswordauthor(C,word_n)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_nicolo),
  haswordauthor(C,word_nicolo)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_p),
  haswordauthor(C,word_p)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_r),
  haswordauthor(C,word_r)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_rivest),
  haswordauthor(C,word_rivest)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_robert),
  haswordauthor(C,word_robert)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_ronald),
  haswordauthor(C,word_ronald)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_s),
  haswordauthor(C,word_s)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_sally),
  haswordauthor(C,word_sally)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_shapire),
  haswordauthor(C,word_shapire)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_t),
  haswordauthor(C,word_t)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_tom),
  haswordauthor(C,word_tom)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_v),
  haswordauthor(C,word_v)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_w),
  haswordauthor(C,word_w)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_william),
  haswordauthor(C,word_william)),

(sameauthor(B,C):0.3 :-
  haswordauthor(B,word_y),
  haswordauthor(C,word_y)),


(sametitle(B,C):0.3 :-
  haswordtitle(B,word_10),
  haswordtitle(C,word_10)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_1321),
  haswordtitle(C,word_1321)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_197),
  haswordtitle(C,word_197)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_1987),
  haswordtitle(C,word_1987)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_1989),
  haswordtitle(C,word_1989)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_1996),
  haswordtitle(C,word_1996)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_1999),
  haswordtitle(C,word_1999)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_2),
  haswordtitle(C,word_2)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_227),
  haswordtitle(C,word_227)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_29),
  haswordtitle(C,word_29)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_5),
  haswordtitle(C,word_5)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_8),
  haswordtitle(C,word_8)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_94),
  haswordtitle(C,word_94)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_a),
  haswordtitle(C,word_a)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_adaptive),
  haswordtitle(C,word_adaptive)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_adversarial),
  haswordtitle(C,word_adversarial)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_adversaries),
  haswordtitle(C,word_adversaries)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_advice),
  haswordtitle(C,word_advice)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_against),
  haswordtitle(C,word_against)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_alence),
  haswordtitle(C,word_alence)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_algo),
  haswordtitle(C,word_algo)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_algorithm),
  haswordtitle(C,word_algorithm)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_algorithmic),
  haswordtitle(C,word_algorithmic)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_almost),
  haswordtitle(C,word_almost)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_alternative),
  haswordtitle(C,word_alternative)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_an),
  haswordtitle(C,word_an)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_applying),
  haswordtitle(C,word_applying)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_approach),
  haswordtitle(C,word_approach)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_approximations),
  haswordtitle(C,word_approximations)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_are),
  haswordtitle(C,word_are)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_armed),
  haswordtitle(C,word_armed)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_article),
  haswordtitle(C,word_article)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_as),
  haswordtitle(C,word_as)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_assignment),
  haswordtitle(C,word_assignment)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_at),
  haswordtitle(C,word_at)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_att),
  haswordtitle(C,word_att)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_au),
  haswordtitle(C,word_au)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_authors),
  haswordtitle(C,word_authors)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_available),
  haswordtitle(C,word_available)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_average),
  haswordtitle(C,word_average)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_awards),
  haswordtitle(C,word_awards)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_b),
  haswordtitle(C,word_b)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_bandit),
  haswordtitle(C,word_bandit)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_bart),
  haswordtitle(C,word_bart)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_bartlett),
  haswordtitle(C,word_bartlett)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_based),
  haswordtitle(C,word_based)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_bayesian),
  haswordtitle(C,word_bayesian)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_behavior),
  haswordtitle(C,word_behavior)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_best),
  haswordtitle(C,word_best)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_biased),
  haswordtitle(C,word_biased)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_binary),
  haswordtitle(C,word_binary)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_bits),
  haswordtitle(C,word_bits)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_boost),
  haswordtitle(C,word_boost)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_bound),
  haswordtitle(C,word_bound)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_bounded),
  haswordtitle(C,word_bounded)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_bounds),
  haswordtitle(C,word_bounds)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_by),
  haswordtitle(C,word_by)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_c),
  haswordtitle(C,word_c)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_c4),
  haswordtitle(C,word_c4)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_case),
  haswordtitle(C,word_case)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_casino),
  haswordtitle(C,word_casino)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_cation),
  haswordtitle(C,word_cation)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_characteristic),
  haswordtitle(C,word_characteristic)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_check),
  haswordtitle(C,word_check)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_circuits),
  haswordtitle(C,word_circuits)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_classi),
  haswordtitle(C,word_classi)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_classification),
  haswordtitle(C,word_classification)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_codes),
  haswordtitle(C,word_codes)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_coin),
  haswordtitle(C,word_coin)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_colt),
  haswordtitle(C,word_colt)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_comittee),
  haswordtitle(C,word_comittee)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_comments),
  haswordtitle(C,word_comments)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_consequences),
  haswordtitle(C,word_consequences)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_convergence),
  haswordtitle(C,word_convergence)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_counting),
  haswordtitle(C,word_counting)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_criptographic),
  haswordtitle(C,word_criptographic)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_cross),
  haswordtitle(C,word_cross)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_curve),
  haswordtitle(C,word_curve)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_d),
  haswordtitle(C,word_d)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_data),
  haswordtitle(C,word_data)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_derstand),
  haswordtitle(C,word_derstand)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_design),
  haswordtitle(C,word_design)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_deterministic),
  haswordtitle(C,word_deterministic)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_discrete),
  haswordtitle(C,word_discrete)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_dissertation),
  haswordtitle(C,word_dissertation)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_distributed),
  haswordtitle(C,word_distributed)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_diversity),
  haswordtitle(C,word_diversity)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_dnf),
  haswordtitle(C,word_dnf)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_donna),
  haswordtitle(C,word_donna)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_down),
  haswordtitle(C,word_down)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_drift),
  haswordtitle(C,word_drift)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_e),
  haswordtitle(C,word_e)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_economic),
  haswordtitle(C,word_economic)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_ed),
  haswordtitle(C,word_ed)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_editor),
  haswordtitle(C,word_editor)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_ef),
  haswordtitle(C,word_ef)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_electronically),
  haswordtitle(C,word_electronically)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_emerging),
  haswordtitle(C,word_emerging)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_empirical),
  haswordtitle(C,word_empirical)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_environments),
  haswordtitle(C,word_environments)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_equivalence),
  haswordtitle(C,word_equivalence)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_error),
  haswordtitle(C,word_error)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_errors),
  haswordtitle(C,word_errors)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_estimates),
  haswordtitle(C,word_estimates)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_euro),
  haswordtitle(C,word_euro)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_evaluation),
  haswordtitle(C,word_evaluation)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_exact),
  haswordtitle(C,word_exact)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_experimental),
  haswordtitle(C,word_experimental)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_experiments),
  haswordtitle(C,word_experiments)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_expert),
  haswordtitle(C,word_expert)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_extended),
  haswordtitle(C,word_extended)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_fi),
  haswordtitle(C,word_fi)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_field),
  haswordtitle(C,word_field)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_fiers),
  haswordtitle(C,word_fiers)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_finite),
  haswordtitle(C,word_finite)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_first),
  haswordtitle(C,word_first)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_fixed),
  haswordtitle(C,word_fixed)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_fourier),
  haswordtitle(C,word_fourier)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_framework),
  haswordtitle(C,word_framework)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_free),
  haswordtitle(C,word_free)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_frei),
  haswordtitle(C,word_frei)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_from),
  haswordtitle(C,word_from)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_fyoav),
  haswordtitle(C,word_fyoav)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_gambling),
  haswordtitle(C,word_gambling)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_games),
  haswordtitle(C,word_games)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_general),
  haswordtitle(C,word_general)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_group),
  haswordtitle(C,word_group)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_hans),
  haswordtitle(C,word_hans)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_hard),
  haswordtitle(C,word_hard)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_harman),
  haswordtitle(C,word_harman)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_helmbold),
  haswordtitle(C,word_helmbold)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_hierarchies),
  haswordtitle(C,word_hierarchies)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_home),
  haswordtitle(C,word_home)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_horn),
  haswordtitle(C,word_horn)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_how),
  haswordtitle(C,word_how)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_http),
  haswordtitle(C,word_http)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_hy),
  haswordtitle(C,word_hy)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_hypotheses),
  haswordtitle(C,word_hypotheses)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_hypothesis),
  haswordtitle(C,word_hypothesis)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_improve),
  haswordtitle(C,word_improve)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_indirect),
  haswordtitle(C,word_indirect)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_informational),
  haswordtitle(C,word_informational)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_is),
  haswordtitle(C,word_is)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_its),
  haswordtitle(C,word_its)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_k),
  haswordtitle(C,word_k)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_l),
  haswordtitle(C,word_l)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_label),
  haswordtitle(C,word_label)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_lae),
  haswordtitle(C,word_lae)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_langley),
  haswordtitle(C,word_langley)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_languages),
  haswordtitle(C,word_languages)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_large),
  haswordtitle(C,word_large)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_layer),
  haswordtitle(C,word_layer)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_learnable),
  haswordtitle(C,word_learnable)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_leave),
  haswordtitle(C,word_leave)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_lee),
  haswordtitle(C,word_lee)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_leee),
  haswordtitle(C,word_leee)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_leslie),
  haswordtitle(C,word_leslie)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_linear),
  haswordtitle(C,word_linear)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_llncs),
  haswordtitle(C,word_llncs)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_lower),
  haswordtitle(C,word_lower)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_m),
  haswordtitle(C,word_m)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_machines),
  haswordtitle(C,word_machines)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_macro),
  haswordtitle(C,word_macro)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_mansour),
  haswordtitle(C,word_mansour)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_manuscript),
  haswordtitle(C,word_manuscript)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_margin),
  haswordtitle(C,word_margin)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_markov),
  haswordtitle(C,word_markov)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_md5),
  haswordtitle(C,word_md5)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_mechanics),
  haswordtitle(C,word_mechanics)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_message),
  haswordtitle(C,word_message)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_methods),
  haswordtitle(C,word_methods)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_mixture),
  haswordtitle(C,word_mixture)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_model),
  haswordtitle(C,word_model)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_models),
  haswordtitle(C,word_models)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_mulae),
  haswordtitle(C,word_mulae)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_multi),
  haswordtitle(C,word_multi)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_muth),
  haswordtitle(C,word_muth)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_near),
  haswordtitle(C,word_near)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_nearly),
  haswordtitle(C,word_nearly)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_needed),
  haswordtitle(C,word_needed)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_network),
  haswordtitle(C,word_network)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_neural),
  haswordtitle(C,word_neural)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_new),
  haswordtitle(C,word_new)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_noise),
  haswordtitle(C,word_noise)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_not),
  haswordtitle(C,word_not)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_number),
  haswordtitle(C,word_number)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_oblivious),
  haswordtitle(C,word_oblivious)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_old),
  haswordtitle(C,word_old)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_one),
  haswordtitle(C,word_one)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_online),
  haswordtitle(C,word_online)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_optimal),
  haswordtitle(C,word_optimal)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_or),
  haswordtitle(C,word_or)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_order),
  haswordtitle(C,word_order)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_orders),
  haswordtitle(C,word_orders)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_orgs),
  haswordtitle(C,word_orgs)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_our),
  haswordtitle(C,word_our)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_out),
  haswordtitle(C,word_out)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_output),
  haswordtitle(C,word_output)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_over),
  haswordtitle(C,word_over)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_p),
  haswordtitle(C,word_p)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_pac),
  haswordtitle(C,word_pac)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_package),
  haswordtitle(C,word_package)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_pages),
  haswordtitle(C,word_pages)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_patrice),
  haswordtitle(C,word_patrice)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_pattern),
  haswordtitle(C,word_pattern)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_people),
  haswordtitle(C,word_people)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_persistent),
  haswordtitle(C,word_persistent)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_peter),
  haswordtitle(C,word_peter)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_play),
  haswordtitle(C,word_play)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_points),
  haswordtitle(C,word_points)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_polynomials),
  haswordtitle(C,word_polynomials)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_portfolio),
  haswordtitle(C,word_portfolio)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_potheses),
  haswordtitle(C,word_potheses)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_pp),
  haswordtitle(C,word_pp)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_precence),
  haswordtitle(C,word_precence)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_predictors),
  haswordtitle(C,word_predictors)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_primitives),
  haswordtitle(C,word_primitives)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_problem),
  haswordtitle(C,word_problem)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_problems),
  haswordtitle(C,word_problems)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_processed),
  haswordtitle(C,word_processed)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_processes),
  haswordtitle(C,word_processes)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_protocols),
  haswordtitle(C,word_protocols)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_pruning),
  haswordtitle(C,word_pruning)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_q),
  haswordtitle(C,word_q)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_queries),
  haswordtitle(C,word_queries)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_query),
  haswordtitle(C,word_query)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_r),
  haswordtitle(C,word_r)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_random),
  haswordtitle(C,word_random)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_read),
  haswordtitle(C,word_read)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_reasoning),
  haswordtitle(C,word_reasoning)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_recent),
  haswordtitle(C,word_recent)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_relations),
  haswordtitle(C,word_relations)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_repeated),
  haswordtitle(C,word_repeated)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_request),
  haswordtitle(C,word_request)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_results),
  haswordtitle(C,word_results)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_rfc),
  haswordtitle(C,word_rfc)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_rigged),
  haswordtitle(C,word_rigged)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_rigorous),
  haswordtitle(C,word_rigorous)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_ro),
  haswordtitle(C,word_ro)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_s),
  haswordtitle(C,word_s)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_saitta),
  haswordtitle(C,word_saitta)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_sanity),
  haswordtitle(C,word_sanity)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_scene),
  haswordtitle(C,word_scene)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_schapireg),
  haswordtitle(C,word_schapireg)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_schemas),
  haswordtitle(C,word_schemas)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_sequence),
  haswordtitle(C,word_sequence)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_seung),
  haswordtitle(C,word_seung)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_shapire),
  haswordtitle(C,word_shapire)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_site),
  haswordtitle(C,word_site)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_snowbird),
  haswordtitle(C,word_snowbird)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_soft),
  haswordtitle(C,word_soft)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_some),
  haswordtitle(C,word_some)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_sparse),
  haswordtitle(C,word_sparse)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_specialize),
  haswordtitle(C,word_specialize)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_split),
  haswordtitle(C,word_split)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_ssr),
  haswordtitle(C,word_ssr)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_stacked),
  haswordtitle(C,word_stacked)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_strategies),
  haswordtitle(C,word_strategies)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_strength),
  haswordtitle(C,word_strength)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_style),
  haswordtitle(C,word_style)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_sub),
  haswordtitle(C,word_sub)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_systems),
  haswordtitle(C,word_systems)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_t),
  haswordtitle(C,word_t)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_temporal),
  haswordtitle(C,word_temporal)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_test),
  haswordtitle(C,word_test)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_testing),
  haswordtitle(C,word_testing)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_text),
  haswordtitle(C,word_text)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_that),
  haswordtitle(C,word_that)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_this),
  haswordtitle(C,word_this)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_thoughts),
  haswordtitle(C,word_thoughts)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_time),
  haswordtitle(C,word_time)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_to),
  haswordtitle(C,word_to)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_top),
  haswordtitle(C,word_top)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_total),
  haswordtitle(C,word_total)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_toward),
  haswordtitle(C,word_toward)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_tree),
  haswordtitle(C,word_tree)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_typical),
  haswordtitle(C,word_typical)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_un),
  haswordtitle(C,word_un)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_under),
  haswordtitle(C,word_under)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_universal),
  haswordtitle(C,word_universal)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_unsupervised),
  haswordtitle(C,word_unsupervised)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_unsupevised),
  haswordtitle(C,word_unsupevised)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_up),
  haswordtitle(C,word_up)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_upyears),
  haswordtitle(C,word_upyears)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_use),
  haswordtitle(C,word_use)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_v),
  haswordtitle(C,word_v)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_valiant),
  haswordtitle(C,word_valiant)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_variable),
  haswordtitle(C,word_variable)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_vazirani),
  haswordtitle(C,word_vazirani)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_vc),
  haswordtitle(C,word_vc)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_vectors),
  haswordtitle(C,word_vectors)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_w),
  haswordtitle(C,word_w)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_walks),
  haswordtitle(C,word_walks)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_was),
  haswordtitle(C,word_was)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_weak),
  haswordtitle(C,word_weak)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_weakly),
  haswordtitle(C,word_weakly)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_web),
  haswordtitle(C,word_web)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_weights),
  haswordtitle(C,word_weights)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_well),
  haswordtitle(C,word_well)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_works),
  haswordtitle(C,word_works)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_workshop),
  haswordtitle(C,word_workshop)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_worst),
  haswordtitle(C,word_worst)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_www),
  haswordtitle(C,word_www)),

(sametitle(B,C):0.3 :-
  haswordtitle(B,word_x),
  haswordtitle(C,word_x))
]).

begin(model(m1)).
author(class_7,author_blum_a_).
author(class_8,author_blum_a_).
author(class_9,author_blum_a_).
author(class_10,author_a_blum_).
author(class_11,author_avrim_blum_).
author(class_12,author_avrim_blum_).
author(class_13,author_avrim_blum_).
author(class_14,author_avrim_blum_).
author(class_15,author_a_blum_).
author(class_16,author_blum_a_).
author(class_81,author_william_w_cohen_).
author(class_82,author_william_w_cohen_).
author(class_83,author_william_w_cohen_).
author(class_84,author_w_w_cohen_).
author(class_95,author_h_drucker_).
author(class_96,author_h_drucker_).
author(class_97,author_drucker_h_).
author(class_98,author_drucker_h_).
author(class_99,author_drucker_h_).
author(class_100,author_drucker_h_).
author(class_101,author_drucker_h_).
author(class_102,author_drucker_h_).
author(class_103,author_drucker_harris_).
author(class_104,author_drucker_h_).
author(class_105,author_drucker_).
author(class_106,author_harris_drucker_).
author(class_107,author_harris_drucker_).
author(class_108,author_h_drucker_).
author(class_109,author_drucker_h_).
author(class_110,author_harris_drucker_).
author(class_111,author_harris_drucker_).
author(class_112,author_harris_drucker_).
author(class_113,author_h_druker_).
author(class_114,author_drucker_h_).
author(class_115,author_drucker_h_).
author(class_138,author_a_ehrenfeucht_).
author(class_245,author_yoav_freund_).
author(class_246,author_freund_y_).
author(class_247,author_freund_y_).
author(class_248,author_yoav_freund_).
author(class_250,author_y_freund_).
author(class_326,author_freud_y_).
author(class_327,author_y_freund_).
author(class_328,author_yoav_freund_).
author(class_329,author_y_freund_).
author(class_330,author_y_freund_).
author(class_331,author_freund_y_).
author(class_332,author_y_freund_).
author(class_333,author_y_freund_).
author(class_334,author_y_freund_).
author(class_335,author_y_freund_).
author(class_336,author_yoav_freund_).
author(class_439,author_goldman_s_).
author(class_440,author_sally_a_goldman_).
author(class_441,author_sally_a_goldman_).
author(class_442,author_s_a_goldman_).
author(class_450,author_sally_a_goldman_).
author(class_451,author_sally_a_goldman_).
author(class_497,author_d_haussler_).
author(class_498,author_d_haussler_).
author(class_499,author_d_haussler_).
author(class_551,author_haussler_d_).
author(class_552,author_d_haussler_).
author(class_553,author_d_haussler_).
author(class_554,author_d_haussler_).
author(class_555,author_d_haussler_).
author(class_556,author_david_haussler_).
author(class_557,author_d_haussler_).
author(class_558,author_d_haussler_).
author(class_559,author_d_haussler_).
author(class_560,author_d_haussler_).
author(class_561,author_david_haussler_).
author(class_562,author_d_haussler_).
author(class_563,author_d_haussler_).
author(class_564,author_d_haussler_).
author(class_565,author_d_haussler_).
author(class_566,author_d_haussler_).
author(class_567,author_d_haussler_).
author(class_568,author_d_haussler_).
author(class_569,author_david_haussler_).
author(class_570,author_d_haussler_).
author(class_571,author_david_haussler_).
author(class_572,author_david_haussler_).
author(class_590,author_d_haussler_).
author(class_591,author_david_haussler_).
author(class_592,author_d_haussler_).
author(class_629,author_kautz_henry_).
author(class_783,author_j_p_kearns_).
author(class_910,author_michael_j_kearns_).
author(class_911,author_m_kearns_).
author(class_912,author_m_kearns_).
author(class_913,author_kearns_m_j_).
author(class_914,author_m_kearns_).
author(class_915,author_michael_kearns_).
author(class_916,author_michael_kearns_).
author(class_917,author_m_j_kearns_).
author(class_918,author_m_kearns_).
author(class_919,author_m_kearns_).
author(class_920,author_m_j_kearns_).
author(class_921,author_m_kearns_).
author(class_922,author_m_kearns_).
author(class_923,author_m_kearns_).
author(class_924,author_m_kearns_).
author(class_925,author_m_kearns_).
author(class_926,author_m_kearns_).
author(class_927,author_m_j_kearns_).
author(class_928,author_m_kearns_).
author(class_929,author_m_kearns_).
author(class_930,author_m_j_kearns_).
author(class_931,author_m_j_kearns_).
author(class_932,author_kearns_m_).
author(class_933,author_kearns_m_).
author(class_934,author_kearns_m_).
author(class_935,author_kearns_m_j_).
author(class_936,author_kearns_m_j_).
author(class_937,author_kearns_m_j_).
author(class_938,author_kearns_m_j_).
author(class_939,author_kearns_m_j_).
author(class_940,author_kearns_m_j_).
author(class_941,author_kearns_m_).
author(class_942,author_kearns_m_).
author(class_943,author_kearns_michael_).
author(class_944,author_m_kearns_).
author(class_945,author_michael_kearns_).
author(class_946,author_m_kearns_).
author(class_947,author_michael_j_kearns_).
author(class_948,author_michael_j_kearns_).
author(class_949,author_m_j_kearns_).
author(class_950,author_m_j_kearns_).
author(class_951,author_m_j_kearns_).
author(class_952,author_michael_j_kearns_).
author(class_953,author_michael_j_kearns_).
author(class_954,author_kearns_m_).
author(class_955,author_m_j_kearns_).
author(class_956,author_michael_j_kearns_).
author(class_957,author_m_kearns_).
author(class_958,author_m_kearns_).
author(class_959,author_m_kearns_).
author(class_960,author_michael_j_kearns_).
author(class_961,author_m_j_kearns_).
author(class_962,author_m_j_kearns_).
author(class_963,author_m_j_kearns_).
author(class_964,author_m_j_kearns_).
author(class_965,author_m_j_kearns_).
author(class_966,author_m_j_kearns_).
author(class_967,author_m_j_kearns_).
author(class_968,author_m_kearns_).
author(class_969,author_m_kearns_).
author(class_970,author_kearns_m_j_).
author(class_971,author_kearns_m_j_).
author(class_972,author_kearns_m_).
author(class_1024,author_kearns_m_).
author(class_1025,author_m_j_kearns_).
author(class_1026,author_michael_kearns_).
author(class_1027,author_m_kearns_).
author(class_1028,author_m_kearns_).
author(class_1029,author_m_kearns_).
author(class_1030,author_m_kearns_).
author(class_1031,author_m_j_kearns_).
author(class_1032,author_m_kearns_).
author(class_1033,author_kearns_m_j_).
author(class_1044,author_kearns_m_).
author(class_1045,author_michael_kearns_).
author(class_1046,author_michael_kearns_).
author(class_1047,author_m_kearns_).
author(class_1048,author_michael_kearns_).
author(class_1062,author_m_kearns_).
author(class_1124,author_r_rivest_).
author(class_1125,author_r_rivest_).
author(class_1126,author_rivest_r_).
author(class_1127,author_rivest_r_l_).
author(class_1128,author_rivest_r_l_).
author(class_1129,author_rivest_r_l_).
author(class_1130,author_rivest_r_l_).
author(class_1131,author_r_rivest_).
author(class_1132,author_r_rivest_).
author(class_1133,author_ronald_l_rivest_).
author(class_1134,author_r_rivest_).
author(class_1135,author_r_e_schapire_).
author(class_1136,author_r_rivest_).
author(class_1137,author_r_l_rivest_).
author(class_1138,author_ronald_l_rivest_).
author(class_1139,author_r_l_rivest_).
author(class_1140,author_r_l_rivest_).
author(class_1141,author_r_rivest_).
author(class_1142,author_ronald_l_rivest_).
author(class_1143,author_ronald_l_rivest_).
author(class_1144,author_r_l_rivest_).
author(class_1145,author_r_l_rivest_).
author(class_1146,author_r_rivest_).
author(class_1147,author_ronald_l_rivest_).
author(class_1148,author_ronald_l_rivest_).
author(class_1149,author_rivest_r_l_).
author(class_1150,author_r_l_rivest_).
author(class_1151,author_ronald_l_rivest_).
author(class_1152,author_ronald_l_rivest_).
author(class_1153,author_r_e_schapire_).
author(class_1154,author_ronald_l_rivest_).
author(class_1155,author_ronald_l_rivest_).
author(class_1156,author_r_l_rivest_).
author(class_1157,author_r_l_rivest_).
author(class_1158,author_r_rivest_).
author(class_1159,author_ronald_l_rivest_).
author(class_1160,author_rivest_r_).
author(class_1161,author_rivest_r_l_).
author(class_1162,author_r_rivest_).
author(class_1163,author_r_l_rivest_).
author(class_1164,author_r_rivest_).
author(class_1165,author_r_rivest_).
author(class_1166,author_r_l_rivest_).
author(class_1247,author_robert_e_schapire_).
author(class_1248,author_robert_e_schapire_).
author(class_1249,author_robert_e_schapire_).
author(class_1250,author_robert_e_schapire_).
author(class_1251,author_r_e_schapire_).
author(class_1252,author_robert_e_schapire_).
author(class_1253,author_r_e_schapire_).
author(class_1254,author_schapire_r_e_).
author(class_1255,author_schapire_r_e_).
author(class_1256,author_robert_e_schapire_).
author(class_1257,author_robert_e_schapire_).
author(class_1258,author_r_e_schapire_).
author(class_1259,author_r_e_schapire_).
author(class_1260,author_r_e_schapire_).
author(class_1261,author_r_e_schapire_).
author(class_1262,author_schapire_r_e_).
author(class_1263,author_schapire_r_e_).
author(class_1264,author_r_e_schapire_).
author(class_1265,author_schapire_r_e_).
author(class_1266,author_schapire_r_e_).
author(class_1267,author_schapire_r_).
author(class_1268,author_schapire_r_).
author(class_1269,author_schapire_r_e_).
author(class_1270,author_schapire_r_e_).
author(class_1271,author_schapire_r_e_).
author(class_1272,author_schapire_r_e_).
author(class_1273,author_schapire_r_e_).
author(class_1274,author_shapire_r_).
author(class_1275,author_r_schapire_).
author(class_1276,author_schapire_r_e_).
author(class_1277,author_robert_e_schapire_).
author(class_1278,author_robert_e_schapire_).
author(class_1279,author_robert_e_schapire_).
author(class_1280,author_robert_e_schapire_).
author(class_1281,author_r_e_schapire_).
author(class_1282,author_r_schapire_).
author(class_1283,author_schapire_).
author(class_1284,author_schapire_r_e_).
author(class_1285,author_robert_e_schapire_).
author(class_1286,author_robert_e_schapire_).
author(class_1287,author_r_e_schapire_).
author(class_1288,author_robert_e_schapire_).
author(class_1289,author_schapire_r_e_).
author(class_1290,author_schapire_r_).
author(class_1291,author_robert_e_schapire_).
author(class_1292,author_robert_e_schapire_).
author(class_1293,author_robert_e_schapire_).
author(class_1294,author_robert_e_schapire_).
author(class_1295,author_robert_e_schapire_).

title(class_7,title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_).
title(class_8,title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_).
title(class_9,title_weakly_learning_learning_dnf_10_characterizing_statistical_query_using_fourier_analysis_).
title(class_10,title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_).
title(class_11,title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_).
title(class_12,title_weakly_8_learning_dnf_characterizing_statistical_query_learn_ing_using_fourier_analysis_).
title(class_13,title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_).
title(class_14,title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_).
title(class_15,title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_).
title(class_16,title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_).
title(class_81,title_learning_to_order_things_).
title(class_82,title_learning_ro_order_things_).
title(class_83,title_learning_to_order_things_).
title(class_84,title_learning_to_order_things_).
title(class_95,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_96,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_97,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_98,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_99,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_100,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_101,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_102,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_103,title_patrice_1993_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_104,title_1993_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_105,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_106,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_107,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_108,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_109,title_1993_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_110,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_111,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_112,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_113,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_114,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_115,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
title(class_138,title_a_general_lower_bound_number_examples_needed_learning_).
title(class_245,title_data_filtering_distribution_modeling_algorithms_machine_learning_).
title(class_246,title_data_filtering_distribution_modeling_algorithms_machine_learning_).
title(class_247,title_data_filtering_distribution_modeling_algorithms_machine_learning_).
title(class_248,title_data_filtering_distribution_modeling_algorithms_machine_learning_).
title(class_250,title_accelerating_learning_using_query_by_committee_).
title(class_326,title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_).
title(class_327,title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_).
title(class_328,title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_).
title(class_329,title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_).
title(class_330,title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_).
title(class_331,title_1995_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_).
title(class_332,title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_).
title(class_333,title_predicting_a_binary_sequence_almost_as_as_well_optimal_biased_coin_).
title(class_334,title_predicting_bits_almost_as_as_well_optimal_biased_coin_).
title(class_335,title_predicting_a_binary_sequence_as_as_well_optimal_biased_coin_).
title(class_336,title_predicting_a_binary_sequence_almost_as_as_well_optimal_biased_coin_).
title(class_439,title_learning_binary_relations_total_orders_).
title(class_440,title_learning_binary_relations_total_orders_).
title(class_441,title_learning_binary_relations_total_orders_).
title(class_442,title_learning_binary_relations_total_orders_).
title(class_450,title_learning_binary_relations_total_orders_).
title(class_451,title_learning_binary_relations_total_orders_).
title(class_497,title_equivalence_models_polynomial_learnability_).
title(class_498,title_equivalence_models_polynomial_learnability_).
title(class_499,title_equivalence_models_polynomial_learnability_).
title(class_551,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_552,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_553,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_554,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_555,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_556,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_557,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_558,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_559,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_560,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_561,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_562,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_563,title_unifying_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_).
title(class_564,title_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_).
title(class_565,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_566,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_567,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_568,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_569,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_570,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_571,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_572,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
title(class_590,title_rigourous_learning_curve_bounds_from_statistical_mechanics_).
title(class_591,title_rigorous_learning_curve_bounds_from_statistical_mechanics_).
title(class_592,title_rigorous_learning_curve_bounds_from_statistical_mechanics_).
title(class_629,title_bart_1993_reasoning_with_characteristic_models_).
title(class_783,title_performance_evaluation_protocols_distributed_systems_an_alternative_to_message_counting_).
title(class_910,title_learning_from_a_population_hypotheses_).
title(class_911,title_learnabil_ity_discrete_distributions_).
title(class_912,title_learnabil_ity_discrete_distributions_).
title(class_913,title_learnability_discrete_distributions_).
title(class_914,title_learnability_discrete_distributions_).
title(class_915,title_learnability_discrete_distributions_extended_abstract_).
title(class_916,title_learnability_discrete_distributions_extended_abstract_).
title(class_917,title_learnability_discrete_distributions_).
title(class_918,title_learnability_discrete_distributions_).
title(class_919,title_learnability_discrete_distributions_).
title(class_920,title_learnability_discrete_distributions_).
title(class_921,title_learnability_discrete_distributions_).
title(class_922,title_learnability_discrete_distributions_).
title(class_923,title_learnability_discrete_distributions_).
title(class_924,title_learnability_discrete_distributions_).
title(class_925,title_learnability_discrete_distributions_).
title(class_926,title_learnability_discrete_distributions_).
title(class_927,title_learnability_discrete_distributions_).
title(class_928,title_learnabil_ity_discrete_distributions_).
title(class_929,title_learnability_discrete_distributions_).
title(class_930,title_an_introduction_to_computational_learning_theory_).
title(class_931,title_an_introduction_to_computational_learning_theory_).
title(class_932,title_an_introduction_to_computational_learning_theory_).
title(class_933,title_an_introduction_to_computational_learning_theory_).
title(class_934,title_an_introduction_to_computational_learning_theory_).
title(class_935,title_an_introduction_to_computational_learning_theory_).
title(class_936,title_an_introduction_to_computational_learning_theory_).
title(class_937,title_an_introduction_to_computational_learning_theory_).
title(class_938,title_an_introduction_to_computational_learning_theory_).
title(class_939,title_an_introduction_to_computational_learning_theory_).
title(class_940,title_an_introduction_to_computational_learning_theory_).
title(class_941,title_an_introduction_to_computational_learning_theory_).
title(class_942,title_introduction_to_computational_learning_theory_).
title(class_943,title_an_introduction_to_computational_learning_theory_).
title(class_944,title_an_introduction_to_computational_learning_theory_).
title(class_945,title_an_introduction_to_computational_learning_theory_).
title(class_946,title_an_introduction_to_computational_learning_theory_).
title(class_947,title_an_introduction_to_computational_learning_theory_).
title(class_948,title_an_introduction_to_computational_learning_theory_).
title(class_949,title_an_introduction_to_computational_learning_theory_).
title(class_950,title_an_introduction_to_computational_learning_theory_).
title(class_951,title_an_introduction_to_computational_learning_theory_).
title(class_952,title_an_introduction_to_computational_learning_theory_).
title(class_953,title_an_introduction_to_computational_learning_theory_).
title(class_954,title_introduction_to_computational_learning_theory_).
title(class_955,title_an_introduction_to_computational_learning_theory_).
title(class_956,title_an_introduction_to_computational_learning_theory_).
title(class_957,title_an_introduction_to_computational_learning_theory_).
title(class_958,title_an_introduction_to_computational_learning_theory_).
title(class_959,title_an_introduction_to_computational_learning_theory_).
title(class_960,title_an_introduction_to_computational_learning_theory_).
title(class_961,title_an_introduction_to_computational_learning_theory_).
title(class_962,title_vazirani_1995_introduction_to_computational_learning_theory_).
title(class_963,title_an_introduction_to_computational_learning_theory_).
title(class_964,title_an_introduction_to_computational_learning_theory_).
title(class_965,title_an_introduction_to_computational_learning_theory_).
title(class_966,title_an_introduction_to_computational_learning_theory_).
title(class_967,title_an_introduction_to_computational_learning_theory_).
title(class_968,title_an_introduction_to_computational_learning_theory_).
title(class_969,title_an_introduction_to_computational_learning_theory_).
title(class_970,title_an_introduction_to_computational_learning_theory_).
title(class_971,title_an_introduction_to_computational_learning_theory_).
title(class_972,title_an_introduction_to_computational_learning_theory_).
title(class_1024,title_an_experimental_theoretical_comparison_model_selection_methods_).
title(class_1025,title_an_experimental_theoretical_comparison_model_selection_methods_).
title(class_1026,title_an_experimental_theoretical_comparison_model_selection_methods_).
title(class_1027,title_an_experimental_theoretical_comparison_model_selection_methods_).
title(class_1028,title_an_experimental_theoretical_comparison_model_selection_methods_).
title(class_1029,title_an_experimental_theoretical_comparison_model_selection_methods_).
title(class_1030,title_an_experimental_theoretical_comparison_model_selection_methods_).
title(class_1031,title_an_experimental_theoretical_comparison_model_selection_methods_).
title(class_1032,title_an_experimental_theoretical_comparison_model_selection_methods_).
title(class_1033,title_an_experimental_theoretical_comparison_model_selection_methods_).
title(class_1044,title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_).
title(class_1045,title_boosting_ability_top_down_decision_tree_learning_algorithms_).
title(class_1046,title_boosting_ability_top_down_decision_tree_learning_algorithms_).
title(class_1047,title_boosting_ability_top_down_decision_tree_learning_algorithms_).
title(class_1048,title_boosting_ability_top_down_decision_tree_learning_algorithms_).
title(class_1062,title_a_sparse_sampling_algorithm_near_optimal_planning_large_markov_decision_processes_1999_).
title(class_1124,title_md5_message_digest_algorithm_request_comments_informational_rfc_1321_).
title(class_1125,title_inference_finite_automata_using_homing_sequences_).
title(class_1126,title_inference_finite_automata_using_homing_sequences_).
title(class_1127,title_inference_finite_automata_using_homing_sequences_).
title(class_1128,title_inference_finite_automata_using_homing_sequences_).
title(class_1129,title_inference_finite_automata_using_homing_sequences_).
title(class_1130,title_1993_inference_finite_automata_using_homing_sequences_).
title(class_1131,title_inference_finite_automata_using_homing_sequences_).
title(class_1132,title_inference_finite_automata_using_homing_sequences_).
title(class_1133,title_inference_finite_automata_using_homing_sequences_).
title(class_1134,title_inference_finite_automata_using_homing_sequences_).
title(class_1135,title_inference_finite_automata_using_homing_sequences_r_l_).
title(class_1136,title_inference_finite_automata_using_homing_sequences_).
title(class_1137,title_inference_finite_automata_using_homing_sequences_).
title(class_1138,title_inference_finite_automata_using_homing_sequences_).
title(class_1139,title_inference_finite_automata_using_homing_sequences_).
title(class_1140,title_inference_finite_automata_using_homing_sequences_).
title(class_1141,title_inference_finite_automata_using_homing_sequences_).
title(class_1142,title_inference_finite_automata_using_homing_sequences_).
title(class_1143,title_inference_finite_automata_using_homing_sequences_).
title(class_1144,title_inference_finite_automata_using_homing_sequences_).
title(class_1145,title_inference_finite_automata_using_homing_sequences_).
title(class_1146,title_inference_finite_automata_using_homing_sequences_).
title(class_1147,title_inference_finite_automata_using_homing_sequences_).
title(class_1148,title_inference_finite_automata_using_homing_sequences_).
title(class_1149,title_inference_finite_automata_using_homing_sequences_).
title(class_1150,title_inference_finite_automata_using_homing_sequences_).
title(class_1151,title_inference_finite_automata_using_homing_sequences_).
title(class_1152,title_inference_finite_automata_using_homing_sequences_).
title(class_1153,title_inference_finite_automata_using_homing_sequences_r_l_).
title(class_1154,title_inference_finite_automata_using_homing_sequences_).
title(class_1155,title_inference_finite_automata_using_homing_sequences_).
title(class_1156,title_inference_finite_automata_using_homing_sequences_).
title(class_1157,title_inference_finite_automata_using_homing_sequences_information_computation_).
title(class_1158,title_inference_finite_automata_using_homing_sequences_).
title(class_1159,title_inference_finite_automata_using_homing_sequences_).
title(class_1160,title_inference_finite_automata_using_homing_sequences_).
title(class_1161,title_inference_finite_automata_using_homing_sequences_).
title(class_1162,title_diversity_based_inference_finite_automata_).
title(class_1163,title_diversity_based_inference_finite_automata_).
title(class_1164,title_diversity_based_inference_finite_automata_).
title(class_1165,title_diversity_based_inference_finite_automata_).
title(class_1166,title_diversity_based_inference_finite_automata_).
title(class_1247,title_design_analysis_efficient_learning_algorithms_).
title(class_1248,title_private_correspondence_).
title(class_1249,title_learning_probabilistic_read_once_formulas_product_distributions_).
title(class_1250,title_learning_probabilistic_read_once_formulas_product_distributions_).
title(class_1251,title_design_analysis_ef_ficient_learning_algorithms_).
title(class_1252,title_private_correspondence_).
title(class_1253,title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_).
title(class_1254,title_worst_case_analysis_temporal_difference_learning_algorithms_).
title(class_1255,title_worst_case_analysis_temporal_difference_learing_algorithms_).
title(class_1256,title_worst_case_analysis_temporal_difference_learning_algorithms_).
title(class_1257,title_worst_case_analysis_temporal_difference_learning_algorithms_).
title(class_1258,title_worst_case_analysis_temporal_difference_learing_algorithms_).
title(class_1259,title_worst_case_analysis_temporal_difference_learing_algorithms_).
title(class_1260,title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_).
title(class_1261,title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_).
title(class_1262,title_private_communication_).
title(class_1263,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
title(class_1264,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
title(class_1265,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
title(class_1266,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
title(class_1267,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_).
title(class_1268,title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
title(class_1269,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
title(class_1270,title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
title(class_1271,title_boosting_margin_).
title(class_1272,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
title(class_1273,title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
title(class_1274,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
title(class_1275,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
title(class_1276,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
title(class_1277,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
title(class_1278,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
title(class_1279,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
title(class_1280,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
title(class_1281,title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_).
title(class_1282,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
title(class_1283,title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
title(class_1284,title_query_by_committee_).
title(class_1285,title_using_output_codes_to_boost_multiclass_learning_problems_).
title(class_1286,title_using_output_codes_to_boost_multiclass_learning_problems_).
title(class_1287,title_using_output_codes_to_boost_multiclass_learning_problems_).
title(class_1288,title_using_output_codes_to_boost_multiclass_learning_problems_).
title(class_1289,title_using_output_codes_to_boost_multiclass_learning_problems_).
title(class_1290,title_improved_boosting_algorithms_using_confidence_rated_predictions_).
title(class_1291,title_improved_boosting_algorithms_using_confidence_rated_predictions_).
title(class_1292,title_improved_boosting_algorithms_using_confidence_rated_predictions_).
title(class_1293,title_improved_boosting_algorithms_using_confidence_rated_predictions_).
title(class_1294,title_improved_boosting_algorithms_using_confidence_rated_predictions_).
title(class_1295,title_a_system_multiclass_multi_label_text_categorization_).

venue(class_7,venue_proceedings_26th_symposium_theory_computing_).
venue(class_8,venue_proceedings_twenty_sixth_symposium_theory_computing_).
venue(class_9,venue_proc_26th_sympos_theory_comput_).
venue(class_10,venue_proceedings_26th_symposium_theory_computing_).
venue(class_11,venue_t_t_he_26_th_symposium_heory_computing_).
venue(class_12,venue_proceedings_26_th_symposium_theory_computing_).
venue(class_13,venue_proceedings_26_th_symposium_theory_computing_).
venue(class_14,venue_proceedings_twenty_sixth_symposium_theory_computing_).
venue(class_15,venue_proceedings_twenty_sixth_symposium_theory_computing_).
venue(class_16,venue_proceedings_26th_symposium_theory_computing_).
venue(class_81,venue_advances_neural_information_processing_systems_10_).
venue(class_82,venue_advances_neural_information_processing_systems_10_).
venue(class_83,venue_advances_neural_information_processing_systems_10_).
venue(class_84,venue_advances_neural_information_processing_systems_10_).
venue(class_95,venue_advances_neural_information_processing_systems_).
venue(class_96,venue_advances_neural_information_processing_systems_).
venue(class_97,venue_advances_neural_information_processing_systems_vol_4_).
venue(class_98,venue_advances_neural_information_processing_systems_volume_5_).
venue(class_99,venue_advances_neural_information_processing_systems_volume_5_).
venue(class_100,venue_advances_neural_information_processing_systems_).
venue(class_101,venue_advances_neural_information_processing_systems_).
venue(class_102,venue_advances_neural_information_processing_systems_5_).
venue(class_103,venue_advances_neural_informations_processing_systems_5_).
venue(class_104,venue_advances_neural_information_processing_systems_5_).
venue(class_105,venue_advances_neural_information_processing_systems_5_).
venue(class_106,venue_advances_neural_information_processing_systems_5_).
venue(class_107,venue_advances_neural_information_processing_systems_).
venue(class_108,venue_advances_neural_information_processing_systems_5_).
venue(class_109,venue_advances_neural_information_processing_systems_5_).
venue(class_110,venue_advances_neural_information_processing_systems_5_).
venue(class_111,venue_advances_neural_informations_processing_systems_5_).
venue(class_112,venue_proceedings_fifth_conf_neural_informations_processing_systems_).
venue(class_113,venue_advances_neural_information_processing_systems_).
venue(class_114,venue_advances_neural_information_processing_systems_).
venue(class_115,venue_neural_information_processing_systems_).
venue(class_138,venue_technical_report_ucsc_crl_87_26_).
venue(class_245,venue_phd_thesis_).
venue(class_246,venue_phd_thesis_).
venue(class_247,venue_phd_thesis_).
venue(class_248,venue_phd_thesis_).
venue(class_250,venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_).
venue(class_326,venue_proceedings_36_th_symposium_foundations_computer_science_).
venue(class_327,venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_).
venue(class_328,venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_).
venue(class_329,venue_proceedings_thirty_seventh_symposium_foundations_computer_science_).
venue(class_330,venue_proceedings_thirty_sixth_symposium_foundations_computer_science_).
venue(class_331,venue_focs_36_).
venue(class_332,venue_ieee_symposium_foundations_computer_science_).
venue(class_333,venue_proceedings_9th_conference_computational_learning_theory_).
venue(class_334,venue_proc_9th_conference_computational_learning_theory_).
venue(class_335,venue_proceedings_ninth_conference_computational_learning_theory_).
venue(class_336,venue_proceedings_ninth_conference_computational_learning_theory_).
venue(class_439,venue_proceedings_30th_symposium_foundation_computer_science_).
venue(class_440,venue_30th_symposium_foundations_computer_science_).
venue(class_441,venue_proceedings_thirtieth_symposium_foundations_computer_science_).
venue(class_442,venue_proceedings_30th_symposium_foundations_computer_science_).
venue(class_450,venue_technical_report_mit_lcs_tm_413_).
venue(class_451,venue_technical_report_mit_lcs_tm_413_).
venue(class_497,venue_technical_report_number_ucsc_crl_88_06_).
venue(class_498,venue_technical_report_number_ucsc_crl_88_06_).
venue(class_499,venue_technical_report_number_ucsc_crl_88_06_).
venue(class_551,venue_proceedings_fourth_conference_computational_learning_theory_colt_91_).
venue(class_552,venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_).
venue(class_553,venue_proceedings_4th_workshop_computational_learning_theory_).
venue(class_554,venue_colt_91_proceedings_4th_workshop_computational_learning_theory_).
venue(class_555,venue_proceedings_4th_workshop_computational_learning_theory_).
venue(class_556,venue_computational_learning_theory_proceedings_fourth_workshop_).
venue(class_557,venue_proc_4th_workshop_comput_learning_theory_).
venue(class_558,venue_colt_91_proceedings_4th_workshop_computational_learning_theory_).
venue(class_559,venue_proc_4th_workshop_computational_learning_theory_).
venue(class_560,venue_colt_91_proceedings_4th_workshop_computational_learning_theory_).
venue(class_561,venue_proceedings_fourth_workshop_computational_learning_theory_).
venue(class_562,venue_1991_workshop_computational_learning_theory_).
venue(class_563,venue_proceedings_4th_workshop_computational_learning_theory_).
venue(class_564,venue_proceedings_4th_workshop_computational_learning_theory_).
venue(class_565,venue_proceedings_1991_workshop_computational_learning_theory_).
venue(class_566,venue_proceedings_fourth_workshop_computational_learning_theory_).
venue(class_567,venue_proceedings_fourth_workshop_computational_learning_theory_).
venue(class_568,venue_proceedings_fourth_workshop_computational_learning_theory_).
venue(class_569,venue_proceedings_fourth_workshop_computational_learning_theory_).
venue(class_570,venue_proc_4th_workshop_comput_learning_theory_).
venue(class_571,venue_proceedings_1991_workshop_computational_learning_theory_).
venue(class_572,venue_proceedings_1991_workshop_computational_learning_theory_).
venue(class_590,venue_machine_learning_).
venue(class_591,venue_machine_learning_).
venue(class_592,venue_machine_learning_).
venue(class_629,venue_technical_report_).
venue(class_783,venue_proceedings_winter_simulation_conference_).
venue(class_910,venue_proceedings_colt_93_).
venue(class_911,venue_25th_symposium_theory_computing_).
venue(class_912,venue_25th_symposium_theory_computing_).
venue(class_913,venue_25th_symposium_theory_computing_1994_).
venue(class_914,venue_25th_symposium_theory_computing_).
venue(class_915,venue_proc_26th_symp_theory_computing_).
venue(class_916,venue_proc_26th_symp_theory_computing_).
venue(class_917,venue_25th_symposium_theory_computing_).
venue(class_918,venue_stoc_).
venue(class_919,venue_stoc_).
venue(class_920,venue_25th_symposium_theory_computing_).
venue(class_921,venue_stoc_94_).
venue(class_922,venue_stoc_94_).
venue(class_923,venue_proceedings_26th_symposium_theory_computing_).
venue(class_924,venue_25th_symp_theory_computing_).
venue(class_925,venue_25th_symp_theory_computing_).
venue(class_926,venue_25th_symposium_theory_computing_to_appear_).
venue(class_927,venue_25th_symposium_theory_computing_).
venue(class_928,venue_stoc_p_).
venue(class_929,venue_symposium_theory_computing_).
venue(class_930,venue_25th_symposium_theory_computing_).
venue(class_931,venue_25th_symposium_theory_computing_).
venue(class_932,venue_25th_symposium_theory_computing_).
venue(class_933,venue_25th_symposium_theory_computing_).
venue(class_934,venue_25th_symposium_theory_computing_).
venue(class_935,venue_25th_symposium_theory_computing_).
venue(class_936,venue_25th_symposium_theory_computing_).
venue(class_937,venue_25th_symposium_theory_computing_).
venue(class_938,venue_25th_symposium_theory_computing_).
venue(class_939,venue_25th_symposium_theory_computing_).
venue(class_940,venue_25th_symposium_theory_computing_).
venue(class_941,venue_25th_symposium_theory_computing_).
venue(class_942,venue_25th_symposium_theory_computing_).
venue(class_943,venue_25th_symposium_theory_computing_).
venue(class_944,venue_25th_symposium_theory_computing_).
venue(class_945,venue_25th_symposium_theory_computing_).
venue(class_946,venue_25th_symposium_theory_computing_).
venue(class_947,venue_25th_symposium_theory_computing_).
venue(class_948,venue_25th_symposium_theory_computing_).
venue(class_949,venue_25th_symposium_theory_computing_).
venue(class_950,venue_25th_symposium_theory_computing_).
venue(class_951,venue_25th_symposium_theory_computing_).
venue(class_952,venue_25th_symposium_theory_computing_).
venue(class_953,venue_25th_symposium_theory_computing_).
venue(class_954,venue_25th_symposium_theory_computing_).
venue(class_955,venue_25th_symposium_theory_computing_).
venue(class_956,venue_25th_symposium_theory_computing_).
venue(class_957,venue_57_62_).
venue(class_958,venue_25th_symposium_theory_computing_).
venue(class_959,venue_25th_symposium_theory_computing_).
venue(class_960,venue_25th_symposium_theory_computing_).
venue(class_961,venue_25th_symposium_theory_computing_).
venue(class_962,venue_25th_symposium_theory_computing_).
venue(class_963,venue_25th_symposium_theory_computing_).
venue(class_964,venue_25th_symposium_theory_computing_).
venue(class_965,venue_25th_symposium_theory_computing_).
venue(class_966,venue_25th_symposium_theory_computing_).
venue(class_967,venue_25th_symposium_theory_computing_).
venue(class_968,venue_25th_symposium_theory_computing_).
venue(class_969,venue_25th_symposium_theory_computing_).
venue(class_970,venue_25th_symposium_theory_computing_).
venue(class_971,venue_chp_).
venue(class_972,venue_25th_symposium_theory_computing_).
venue(class_1024,venue_proceedings_colt_95_).
venue(class_1025,venue_proceedings_eighth_workshop_computational_learning_theory_).
venue(class_1026,venue_proceedings_eighth_conference_computational_learning_theory_).
venue(class_1027,venue_proceedings_eighth_conference_computational_learning_theory_).
venue(class_1028,venue_proceedings_eigth_conference_computational_learning_theory_).
venue(class_1029,venue_colt_95_).
venue(class_1030,venue_proceedings_seventh_workshop_computational_learning_theory_).
venue(class_1031,venue_proceedings_eighth_workshop_computational_learning_theory_).
venue(class_1032,venue_proc_workshop_computational_learning_theory_).
venue(class_1033,venue_proceedings_eighth_conference_computational_learning_theory_).
venue(class_1044,venue_to_appear_proceedings_28th_symposium_theory_computing_).
venue(class_1045,venue_proceedings_twenty_eighth_symposium_theory_computing_).
venue(class_1046,venue_proceedings_twenty_eighth_symposium_theory_computing_).
venue(class_1047,venue_proceedings_28th_symposium_theory_computing_).
venue(class_1048,venue_proceedings_28th_symposium_theory_computing_).
venue(class_1062,venue_unpublished_manuscript_).
venue(class_1124,venue_proc_21st_symp_theoretical_computing_).
venue(class_1125,venue_information_computation_).
venue(class_1126,venue_information_computation_103_2_).
venue(class_1127,venue_information_computation_).
venue(class_1128,venue_information_computation_).
venue(class_1129,venue_information_computation_103_2_).
venue(class_1130,venue_information_computation_).
venue(class_1131,venue_information_computation_).
venue(class_1132,venue_information_computation_).
venue(class_1133,venue_information_computation_).
venue(class_1134,venue_information_computation_).
venue(class_1135,venue_machine_learning_from_theory_to_applications_).
venue(class_1136,venue_information_computation_).
venue(class_1137,venue_formation_computation_103_2_1993_).
venue(class_1138,venue_information_computation_).
venue(class_1139,venue_information_computation_).
venue(class_1140,venue_information_computation_).
venue(class_1141,venue_information_computation_).
venue(class_1142,venue_information_computation_).
venue(class_1143,venue_information_computation_).
venue(class_1144,venue_information_computation_).
venue(class_1145,venue_information_computation_).
venue(class_1146,venue_information_computation_).
venue(class_1147,venue_information_computation_).
venue(class_1148,venue_information_computation_).
venue(class_1149,venue_information_computation_).
venue(class_1150,venue_information_computation_).
venue(class_1151,venue_information_computation_).
venue(class_1152,venue_information_computation_).
venue(class_1153,venue_machine_learning_from_theory_to_applications_).
venue(class_1154,venue_information_computation_).
venue(class_1155,venue_information_computation_).
venue(class_1156,venue_information_computation_).
venue(class_1157,venue_information_computation_).
venue(class_1158,venue_information_compuation_).
venue(class_1159,venue_information_computation_).
venue(class_1160,venue_information_computation_103_).
venue(class_1161,venue_information_computation_).
venue(class_1162,venue_journal_association_computing_machinery_).
venue(class_1163,venue_journal_association_computing_machinery_).
venue(class_1164,venue_journal_association_computing_machinery_).
venue(class_1165,venue_journal_association_computing_machinery_).
venue(class_1166,venue_journal_association_computing_machinery_).
venue(class_1247,venue_technical_report_mit_lcs_tr_493_).
venue(class_1248,venue_phd_thesis_).
venue(class_1249,venue_computation_learning_theory_proceedings_fourth_workshop_).
venue(class_1250,venue_proceedings_fourth_workshop_computational_learning_theory_).
venue(class_1251,venue_proceedings_fourth_workshop_computational_learning_theory_).
venue(class_1252,venue_proceedings_fourth_workshop_computational_learning_theory_).
venue(class_1253,venue_proceedings_sixth_workshop_computational_learning_theory_).
venue(class_1254,venue_machine_learning_proceedings_eleventh_international_conference_).
venue(class_1255,venue_proceddings_11th_international_conference_machine_learning_).
venue(class_1256,venue_proceedings_eleventh_international_conference_machine_learning_).
venue(class_1257,venue_proc_11th_international_conference_machine_learning_).
venue(class_1258,venue_proc_11th_international_conf_machine_learning_).
venue(class_1259,venue_proc_11th_international_conference_machine_learning_).
venue(class_1260,venue_j_computer_system_sciences_).
venue(class_1261,venue_j_computer_system_sciences_).
venue(class_1262,venue_proceedings_fourteenth_international_conference_machine_learning_).
venue(class_1263,venue_proceedings_fourteenth_international_conference_machine_learning_).
venue(class_1264,venue_machine_learning_proceedings_fourteenth_international_conference_).
venue(class_1265,venue_machine_learning_proceedings_fourteenth_international_conference_).
venue(class_1266,venue_machine_learning_proceedings_fourteenth_international_conference_).
venue(class_1267,venue_proceedings_14th_international_conference_machine_learning_).
venue(class_1268,venue_proceedings_14th_international_conference_machine_learning_).
venue(class_1269,venue_proceedings_11th_international_conference_machine_learning_).
venue(class_1270,venue_proceedings_fourteenth_international_conference_machine_learning_).
venue(class_1271,venue_proceedings_fourteenth_international_conference_machine_learning_).
venue(class_1272,venue_proceedings_14th_international_conference_machine_learning_).
venue(class_1273,venue_proceedings_fourteenth_international_conference_machine_learning_).
venue(class_1274,venue_snowbird_1997_conference_).
venue(class_1275,venue_proceedings_fourteenth_international_conference_machine_learning_).
venue(class_1276,venue_proceedings_fourteenth_international_conference_machine_learning_).
venue(class_1277,venue_machine_learning_proceedings_fourteenth_international_conference_).
venue(class_1278,venue_machine_learning_proceedings_fourteenth_international_conference_).
venue(class_1279,venue_machine_learning_proceedings_fourteenth_international_conference_).
venue(class_1280,venue_machine_learning_proceedings_fourteenth_international_conference_).
venue(class_1281,venue_1997_).
venue(class_1282,venue_icml_97_).
venue(class_1283,venue_proceedings_14th_international_conference_machine_learning_).
venue(class_1284,venue_proceedings_14th_international_conference_machine_learning_).
venue(class_1285,venue_machine_learning_proceedings_fourteenth_international_conference_).
venue(class_1286,venue_machine_learning_proceedings_fourteenth_international_conference_).
venue(class_1287,venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_).
venue(class_1288,venue_machine_learning_proceedings_fourteenth_international_conference_).
venue(class_1289,venue_tech_rep_).
venue(class_1290,venue_proceedings_eleventh_conference_computational_learning_theory_).
venue(class_1291,venue_proceedings_eleventh_conference_computational_learning_theory_).
venue(class_1292,venue_proceedings_eleventh_conference_computational_learning_theory_).
venue(class_1293,venue_proceedings_eleventh_conference_computational_learning_theory_).
venue(class_1294,venue_proceedings_eleventh_conference_computational_learning_theory_).
venue(class_1295,venue_unpublished_manuscript_).

haswordauthor(author_blum_a_, word_a).
haswordauthor(author_blum_a_, word_blum).
haswordauthor(author_a_blum_, word_a).
haswordauthor(author_a_blum_, word_blum).
haswordauthor(author_avrim_blum_, word_avrim).
haswordauthor(author_avrim_blum_, word_blum).
haswordauthor(author_william_w_cohen_, word_cohen).
haswordauthor(author_william_w_cohen_, word_w).
haswordauthor(author_william_w_cohen_, word_william).
haswordauthor(author_w_w_cohen_, word_cohen).
haswordauthor(author_w_w_cohen_, word_w).
haswordauthor(author_h_drucker_, word_druker).
haswordauthor(author_h_drucker_, word_h).
haswordauthor(author_drucker_h_, word_druker).
haswordauthor(author_drucker_h_, word_h).
haswordauthor(author_drucker_harris_, word_druker).
haswordauthor(author_drucker_harris_, word_harris).
haswordauthor(author_drucker_, word_druker).
haswordauthor(author_harris_drucker_, word_druker).
haswordauthor(author_harris_drucker_, word_harris).
haswordauthor(author_h_druker_, word_druker).
haswordauthor(author_h_druker_, word_h).
haswordauthor(author_a_ehrenfeucht_, word_a).
haswordauthor(author_a_ehrenfeucht_, word_ehrenfeucht).
haswordauthor(author_yoav_freund_, word_freund).
haswordauthor(author_yoav_freund_, word_fyoav).
haswordauthor(author_freund_y_, word_freund).
haswordauthor(author_freund_y_, word_y).
haswordauthor(author_y_freund_, word_freund).
haswordauthor(author_y_freund_, word_y).
haswordauthor(author_freud_y_, word_freund).
haswordauthor(author_freud_y_, word_y).
haswordauthor(author_goldman_s_, word_goldman).
haswordauthor(author_goldman_s_, word_s).
haswordauthor(author_sally_a_goldman_, word_a).
haswordauthor(author_sally_a_goldman_, word_goldman).
haswordauthor(author_sally_a_goldman_, word_sally).
haswordauthor(author_s_a_goldman_, word_a).
haswordauthor(author_s_a_goldman_, word_goldman).
haswordauthor(author_s_a_goldman_, word_s).
haswordauthor(author_d_haussler_, word_d).
haswordauthor(author_d_haussler_, word_haussler).
haswordauthor(author_haussler_d_, word_d).
haswordauthor(author_haussler_d_, word_haussler).
haswordauthor(author_david_haussler_, word_david).
haswordauthor(author_david_haussler_, word_haussler).
haswordauthor(author_kautz_henry_, word_henry).
haswordauthor(author_kautz_henry_, word_kautz).
haswordauthor(author_j_p_kearns_, word_j).
haswordauthor(author_j_p_kearns_, word_kearns).
haswordauthor(author_j_p_kearns_, word_p).
haswordauthor(author_michael_j_kearns_, word_j).
haswordauthor(author_michael_j_kearns_, word_kearns).
haswordauthor(author_michael_j_kearns_, word_michael).
haswordauthor(author_m_kearns_, word_kearns).
haswordauthor(author_m_kearns_, word_m).
haswordauthor(author_kearns_m_j_, word_j).
haswordauthor(author_kearns_m_j_, word_kearns).
haswordauthor(author_kearns_m_j_, word_m).
haswordauthor(author_michael_kearns_, word_kearns).
haswordauthor(author_michael_kearns_, word_michael).
haswordauthor(author_m_j_kearns_, word_j).
haswordauthor(author_m_j_kearns_, word_kearns).
haswordauthor(author_m_j_kearns_, word_m).
haswordauthor(author_kearns_m_, word_kearns).
haswordauthor(author_kearns_m_, word_m).
haswordauthor(author_kearns_michael_, word_kearns).
haswordauthor(author_kearns_michael_, word_michael).
haswordauthor(author_r_rivest_, word_r).
haswordauthor(author_r_rivest_, word_rivest).
haswordauthor(author_rivest_r_, word_r).
haswordauthor(author_rivest_r_, word_rivest).
haswordauthor(author_rivest_r_l_, word_l).
haswordauthor(author_rivest_r_l_, word_r).
haswordauthor(author_rivest_r_l_, word_rivest).
haswordauthor(author_ronald_l_rivest_, word_l).
haswordauthor(author_ronald_l_rivest_, word_rivest).
haswordauthor(author_ronald_l_rivest_, word_ronald).
haswordauthor(author_r_e_schapire_, word_e).
haswordauthor(author_r_e_schapire_, word_r).
haswordauthor(author_r_e_schapire_, word_shapire).
haswordauthor(author_r_l_rivest_, word_l).
haswordauthor(author_r_l_rivest_, word_r).
haswordauthor(author_r_l_rivest_, word_rivest).
haswordauthor(author_robert_e_schapire_, word_e).
haswordauthor(author_robert_e_schapire_, word_robert).
haswordauthor(author_robert_e_schapire_, word_shapire).
haswordauthor(author_schapire_r_e_, word_e).
haswordauthor(author_schapire_r_e_, word_r).
haswordauthor(author_schapire_r_e_, word_shapire).
haswordauthor(author_schapire_r_, word_r).
haswordauthor(author_schapire_r_, word_shapire).
haswordauthor(author_shapire_r_, word_r).
haswordauthor(author_shapire_r_, word_shapire).
haswordauthor(author_r_schapire_, word_r).
haswordauthor(author_r_schapire_, word_shapire).
haswordauthor(author_schapire_, word_shapire).

haswordtitle(title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_, word_dnf).
haswordtitle(title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_, word_fourier).
haswordtitle(title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_, word_query).
haswordtitle(title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_, word_test).
haswordtitle(title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_, word_weakly).
haswordtitle(title_weakly_learning_learning_dnf_10_characterizing_statistical_query_using_fourier_analysis_, word_10).
haswordtitle(title_weakly_learning_learning_dnf_10_characterizing_statistical_query_using_fourier_analysis_, word_dnf).
haswordtitle(title_weakly_learning_learning_dnf_10_characterizing_statistical_query_using_fourier_analysis_, word_fourier).
haswordtitle(title_weakly_learning_learning_dnf_10_characterizing_statistical_query_using_fourier_analysis_, word_query).
haswordtitle(title_weakly_learning_learning_dnf_10_characterizing_statistical_query_using_fourier_analysis_, word_test).
haswordtitle(title_weakly_learning_learning_dnf_10_characterizing_statistical_query_using_fourier_analysis_, word_weakly).
haswordtitle(title_weakly_8_learning_dnf_characterizing_statistical_query_learn_ing_using_fourier_analysis_, word_8).
haswordtitle(title_weakly_8_learning_dnf_characterizing_statistical_query_learn_ing_using_fourier_analysis_, word_dnf).
haswordtitle(title_weakly_8_learning_dnf_characterizing_statistical_query_learn_ing_using_fourier_analysis_, word_fourier).
haswordtitle(title_weakly_8_learning_dnf_characterizing_statistical_query_learn_ing_using_fourier_analysis_, word_query).
haswordtitle(title_weakly_8_learning_dnf_characterizing_statistical_query_learn_ing_using_fourier_analysis_, word_test).
haswordtitle(title_weakly_8_learning_dnf_characterizing_statistical_query_learn_ing_using_fourier_analysis_, word_weakly).
haswordtitle(title_learning_to_order_things_, word_order).
haswordtitle(title_learning_to_order_things_, word_test).
haswordtitle(title_learning_to_order_things_, word_to).
haswordtitle(title_learning_ro_order_things_, word_order).
haswordtitle(title_learning_ro_order_things_, word_ro).
haswordtitle(title_learning_ro_order_things_, word_test).
haswordtitle(title_improving_performance_neural_networks_using_a_boosting_algorithm_, word_a).
haswordtitle(title_improving_performance_neural_networks_using_a_boosting_algorithm_, word_algorithm).
haswordtitle(title_improving_performance_neural_networks_using_a_boosting_algorithm_, word_neural).
haswordtitle(title_improving_performance_neural_networks_using_a_boosting_algorithm_, word_test).
haswordtitle(title_improving_performance_neural_networks_using_a_boosting_algorithm_, word_works).
haswordtitle(title_patrice_1993_improving_performance_neural_networks_using_a_boosting_algorithm_, word_1996).
haswordtitle(title_patrice_1993_improving_performance_neural_networks_using_a_boosting_algorithm_, word_a).
haswordtitle(title_patrice_1993_improving_performance_neural_networks_using_a_boosting_algorithm_, word_algorithm).
haswordtitle(title_patrice_1993_improving_performance_neural_networks_using_a_boosting_algorithm_, word_neural).
haswordtitle(title_patrice_1993_improving_performance_neural_networks_using_a_boosting_algorithm_, word_patrice).
haswordtitle(title_patrice_1993_improving_performance_neural_networks_using_a_boosting_algorithm_, word_test).
haswordtitle(title_patrice_1993_improving_performance_neural_networks_using_a_boosting_algorithm_, word_works).
haswordtitle(title_1993_improving_performance_neural_networks_using_a_boosting_algorithm_, word_1996).
haswordtitle(title_1993_improving_performance_neural_networks_using_a_boosting_algorithm_, word_a).
haswordtitle(title_1993_improving_performance_neural_networks_using_a_boosting_algorithm_, word_algorithm).
haswordtitle(title_1993_improving_performance_neural_networks_using_a_boosting_algorithm_, word_neural).
haswordtitle(title_1993_improving_performance_neural_networks_using_a_boosting_algorithm_, word_test).
haswordtitle(title_1993_improving_performance_neural_networks_using_a_boosting_algorithm_, word_works).
haswordtitle(title_a_general_lower_bound_number_examples_needed_learning_, word_a).
haswordtitle(title_a_general_lower_bound_number_examples_needed_learning_, word_bound).
haswordtitle(title_a_general_lower_bound_number_examples_needed_learning_, word_general).
haswordtitle(title_a_general_lower_bound_number_examples_needed_learning_, word_leslie).
haswordtitle(title_a_general_lower_bound_number_examples_needed_learning_, word_lower).
haswordtitle(title_a_general_lower_bound_number_examples_needed_learning_, word_needed).
haswordtitle(title_a_general_lower_bound_number_examples_needed_learning_, word_number).
haswordtitle(title_a_general_lower_bound_number_examples_needed_learning_, word_test).
haswordtitle(title_data_filtering_distribution_modeling_algorithms_machine_learning_, word_algorithm).
haswordtitle(title_data_filtering_distribution_modeling_algorithms_machine_learning_, word_data).
haswordtitle(title_data_filtering_distribution_modeling_algorithms_machine_learning_, word_macro).
haswordtitle(title_data_filtering_distribution_modeling_algorithms_machine_learning_, word_test).
haswordtitle(title_accelerating_learning_using_query_by_committee_, word_by).
haswordtitle(title_accelerating_learning_using_query_by_committee_, word_query).
haswordtitle(title_accelerating_learning_using_query_by_committee_, word_test).
haswordtitle(title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_adversaries).
haswordtitle(title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_against).
haswordtitle(title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_algorithm).
haswordtitle(title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_bounded).
haswordtitle(title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_games).
haswordtitle(title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_play).
haswordtitle(title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_recent).
haswordtitle(title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_repeated).
haswordtitle(title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_test).
haswordtitle(title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_to).
haswordtitle(title_1995_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_1996).
haswordtitle(title_1995_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_adversaries).
haswordtitle(title_1995_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_against).
haswordtitle(title_1995_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_algorithm).
haswordtitle(title_1995_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_bounded).
haswordtitle(title_1995_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_games).
haswordtitle(title_1995_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_play).
haswordtitle(title_1995_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_recent).
haswordtitle(title_1995_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_repeated).
haswordtitle(title_1995_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_test).
haswordtitle(title_1995_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_, word_to).
haswordtitle(title_predicting_a_binary_sequence_almost_as_as_well_optimal_biased_coin_, word_a).
haswordtitle(title_predicting_a_binary_sequence_almost_as_as_well_optimal_biased_coin_, word_almost).
haswordtitle(title_predicting_a_binary_sequence_almost_as_as_well_optimal_biased_coin_, word_as).
haswordtitle(title_predicting_a_binary_sequence_almost_as_as_well_optimal_biased_coin_, word_biased).
haswordtitle(title_predicting_a_binary_sequence_almost_as_as_well_optimal_biased_coin_, word_binary).
haswordtitle(title_predicting_a_binary_sequence_almost_as_as_well_optimal_biased_coin_, word_coin).
haswordtitle(title_predicting_a_binary_sequence_almost_as_as_well_optimal_biased_coin_, word_optimal).
haswordtitle(title_predicting_a_binary_sequence_almost_as_as_well_optimal_biased_coin_, word_pruning).
haswordtitle(title_predicting_a_binary_sequence_almost_as_as_well_optimal_biased_coin_, word_sequence).
haswordtitle(title_predicting_a_binary_sequence_almost_as_as_well_optimal_biased_coin_, word_well).
haswordtitle(title_predicting_bits_almost_as_as_well_optimal_biased_coin_, word_almost).
haswordtitle(title_predicting_bits_almost_as_as_well_optimal_biased_coin_, word_as).
haswordtitle(title_predicting_bits_almost_as_as_well_optimal_biased_coin_, word_biased).
haswordtitle(title_predicting_bits_almost_as_as_well_optimal_biased_coin_, word_bits).
haswordtitle(title_predicting_bits_almost_as_as_well_optimal_biased_coin_, word_coin).
haswordtitle(title_predicting_bits_almost_as_as_well_optimal_biased_coin_, word_optimal).
haswordtitle(title_predicting_bits_almost_as_as_well_optimal_biased_coin_, word_pruning).
haswordtitle(title_predicting_bits_almost_as_as_well_optimal_biased_coin_, word_well).
haswordtitle(title_predicting_a_binary_sequence_as_as_well_optimal_biased_coin_, word_a).
haswordtitle(title_predicting_a_binary_sequence_as_as_well_optimal_biased_coin_, word_as).
haswordtitle(title_predicting_a_binary_sequence_as_as_well_optimal_biased_coin_, word_biased).
haswordtitle(title_predicting_a_binary_sequence_as_as_well_optimal_biased_coin_, word_binary).
haswordtitle(title_predicting_a_binary_sequence_as_as_well_optimal_biased_coin_, word_coin).
haswordtitle(title_predicting_a_binary_sequence_as_as_well_optimal_biased_coin_, word_optimal).
haswordtitle(title_predicting_a_binary_sequence_as_as_well_optimal_biased_coin_, word_pruning).
haswordtitle(title_predicting_a_binary_sequence_as_as_well_optimal_biased_coin_, word_sequence).
haswordtitle(title_predicting_a_binary_sequence_as_as_well_optimal_biased_coin_, word_well).
haswordtitle(title_learning_binary_relations_total_orders_, word_binary).
haswordtitle(title_learning_binary_relations_total_orders_, word_orders).
haswordtitle(title_learning_binary_relations_total_orders_, word_relations).
haswordtitle(title_learning_binary_relations_total_orders_, word_test).
haswordtitle(title_learning_binary_relations_total_orders_, word_total).
haswordtitle(title_equivalence_models_polynomial_learnability_, word_equivalence).
haswordtitle(title_equivalence_models_polynomial_learnability_, word_models).
haswordtitle(title_equivalence_models_polynomial_learnability_, word_polynomials).
haswordtitle(title_equivalence_models_polynomial_learnability_, word_test).
haswordtitle(title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_, word_bayesian).
haswordtitle(title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_, word_bounds).
haswordtitle(title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_, word_test).
haswordtitle(title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_, word_vc).
haswordtitle(title_unifying_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_, word_bayesian).
haswordtitle(title_unifying_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_, word_bounds).
haswordtitle(title_unifying_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_, word_test).
haswordtitle(title_unifying_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_, word_vc).
haswordtitle(title_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_, word_bayesian).
haswordtitle(title_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_, word_bounds).
haswordtitle(title_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_, word_test).
haswordtitle(title_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_, word_vc).
haswordtitle(title_rigourous_learning_curve_bounds_from_statistical_mechanics_, word_bounds).
haswordtitle(title_rigourous_learning_curve_bounds_from_statistical_mechanics_, word_curve).
haswordtitle(title_rigourous_learning_curve_bounds_from_statistical_mechanics_, word_from).
haswordtitle(title_rigourous_learning_curve_bounds_from_statistical_mechanics_, word_mechanics).
haswordtitle(title_rigourous_learning_curve_bounds_from_statistical_mechanics_, word_rigorous).
haswordtitle(title_rigourous_learning_curve_bounds_from_statistical_mechanics_, word_test).
haswordtitle(title_rigorous_learning_curve_bounds_from_statistical_mechanics_, word_bounds).
haswordtitle(title_rigorous_learning_curve_bounds_from_statistical_mechanics_, word_curve).
haswordtitle(title_rigorous_learning_curve_bounds_from_statistical_mechanics_, word_from).
haswordtitle(title_rigorous_learning_curve_bounds_from_statistical_mechanics_, word_mechanics).
haswordtitle(title_rigorous_learning_curve_bounds_from_statistical_mechanics_, word_rigorous).
haswordtitle(title_rigorous_learning_curve_bounds_from_statistical_mechanics_, word_test).
haswordtitle(title_bart_1993_reasoning_with_characteristic_models_, word_1996).
haswordtitle(title_bart_1993_reasoning_with_characteristic_models_, word_algorithm).
haswordtitle(title_bart_1993_reasoning_with_characteristic_models_, word_bart).
haswordtitle(title_bart_1993_reasoning_with_characteristic_models_, word_characteristic).
haswordtitle(title_bart_1993_reasoning_with_characteristic_models_, word_models).
haswordtitle(title_bart_1993_reasoning_with_characteristic_models_, word_reasoning).
haswordtitle(title_performance_evaluation_protocols_distributed_systems_an_alternative_to_message_counting_, word_alternative).
haswordtitle(title_performance_evaluation_protocols_distributed_systems_an_alternative_to_message_counting_, word_an).
haswordtitle(title_performance_evaluation_protocols_distributed_systems_an_alternative_to_message_counting_, word_counting).
haswordtitle(title_performance_evaluation_protocols_distributed_systems_an_alternative_to_message_counting_, word_distributed).
haswordtitle(title_performance_evaluation_protocols_distributed_systems_an_alternative_to_message_counting_, word_evaluation).
haswordtitle(title_performance_evaluation_protocols_distributed_systems_an_alternative_to_message_counting_, word_message).
haswordtitle(title_performance_evaluation_protocols_distributed_systems_an_alternative_to_message_counting_, word_protocols).
haswordtitle(title_performance_evaluation_protocols_distributed_systems_an_alternative_to_message_counting_, word_systems).
haswordtitle(title_performance_evaluation_protocols_distributed_systems_an_alternative_to_message_counting_, word_test).
haswordtitle(title_performance_evaluation_protocols_distributed_systems_an_alternative_to_message_counting_, word_to).
haswordtitle(title_learning_from_a_population_hypotheses_, word_a).
haswordtitle(title_learning_from_a_population_hypotheses_, word_from).
haswordtitle(title_learning_from_a_population_hypotheses_, word_hypotheses).
haswordtitle(title_learning_from_a_population_hypotheses_, word_test).
haswordtitle(title_learnabil_ity_discrete_distributions_, word_discrete).
haswordtitle(title_learnabil_ity_discrete_distributions_, word_test).
haswordtitle(title_learnability_discrete_distributions_, word_discrete).
haswordtitle(title_learnability_discrete_distributions_, word_test).
haswordtitle(title_learnability_discrete_distributions_extended_abstract_, word_discrete).
haswordtitle(title_learnability_discrete_distributions_extended_abstract_, word_extended).
haswordtitle(title_learnability_discrete_distributions_extended_abstract_, word_test).
haswordtitle(title_an_introduction_to_computational_learning_theory_, word_an).
haswordtitle(title_an_introduction_to_computational_learning_theory_, word_test).
haswordtitle(title_an_introduction_to_computational_learning_theory_, word_to).
haswordtitle(title_introduction_to_computational_learning_theory_, word_test).
haswordtitle(title_introduction_to_computational_learning_theory_, word_to).
haswordtitle(title_vazirani_1995_introduction_to_computational_learning_theory_, word_1996).
haswordtitle(title_vazirani_1995_introduction_to_computational_learning_theory_, word_test).
haswordtitle(title_vazirani_1995_introduction_to_computational_learning_theory_, word_to).
haswordtitle(title_vazirani_1995_introduction_to_computational_learning_theory_, word_vazirani).
haswordtitle(title_an_experimental_theoretical_comparison_model_selection_methods_, word_an).
haswordtitle(title_an_experimental_theoretical_comparison_model_selection_methods_, word_experimental).
haswordtitle(title_an_experimental_theoretical_comparison_model_selection_methods_, word_methods).
haswordtitle(title_an_experimental_theoretical_comparison_model_selection_methods_, word_model).
haswordtitle(title_an_experimental_theoretical_comparison_model_selection_methods_, word_test).
haswordtitle(title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_, word_1996).
haswordtitle(title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_, word_algorithm).
haswordtitle(title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_, word_down).
haswordtitle(title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_, word_mansour).
haswordtitle(title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_, word_test).
haswordtitle(title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_, word_top).
haswordtitle(title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_, word_tree).
haswordtitle(title_boosting_ability_top_down_decision_tree_learning_algorithms_, word_algorithm).
haswordtitle(title_boosting_ability_top_down_decision_tree_learning_algorithms_, word_down).
haswordtitle(title_boosting_ability_top_down_decision_tree_learning_algorithms_, word_test).
haswordtitle(title_boosting_ability_top_down_decision_tree_learning_algorithms_, word_top).
haswordtitle(title_boosting_ability_top_down_decision_tree_learning_algorithms_, word_tree).
haswordtitle(title_a_sparse_sampling_algorithm_near_optimal_planning_large_markov_decision_processes_1999_, word_1999).
haswordtitle(title_a_sparse_sampling_algorithm_near_optimal_planning_large_markov_decision_processes_1999_, word_a).
haswordtitle(title_a_sparse_sampling_algorithm_near_optimal_planning_large_markov_decision_processes_1999_, word_algorithm).
haswordtitle(title_a_sparse_sampling_algorithm_near_optimal_planning_large_markov_decision_processes_1999_, word_large).
haswordtitle(title_a_sparse_sampling_algorithm_near_optimal_planning_large_markov_decision_processes_1999_, word_markov).
haswordtitle(title_a_sparse_sampling_algorithm_near_optimal_planning_large_markov_decision_processes_1999_, word_near).
haswordtitle(title_a_sparse_sampling_algorithm_near_optimal_planning_large_markov_decision_processes_1999_, word_optimal).
haswordtitle(title_a_sparse_sampling_algorithm_near_optimal_planning_large_markov_decision_processes_1999_, word_processes).
haswordtitle(title_a_sparse_sampling_algorithm_near_optimal_planning_large_markov_decision_processes_1999_, word_sparse).
haswordtitle(title_a_sparse_sampling_algorithm_near_optimal_planning_large_markov_decision_processes_1999_, word_test).
haswordtitle(title_md5_message_digest_algorithm_request_comments_informational_rfc_1321_, word_1321).
haswordtitle(title_md5_message_digest_algorithm_request_comments_informational_rfc_1321_, word_algorithm).
haswordtitle(title_md5_message_digest_algorithm_request_comments_informational_rfc_1321_, word_comments).
haswordtitle(title_md5_message_digest_algorithm_request_comments_informational_rfc_1321_, word_informational).
haswordtitle(title_md5_message_digest_algorithm_request_comments_informational_rfc_1321_, word_md5).
haswordtitle(title_md5_message_digest_algorithm_request_comments_informational_rfc_1321_, word_message).
haswordtitle(title_md5_message_digest_algorithm_request_comments_informational_rfc_1321_, word_request).
haswordtitle(title_md5_message_digest_algorithm_request_comments_informational_rfc_1321_, word_rfc).
haswordtitle(title_inference_finite_automata_using_homing_sequences_, word_finite).
haswordtitle(title_inference_finite_automata_using_homing_sequences_, word_test).
haswordtitle(title_1993_inference_finite_automata_using_homing_sequences_, word_1996).
haswordtitle(title_1993_inference_finite_automata_using_homing_sequences_, word_finite).
haswordtitle(title_1993_inference_finite_automata_using_homing_sequences_, word_test).
haswordtitle(title_inference_finite_automata_using_homing_sequences_r_l_, word_finite).
haswordtitle(title_inference_finite_automata_using_homing_sequences_r_l_, word_l).
haswordtitle(title_inference_finite_automata_using_homing_sequences_r_l_, word_r).
haswordtitle(title_inference_finite_automata_using_homing_sequences_r_l_, word_test).
haswordtitle(title_inference_finite_automata_using_homing_sequences_information_computation_, word_finite).
haswordtitle(title_inference_finite_automata_using_homing_sequences_information_computation_, word_test).
haswordtitle(title_diversity_based_inference_finite_automata_, word_based).
haswordtitle(title_diversity_based_inference_finite_automata_, word_diversity).
haswordtitle(title_diversity_based_inference_finite_automata_, word_finite).
haswordtitle(title_diversity_based_inference_finite_automata_, word_test).
haswordtitle(title_design_analysis_efficient_learning_algorithms_, word_algorithm).
haswordtitle(title_design_analysis_efficient_learning_algorithms_, word_design).
haswordtitle(title_design_analysis_efficient_learning_algorithms_, word_recent).
haswordtitle(title_design_analysis_efficient_learning_algorithms_, word_test).
haswordtitle(title_private_correspondence_, word_test).
haswordtitle(title_learning_probabilistic_read_once_formulas_product_distributions_, word_read).
haswordtitle(title_learning_probabilistic_read_once_formulas_product_distributions_, word_test).
haswordtitle(title_design_analysis_ef_ficient_learning_algorithms_, word_algorithm).
haswordtitle(title_design_analysis_ef_ficient_learning_algorithms_, word_design).
haswordtitle(title_design_analysis_ef_ficient_learning_algorithms_, word_ef).
haswordtitle(title_design_analysis_ef_ficient_learning_algorithms_, word_recent).
haswordtitle(title_design_analysis_ef_ficient_learning_algorithms_, word_test).
haswordtitle(title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_, word_a).
haswordtitle(title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_, word_algorithm).
haswordtitle(title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_, word_field).
haswordtitle(title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_, word_over).
haswordtitle(title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_, word_polynomials).
haswordtitle(title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_, word_queries).
haswordtitle(title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_, word_sparse).
haswordtitle(title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_, word_test).
haswordtitle(title_worst_case_analysis_temporal_difference_learning_algorithms_, word_algorithm).
haswordtitle(title_worst_case_analysis_temporal_difference_learning_algorithms_, word_case).
haswordtitle(title_worst_case_analysis_temporal_difference_learning_algorithms_, word_temporal).
haswordtitle(title_worst_case_analysis_temporal_difference_learning_algorithms_, word_test).
haswordtitle(title_worst_case_analysis_temporal_difference_learning_algorithms_, word_worst).
haswordtitle(title_worst_case_analysis_temporal_difference_learing_algorithms_, word_algorithm).
haswordtitle(title_worst_case_analysis_temporal_difference_learing_algorithms_, word_case).
haswordtitle(title_worst_case_analysis_temporal_difference_learing_algorithms_, word_temporal).
haswordtitle(title_worst_case_analysis_temporal_difference_learing_algorithms_, word_test).
haswordtitle(title_worst_case_analysis_temporal_difference_learing_algorithms_, word_worst).
haswordtitle(title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_, word_a).
haswordtitle(title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_, word_algorithm).
haswordtitle(title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_, word_field).
haswordtitle(title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_, word_over).
haswordtitle(title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_, word_polynomials).
haswordtitle(title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_, word_queries).
haswordtitle(title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_, word_sparse).
haswordtitle(title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_, word_test).
haswordtitle(title_private_communication_, word_test).
haswordtitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_a).
haswordtitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_margin).
haswordtitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_methods).
haswordtitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_new).
haswordtitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_test).
haswordtitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_a).
haswordtitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_machines).
haswordtitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_margin).
haswordtitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_methods).
haswordtitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_new).
haswordtitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_snowbird).
haswordtitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_test).
haswordtitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_that).
haswordtitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_1996).
haswordtitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_a).
haswordtitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_lee).
haswordtitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_margin).
haswordtitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_methods).
haswordtitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_new).
haswordtitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_test).
haswordtitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_a).
haswordtitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_margin).
haswordtitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_methods).
haswordtitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_new).
haswordtitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_s).
haswordtitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_test).
haswordtitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_w).
haswordtitle(title_boosting_margin_, word_margin).
haswordtitle(title_boosting_margin_, word_test).
haswordtitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_1996).
haswordtitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_a).
haswordtitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_bartlett).
haswordtitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_lee).
haswordtitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_margin).
haswordtitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_methods).
haswordtitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_new).
haswordtitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_s).
haswordtitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_test).
haswordtitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_, word_w).
haswordtitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_a).
haswordtitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_leee).
haswordtitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_machines).
haswordtitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_margin).
haswordtitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_methods).
haswordtitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_new).
haswordtitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_s).
haswordtitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_snowbird).
haswordtitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_test).
haswordtitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_that).
haswordtitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_, word_w).
haswordtitle(title_query_by_committee_, word_by).
haswordtitle(title_query_by_committee_, word_query).
haswordtitle(title_query_by_committee_, word_test).
haswordtitle(title_using_output_codes_to_boost_multiclass_learning_problems_, word_boost).
haswordtitle(title_using_output_codes_to_boost_multiclass_learning_problems_, word_codes).
haswordtitle(title_using_output_codes_to_boost_multiclass_learning_problems_, word_multi).
haswordtitle(title_using_output_codes_to_boost_multiclass_learning_problems_, word_output).
haswordtitle(title_using_output_codes_to_boost_multiclass_learning_problems_, word_problems).
haswordtitle(title_using_output_codes_to_boost_multiclass_learning_problems_, word_test).
haswordtitle(title_using_output_codes_to_boost_multiclass_learning_problems_, word_to).
haswordtitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_, word_algorithm).
haswordtitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_, word_test).
haswordtitle(title_a_system_multiclass_multi_label_text_categorization_, word_a).
haswordtitle(title_a_system_multiclass_multi_label_text_categorization_, word_label).
haswordtitle(title_a_system_multiclass_multi_label_text_categorization_, word_multi).
haswordtitle(title_a_system_multiclass_multi_label_text_categorization_, word_systems).
haswordtitle(title_a_system_multiclass_multi_label_text_categorization_, word_test).
haswordtitle(title_a_system_multiclass_multi_label_text_categorization_, word_text).

haswordvenue(venue_proceedings_26th_symposium_theory_computing_, word_26th).
haswordvenue(venue_proceedings_26th_symposium_theory_computing_, word_pos).
haswordvenue(venue_proceedings_26th_symposium_theory_computing_, word_test).
haswordvenue(venue_proceedings_twenty_sixth_symposium_theory_computing_, word_pos).
haswordvenue(venue_proceedings_twenty_sixth_symposium_theory_computing_, word_sixth).
haswordvenue(venue_proceedings_twenty_sixth_symposium_theory_computing_, word_test).
haswordvenue(venue_proc_26th_sympos_theory_comput_, word_26th).
haswordvenue(venue_proc_26th_sympos_theory_comput_, word_pos).
haswordvenue(venue_proc_26th_sympos_theory_comput_, word_test).
haswordvenue(venue_t_t_he_26_th_symposium_heory_computing_, word_26).
haswordvenue(venue_t_t_he_26_th_symposium_heory_computing_, word_he).
haswordvenue(venue_t_t_he_26_th_symposium_heory_computing_, word_pos).
haswordvenue(venue_t_t_he_26_th_symposium_heory_computing_, word_t).
haswordvenue(venue_t_t_he_26_th_symposium_heory_computing_, word_test).
haswordvenue(venue_t_t_he_26_th_symposium_heory_computing_, word_th).
haswordvenue(venue_proceedings_26_th_symposium_theory_computing_, word_26).
haswordvenue(venue_proceedings_26_th_symposium_theory_computing_, word_pos).
haswordvenue(venue_proceedings_26_th_symposium_theory_computing_, word_test).
haswordvenue(venue_proceedings_26_th_symposium_theory_computing_, word_th).
haswordvenue(venue_advances_neural_information_processing_systems_10_, word_10).
haswordvenue(venue_advances_neural_information_processing_systems_10_, word_neural).
haswordvenue(venue_advances_neural_information_processing_systems_10_, word_systems).
haswordvenue(venue_advances_neural_information_processing_systems_10_, word_test).
haswordvenue(venue_advances_neural_information_processing_systems_, word_neural).
haswordvenue(venue_advances_neural_information_processing_systems_, word_systems).
haswordvenue(venue_advances_neural_information_processing_systems_, word_test).
haswordvenue(venue_advances_neural_information_processing_systems_vol_4_, word_4).
haswordvenue(venue_advances_neural_information_processing_systems_vol_4_, word_neural).
haswordvenue(venue_advances_neural_information_processing_systems_vol_4_, word_systems).
haswordvenue(venue_advances_neural_information_processing_systems_vol_4_, word_test).
haswordvenue(venue_advances_neural_information_processing_systems_vol_4_, word_vol).
haswordvenue(venue_advances_neural_information_processing_systems_volume_5_, word_5).
haswordvenue(venue_advances_neural_information_processing_systems_volume_5_, word_neural).
haswordvenue(venue_advances_neural_information_processing_systems_volume_5_, word_systems).
haswordvenue(venue_advances_neural_information_processing_systems_volume_5_, word_test).
haswordvenue(venue_advances_neural_information_processing_systems_volume_5_, word_volume).
haswordvenue(venue_advances_neural_information_processing_systems_5_, word_5).
haswordvenue(venue_advances_neural_information_processing_systems_5_, word_neural).
haswordvenue(venue_advances_neural_information_processing_systems_5_, word_systems).
haswordvenue(venue_advances_neural_information_processing_systems_5_, word_test).
haswordvenue(venue_advances_neural_informations_processing_systems_5_, word_5).
haswordvenue(venue_advances_neural_informations_processing_systems_5_, word_neural).
haswordvenue(venue_advances_neural_informations_processing_systems_5_, word_systems).
haswordvenue(venue_advances_neural_informations_processing_systems_5_, word_test).
haswordvenue(venue_proceedings_fifth_conf_neural_informations_processing_systems_, word_fifth).
haswordvenue(venue_proceedings_fifth_conf_neural_informations_processing_systems_, word_neural).
haswordvenue(venue_proceedings_fifth_conf_neural_informations_processing_systems_, word_systems).
haswordvenue(venue_proceedings_fifth_conf_neural_informations_processing_systems_, word_test).
haswordvenue(venue_neural_information_processing_systems_, word_neural).
haswordvenue(venue_neural_information_processing_systems_, word_systems).
haswordvenue(venue_neural_information_processing_systems_, word_test).
haswordvenue(venue_technical_report_ucsc_crl_87_26_, word_26).
haswordvenue(venue_technical_report_ucsc_crl_87_26_, word_87).
haswordvenue(venue_technical_report_ucsc_crl_87_26_, word_crl).
haswordvenue(venue_technical_report_ucsc_crl_87_26_, word_rep).
haswordvenue(venue_technical_report_ucsc_crl_87_26_, word_technical).
haswordvenue(venue_technical_report_ucsc_crl_87_26_, word_ucsc).
haswordvenue(venue_phd_thesis_, word_phd).
haswordvenue(venue_phd_thesis_, word_test).
haswordvenue(venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_, word_1996).
haswordvenue(venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_, word_neural).
haswordvenue(venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_, word_systems).
haswordvenue(venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_, word_test).
haswordvenue(venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_, word_to).
haswordvenue(venue_proceedings_36_th_symposium_foundations_computer_science_, word_36).
haswordvenue(venue_proceedings_36_th_symposium_foundations_computer_science_, word_pos).
haswordvenue(venue_proceedings_36_th_symposium_foundations_computer_science_, word_test).
haswordvenue(venue_proceedings_36_th_symposium_foundations_computer_science_, word_th).
haswordvenue(venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_, word_pos).
haswordvenue(venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_, word_sixth).
haswordvenue(venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_, word_test).
haswordvenue(venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_, word_thirtieth).
haswordvenue(venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_, word_to).
haswordvenue(venue_proceedings_thirty_seventh_symposium_foundations_computer_science_, word_pos).
haswordvenue(venue_proceedings_thirty_seventh_symposium_foundations_computer_science_, word_seventh).
haswordvenue(venue_proceedings_thirty_seventh_symposium_foundations_computer_science_, word_test).
haswordvenue(venue_proceedings_thirty_seventh_symposium_foundations_computer_science_, word_thirtieth).
haswordvenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_, word_pos).
haswordvenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_, word_sixth).
haswordvenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_, word_test).
haswordvenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_, word_thirtieth).
haswordvenue(venue_focs_36_, word_36).
haswordvenue(venue_focs_36_, word_focs).
haswordvenue(venue_ieee_symposium_foundations_computer_science_, word_ieee).
haswordvenue(venue_ieee_symposium_foundations_computer_science_, word_pos).
haswordvenue(venue_ieee_symposium_foundations_computer_science_, word_test).
haswordvenue(venue_proceedings_9th_conference_computational_learning_theory_, word_9th).
haswordvenue(venue_proceedings_9th_conference_computational_learning_theory_, word_test).
haswordvenue(venue_proc_9th_conference_computational_learning_theory_, word_9th).
haswordvenue(venue_proc_9th_conference_computational_learning_theory_, word_test).
haswordvenue(venue_proceedings_ninth_conference_computational_learning_theory_, word_test).
haswordvenue(venue_proceedings_30th_symposium_foundation_computer_science_, word_30th).
haswordvenue(venue_proceedings_30th_symposium_foundation_computer_science_, word_pos).
haswordvenue(venue_proceedings_30th_symposium_foundation_computer_science_, word_test).
haswordvenue(venue_30th_symposium_foundations_computer_science_, word_30th).
haswordvenue(venue_30th_symposium_foundations_computer_science_, word_pos).
haswordvenue(venue_30th_symposium_foundations_computer_science_, word_test).
haswordvenue(venue_proceedings_thirtieth_symposium_foundations_computer_science_, word_pos).
haswordvenue(venue_proceedings_thirtieth_symposium_foundations_computer_science_, word_test).
haswordvenue(venue_proceedings_thirtieth_symposium_foundations_computer_science_, word_thirtieth).
haswordvenue(venue_proceedings_30th_symposium_foundations_computer_science_, word_30th).
haswordvenue(venue_proceedings_30th_symposium_foundations_computer_science_, word_pos).
haswordvenue(venue_proceedings_30th_symposium_foundations_computer_science_, word_test).
haswordvenue(venue_technical_report_mit_lcs_tm_413_, word_413).
haswordvenue(venue_technical_report_mit_lcs_tm_413_, word_lcs).
haswordvenue(venue_technical_report_mit_lcs_tm_413_, word_mit).
haswordvenue(venue_technical_report_mit_lcs_tm_413_, word_rep).
haswordvenue(venue_technical_report_mit_lcs_tm_413_, word_technical).
haswordvenue(venue_technical_report_mit_lcs_tm_413_, word_tm).
haswordvenue(venue_technical_report_number_ucsc_crl_88_06_, word_06).
haswordvenue(venue_technical_report_number_ucsc_crl_88_06_, word_88).
haswordvenue(venue_technical_report_number_ucsc_crl_88_06_, word_crl).
haswordvenue(venue_technical_report_number_ucsc_crl_88_06_, word_number).
haswordvenue(venue_technical_report_number_ucsc_crl_88_06_, word_rep).
haswordvenue(venue_technical_report_number_ucsc_crl_88_06_, word_technical).
haswordvenue(venue_technical_report_number_ucsc_crl_88_06_, word_ucsc).
haswordvenue(venue_proceedings_fourth_conference_computational_learning_theory_colt_91_, word_91).
haswordvenue(venue_proceedings_fourth_conference_computational_learning_theory_colt_91_, word_colt).
haswordvenue(venue_proceedings_fourth_conference_computational_learning_theory_colt_91_, word_test).
haswordvenue(venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_, word_61).
haswordvenue(venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_, word_74).
haswordvenue(venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_, word_pages).
haswordvenue(venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_, word_test).
haswordvenue(venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_, word_workshop).
haswordvenue(venue_proceedings_4th_workshop_computational_learning_theory_, word_4th).
haswordvenue(venue_proceedings_4th_workshop_computational_learning_theory_, word_test).
haswordvenue(venue_proceedings_4th_workshop_computational_learning_theory_, word_workshop).
haswordvenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_, word_4th).
haswordvenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_, word_91).
haswordvenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_, word_colt).
haswordvenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_, word_test).
haswordvenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_, word_workshop).
haswordvenue(venue_computational_learning_theory_proceedings_fourth_workshop_, word_test).
haswordvenue(venue_computational_learning_theory_proceedings_fourth_workshop_, word_workshop).
haswordvenue(venue_proc_4th_workshop_comput_learning_theory_, word_4th).
haswordvenue(venue_proc_4th_workshop_comput_learning_theory_, word_test).
haswordvenue(venue_proc_4th_workshop_comput_learning_theory_, word_workshop).
haswordvenue(venue_proc_4th_workshop_computational_learning_theory_, word_4th).
haswordvenue(venue_proc_4th_workshop_computational_learning_theory_, word_test).
haswordvenue(venue_proc_4th_workshop_computational_learning_theory_, word_workshop).
haswordvenue(venue_proceedings_fourth_workshop_computational_learning_theory_, word_test).
haswordvenue(venue_proceedings_fourth_workshop_computational_learning_theory_, word_workshop).
haswordvenue(venue_1991_workshop_computational_learning_theory_, word_1996).
haswordvenue(venue_1991_workshop_computational_learning_theory_, word_test).
haswordvenue(venue_1991_workshop_computational_learning_theory_, word_workshop).
haswordvenue(venue_proceedings_1991_workshop_computational_learning_theory_, word_1996).
haswordvenue(venue_proceedings_1991_workshop_computational_learning_theory_, word_test).
haswordvenue(venue_proceedings_1991_workshop_computational_learning_theory_, word_workshop).
haswordvenue(venue_machine_learning_, word_macro).
haswordvenue(venue_machine_learning_, word_test).
haswordvenue(venue_technical_report_, word_rep).
haswordvenue(venue_technical_report_, word_technical).
haswordvenue(venue_proceedings_winter_simulation_conference_, word_simulation).
haswordvenue(venue_proceedings_winter_simulation_conference_, word_test).
haswordvenue(venue_proceedings_winter_simulation_conference_, word_winter).
haswordvenue(venue_proceedings_colt_93_, word_93).
haswordvenue(venue_proceedings_colt_93_, word_colt).
haswordvenue(venue_proceedings_colt_93_, word_test).
haswordvenue(venue_25th_symposium_theory_computing_, word_5th).
haswordvenue(venue_25th_symposium_theory_computing_, word_pos).
haswordvenue(venue_25th_symposium_theory_computing_, word_test).
haswordvenue(venue_25th_symposium_theory_computing_1994_, word_1994).
haswordvenue(venue_25th_symposium_theory_computing_1994_, word_5th).
haswordvenue(venue_25th_symposium_theory_computing_1994_, word_pos).
haswordvenue(venue_25th_symposium_theory_computing_1994_, word_test).
haswordvenue(venue_proc_26th_symp_theory_computing_, word_26th).
haswordvenue(venue_proc_26th_symp_theory_computing_, word_pos).
haswordvenue(venue_proc_26th_symp_theory_computing_, word_test).
haswordvenue(venue_stoc_, word_stoc).
haswordvenue(venue_stoc_94_, word_94).
haswordvenue(venue_stoc_94_, word_stoc).
haswordvenue(venue_25th_symp_theory_computing_, word_5th).
haswordvenue(venue_25th_symp_theory_computing_, word_pos).
haswordvenue(venue_25th_symp_theory_computing_, word_test).
haswordvenue(venue_25th_symposium_theory_computing_to_appear_, word_5th).
haswordvenue(venue_25th_symposium_theory_computing_to_appear_, word_pos).
haswordvenue(venue_25th_symposium_theory_computing_to_appear_, word_test).
haswordvenue(venue_25th_symposium_theory_computing_to_appear_, word_to).
haswordvenue(venue_stoc_p_, word_p).
haswordvenue(venue_stoc_p_, word_stoc).
haswordvenue(venue_symposium_theory_computing_, word_pos).
haswordvenue(venue_symposium_theory_computing_, word_test).
haswordvenue(venue_57_62_, word_57).
haswordvenue(venue_57_62_, word_62).
haswordvenue(venue_chp_, word_chp).
haswordvenue(venue_proceedings_colt_95_, word_95).
haswordvenue(venue_proceedings_colt_95_, word_colt).
haswordvenue(venue_proceedings_colt_95_, word_test).
haswordvenue(venue_proceedings_eighth_workshop_computational_learning_theory_, word_eighth).
haswordvenue(venue_proceedings_eighth_workshop_computational_learning_theory_, word_test).
haswordvenue(venue_proceedings_eighth_workshop_computational_learning_theory_, word_workshop).
haswordvenue(venue_proceedings_eighth_conference_computational_learning_theory_, word_eighth).
haswordvenue(venue_proceedings_eighth_conference_computational_learning_theory_, word_test).
haswordvenue(venue_proceedings_eigth_conference_computational_learning_theory_, word_eighth).
haswordvenue(venue_proceedings_eigth_conference_computational_learning_theory_, word_test).
haswordvenue(venue_colt_95_, word_95).
haswordvenue(venue_colt_95_, word_colt).
haswordvenue(venue_proceedings_seventh_workshop_computational_learning_theory_, word_seventh).
haswordvenue(venue_proceedings_seventh_workshop_computational_learning_theory_, word_test).
haswordvenue(venue_proceedings_seventh_workshop_computational_learning_theory_, word_workshop).
haswordvenue(venue_proc_workshop_computational_learning_theory_, word_test).
haswordvenue(venue_proc_workshop_computational_learning_theory_, word_workshop).
haswordvenue(venue_to_appear_proceedings_28th_symposium_theory_computing_, word_28th).
haswordvenue(venue_to_appear_proceedings_28th_symposium_theory_computing_, word_pos).
haswordvenue(venue_to_appear_proceedings_28th_symposium_theory_computing_, word_test).
haswordvenue(venue_to_appear_proceedings_28th_symposium_theory_computing_, word_to).
haswordvenue(venue_proceedings_twenty_eighth_symposium_theory_computing_, word_eighth).
haswordvenue(venue_proceedings_twenty_eighth_symposium_theory_computing_, word_pos).
haswordvenue(venue_proceedings_twenty_eighth_symposium_theory_computing_, word_test).
haswordvenue(venue_proceedings_28th_symposium_theory_computing_, word_28th).
haswordvenue(venue_proceedings_28th_symposium_theory_computing_, word_pos).
haswordvenue(venue_proceedings_28th_symposium_theory_computing_, word_test).
haswordvenue(venue_unpublished_manuscript_, word_manuscript).
haswordvenue(venue_proc_21st_symp_theoretical_computing_, word_21st).
haswordvenue(venue_proc_21st_symp_theoretical_computing_, word_pos).
haswordvenue(venue_proc_21st_symp_theoretical_computing_, word_test).
haswordvenue(venue_information_computation_, word_test).
haswordvenue(venue_information_computation_103_2_, word_103).
haswordvenue(venue_information_computation_103_2_, word_2).
haswordvenue(venue_information_computation_103_2_, word_test).
haswordvenue(venue_machine_learning_from_theory_to_applications_, word_from).
haswordvenue(venue_machine_learning_from_theory_to_applications_, word_macro).
haswordvenue(venue_machine_learning_from_theory_to_applications_, word_test).
haswordvenue(venue_machine_learning_from_theory_to_applications_, word_to).
haswordvenue(venue_formation_computation_103_2_1993_, word_103).
haswordvenue(venue_formation_computation_103_2_1993_, word_1996).
haswordvenue(venue_formation_computation_103_2_1993_, word_2).
haswordvenue(venue_formation_computation_103_2_1993_, word_test).
haswordvenue(venue_information_compuation_, word_test).
haswordvenue(venue_information_computation_103_, word_103).
haswordvenue(venue_information_computation_103_, word_test).
haswordvenue(venue_journal_association_computing_machinery_, word_nineteenth).
haswordvenue(venue_journal_association_computing_machinery_, word_test).
haswordvenue(venue_technical_report_mit_lcs_tr_493_, word_493).
haswordvenue(venue_technical_report_mit_lcs_tr_493_, word_lcs).
haswordvenue(venue_technical_report_mit_lcs_tr_493_, word_mit).
haswordvenue(venue_technical_report_mit_lcs_tr_493_, word_rep).
haswordvenue(venue_technical_report_mit_lcs_tr_493_, word_technical).
haswordvenue(venue_technical_report_mit_lcs_tr_493_, word_tr).
haswordvenue(venue_computation_learning_theory_proceedings_fourth_workshop_, word_test).
haswordvenue(venue_computation_learning_theory_proceedings_fourth_workshop_, word_workshop).
haswordvenue(venue_proceedings_sixth_workshop_computational_learning_theory_, word_sixth).
haswordvenue(venue_proceedings_sixth_workshop_computational_learning_theory_, word_test).
haswordvenue(venue_proceedings_sixth_workshop_computational_learning_theory_, word_workshop).
haswordvenue(venue_machine_learning_proceedings_eleventh_international_conference_, word_eleventh).
haswordvenue(venue_machine_learning_proceedings_eleventh_international_conference_, word_macro).
haswordvenue(venue_machine_learning_proceedings_eleventh_international_conference_, word_test).
haswordvenue(venue_proceddings_11th_international_conference_machine_learning_, word_11th).
haswordvenue(venue_proceddings_11th_international_conference_machine_learning_, word_macro).
haswordvenue(venue_proceddings_11th_international_conference_machine_learning_, word_test).
haswordvenue(venue_proceedings_eleventh_international_conference_machine_learning_, word_eleventh).
haswordvenue(venue_proceedings_eleventh_international_conference_machine_learning_, word_macro).
haswordvenue(venue_proceedings_eleventh_international_conference_machine_learning_, word_test).
haswordvenue(venue_proc_11th_international_conference_machine_learning_, word_11th).
haswordvenue(venue_proc_11th_international_conference_machine_learning_, word_macro).
haswordvenue(venue_proc_11th_international_conference_machine_learning_, word_test).
haswordvenue(venue_proc_11th_international_conf_machine_learning_, word_11th).
haswordvenue(venue_proc_11th_international_conf_machine_learning_, word_macro).
haswordvenue(venue_proc_11th_international_conf_machine_learning_, word_test).
haswordvenue(venue_j_computer_system_sciences_, word_j).
haswordvenue(venue_j_computer_system_sciences_, word_systems).
haswordvenue(venue_j_computer_system_sciences_, word_test).
haswordvenue(venue_proceedings_fourteenth_international_conference_machine_learning_, word_fourteenth).
haswordvenue(venue_proceedings_fourteenth_international_conference_machine_learning_, word_macro).
haswordvenue(venue_proceedings_fourteenth_international_conference_machine_learning_, word_test).
haswordvenue(venue_machine_learning_proceedings_fourteenth_international_conference_, word_fourteenth).
haswordvenue(venue_machine_learning_proceedings_fourteenth_international_conference_, word_macro).
haswordvenue(venue_machine_learning_proceedings_fourteenth_international_conference_, word_test).
haswordvenue(venue_proceedings_14th_international_conference_machine_learning_, word_14th).
haswordvenue(venue_proceedings_14th_international_conference_machine_learning_, word_macro).
haswordvenue(venue_proceedings_14th_international_conference_machine_learning_, word_test).
haswordvenue(venue_proceedings_11th_international_conference_machine_learning_, word_11th).
haswordvenue(venue_proceedings_11th_international_conference_machine_learning_, word_macro).
haswordvenue(venue_proceedings_11th_international_conference_machine_learning_, word_test).
haswordvenue(venue_snowbird_1997_conference_, word_1996).
haswordvenue(venue_snowbird_1997_conference_, word_snowbird).
haswordvenue(venue_snowbird_1997_conference_, word_test).
haswordvenue(venue_1997_, word_1996).
haswordvenue(venue_icml_97_, word_97).
haswordvenue(venue_icml_97_, word_icml).
haswordvenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_, word_97).
haswordvenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_, word_fourteenth).
haswordvenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_, word_icml).
haswordvenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_, word_macro).
haswordvenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_, word_test).
haswordvenue(venue_tech_rep_, word_rep).
haswordvenue(venue_tech_rep_, word_technical).
haswordvenue(venue_proceedings_eleventh_conference_computational_learning_theory_, word_eleventh).
haswordvenue(venue_proceedings_eleventh_conference_computational_learning_theory_, word_test).

samebib(class_7,class_7).
samebib(class_7,class_8).
samebib(class_7,class_9).
samebib(class_7,class_10).
samebib(class_7,class_11).
samebib(class_7,class_12).
samebib(class_7,class_13).
samebib(class_7,class_14).
samebib(class_7,class_15).
samebib(class_7,class_16).
samebib(class_8,class_7).
samebib(class_8,class_8).
samebib(class_8,class_9).
samebib(class_8,class_10).
samebib(class_8,class_11).
samebib(class_8,class_12).
samebib(class_8,class_13).
samebib(class_8,class_14).
samebib(class_8,class_15).
samebib(class_8,class_16).
samebib(class_9,class_7).
samebib(class_9,class_8).
samebib(class_9,class_9).
samebib(class_9,class_10).
samebib(class_9,class_11).
samebib(class_9,class_12).
samebib(class_9,class_13).
samebib(class_9,class_14).
samebib(class_9,class_15).
samebib(class_9,class_16).
samebib(class_10,class_7).
samebib(class_10,class_8).
samebib(class_10,class_9).
samebib(class_10,class_10).
samebib(class_10,class_11).
samebib(class_10,class_12).
samebib(class_10,class_13).
samebib(class_10,class_14).
samebib(class_10,class_15).
samebib(class_10,class_16).
samebib(class_11,class_7).
samebib(class_11,class_8).
samebib(class_11,class_9).
samebib(class_11,class_10).
samebib(class_11,class_11).
samebib(class_11,class_12).
samebib(class_11,class_13).
samebib(class_11,class_14).
samebib(class_11,class_15).
samebib(class_11,class_16).
samebib(class_12,class_7).
samebib(class_12,class_8).
samebib(class_12,class_9).
samebib(class_12,class_10).
samebib(class_12,class_11).
samebib(class_12,class_12).
samebib(class_12,class_13).
samebib(class_12,class_14).
samebib(class_12,class_15).
samebib(class_12,class_16).
samebib(class_13,class_7).
samebib(class_13,class_8).
samebib(class_13,class_9).
samebib(class_13,class_10).
samebib(class_13,class_11).
samebib(class_13,class_12).
samebib(class_13,class_13).
samebib(class_13,class_14).
samebib(class_13,class_15).
samebib(class_13,class_16).
samebib(class_14,class_7).
samebib(class_14,class_8).
samebib(class_14,class_9).
samebib(class_14,class_10).
samebib(class_14,class_11).
samebib(class_14,class_12).
samebib(class_14,class_13).
samebib(class_14,class_14).
samebib(class_14,class_15).
samebib(class_14,class_16).
samebib(class_15,class_7).
samebib(class_15,class_8).
samebib(class_15,class_9).
samebib(class_15,class_10).
samebib(class_15,class_11).
samebib(class_15,class_12).
samebib(class_15,class_13).
samebib(class_15,class_14).
samebib(class_15,class_15).
samebib(class_15,class_16).
samebib(class_16,class_7).
samebib(class_16,class_8).
samebib(class_16,class_9).
samebib(class_16,class_10).
samebib(class_16,class_11).
samebib(class_16,class_12).
samebib(class_16,class_13).
samebib(class_16,class_14).
samebib(class_16,class_15).
samebib(class_16,class_16).
samebib(class_81,class_81).
samebib(class_81,class_82).
samebib(class_81,class_83).
samebib(class_81,class_84).
samebib(class_82,class_81).
samebib(class_82,class_82).
samebib(class_82,class_83).
samebib(class_82,class_84).
samebib(class_83,class_81).
samebib(class_83,class_82).
samebib(class_83,class_83).
samebib(class_83,class_84).
samebib(class_84,class_81).
samebib(class_84,class_82).
samebib(class_84,class_83).
samebib(class_84,class_84).
samebib(class_95,class_95).
samebib(class_95,class_96).
samebib(class_95,class_97).
samebib(class_95,class_98).
samebib(class_95,class_99).
samebib(class_95,class_100).
samebib(class_95,class_101).
samebib(class_95,class_102).
samebib(class_95,class_103).
samebib(class_95,class_104).
samebib(class_95,class_105).
samebib(class_95,class_106).
samebib(class_95,class_107).
samebib(class_95,class_108).
samebib(class_95,class_109).
samebib(class_95,class_110).
samebib(class_95,class_111).
samebib(class_95,class_112).
samebib(class_95,class_113).
samebib(class_95,class_114).
samebib(class_95,class_115).
samebib(class_96,class_95).
samebib(class_96,class_96).
samebib(class_96,class_97).
samebib(class_96,class_98).
samebib(class_96,class_99).
samebib(class_96,class_100).
samebib(class_96,class_101).
samebib(class_96,class_102).
samebib(class_96,class_103).
samebib(class_96,class_104).
samebib(class_96,class_105).
samebib(class_96,class_106).
samebib(class_96,class_107).
samebib(class_96,class_108).
samebib(class_96,class_109).
samebib(class_96,class_110).
samebib(class_96,class_111).
samebib(class_96,class_112).
samebib(class_96,class_113).
samebib(class_96,class_114).
samebib(class_96,class_115).
samebib(class_97,class_95).
samebib(class_97,class_96).
samebib(class_97,class_97).
samebib(class_97,class_98).
samebib(class_97,class_99).
samebib(class_97,class_100).
samebib(class_97,class_101).
samebib(class_97,class_102).
samebib(class_97,class_103).
samebib(class_97,class_104).
samebib(class_97,class_105).
samebib(class_97,class_106).
samebib(class_97,class_107).
samebib(class_97,class_108).
samebib(class_97,class_109).
samebib(class_97,class_110).
samebib(class_97,class_111).
samebib(class_97,class_112).
samebib(class_97,class_113).
samebib(class_97,class_114).
samebib(class_97,class_115).
samebib(class_98,class_95).
samebib(class_98,class_96).
samebib(class_98,class_97).
samebib(class_98,class_98).
samebib(class_98,class_99).
samebib(class_98,class_100).
samebib(class_98,class_101).
samebib(class_98,class_102).
samebib(class_98,class_103).
samebib(class_98,class_104).
samebib(class_98,class_105).
samebib(class_98,class_106).
samebib(class_98,class_107).
samebib(class_98,class_108).
samebib(class_98,class_109).
samebib(class_98,class_110).
samebib(class_98,class_111).
samebib(class_98,class_112).
samebib(class_98,class_113).
samebib(class_98,class_114).
samebib(class_98,class_115).
samebib(class_99,class_95).
samebib(class_99,class_96).
samebib(class_99,class_97).
samebib(class_99,class_98).
samebib(class_99,class_99).
samebib(class_99,class_100).
samebib(class_99,class_101).
samebib(class_99,class_102).
samebib(class_99,class_103).
samebib(class_99,class_104).
samebib(class_99,class_105).
samebib(class_99,class_106).
samebib(class_99,class_107).
samebib(class_99,class_108).
samebib(class_99,class_109).
samebib(class_99,class_110).
samebib(class_99,class_111).
samebib(class_99,class_112).
samebib(class_99,class_113).
samebib(class_99,class_114).
samebib(class_99,class_115).
samebib(class_100,class_95).
samebib(class_100,class_96).
samebib(class_100,class_97).
samebib(class_100,class_98).
samebib(class_100,class_99).
samebib(class_100,class_100).
samebib(class_100,class_101).
samebib(class_100,class_102).
samebib(class_100,class_103).
samebib(class_100,class_104).
samebib(class_100,class_105).
samebib(class_100,class_106).
samebib(class_100,class_107).
samebib(class_100,class_108).
samebib(class_100,class_109).
samebib(class_100,class_110).
samebib(class_100,class_111).
samebib(class_100,class_112).
samebib(class_100,class_113).
samebib(class_100,class_114).
samebib(class_100,class_115).
samebib(class_101,class_95).
samebib(class_101,class_96).
samebib(class_101,class_97).
samebib(class_101,class_98).
samebib(class_101,class_99).
samebib(class_101,class_100).
samebib(class_101,class_101).
samebib(class_101,class_102).
samebib(class_101,class_103).
samebib(class_101,class_104).
samebib(class_101,class_105).
samebib(class_101,class_106).
samebib(class_101,class_107).
samebib(class_101,class_108).
samebib(class_101,class_109).
samebib(class_101,class_110).
samebib(class_101,class_111).
samebib(class_101,class_112).
samebib(class_101,class_113).
samebib(class_101,class_114).
samebib(class_101,class_115).
samebib(class_102,class_95).
samebib(class_102,class_96).
samebib(class_102,class_97).
samebib(class_102,class_98).
samebib(class_102,class_99).
samebib(class_102,class_100).
samebib(class_102,class_101).
samebib(class_102,class_102).
samebib(class_102,class_103).
samebib(class_102,class_104).
samebib(class_102,class_105).
samebib(class_102,class_106).
samebib(class_102,class_107).
samebib(class_102,class_108).
samebib(class_102,class_109).
samebib(class_102,class_110).
samebib(class_102,class_111).
samebib(class_102,class_112).
samebib(class_102,class_113).
samebib(class_102,class_114).
samebib(class_102,class_115).
samebib(class_103,class_95).
samebib(class_103,class_96).
samebib(class_103,class_97).
samebib(class_103,class_98).
samebib(class_103,class_99).
samebib(class_103,class_100).
samebib(class_103,class_101).
samebib(class_103,class_102).
samebib(class_103,class_103).
samebib(class_103,class_104).
samebib(class_103,class_105).
samebib(class_103,class_106).
samebib(class_103,class_107).
samebib(class_103,class_108).
samebib(class_103,class_109).
samebib(class_103,class_110).
samebib(class_103,class_111).
samebib(class_103,class_112).
samebib(class_103,class_113).
samebib(class_103,class_114).
samebib(class_103,class_115).
samebib(class_104,class_95).
samebib(class_104,class_96).
samebib(class_104,class_97).
samebib(class_104,class_98).
samebib(class_104,class_99).
samebib(class_104,class_100).
samebib(class_104,class_101).
samebib(class_104,class_102).
samebib(class_104,class_103).
samebib(class_104,class_104).
samebib(class_104,class_105).
samebib(class_104,class_106).
samebib(class_104,class_107).
samebib(class_104,class_108).
samebib(class_104,class_109).
samebib(class_104,class_110).
samebib(class_104,class_111).
samebib(class_104,class_112).
samebib(class_104,class_113).
samebib(class_104,class_114).
samebib(class_104,class_115).
samebib(class_105,class_95).
samebib(class_105,class_96).
samebib(class_105,class_97).
samebib(class_105,class_98).
samebib(class_105,class_99).
samebib(class_105,class_100).
samebib(class_105,class_101).
samebib(class_105,class_102).
samebib(class_105,class_103).
samebib(class_105,class_104).
samebib(class_105,class_105).
samebib(class_105,class_106).
samebib(class_105,class_107).
samebib(class_105,class_108).
samebib(class_105,class_109).
samebib(class_105,class_110).
samebib(class_105,class_111).
samebib(class_105,class_112).
samebib(class_105,class_113).
samebib(class_105,class_114).
samebib(class_105,class_115).
samebib(class_106,class_95).
samebib(class_106,class_96).
samebib(class_106,class_97).
samebib(class_106,class_98).
samebib(class_106,class_99).
samebib(class_106,class_100).
samebib(class_106,class_101).
samebib(class_106,class_102).
samebib(class_106,class_103).
samebib(class_106,class_104).
samebib(class_106,class_105).
samebib(class_106,class_106).
samebib(class_106,class_107).
samebib(class_106,class_108).
samebib(class_106,class_109).
samebib(class_106,class_110).
samebib(class_106,class_111).
samebib(class_106,class_112).
samebib(class_106,class_113).
samebib(class_106,class_114).
samebib(class_106,class_115).
samebib(class_107,class_95).
samebib(class_107,class_96).
samebib(class_107,class_97).
samebib(class_107,class_98).
samebib(class_107,class_99).
samebib(class_107,class_100).
samebib(class_107,class_101).
samebib(class_107,class_102).
samebib(class_107,class_103).
samebib(class_107,class_104).
samebib(class_107,class_105).
samebib(class_107,class_106).
samebib(class_107,class_107).
samebib(class_107,class_108).
samebib(class_107,class_109).
samebib(class_107,class_110).
samebib(class_107,class_111).
samebib(class_107,class_112).
samebib(class_107,class_113).
samebib(class_107,class_114).
samebib(class_107,class_115).
samebib(class_108,class_95).
samebib(class_108,class_96).
samebib(class_108,class_97).
samebib(class_108,class_98).
samebib(class_108,class_99).
samebib(class_108,class_100).
samebib(class_108,class_101).
samebib(class_108,class_102).
samebib(class_108,class_103).
samebib(class_108,class_104).
samebib(class_108,class_105).
samebib(class_108,class_106).
samebib(class_108,class_107).
samebib(class_108,class_108).
samebib(class_108,class_109).
samebib(class_108,class_110).
samebib(class_108,class_111).
samebib(class_108,class_112).
samebib(class_108,class_113).
samebib(class_108,class_114).
samebib(class_108,class_115).
samebib(class_109,class_95).
samebib(class_109,class_96).
samebib(class_109,class_97).
samebib(class_109,class_98).
samebib(class_109,class_99).
samebib(class_109,class_100).
samebib(class_109,class_101).
samebib(class_109,class_102).
samebib(class_109,class_103).
samebib(class_109,class_104).
samebib(class_109,class_105).
samebib(class_109,class_106).
samebib(class_109,class_107).
samebib(class_109,class_108).
samebib(class_109,class_109).
samebib(class_109,class_110).
samebib(class_109,class_111).
samebib(class_109,class_112).
samebib(class_109,class_113).
samebib(class_109,class_114).
samebib(class_109,class_115).
samebib(class_110,class_95).
samebib(class_110,class_96).
samebib(class_110,class_97).
samebib(class_110,class_98).
samebib(class_110,class_99).
samebib(class_110,class_100).
samebib(class_110,class_101).
samebib(class_110,class_102).
samebib(class_110,class_103).
samebib(class_110,class_104).
samebib(class_110,class_105).
samebib(class_110,class_106).
samebib(class_110,class_107).
samebib(class_110,class_108).
samebib(class_110,class_109).
samebib(class_110,class_110).
samebib(class_110,class_111).
samebib(class_110,class_112).
samebib(class_110,class_113).
samebib(class_110,class_114).
samebib(class_110,class_115).
samebib(class_111,class_95).
samebib(class_111,class_96).
samebib(class_111,class_97).
samebib(class_111,class_98).
samebib(class_111,class_99).
samebib(class_111,class_100).
samebib(class_111,class_101).
samebib(class_111,class_102).
samebib(class_111,class_103).
samebib(class_111,class_104).
samebib(class_111,class_105).
samebib(class_111,class_106).
samebib(class_111,class_107).
samebib(class_111,class_108).
samebib(class_111,class_109).
samebib(class_111,class_110).
samebib(class_111,class_111).
samebib(class_111,class_112).
samebib(class_111,class_113).
samebib(class_111,class_114).
samebib(class_111,class_115).
samebib(class_112,class_95).
samebib(class_112,class_96).
samebib(class_112,class_97).
samebib(class_112,class_98).
samebib(class_112,class_99).
samebib(class_112,class_100).
samebib(class_112,class_101).
samebib(class_112,class_102).
samebib(class_112,class_103).
samebib(class_112,class_104).
samebib(class_112,class_105).
samebib(class_112,class_106).
samebib(class_112,class_107).
samebib(class_112,class_108).
samebib(class_112,class_109).
samebib(class_112,class_110).
samebib(class_112,class_111).
samebib(class_112,class_112).
samebib(class_112,class_113).
samebib(class_112,class_114).
samebib(class_112,class_115).
samebib(class_113,class_95).
samebib(class_113,class_96).
samebib(class_113,class_97).
samebib(class_113,class_98).
samebib(class_113,class_99).
samebib(class_113,class_100).
samebib(class_113,class_101).
samebib(class_113,class_102).
samebib(class_113,class_103).
samebib(class_113,class_104).
samebib(class_113,class_105).
samebib(class_113,class_106).
samebib(class_113,class_107).
samebib(class_113,class_108).
samebib(class_113,class_109).
samebib(class_113,class_110).
samebib(class_113,class_111).
samebib(class_113,class_112).
samebib(class_113,class_113).
samebib(class_113,class_114).
samebib(class_113,class_115).
samebib(class_114,class_95).
samebib(class_114,class_96).
samebib(class_114,class_97).
samebib(class_114,class_98).
samebib(class_114,class_99).
samebib(class_114,class_100).
samebib(class_114,class_101).
samebib(class_114,class_102).
samebib(class_114,class_103).
samebib(class_114,class_104).
samebib(class_114,class_105).
samebib(class_114,class_106).
samebib(class_114,class_107).
samebib(class_114,class_108).
samebib(class_114,class_109).
samebib(class_114,class_110).
samebib(class_114,class_111).
samebib(class_114,class_112).
samebib(class_114,class_113).
samebib(class_114,class_114).
samebib(class_114,class_115).
samebib(class_115,class_95).
samebib(class_115,class_96).
samebib(class_115,class_97).
samebib(class_115,class_98).
samebib(class_115,class_99).
samebib(class_115,class_100).
samebib(class_115,class_101).
samebib(class_115,class_102).
samebib(class_115,class_103).
samebib(class_115,class_104).
samebib(class_115,class_105).
samebib(class_115,class_106).
samebib(class_115,class_107).
samebib(class_115,class_108).
samebib(class_115,class_109).
samebib(class_115,class_110).
samebib(class_115,class_111).
samebib(class_115,class_112).
samebib(class_115,class_113).
samebib(class_115,class_114).
samebib(class_115,class_115).
samebib(class_138,class_138).
samebib(class_245,class_245).
samebib(class_245,class_246).
samebib(class_245,class_247).
samebib(class_245,class_248).
samebib(class_246,class_245).
samebib(class_246,class_246).
samebib(class_246,class_247).
samebib(class_246,class_248).
samebib(class_247,class_245).
samebib(class_247,class_246).
samebib(class_247,class_247).
samebib(class_247,class_248).
samebib(class_248,class_245).
samebib(class_248,class_246).
samebib(class_248,class_247).
samebib(class_248,class_248).
samebib(class_250,class_250).
neg(samebib(class_250,class_327)).
samebib(class_326,class_326).
samebib(class_326,class_327).
samebib(class_326,class_328).
samebib(class_326,class_329).
samebib(class_326,class_330).
samebib(class_326,class_331).
samebib(class_326,class_332).
neg(samebib(class_327,class_250)).
samebib(class_327,class_326).
samebib(class_327,class_327).
samebib(class_327,class_328).
samebib(class_327,class_329).
samebib(class_327,class_330).
samebib(class_327,class_331).
samebib(class_327,class_332).
samebib(class_328,class_326).
samebib(class_328,class_327).
samebib(class_328,class_328).
samebib(class_328,class_329).
samebib(class_328,class_330).
samebib(class_328,class_331).
samebib(class_328,class_332).
samebib(class_329,class_326).
samebib(class_329,class_327).
samebib(class_329,class_328).
samebib(class_329,class_329).
samebib(class_329,class_330).
samebib(class_329,class_331).
samebib(class_329,class_332).
samebib(class_330,class_326).
samebib(class_330,class_327).
samebib(class_330,class_328).
samebib(class_330,class_329).
samebib(class_330,class_330).
samebib(class_330,class_331).
samebib(class_330,class_332).
samebib(class_331,class_326).
samebib(class_331,class_327).
samebib(class_331,class_328).
samebib(class_331,class_329).
samebib(class_331,class_330).
samebib(class_331,class_331).
samebib(class_331,class_332).
samebib(class_332,class_326).
samebib(class_332,class_327).
samebib(class_332,class_328).
samebib(class_332,class_329).
samebib(class_332,class_330).
samebib(class_332,class_331).
samebib(class_332,class_332).
samebib(class_333,class_333).
samebib(class_333,class_334).
samebib(class_333,class_335).
samebib(class_333,class_336).
samebib(class_334,class_333).
samebib(class_334,class_334).
samebib(class_334,class_335).
samebib(class_334,class_336).
samebib(class_335,class_333).
samebib(class_335,class_334).
samebib(class_335,class_335).
samebib(class_335,class_336).
samebib(class_336,class_333).
samebib(class_336,class_334).
samebib(class_336,class_335).
samebib(class_336,class_336).
samebib(class_439,class_439).
samebib(class_439,class_440).
samebib(class_439,class_441).
samebib(class_439,class_442).
neg(samebib(class_439,class_450)).
neg(samebib(class_439,class_451)).
samebib(class_440,class_439).
samebib(class_440,class_440).
samebib(class_440,class_441).
samebib(class_440,class_442).
neg(samebib(class_440,class_450)).
neg(samebib(class_440,class_451)).
samebib(class_441,class_439).
samebib(class_441,class_440).
samebib(class_441,class_441).
samebib(class_441,class_442).
neg(samebib(class_441,class_450)).
neg(samebib(class_441,class_451)).
samebib(class_442,class_439).
samebib(class_442,class_440).
samebib(class_442,class_441).
samebib(class_442,class_442).
neg(samebib(class_442,class_450)).
neg(samebib(class_442,class_451)).
neg(samebib(class_450,class_439)).
neg(samebib(class_450,class_440)).
neg(samebib(class_450,class_441)).
neg(samebib(class_450,class_442)).
samebib(class_450,class_450).
samebib(class_450,class_451).
neg(samebib(class_451,class_439)).
neg(samebib(class_451,class_440)).
neg(samebib(class_451,class_441)).
neg(samebib(class_451,class_442)).
samebib(class_451,class_450).
samebib(class_451,class_451).
samebib(class_497,class_497).
samebib(class_497,class_498).
samebib(class_497,class_499).
samebib(class_498,class_497).
samebib(class_498,class_498).
samebib(class_498,class_499).
samebib(class_499,class_497).
samebib(class_499,class_498).
samebib(class_499,class_499).
samebib(class_551,class_551).
samebib(class_551,class_552).
samebib(class_551,class_553).
samebib(class_551,class_554).
samebib(class_551,class_555).
samebib(class_551,class_556).
samebib(class_551,class_557).
samebib(class_551,class_558).
samebib(class_551,class_559).
samebib(class_551,class_560).
samebib(class_551,class_561).
samebib(class_551,class_562).
samebib(class_551,class_563).
samebib(class_551,class_564).
samebib(class_551,class_565).
samebib(class_551,class_566).
samebib(class_551,class_567).
samebib(class_551,class_568).
samebib(class_551,class_569).
samebib(class_551,class_570).
samebib(class_551,class_571).
samebib(class_551,class_572).
neg(samebib(class_551,class_590)).
neg(samebib(class_551,class_592)).
samebib(class_552,class_551).
samebib(class_552,class_552).
samebib(class_552,class_553).
samebib(class_552,class_554).
samebib(class_552,class_555).
samebib(class_552,class_556).
samebib(class_552,class_557).
samebib(class_552,class_558).
samebib(class_552,class_559).
samebib(class_552,class_560).
samebib(class_552,class_561).
samebib(class_552,class_562).
samebib(class_552,class_563).
samebib(class_552,class_564).
samebib(class_552,class_565).
samebib(class_552,class_566).
samebib(class_552,class_567).
samebib(class_552,class_568).
samebib(class_552,class_569).
samebib(class_552,class_570).
samebib(class_552,class_571).
samebib(class_552,class_572).
neg(samebib(class_552,class_590)).
neg(samebib(class_552,class_592)).
samebib(class_553,class_551).
samebib(class_553,class_552).
samebib(class_553,class_553).
samebib(class_553,class_554).
samebib(class_553,class_555).
samebib(class_553,class_556).
samebib(class_553,class_557).
samebib(class_553,class_558).
samebib(class_553,class_559).
samebib(class_553,class_560).
samebib(class_553,class_561).
samebib(class_553,class_562).
samebib(class_553,class_563).
samebib(class_553,class_564).
samebib(class_553,class_565).
samebib(class_553,class_566).
samebib(class_553,class_567).
samebib(class_553,class_568).
samebib(class_553,class_569).
samebib(class_553,class_570).
samebib(class_553,class_571).
samebib(class_553,class_572).
neg(samebib(class_553,class_590)).
neg(samebib(class_553,class_592)).
samebib(class_554,class_551).
samebib(class_554,class_552).
samebib(class_554,class_553).
samebib(class_554,class_554).
samebib(class_554,class_555).
samebib(class_554,class_556).
samebib(class_554,class_557).
samebib(class_554,class_558).
samebib(class_554,class_559).
samebib(class_554,class_560).
samebib(class_554,class_561).
samebib(class_554,class_562).
samebib(class_554,class_563).
samebib(class_554,class_564).
samebib(class_554,class_565).
samebib(class_554,class_566).
samebib(class_554,class_567).
samebib(class_554,class_568).
samebib(class_554,class_569).
samebib(class_554,class_570).
samebib(class_554,class_571).
samebib(class_554,class_572).
neg(samebib(class_554,class_590)).
neg(samebib(class_554,class_592)).
samebib(class_555,class_551).
samebib(class_555,class_552).
samebib(class_555,class_553).
samebib(class_555,class_554).
samebib(class_555,class_555).
samebib(class_555,class_556).
samebib(class_555,class_557).
samebib(class_555,class_558).
samebib(class_555,class_559).
samebib(class_555,class_560).
samebib(class_555,class_561).
samebib(class_555,class_562).
samebib(class_555,class_563).
samebib(class_555,class_564).
samebib(class_555,class_565).
samebib(class_555,class_566).
samebib(class_555,class_567).
samebib(class_555,class_568).
samebib(class_555,class_569).
samebib(class_555,class_570).
samebib(class_555,class_571).
samebib(class_555,class_572).
neg(samebib(class_555,class_590)).
neg(samebib(class_555,class_592)).
samebib(class_556,class_551).
samebib(class_556,class_552).
samebib(class_556,class_553).
samebib(class_556,class_554).
samebib(class_556,class_555).
samebib(class_556,class_556).
samebib(class_556,class_557).
samebib(class_556,class_558).
samebib(class_556,class_559).
samebib(class_556,class_560).
samebib(class_556,class_561).
samebib(class_556,class_562).
samebib(class_556,class_563).
samebib(class_556,class_564).
samebib(class_556,class_565).
samebib(class_556,class_566).
samebib(class_556,class_567).
samebib(class_556,class_568).
samebib(class_556,class_569).
samebib(class_556,class_570).
samebib(class_556,class_571).
samebib(class_556,class_572).
neg(samebib(class_556,class_590)).
neg(samebib(class_556,class_591)).
neg(samebib(class_556,class_592)).
samebib(class_557,class_551).
samebib(class_557,class_552).
samebib(class_557,class_553).
samebib(class_557,class_554).
samebib(class_557,class_555).
samebib(class_557,class_556).
samebib(class_557,class_557).
samebib(class_557,class_558).
samebib(class_557,class_559).
samebib(class_557,class_560).
samebib(class_557,class_561).
samebib(class_557,class_562).
samebib(class_557,class_563).
samebib(class_557,class_564).
samebib(class_557,class_565).
samebib(class_557,class_566).
samebib(class_557,class_567).
samebib(class_557,class_568).
samebib(class_557,class_569).
samebib(class_557,class_570).
samebib(class_557,class_571).
samebib(class_557,class_572).
neg(samebib(class_557,class_590)).
neg(samebib(class_557,class_591)).
neg(samebib(class_557,class_592)).
samebib(class_558,class_551).
samebib(class_558,class_552).
samebib(class_558,class_553).
samebib(class_558,class_554).
samebib(class_558,class_555).
samebib(class_558,class_556).
samebib(class_558,class_557).
samebib(class_558,class_558).
samebib(class_558,class_559).
samebib(class_558,class_560).
samebib(class_558,class_561).
samebib(class_558,class_562).
samebib(class_558,class_563).
samebib(class_558,class_564).
samebib(class_558,class_565).
samebib(class_558,class_566).
samebib(class_558,class_567).
samebib(class_558,class_568).
samebib(class_558,class_569).
samebib(class_558,class_570).
samebib(class_558,class_571).
samebib(class_558,class_572).
neg(samebib(class_558,class_590)).
neg(samebib(class_558,class_591)).
neg(samebib(class_558,class_592)).
samebib(class_559,class_551).
samebib(class_559,class_552).
samebib(class_559,class_553).
samebib(class_559,class_554).
samebib(class_559,class_555).
samebib(class_559,class_556).
samebib(class_559,class_557).
samebib(class_559,class_558).
samebib(class_559,class_559).
samebib(class_559,class_560).
samebib(class_559,class_561).
samebib(class_559,class_562).
samebib(class_559,class_563).
samebib(class_559,class_564).
samebib(class_559,class_565).
samebib(class_559,class_566).
samebib(class_559,class_567).
samebib(class_559,class_568).
samebib(class_559,class_569).
samebib(class_559,class_570).
samebib(class_559,class_571).
samebib(class_559,class_572).
neg(samebib(class_559,class_590)).
neg(samebib(class_559,class_591)).
neg(samebib(class_559,class_592)).
samebib(class_560,class_551).
samebib(class_560,class_552).
samebib(class_560,class_553).
samebib(class_560,class_554).
samebib(class_560,class_555).
samebib(class_560,class_556).
samebib(class_560,class_557).
samebib(class_560,class_558).
samebib(class_560,class_559).
samebib(class_560,class_560).
samebib(class_560,class_561).
samebib(class_560,class_562).
samebib(class_560,class_563).
samebib(class_560,class_564).
samebib(class_560,class_565).
samebib(class_560,class_566).
samebib(class_560,class_567).
samebib(class_560,class_568).
samebib(class_560,class_569).
samebib(class_560,class_570).
samebib(class_560,class_571).
samebib(class_560,class_572).
neg(samebib(class_560,class_590)).
neg(samebib(class_560,class_591)).
neg(samebib(class_560,class_592)).
samebib(class_561,class_551).
samebib(class_561,class_552).
samebib(class_561,class_553).
samebib(class_561,class_554).
samebib(class_561,class_555).
samebib(class_561,class_556).
samebib(class_561,class_557).
samebib(class_561,class_558).
samebib(class_561,class_559).
samebib(class_561,class_560).
samebib(class_561,class_561).
samebib(class_561,class_562).
samebib(class_561,class_563).
samebib(class_561,class_564).
samebib(class_561,class_565).
samebib(class_561,class_566).
samebib(class_561,class_567).
samebib(class_561,class_568).
samebib(class_561,class_569).
samebib(class_561,class_570).
samebib(class_561,class_571).
samebib(class_561,class_572).
neg(samebib(class_561,class_590)).
neg(samebib(class_561,class_591)).
neg(samebib(class_561,class_592)).
samebib(class_562,class_551).
samebib(class_562,class_552).
samebib(class_562,class_553).
samebib(class_562,class_554).
samebib(class_562,class_555).
samebib(class_562,class_556).
samebib(class_562,class_557).
samebib(class_562,class_558).
samebib(class_562,class_559).
samebib(class_562,class_560).
samebib(class_562,class_561).
samebib(class_562,class_562).
samebib(class_562,class_563).
samebib(class_562,class_564).
samebib(class_562,class_565).
samebib(class_562,class_566).
samebib(class_562,class_567).
samebib(class_562,class_568).
samebib(class_562,class_569).
samebib(class_562,class_570).
samebib(class_562,class_571).
samebib(class_562,class_572).
neg(samebib(class_562,class_590)).
neg(samebib(class_562,class_591)).
neg(samebib(class_562,class_592)).
samebib(class_563,class_551).
samebib(class_563,class_552).
samebib(class_563,class_553).
samebib(class_563,class_554).
samebib(class_563,class_555).
samebib(class_563,class_556).
samebib(class_563,class_557).
samebib(class_563,class_558).
samebib(class_563,class_559).
samebib(class_563,class_560).
samebib(class_563,class_561).
samebib(class_563,class_562).
samebib(class_563,class_563).
samebib(class_563,class_564).
samebib(class_563,class_565).
samebib(class_563,class_566).
samebib(class_563,class_567).
samebib(class_563,class_568).
samebib(class_563,class_569).
samebib(class_563,class_570).
samebib(class_563,class_571).
samebib(class_563,class_572).
neg(samebib(class_563,class_590)).
neg(samebib(class_563,class_591)).
neg(samebib(class_563,class_592)).
samebib(class_564,class_551).
samebib(class_564,class_552).
samebib(class_564,class_553).
samebib(class_564,class_554).
samebib(class_564,class_555).
samebib(class_564,class_556).
samebib(class_564,class_557).
samebib(class_564,class_558).
samebib(class_564,class_559).
samebib(class_564,class_560).
samebib(class_564,class_561).
samebib(class_564,class_562).
samebib(class_564,class_563).
samebib(class_564,class_564).
samebib(class_564,class_565).
samebib(class_564,class_566).
samebib(class_564,class_567).
samebib(class_564,class_568).
samebib(class_564,class_569).
samebib(class_564,class_570).
samebib(class_564,class_571).
samebib(class_564,class_572).
neg(samebib(class_564,class_590)).
neg(samebib(class_564,class_591)).
neg(samebib(class_564,class_592)).
samebib(class_565,class_551).
samebib(class_565,class_552).
samebib(class_565,class_553).
samebib(class_565,class_554).
samebib(class_565,class_555).
samebib(class_565,class_556).
samebib(class_565,class_557).
samebib(class_565,class_558).
samebib(class_565,class_559).
samebib(class_565,class_560).
samebib(class_565,class_561).
samebib(class_565,class_562).
samebib(class_565,class_563).
samebib(class_565,class_564).
samebib(class_565,class_565).
samebib(class_565,class_566).
samebib(class_565,class_567).
samebib(class_565,class_568).
samebib(class_565,class_569).
samebib(class_565,class_570).
samebib(class_565,class_571).
samebib(class_565,class_572).
neg(samebib(class_565,class_590)).
neg(samebib(class_565,class_591)).
neg(samebib(class_565,class_592)).
samebib(class_566,class_551).
samebib(class_566,class_552).
samebib(class_566,class_553).
samebib(class_566,class_554).
samebib(class_566,class_555).
samebib(class_566,class_556).
samebib(class_566,class_557).
samebib(class_566,class_558).
samebib(class_566,class_559).
samebib(class_566,class_560).
samebib(class_566,class_561).
samebib(class_566,class_562).
samebib(class_566,class_563).
samebib(class_566,class_564).
samebib(class_566,class_565).
samebib(class_566,class_566).
samebib(class_566,class_567).
samebib(class_566,class_568).
samebib(class_566,class_569).
samebib(class_566,class_570).
samebib(class_566,class_571).
samebib(class_566,class_572).
neg(samebib(class_566,class_590)).
neg(samebib(class_566,class_591)).
neg(samebib(class_566,class_592)).
samebib(class_567,class_551).
samebib(class_567,class_552).
samebib(class_567,class_553).
samebib(class_567,class_554).
samebib(class_567,class_555).
samebib(class_567,class_556).
samebib(class_567,class_557).
samebib(class_567,class_558).
samebib(class_567,class_559).
samebib(class_567,class_560).
samebib(class_567,class_561).
samebib(class_567,class_562).
samebib(class_567,class_563).
samebib(class_567,class_564).
samebib(class_567,class_565).
samebib(class_567,class_566).
samebib(class_567,class_567).
samebib(class_567,class_568).
samebib(class_567,class_569).
samebib(class_567,class_570).
samebib(class_567,class_571).
samebib(class_567,class_572).
neg(samebib(class_567,class_590)).
neg(samebib(class_567,class_591)).
neg(samebib(class_567,class_592)).
samebib(class_568,class_551).
samebib(class_568,class_552).
samebib(class_568,class_553).
samebib(class_568,class_554).
samebib(class_568,class_555).
samebib(class_568,class_556).
samebib(class_568,class_557).
samebib(class_568,class_558).
samebib(class_568,class_559).
samebib(class_568,class_560).
samebib(class_568,class_561).
samebib(class_568,class_562).
samebib(class_568,class_563).
samebib(class_568,class_564).
samebib(class_568,class_565).
samebib(class_568,class_566).
samebib(class_568,class_567).
samebib(class_568,class_568).
samebib(class_568,class_569).
samebib(class_568,class_570).
samebib(class_568,class_571).
samebib(class_568,class_572).
neg(samebib(class_568,class_590)).
neg(samebib(class_568,class_591)).
neg(samebib(class_568,class_592)).
samebib(class_569,class_551).
samebib(class_569,class_552).
samebib(class_569,class_553).
samebib(class_569,class_554).
samebib(class_569,class_555).
samebib(class_569,class_556).
samebib(class_569,class_557).
samebib(class_569,class_558).
samebib(class_569,class_559).
samebib(class_569,class_560).
samebib(class_569,class_561).
samebib(class_569,class_562).
samebib(class_569,class_563).
samebib(class_569,class_564).
samebib(class_569,class_565).
samebib(class_569,class_566).
samebib(class_569,class_567).
samebib(class_569,class_568).
samebib(class_569,class_569).
samebib(class_569,class_570).
samebib(class_569,class_571).
samebib(class_569,class_572).
neg(samebib(class_569,class_590)).
neg(samebib(class_569,class_591)).
neg(samebib(class_569,class_592)).
samebib(class_570,class_551).
samebib(class_570,class_552).
samebib(class_570,class_553).
samebib(class_570,class_554).
samebib(class_570,class_555).
samebib(class_570,class_556).
samebib(class_570,class_557).
samebib(class_570,class_558).
samebib(class_570,class_559).
samebib(class_570,class_560).
samebib(class_570,class_561).
samebib(class_570,class_562).
samebib(class_570,class_563).
samebib(class_570,class_564).
samebib(class_570,class_565).
samebib(class_570,class_566).
samebib(class_570,class_567).
samebib(class_570,class_568).
samebib(class_570,class_569).
samebib(class_570,class_570).
samebib(class_570,class_571).
samebib(class_570,class_572).
neg(samebib(class_570,class_590)).
neg(samebib(class_570,class_591)).
neg(samebib(class_570,class_592)).
samebib(class_571,class_551).
samebib(class_571,class_552).
samebib(class_571,class_553).
samebib(class_571,class_554).
samebib(class_571,class_555).
samebib(class_571,class_556).
samebib(class_571,class_557).
samebib(class_571,class_558).
samebib(class_571,class_559).
samebib(class_571,class_560).
samebib(class_571,class_561).
samebib(class_571,class_562).
samebib(class_571,class_563).
samebib(class_571,class_564).
samebib(class_571,class_565).
samebib(class_571,class_566).
samebib(class_571,class_567).
samebib(class_571,class_568).
samebib(class_571,class_569).
samebib(class_571,class_570).
samebib(class_571,class_571).
samebib(class_571,class_572).
neg(samebib(class_571,class_590)).
neg(samebib(class_571,class_591)).
neg(samebib(class_571,class_592)).
samebib(class_572,class_551).
samebib(class_572,class_552).
samebib(class_572,class_553).
samebib(class_572,class_554).
samebib(class_572,class_555).
samebib(class_572,class_556).
samebib(class_572,class_557).
samebib(class_572,class_558).
samebib(class_572,class_559).
samebib(class_572,class_560).
samebib(class_572,class_561).
samebib(class_572,class_562).
samebib(class_572,class_563).
samebib(class_572,class_564).
samebib(class_572,class_565).
samebib(class_572,class_566).
samebib(class_572,class_567).
samebib(class_572,class_568).
samebib(class_572,class_569).
samebib(class_572,class_570).
samebib(class_572,class_571).
samebib(class_572,class_572).
neg(samebib(class_572,class_590)).
neg(samebib(class_572,class_591)).
neg(samebib(class_572,class_592)).
neg(samebib(class_590,class_551)).
neg(samebib(class_590,class_552)).
neg(samebib(class_590,class_553)).
neg(samebib(class_590,class_554)).
neg(samebib(class_590,class_555)).
neg(samebib(class_590,class_556)).
neg(samebib(class_590,class_557)).
neg(samebib(class_590,class_558)).
neg(samebib(class_590,class_559)).
neg(samebib(class_590,class_560)).
neg(samebib(class_590,class_561)).
neg(samebib(class_590,class_562)).
neg(samebib(class_590,class_563)).
neg(samebib(class_590,class_564)).
neg(samebib(class_590,class_565)).
neg(samebib(class_590,class_566)).
neg(samebib(class_590,class_567)).
neg(samebib(class_590,class_568)).
neg(samebib(class_590,class_569)).
neg(samebib(class_590,class_570)).
neg(samebib(class_590,class_571)).
neg(samebib(class_590,class_572)).
samebib(class_590,class_590).
samebib(class_590,class_591).
samebib(class_590,class_592).
neg(samebib(class_591,class_556)).
neg(samebib(class_591,class_557)).
neg(samebib(class_591,class_558)).
neg(samebib(class_591,class_559)).
neg(samebib(class_591,class_560)).
neg(samebib(class_591,class_561)).
neg(samebib(class_591,class_562)).
neg(samebib(class_591,class_563)).
neg(samebib(class_591,class_564)).
neg(samebib(class_591,class_565)).
neg(samebib(class_591,class_566)).
neg(samebib(class_591,class_567)).
neg(samebib(class_591,class_568)).
neg(samebib(class_591,class_569)).
neg(samebib(class_591,class_570)).
neg(samebib(class_591,class_571)).
neg(samebib(class_591,class_572)).
samebib(class_591,class_590).
samebib(class_591,class_591).
samebib(class_591,class_592).
neg(samebib(class_592,class_551)).
neg(samebib(class_592,class_552)).
neg(samebib(class_592,class_553)).
neg(samebib(class_592,class_554)).
neg(samebib(class_592,class_555)).
neg(samebib(class_592,class_556)).
neg(samebib(class_592,class_557)).
neg(samebib(class_592,class_558)).
neg(samebib(class_592,class_559)).
neg(samebib(class_592,class_560)).
neg(samebib(class_592,class_561)).
neg(samebib(class_592,class_562)).
neg(samebib(class_592,class_563)).
neg(samebib(class_592,class_564)).
neg(samebib(class_592,class_565)).
neg(samebib(class_592,class_566)).
neg(samebib(class_592,class_567)).
neg(samebib(class_592,class_568)).
neg(samebib(class_592,class_569)).
neg(samebib(class_592,class_570)).
neg(samebib(class_592,class_571)).
neg(samebib(class_592,class_572)).
samebib(class_592,class_590).
samebib(class_592,class_591).
samebib(class_592,class_592).
samebib(class_629,class_629).
samebib(class_783,class_783).
samebib(class_910,class_910).
samebib(class_911,class_911).
samebib(class_911,class_912).
samebib(class_911,class_913).
samebib(class_911,class_914).
samebib(class_911,class_917).
samebib(class_911,class_918).
samebib(class_911,class_919).
samebib(class_911,class_920).
samebib(class_911,class_921).
samebib(class_911,class_922).
samebib(class_911,class_923).
samebib(class_911,class_924).
samebib(class_911,class_925).
samebib(class_911,class_926).
samebib(class_911,class_927).
samebib(class_911,class_928).
samebib(class_911,class_929).
neg(samebib(class_911,class_930)).
neg(samebib(class_911,class_931)).
neg(samebib(class_911,class_932)).
neg(samebib(class_911,class_933)).
neg(samebib(class_911,class_934)).
neg(samebib(class_911,class_935)).
neg(samebib(class_911,class_936)).
neg(samebib(class_911,class_937)).
neg(samebib(class_911,class_938)).
neg(samebib(class_911,class_939)).
neg(samebib(class_911,class_940)).
neg(samebib(class_911,class_941)).
neg(samebib(class_911,class_942)).
neg(samebib(class_911,class_943)).
neg(samebib(class_911,class_944)).
neg(samebib(class_911,class_945)).
neg(samebib(class_911,class_946)).
neg(samebib(class_911,class_947)).
neg(samebib(class_911,class_948)).
neg(samebib(class_911,class_949)).
neg(samebib(class_911,class_950)).
neg(samebib(class_911,class_951)).
neg(samebib(class_911,class_952)).
neg(samebib(class_911,class_953)).
neg(samebib(class_911,class_954)).
neg(samebib(class_911,class_955)).
neg(samebib(class_911,class_956)).
neg(samebib(class_911,class_958)).
neg(samebib(class_911,class_959)).
neg(samebib(class_911,class_960)).
neg(samebib(class_911,class_961)).
neg(samebib(class_911,class_962)).
neg(samebib(class_911,class_963)).
neg(samebib(class_911,class_964)).
neg(samebib(class_911,class_965)).
neg(samebib(class_911,class_966)).
neg(samebib(class_911,class_967)).
neg(samebib(class_911,class_968)).
neg(samebib(class_911,class_969)).
neg(samebib(class_911,class_970)).
neg(samebib(class_911,class_972)).
neg(samebib(class_911,class_1044)).
neg(samebib(class_911,class_1047)).
samebib(class_912,class_911).
samebib(class_912,class_912).
samebib(class_912,class_913).
samebib(class_912,class_914).
samebib(class_912,class_917).
samebib(class_912,class_918).
samebib(class_912,class_919).
samebib(class_912,class_920).
samebib(class_912,class_921).
samebib(class_912,class_922).
samebib(class_912,class_923).
samebib(class_912,class_924).
samebib(class_912,class_925).
samebib(class_912,class_926).
samebib(class_912,class_927).
samebib(class_912,class_928).
samebib(class_912,class_929).
neg(samebib(class_912,class_930)).
neg(samebib(class_912,class_931)).
neg(samebib(class_912,class_932)).
neg(samebib(class_912,class_933)).
neg(samebib(class_912,class_934)).
neg(samebib(class_912,class_935)).
neg(samebib(class_912,class_936)).
neg(samebib(class_912,class_937)).
neg(samebib(class_912,class_938)).
neg(samebib(class_912,class_939)).
neg(samebib(class_912,class_940)).
neg(samebib(class_912,class_941)).
neg(samebib(class_912,class_942)).
neg(samebib(class_912,class_943)).
neg(samebib(class_912,class_944)).
neg(samebib(class_912,class_945)).
neg(samebib(class_912,class_946)).
neg(samebib(class_912,class_947)).
neg(samebib(class_912,class_948)).
neg(samebib(class_912,class_949)).
neg(samebib(class_912,class_950)).
neg(samebib(class_912,class_951)).
neg(samebib(class_912,class_952)).
neg(samebib(class_912,class_953)).
neg(samebib(class_912,class_954)).
neg(samebib(class_912,class_955)).
neg(samebib(class_912,class_956)).
neg(samebib(class_912,class_958)).
neg(samebib(class_912,class_959)).
neg(samebib(class_912,class_960)).
neg(samebib(class_912,class_961)).
neg(samebib(class_912,class_962)).
neg(samebib(class_912,class_963)).
neg(samebib(class_912,class_964)).
neg(samebib(class_912,class_965)).
neg(samebib(class_912,class_966)).
neg(samebib(class_912,class_967)).
neg(samebib(class_912,class_968)).
neg(samebib(class_912,class_969)).
neg(samebib(class_912,class_970)).
neg(samebib(class_912,class_972)).
neg(samebib(class_912,class_1044)).
neg(samebib(class_912,class_1047)).
samebib(class_913,class_911).
samebib(class_913,class_912).
samebib(class_913,class_913).
samebib(class_913,class_914).
samebib(class_913,class_917).
samebib(class_913,class_918).
samebib(class_913,class_919).
samebib(class_913,class_920).
samebib(class_913,class_921).
samebib(class_913,class_922).
samebib(class_913,class_923).
samebib(class_913,class_924).
samebib(class_913,class_925).
samebib(class_913,class_926).
samebib(class_913,class_927).
samebib(class_913,class_928).
samebib(class_913,class_929).
neg(samebib(class_913,class_930)).
neg(samebib(class_913,class_931)).
neg(samebib(class_913,class_932)).
neg(samebib(class_913,class_933)).
neg(samebib(class_913,class_934)).
neg(samebib(class_913,class_935)).
neg(samebib(class_913,class_936)).
neg(samebib(class_913,class_937)).
neg(samebib(class_913,class_938)).
neg(samebib(class_913,class_939)).
neg(samebib(class_913,class_940)).
neg(samebib(class_913,class_941)).
neg(samebib(class_913,class_942)).
neg(samebib(class_913,class_943)).
neg(samebib(class_913,class_944)).
neg(samebib(class_913,class_945)).
neg(samebib(class_913,class_946)).
neg(samebib(class_913,class_947)).
neg(samebib(class_913,class_948)).
neg(samebib(class_913,class_949)).
neg(samebib(class_913,class_950)).
neg(samebib(class_913,class_951)).
neg(samebib(class_913,class_952)).
neg(samebib(class_913,class_953)).
neg(samebib(class_913,class_954)).
neg(samebib(class_913,class_955)).
neg(samebib(class_913,class_956)).
neg(samebib(class_913,class_958)).
neg(samebib(class_913,class_959)).
neg(samebib(class_913,class_960)).
neg(samebib(class_913,class_961)).
neg(samebib(class_913,class_962)).
neg(samebib(class_913,class_963)).
neg(samebib(class_913,class_964)).
neg(samebib(class_913,class_965)).
neg(samebib(class_913,class_966)).
neg(samebib(class_913,class_967)).
neg(samebib(class_913,class_968)).
neg(samebib(class_913,class_969)).
neg(samebib(class_913,class_970)).
neg(samebib(class_913,class_972)).
neg(samebib(class_913,class_1044)).
neg(samebib(class_913,class_1047)).
samebib(class_914,class_911).
samebib(class_914,class_912).
samebib(class_914,class_913).
samebib(class_914,class_914).
samebib(class_914,class_917).
samebib(class_914,class_918).
samebib(class_914,class_919).
samebib(class_914,class_920).
samebib(class_914,class_921).
samebib(class_914,class_922).
samebib(class_914,class_923).
samebib(class_914,class_924).
samebib(class_914,class_925).
samebib(class_914,class_926).
samebib(class_914,class_927).
samebib(class_914,class_928).
samebib(class_914,class_929).
neg(samebib(class_914,class_930)).
neg(samebib(class_914,class_931)).
neg(samebib(class_914,class_932)).
neg(samebib(class_914,class_933)).
neg(samebib(class_914,class_934)).
neg(samebib(class_914,class_935)).
neg(samebib(class_914,class_936)).
neg(samebib(class_914,class_937)).
neg(samebib(class_914,class_938)).
neg(samebib(class_914,class_939)).
neg(samebib(class_914,class_940)).
neg(samebib(class_914,class_941)).
neg(samebib(class_914,class_942)).
neg(samebib(class_914,class_943)).
neg(samebib(class_914,class_944)).
neg(samebib(class_914,class_945)).
neg(samebib(class_914,class_946)).
neg(samebib(class_914,class_947)).
neg(samebib(class_914,class_948)).
neg(samebib(class_914,class_949)).
neg(samebib(class_914,class_950)).
neg(samebib(class_914,class_951)).
neg(samebib(class_914,class_952)).
neg(samebib(class_914,class_953)).
neg(samebib(class_914,class_954)).
neg(samebib(class_914,class_955)).
neg(samebib(class_914,class_956)).
neg(samebib(class_914,class_958)).
neg(samebib(class_914,class_959)).
neg(samebib(class_914,class_960)).
neg(samebib(class_914,class_961)).
neg(samebib(class_914,class_962)).
neg(samebib(class_914,class_963)).
neg(samebib(class_914,class_964)).
neg(samebib(class_914,class_965)).
neg(samebib(class_914,class_966)).
neg(samebib(class_914,class_967)).
neg(samebib(class_914,class_968)).
neg(samebib(class_914,class_969)).
neg(samebib(class_914,class_970)).
neg(samebib(class_914,class_972)).
neg(samebib(class_914,class_1044)).
neg(samebib(class_914,class_1047)).
samebib(class_915,class_915).
samebib(class_915,class_916).
samebib(class_915,class_923).
samebib(class_915,class_924).
samebib(class_915,class_925).
samebib(class_916,class_915).
samebib(class_916,class_916).
samebib(class_916,class_923).
samebib(class_916,class_924).
samebib(class_916,class_925).
samebib(class_917,class_911).
samebib(class_917,class_912).
samebib(class_917,class_913).
samebib(class_917,class_914).
samebib(class_917,class_917).
samebib(class_917,class_918).
samebib(class_917,class_919).
samebib(class_917,class_920).
samebib(class_917,class_921).
samebib(class_917,class_922).
samebib(class_917,class_923).
samebib(class_917,class_924).
samebib(class_917,class_925).
samebib(class_917,class_926).
samebib(class_917,class_927).
samebib(class_917,class_928).
samebib(class_917,class_929).
neg(samebib(class_917,class_930)).
neg(samebib(class_917,class_931)).
neg(samebib(class_917,class_932)).
neg(samebib(class_917,class_933)).
neg(samebib(class_917,class_934)).
neg(samebib(class_917,class_935)).
neg(samebib(class_917,class_936)).
neg(samebib(class_917,class_937)).
neg(samebib(class_917,class_938)).
neg(samebib(class_917,class_939)).
neg(samebib(class_917,class_940)).
neg(samebib(class_917,class_941)).
neg(samebib(class_917,class_942)).
neg(samebib(class_917,class_943)).
neg(samebib(class_917,class_944)).
neg(samebib(class_917,class_945)).
neg(samebib(class_917,class_946)).
neg(samebib(class_917,class_947)).
neg(samebib(class_917,class_948)).
neg(samebib(class_917,class_949)).
neg(samebib(class_917,class_950)).
neg(samebib(class_917,class_951)).
neg(samebib(class_917,class_952)).
neg(samebib(class_917,class_953)).
neg(samebib(class_917,class_954)).
neg(samebib(class_917,class_955)).
neg(samebib(class_917,class_956)).
neg(samebib(class_917,class_958)).
neg(samebib(class_917,class_959)).
neg(samebib(class_917,class_960)).
neg(samebib(class_917,class_961)).
neg(samebib(class_917,class_962)).
neg(samebib(class_917,class_963)).
neg(samebib(class_917,class_964)).
neg(samebib(class_917,class_965)).
neg(samebib(class_917,class_966)).
neg(samebib(class_917,class_967)).
neg(samebib(class_917,class_968)).
neg(samebib(class_917,class_969)).
neg(samebib(class_917,class_970)).
neg(samebib(class_917,class_972)).
neg(samebib(class_917,class_1044)).
neg(samebib(class_917,class_1047)).
samebib(class_918,class_911).
samebib(class_918,class_912).
samebib(class_918,class_913).
samebib(class_918,class_914).
samebib(class_918,class_917).
samebib(class_918,class_918).
samebib(class_918,class_919).
samebib(class_918,class_920).
samebib(class_918,class_921).
samebib(class_918,class_922).
samebib(class_918,class_923).
samebib(class_918,class_924).
samebib(class_918,class_925).
samebib(class_918,class_926).
samebib(class_918,class_927).
samebib(class_918,class_928).
samebib(class_918,class_929).
neg(samebib(class_918,class_930)).
neg(samebib(class_918,class_931)).
neg(samebib(class_918,class_932)).
neg(samebib(class_918,class_933)).
neg(samebib(class_918,class_934)).
neg(samebib(class_918,class_935)).
neg(samebib(class_918,class_936)).
neg(samebib(class_918,class_937)).
neg(samebib(class_918,class_938)).
neg(samebib(class_918,class_939)).
neg(samebib(class_918,class_940)).
neg(samebib(class_918,class_941)).
neg(samebib(class_918,class_942)).
neg(samebib(class_918,class_943)).
neg(samebib(class_918,class_944)).
neg(samebib(class_918,class_945)).
neg(samebib(class_918,class_946)).
neg(samebib(class_918,class_947)).
neg(samebib(class_918,class_948)).
neg(samebib(class_918,class_949)).
neg(samebib(class_918,class_950)).
neg(samebib(class_918,class_951)).
neg(samebib(class_918,class_952)).
neg(samebib(class_918,class_953)).
neg(samebib(class_918,class_954)).
neg(samebib(class_918,class_955)).
neg(samebib(class_918,class_956)).
neg(samebib(class_918,class_958)).
neg(samebib(class_918,class_959)).
neg(samebib(class_918,class_960)).
neg(samebib(class_918,class_961)).
neg(samebib(class_918,class_962)).
neg(samebib(class_918,class_963)).
neg(samebib(class_918,class_964)).
neg(samebib(class_918,class_965)).
neg(samebib(class_918,class_966)).
neg(samebib(class_918,class_967)).
neg(samebib(class_918,class_968)).
neg(samebib(class_918,class_969)).
neg(samebib(class_918,class_970)).
neg(samebib(class_918,class_972)).
neg(samebib(class_918,class_1044)).
neg(samebib(class_918,class_1047)).
samebib(class_919,class_911).
samebib(class_919,class_912).
samebib(class_919,class_913).
samebib(class_919,class_914).
samebib(class_919,class_917).
samebib(class_919,class_918).
samebib(class_919,class_919).
samebib(class_919,class_920).
samebib(class_919,class_921).
samebib(class_919,class_922).
samebib(class_919,class_923).
samebib(class_919,class_924).
samebib(class_919,class_925).
samebib(class_919,class_926).
samebib(class_919,class_927).
samebib(class_919,class_928).
samebib(class_919,class_929).
neg(samebib(class_919,class_930)).
neg(samebib(class_919,class_931)).
neg(samebib(class_919,class_932)).
neg(samebib(class_919,class_933)).
neg(samebib(class_919,class_934)).
neg(samebib(class_919,class_935)).
neg(samebib(class_919,class_936)).
neg(samebib(class_919,class_937)).
neg(samebib(class_919,class_938)).
neg(samebib(class_919,class_939)).
neg(samebib(class_919,class_940)).
neg(samebib(class_919,class_941)).
neg(samebib(class_919,class_942)).
neg(samebib(class_919,class_943)).
neg(samebib(class_919,class_944)).
neg(samebib(class_919,class_945)).
neg(samebib(class_919,class_946)).
neg(samebib(class_919,class_947)).
neg(samebib(class_919,class_948)).
neg(samebib(class_919,class_949)).
neg(samebib(class_919,class_950)).
neg(samebib(class_919,class_951)).
neg(samebib(class_919,class_952)).
neg(samebib(class_919,class_953)).
neg(samebib(class_919,class_954)).
neg(samebib(class_919,class_955)).
neg(samebib(class_919,class_956)).
neg(samebib(class_919,class_958)).
neg(samebib(class_919,class_959)).
neg(samebib(class_919,class_960)).
neg(samebib(class_919,class_961)).
neg(samebib(class_919,class_962)).
neg(samebib(class_919,class_963)).
neg(samebib(class_919,class_964)).
neg(samebib(class_919,class_965)).
neg(samebib(class_919,class_966)).
neg(samebib(class_919,class_967)).
neg(samebib(class_919,class_968)).
neg(samebib(class_919,class_969)).
neg(samebib(class_919,class_970)).
neg(samebib(class_919,class_972)).
neg(samebib(class_919,class_1044)).
neg(samebib(class_919,class_1047)).
samebib(class_920,class_911).
samebib(class_920,class_912).
samebib(class_920,class_913).
samebib(class_920,class_914).
samebib(class_920,class_917).
samebib(class_920,class_918).
samebib(class_920,class_919).
samebib(class_920,class_920).
samebib(class_920,class_921).
samebib(class_920,class_922).
samebib(class_920,class_923).
samebib(class_920,class_924).
samebib(class_920,class_925).
samebib(class_920,class_926).
samebib(class_920,class_927).
samebib(class_920,class_928).
samebib(class_920,class_929).
neg(samebib(class_920,class_930)).
neg(samebib(class_920,class_931)).
neg(samebib(class_920,class_932)).
neg(samebib(class_920,class_933)).
neg(samebib(class_920,class_934)).
neg(samebib(class_920,class_935)).
neg(samebib(class_920,class_936)).
neg(samebib(class_920,class_937)).
neg(samebib(class_920,class_938)).
neg(samebib(class_920,class_939)).
neg(samebib(class_920,class_940)).
neg(samebib(class_920,class_941)).
neg(samebib(class_920,class_942)).
neg(samebib(class_920,class_943)).
neg(samebib(class_920,class_944)).
neg(samebib(class_920,class_945)).
neg(samebib(class_920,class_946)).
neg(samebib(class_920,class_947)).
neg(samebib(class_920,class_948)).
neg(samebib(class_920,class_949)).
neg(samebib(class_920,class_950)).
neg(samebib(class_920,class_951)).
neg(samebib(class_920,class_952)).
neg(samebib(class_920,class_953)).
neg(samebib(class_920,class_954)).
neg(samebib(class_920,class_955)).
neg(samebib(class_920,class_956)).
neg(samebib(class_920,class_958)).
neg(samebib(class_920,class_959)).
neg(samebib(class_920,class_960)).
neg(samebib(class_920,class_961)).
neg(samebib(class_920,class_962)).
neg(samebib(class_920,class_963)).
neg(samebib(class_920,class_964)).
neg(samebib(class_920,class_965)).
neg(samebib(class_920,class_966)).
neg(samebib(class_920,class_967)).
neg(samebib(class_920,class_968)).
neg(samebib(class_920,class_969)).
neg(samebib(class_920,class_970)).
neg(samebib(class_920,class_972)).
neg(samebib(class_920,class_1044)).
neg(samebib(class_920,class_1047)).
samebib(class_921,class_911).
samebib(class_921,class_912).
samebib(class_921,class_913).
samebib(class_921,class_914).
samebib(class_921,class_917).
samebib(class_921,class_918).
samebib(class_921,class_919).
samebib(class_921,class_920).
samebib(class_921,class_921).
samebib(class_921,class_922).
samebib(class_921,class_923).
samebib(class_921,class_924).
samebib(class_921,class_925).
samebib(class_921,class_926).
samebib(class_921,class_927).
samebib(class_921,class_928).
samebib(class_921,class_929).
neg(samebib(class_921,class_930)).
neg(samebib(class_921,class_931)).
neg(samebib(class_921,class_932)).
neg(samebib(class_921,class_933)).
neg(samebib(class_921,class_934)).
neg(samebib(class_921,class_935)).
neg(samebib(class_921,class_936)).
neg(samebib(class_921,class_937)).
neg(samebib(class_921,class_938)).
neg(samebib(class_921,class_939)).
neg(samebib(class_921,class_940)).
neg(samebib(class_921,class_941)).
neg(samebib(class_921,class_942)).
neg(samebib(class_921,class_943)).
neg(samebib(class_921,class_944)).
neg(samebib(class_921,class_945)).
neg(samebib(class_921,class_946)).
neg(samebib(class_921,class_947)).
neg(samebib(class_921,class_948)).
neg(samebib(class_921,class_949)).
neg(samebib(class_921,class_950)).
neg(samebib(class_921,class_951)).
neg(samebib(class_921,class_952)).
neg(samebib(class_921,class_953)).
neg(samebib(class_921,class_954)).
neg(samebib(class_921,class_955)).
neg(samebib(class_921,class_956)).
neg(samebib(class_921,class_958)).
neg(samebib(class_921,class_959)).
neg(samebib(class_921,class_960)).
neg(samebib(class_921,class_961)).
neg(samebib(class_921,class_962)).
neg(samebib(class_921,class_963)).
neg(samebib(class_921,class_964)).
neg(samebib(class_921,class_965)).
neg(samebib(class_921,class_966)).
neg(samebib(class_921,class_967)).
neg(samebib(class_921,class_968)).
neg(samebib(class_921,class_969)).
neg(samebib(class_921,class_970)).
neg(samebib(class_921,class_972)).
neg(samebib(class_921,class_1044)).
neg(samebib(class_921,class_1047)).
samebib(class_922,class_911).
samebib(class_922,class_912).
samebib(class_922,class_913).
samebib(class_922,class_914).
samebib(class_922,class_917).
samebib(class_922,class_918).
samebib(class_922,class_919).
samebib(class_922,class_920).
samebib(class_922,class_921).
samebib(class_922,class_922).
samebib(class_922,class_923).
samebib(class_922,class_924).
samebib(class_922,class_925).
samebib(class_922,class_926).
samebib(class_922,class_927).
samebib(class_922,class_928).
samebib(class_922,class_929).
neg(samebib(class_922,class_930)).
neg(samebib(class_922,class_931)).
neg(samebib(class_922,class_932)).
neg(samebib(class_922,class_933)).
neg(samebib(class_922,class_934)).
neg(samebib(class_922,class_935)).
neg(samebib(class_922,class_936)).
neg(samebib(class_922,class_937)).
neg(samebib(class_922,class_938)).
neg(samebib(class_922,class_939)).
neg(samebib(class_922,class_940)).
neg(samebib(class_922,class_941)).
neg(samebib(class_922,class_942)).
neg(samebib(class_922,class_943)).
neg(samebib(class_922,class_944)).
neg(samebib(class_922,class_945)).
neg(samebib(class_922,class_946)).
neg(samebib(class_922,class_947)).
neg(samebib(class_922,class_948)).
neg(samebib(class_922,class_949)).
neg(samebib(class_922,class_950)).
neg(samebib(class_922,class_951)).
neg(samebib(class_922,class_952)).
neg(samebib(class_922,class_953)).
neg(samebib(class_922,class_954)).
neg(samebib(class_922,class_955)).
neg(samebib(class_922,class_956)).
neg(samebib(class_922,class_958)).
neg(samebib(class_922,class_959)).
neg(samebib(class_922,class_960)).
neg(samebib(class_922,class_961)).
neg(samebib(class_922,class_962)).
neg(samebib(class_922,class_963)).
neg(samebib(class_922,class_964)).
neg(samebib(class_922,class_965)).
neg(samebib(class_922,class_966)).
neg(samebib(class_922,class_967)).
neg(samebib(class_922,class_968)).
neg(samebib(class_922,class_969)).
neg(samebib(class_922,class_970)).
neg(samebib(class_922,class_972)).
neg(samebib(class_922,class_1044)).
neg(samebib(class_922,class_1047)).
samebib(class_923,class_911).
samebib(class_923,class_912).
samebib(class_923,class_913).
samebib(class_923,class_914).
samebib(class_923,class_915).
samebib(class_923,class_916).
samebib(class_923,class_917).
samebib(class_923,class_918).
samebib(class_923,class_919).
samebib(class_923,class_920).
samebib(class_923,class_921).
samebib(class_923,class_922).
samebib(class_923,class_923).
samebib(class_923,class_924).
samebib(class_923,class_925).
samebib(class_923,class_926).
samebib(class_923,class_927).
samebib(class_923,class_928).
samebib(class_923,class_929).
neg(samebib(class_923,class_930)).
neg(samebib(class_923,class_931)).
neg(samebib(class_923,class_932)).
neg(samebib(class_923,class_933)).
neg(samebib(class_923,class_934)).
neg(samebib(class_923,class_935)).
neg(samebib(class_923,class_936)).
neg(samebib(class_923,class_937)).
neg(samebib(class_923,class_938)).
neg(samebib(class_923,class_939)).
neg(samebib(class_923,class_940)).
neg(samebib(class_923,class_941)).
neg(samebib(class_923,class_942)).
neg(samebib(class_923,class_943)).
neg(samebib(class_923,class_944)).
neg(samebib(class_923,class_945)).
neg(samebib(class_923,class_946)).
neg(samebib(class_923,class_947)).
neg(samebib(class_923,class_948)).
neg(samebib(class_923,class_949)).
neg(samebib(class_923,class_950)).
neg(samebib(class_923,class_951)).
neg(samebib(class_923,class_952)).
neg(samebib(class_923,class_953)).
neg(samebib(class_923,class_954)).
neg(samebib(class_923,class_955)).
neg(samebib(class_923,class_956)).
neg(samebib(class_923,class_958)).
neg(samebib(class_923,class_959)).
neg(samebib(class_923,class_960)).
neg(samebib(class_923,class_961)).
neg(samebib(class_923,class_962)).
neg(samebib(class_923,class_963)).
neg(samebib(class_923,class_964)).
neg(samebib(class_923,class_965)).
neg(samebib(class_923,class_966)).
neg(samebib(class_923,class_967)).
neg(samebib(class_923,class_968)).
neg(samebib(class_923,class_969)).
neg(samebib(class_923,class_970)).
neg(samebib(class_923,class_972)).
neg(samebib(class_923,class_1044)).
neg(samebib(class_923,class_1047)).
samebib(class_924,class_911).
samebib(class_924,class_912).
samebib(class_924,class_913).
samebib(class_924,class_914).
samebib(class_924,class_915).
samebib(class_924,class_916).
samebib(class_924,class_917).
samebib(class_924,class_918).
samebib(class_924,class_919).
samebib(class_924,class_920).
samebib(class_924,class_921).
samebib(class_924,class_922).
samebib(class_924,class_923).
samebib(class_924,class_924).
samebib(class_924,class_925).
samebib(class_924,class_926).
samebib(class_924,class_927).
samebib(class_924,class_928).
samebib(class_924,class_929).
neg(samebib(class_924,class_930)).
neg(samebib(class_924,class_931)).
neg(samebib(class_924,class_932)).
neg(samebib(class_924,class_933)).
neg(samebib(class_924,class_934)).
neg(samebib(class_924,class_935)).
neg(samebib(class_924,class_936)).
neg(samebib(class_924,class_937)).
neg(samebib(class_924,class_938)).
neg(samebib(class_924,class_939)).
neg(samebib(class_924,class_940)).
neg(samebib(class_924,class_941)).
neg(samebib(class_924,class_942)).
neg(samebib(class_924,class_943)).
neg(samebib(class_924,class_944)).
neg(samebib(class_924,class_945)).
neg(samebib(class_924,class_946)).
neg(samebib(class_924,class_947)).
neg(samebib(class_924,class_948)).
neg(samebib(class_924,class_949)).
neg(samebib(class_924,class_950)).
neg(samebib(class_924,class_951)).
neg(samebib(class_924,class_952)).
neg(samebib(class_924,class_953)).
neg(samebib(class_924,class_954)).
neg(samebib(class_924,class_955)).
neg(samebib(class_924,class_956)).
neg(samebib(class_924,class_958)).
neg(samebib(class_924,class_959)).
neg(samebib(class_924,class_960)).
neg(samebib(class_924,class_961)).
neg(samebib(class_924,class_962)).
neg(samebib(class_924,class_963)).
neg(samebib(class_924,class_964)).
neg(samebib(class_924,class_965)).
neg(samebib(class_924,class_966)).
neg(samebib(class_924,class_967)).
neg(samebib(class_924,class_968)).
neg(samebib(class_924,class_969)).
neg(samebib(class_924,class_970)).
neg(samebib(class_924,class_972)).
neg(samebib(class_924,class_1044)).
neg(samebib(class_924,class_1047)).
samebib(class_925,class_911).
samebib(class_925,class_912).
samebib(class_925,class_913).
samebib(class_925,class_914).
samebib(class_925,class_915).
samebib(class_925,class_916).
samebib(class_925,class_917).
samebib(class_925,class_918).
samebib(class_925,class_919).
samebib(class_925,class_920).
samebib(class_925,class_921).
samebib(class_925,class_922).
samebib(class_925,class_923).
samebib(class_925,class_924).
samebib(class_925,class_925).
samebib(class_925,class_926).
samebib(class_925,class_927).
samebib(class_925,class_928).
samebib(class_925,class_929).
neg(samebib(class_925,class_930)).
neg(samebib(class_925,class_931)).
neg(samebib(class_925,class_932)).
neg(samebib(class_925,class_933)).
neg(samebib(class_925,class_934)).
neg(samebib(class_925,class_935)).
neg(samebib(class_925,class_936)).
neg(samebib(class_925,class_937)).
neg(samebib(class_925,class_938)).
neg(samebib(class_925,class_939)).
neg(samebib(class_925,class_940)).
neg(samebib(class_925,class_941)).
neg(samebib(class_925,class_942)).
neg(samebib(class_925,class_943)).
neg(samebib(class_925,class_944)).
neg(samebib(class_925,class_945)).
neg(samebib(class_925,class_946)).
neg(samebib(class_925,class_947)).
neg(samebib(class_925,class_948)).
neg(samebib(class_925,class_949)).
neg(samebib(class_925,class_950)).
neg(samebib(class_925,class_951)).
neg(samebib(class_925,class_952)).
neg(samebib(class_925,class_953)).
neg(samebib(class_925,class_954)).
neg(samebib(class_925,class_955)).
neg(samebib(class_925,class_956)).
neg(samebib(class_925,class_958)).
neg(samebib(class_925,class_959)).
neg(samebib(class_925,class_960)).
neg(samebib(class_925,class_961)).
neg(samebib(class_925,class_962)).
neg(samebib(class_925,class_963)).
neg(samebib(class_925,class_964)).
neg(samebib(class_925,class_965)).
neg(samebib(class_925,class_966)).
neg(samebib(class_925,class_967)).
neg(samebib(class_925,class_968)).
neg(samebib(class_925,class_969)).
neg(samebib(class_925,class_970)).
neg(samebib(class_925,class_972)).
neg(samebib(class_925,class_1044)).
neg(samebib(class_925,class_1047)).
samebib(class_926,class_911).
samebib(class_926,class_912).
samebib(class_926,class_913).
samebib(class_926,class_914).
samebib(class_926,class_917).
samebib(class_926,class_918).
samebib(class_926,class_919).
samebib(class_926,class_920).
samebib(class_926,class_921).
samebib(class_926,class_922).
samebib(class_926,class_923).
samebib(class_926,class_924).
samebib(class_926,class_925).
samebib(class_926,class_926).
samebib(class_926,class_927).
samebib(class_926,class_928).
samebib(class_926,class_929).
neg(samebib(class_926,class_930)).
neg(samebib(class_926,class_931)).
neg(samebib(class_926,class_932)).
neg(samebib(class_926,class_933)).
neg(samebib(class_926,class_934)).
neg(samebib(class_926,class_935)).
neg(samebib(class_926,class_936)).
neg(samebib(class_926,class_937)).
neg(samebib(class_926,class_938)).
neg(samebib(class_926,class_939)).
neg(samebib(class_926,class_940)).
neg(samebib(class_926,class_941)).
neg(samebib(class_926,class_942)).
neg(samebib(class_926,class_943)).
neg(samebib(class_926,class_944)).
neg(samebib(class_926,class_945)).
neg(samebib(class_926,class_946)).
neg(samebib(class_926,class_947)).
neg(samebib(class_926,class_948)).
neg(samebib(class_926,class_949)).
neg(samebib(class_926,class_950)).
neg(samebib(class_926,class_951)).
neg(samebib(class_926,class_952)).
neg(samebib(class_926,class_953)).
neg(samebib(class_926,class_954)).
neg(samebib(class_926,class_955)).
neg(samebib(class_926,class_956)).
neg(samebib(class_926,class_958)).
neg(samebib(class_926,class_959)).
neg(samebib(class_926,class_960)).
neg(samebib(class_926,class_961)).
neg(samebib(class_926,class_962)).
neg(samebib(class_926,class_963)).
neg(samebib(class_926,class_964)).
neg(samebib(class_926,class_965)).
neg(samebib(class_926,class_966)).
neg(samebib(class_926,class_967)).
neg(samebib(class_926,class_968)).
neg(samebib(class_926,class_969)).
neg(samebib(class_926,class_970)).
neg(samebib(class_926,class_972)).
neg(samebib(class_926,class_1044)).
neg(samebib(class_926,class_1047)).
samebib(class_927,class_911).
samebib(class_927,class_912).
samebib(class_927,class_913).
samebib(class_927,class_914).
samebib(class_927,class_917).
samebib(class_927,class_918).
samebib(class_927,class_919).
samebib(class_927,class_920).
samebib(class_927,class_921).
samebib(class_927,class_922).
samebib(class_927,class_923).
samebib(class_927,class_924).
samebib(class_927,class_925).
samebib(class_927,class_926).
samebib(class_927,class_927).
samebib(class_927,class_928).
samebib(class_927,class_929).
neg(samebib(class_927,class_930)).
neg(samebib(class_927,class_931)).
neg(samebib(class_927,class_932)).
neg(samebib(class_927,class_933)).
neg(samebib(class_927,class_934)).
neg(samebib(class_927,class_935)).
neg(samebib(class_927,class_936)).
neg(samebib(class_927,class_937)).
neg(samebib(class_927,class_938)).
neg(samebib(class_927,class_939)).
neg(samebib(class_927,class_940)).
neg(samebib(class_927,class_941)).
neg(samebib(class_927,class_942)).
neg(samebib(class_927,class_943)).
neg(samebib(class_927,class_944)).
neg(samebib(class_927,class_945)).
neg(samebib(class_927,class_946)).
neg(samebib(class_927,class_947)).
neg(samebib(class_927,class_948)).
neg(samebib(class_927,class_949)).
neg(samebib(class_927,class_950)).
neg(samebib(class_927,class_951)).
neg(samebib(class_927,class_952)).
neg(samebib(class_927,class_953)).
neg(samebib(class_927,class_954)).
neg(samebib(class_927,class_955)).
neg(samebib(class_927,class_956)).
neg(samebib(class_927,class_958)).
neg(samebib(class_927,class_959)).
neg(samebib(class_927,class_960)).
neg(samebib(class_927,class_961)).
neg(samebib(class_927,class_962)).
neg(samebib(class_927,class_963)).
neg(samebib(class_927,class_964)).
neg(samebib(class_927,class_965)).
neg(samebib(class_927,class_966)).
neg(samebib(class_927,class_967)).
neg(samebib(class_927,class_968)).
neg(samebib(class_927,class_969)).
neg(samebib(class_927,class_970)).
neg(samebib(class_927,class_972)).
neg(samebib(class_927,class_1044)).
neg(samebib(class_927,class_1047)).
samebib(class_928,class_911).
samebib(class_928,class_912).
samebib(class_928,class_913).
samebib(class_928,class_914).
samebib(class_928,class_917).
samebib(class_928,class_918).
samebib(class_928,class_919).
samebib(class_928,class_920).
samebib(class_928,class_921).
samebib(class_928,class_922).
samebib(class_928,class_923).
samebib(class_928,class_924).
samebib(class_928,class_925).
samebib(class_928,class_926).
samebib(class_928,class_927).
samebib(class_928,class_928).
samebib(class_928,class_929).
neg(samebib(class_928,class_930)).
neg(samebib(class_928,class_931)).
neg(samebib(class_928,class_932)).
neg(samebib(class_928,class_933)).
neg(samebib(class_928,class_934)).
neg(samebib(class_928,class_935)).
neg(samebib(class_928,class_936)).
neg(samebib(class_928,class_937)).
neg(samebib(class_928,class_938)).
neg(samebib(class_928,class_939)).
neg(samebib(class_928,class_940)).
neg(samebib(class_928,class_941)).
neg(samebib(class_928,class_942)).
neg(samebib(class_928,class_943)).
neg(samebib(class_928,class_944)).
neg(samebib(class_928,class_945)).
neg(samebib(class_928,class_946)).
neg(samebib(class_928,class_947)).
neg(samebib(class_928,class_948)).
neg(samebib(class_928,class_949)).
neg(samebib(class_928,class_950)).
neg(samebib(class_928,class_951)).
neg(samebib(class_928,class_952)).
neg(samebib(class_928,class_953)).
neg(samebib(class_928,class_954)).
neg(samebib(class_928,class_955)).
neg(samebib(class_928,class_956)).
neg(samebib(class_928,class_958)).
neg(samebib(class_928,class_959)).
neg(samebib(class_928,class_960)).
neg(samebib(class_928,class_961)).
neg(samebib(class_928,class_962)).
neg(samebib(class_928,class_963)).
neg(samebib(class_928,class_964)).
neg(samebib(class_928,class_965)).
neg(samebib(class_928,class_966)).
neg(samebib(class_928,class_967)).
neg(samebib(class_928,class_968)).
neg(samebib(class_928,class_969)).
neg(samebib(class_928,class_970)).
neg(samebib(class_928,class_972)).
neg(samebib(class_928,class_1044)).
neg(samebib(class_928,class_1047)).
samebib(class_929,class_911).
samebib(class_929,class_912).
samebib(class_929,class_913).
samebib(class_929,class_914).
samebib(class_929,class_917).
samebib(class_929,class_918).
samebib(class_929,class_919).
samebib(class_929,class_920).
samebib(class_929,class_921).
samebib(class_929,class_922).
samebib(class_929,class_923).
samebib(class_929,class_924).
samebib(class_929,class_925).
samebib(class_929,class_926).
samebib(class_929,class_927).
samebib(class_929,class_928).
samebib(class_929,class_929).
neg(samebib(class_929,class_930)).
neg(samebib(class_929,class_931)).
neg(samebib(class_929,class_932)).
neg(samebib(class_929,class_933)).
neg(samebib(class_929,class_934)).
neg(samebib(class_929,class_935)).
neg(samebib(class_929,class_936)).
neg(samebib(class_929,class_937)).
neg(samebib(class_929,class_938)).
neg(samebib(class_929,class_939)).
neg(samebib(class_929,class_940)).
neg(samebib(class_929,class_941)).
neg(samebib(class_929,class_942)).
neg(samebib(class_929,class_943)).
neg(samebib(class_929,class_944)).
neg(samebib(class_929,class_945)).
neg(samebib(class_929,class_946)).
neg(samebib(class_929,class_947)).
neg(samebib(class_929,class_948)).
neg(samebib(class_929,class_949)).
neg(samebib(class_929,class_950)).
neg(samebib(class_929,class_951)).
neg(samebib(class_929,class_952)).
neg(samebib(class_929,class_953)).
neg(samebib(class_929,class_954)).
neg(samebib(class_929,class_955)).
neg(samebib(class_929,class_956)).
neg(samebib(class_929,class_958)).
neg(samebib(class_929,class_959)).
neg(samebib(class_929,class_960)).
neg(samebib(class_929,class_961)).
neg(samebib(class_929,class_962)).
neg(samebib(class_929,class_963)).
neg(samebib(class_929,class_964)).
neg(samebib(class_929,class_965)).
neg(samebib(class_929,class_966)).
neg(samebib(class_929,class_967)).
neg(samebib(class_929,class_968)).
neg(samebib(class_929,class_969)).
neg(samebib(class_929,class_970)).
neg(samebib(class_929,class_972)).
neg(samebib(class_929,class_1044)).
neg(samebib(class_929,class_1047)).
neg(samebib(class_930,class_911)).
neg(samebib(class_930,class_912)).
neg(samebib(class_930,class_913)).
neg(samebib(class_930,class_914)).
neg(samebib(class_930,class_917)).
neg(samebib(class_930,class_918)).
neg(samebib(class_930,class_919)).
neg(samebib(class_930,class_920)).
neg(samebib(class_930,class_921)).
neg(samebib(class_930,class_922)).
neg(samebib(class_930,class_923)).
neg(samebib(class_930,class_924)).
neg(samebib(class_930,class_925)).
neg(samebib(class_930,class_926)).
neg(samebib(class_930,class_927)).
neg(samebib(class_930,class_928)).
neg(samebib(class_930,class_929)).
samebib(class_930,class_930).
samebib(class_930,class_931).
samebib(class_930,class_932).
samebib(class_930,class_933).
samebib(class_930,class_934).
samebib(class_930,class_935).
samebib(class_930,class_936).
samebib(class_930,class_937).
samebib(class_930,class_938).
samebib(class_930,class_939).
samebib(class_930,class_940).
samebib(class_930,class_941).
samebib(class_930,class_942).
samebib(class_930,class_943).
samebib(class_930,class_944).
samebib(class_930,class_945).
samebib(class_930,class_946).
samebib(class_930,class_947).
samebib(class_930,class_948).
samebib(class_930,class_949).
samebib(class_930,class_950).
samebib(class_930,class_951).
samebib(class_930,class_952).
samebib(class_930,class_953).
samebib(class_930,class_954).
samebib(class_930,class_955).
samebib(class_930,class_956).
samebib(class_930,class_957).
samebib(class_930,class_958).
samebib(class_930,class_959).
samebib(class_930,class_960).
samebib(class_930,class_961).
samebib(class_930,class_962).
samebib(class_930,class_963).
samebib(class_930,class_964).
samebib(class_930,class_965).
samebib(class_930,class_966).
samebib(class_930,class_967).
samebib(class_930,class_968).
samebib(class_930,class_969).
samebib(class_930,class_970).
samebib(class_930,class_971).
samebib(class_930,class_972).
neg(samebib(class_930,class_1025)).
neg(samebib(class_930,class_1031)).
neg(samebib(class_930,class_1033)).
neg(samebib(class_930,class_1044)).
neg(samebib(class_930,class_1047)).
neg(samebib(class_931,class_911)).
neg(samebib(class_931,class_912)).
neg(samebib(class_931,class_913)).
neg(samebib(class_931,class_914)).
neg(samebib(class_931,class_917)).
neg(samebib(class_931,class_918)).
neg(samebib(class_931,class_919)).
neg(samebib(class_931,class_920)).
neg(samebib(class_931,class_921)).
neg(samebib(class_931,class_922)).
neg(samebib(class_931,class_923)).
neg(samebib(class_931,class_924)).
neg(samebib(class_931,class_925)).
neg(samebib(class_931,class_926)).
neg(samebib(class_931,class_927)).
neg(samebib(class_931,class_928)).
neg(samebib(class_931,class_929)).
samebib(class_931,class_930).
samebib(class_931,class_931).
samebib(class_931,class_932).
samebib(class_931,class_933).
samebib(class_931,class_934).
samebib(class_931,class_935).
samebib(class_931,class_936).
samebib(class_931,class_937).
samebib(class_931,class_938).
samebib(class_931,class_939).
samebib(class_931,class_940).
samebib(class_931,class_941).
samebib(class_931,class_942).
samebib(class_931,class_943).
samebib(class_931,class_944).
samebib(class_931,class_945).
samebib(class_931,class_946).
samebib(class_931,class_947).
samebib(class_931,class_948).
samebib(class_931,class_949).
samebib(class_931,class_950).
samebib(class_931,class_951).
samebib(class_931,class_952).
samebib(class_931,class_953).
samebib(class_931,class_954).
samebib(class_931,class_955).
samebib(class_931,class_956).
samebib(class_931,class_957).
samebib(class_931,class_958).
samebib(class_931,class_959).
samebib(class_931,class_960).
samebib(class_931,class_961).
samebib(class_931,class_962).
samebib(class_931,class_963).
samebib(class_931,class_964).
samebib(class_931,class_965).
samebib(class_931,class_966).
samebib(class_931,class_967).
samebib(class_931,class_968).
samebib(class_931,class_969).
samebib(class_931,class_970).
samebib(class_931,class_971).
samebib(class_931,class_972).
neg(samebib(class_931,class_1025)).
neg(samebib(class_931,class_1031)).
neg(samebib(class_931,class_1033)).
neg(samebib(class_931,class_1044)).
neg(samebib(class_931,class_1047)).
neg(samebib(class_932,class_911)).
neg(samebib(class_932,class_912)).
neg(samebib(class_932,class_913)).
neg(samebib(class_932,class_914)).
neg(samebib(class_932,class_917)).
neg(samebib(class_932,class_918)).
neg(samebib(class_932,class_919)).
neg(samebib(class_932,class_920)).
neg(samebib(class_932,class_921)).
neg(samebib(class_932,class_922)).
neg(samebib(class_932,class_923)).
neg(samebib(class_932,class_924)).
neg(samebib(class_932,class_925)).
neg(samebib(class_932,class_926)).
neg(samebib(class_932,class_927)).
neg(samebib(class_932,class_928)).
neg(samebib(class_932,class_929)).
samebib(class_932,class_930).
samebib(class_932,class_931).
samebib(class_932,class_932).
samebib(class_932,class_933).
samebib(class_932,class_934).
samebib(class_932,class_935).
samebib(class_932,class_936).
samebib(class_932,class_937).
samebib(class_932,class_938).
samebib(class_932,class_939).
samebib(class_932,class_940).
samebib(class_932,class_941).
samebib(class_932,class_942).
samebib(class_932,class_943).
samebib(class_932,class_944).
samebib(class_932,class_945).
samebib(class_932,class_946).
samebib(class_932,class_947).
samebib(class_932,class_948).
samebib(class_932,class_949).
samebib(class_932,class_950).
samebib(class_932,class_951).
samebib(class_932,class_952).
samebib(class_932,class_953).
samebib(class_932,class_954).
samebib(class_932,class_955).
samebib(class_932,class_956).
samebib(class_932,class_957).
samebib(class_932,class_958).
samebib(class_932,class_959).
samebib(class_932,class_960).
samebib(class_932,class_961).
samebib(class_932,class_962).
samebib(class_932,class_963).
samebib(class_932,class_964).
samebib(class_932,class_965).
samebib(class_932,class_966).
samebib(class_932,class_967).
samebib(class_932,class_968).
samebib(class_932,class_969).
samebib(class_932,class_970).
samebib(class_932,class_971).
samebib(class_932,class_972).
neg(samebib(class_932,class_1025)).
neg(samebib(class_932,class_1027)).
neg(samebib(class_932,class_1028)).
neg(samebib(class_932,class_1030)).
neg(samebib(class_932,class_1031)).
neg(samebib(class_932,class_1032)).
neg(samebib(class_932,class_1033)).
neg(samebib(class_932,class_1044)).
neg(samebib(class_932,class_1047)).
neg(samebib(class_933,class_911)).
neg(samebib(class_933,class_912)).
neg(samebib(class_933,class_913)).
neg(samebib(class_933,class_914)).
neg(samebib(class_933,class_917)).
neg(samebib(class_933,class_918)).
neg(samebib(class_933,class_919)).
neg(samebib(class_933,class_920)).
neg(samebib(class_933,class_921)).
neg(samebib(class_933,class_922)).
neg(samebib(class_933,class_923)).
neg(samebib(class_933,class_924)).
neg(samebib(class_933,class_925)).
neg(samebib(class_933,class_926)).
neg(samebib(class_933,class_927)).
neg(samebib(class_933,class_928)).
neg(samebib(class_933,class_929)).
samebib(class_933,class_930).
samebib(class_933,class_931).
samebib(class_933,class_932).
samebib(class_933,class_933).
samebib(class_933,class_934).
samebib(class_933,class_935).
samebib(class_933,class_936).
samebib(class_933,class_937).
samebib(class_933,class_938).
samebib(class_933,class_939).
samebib(class_933,class_940).
samebib(class_933,class_941).
samebib(class_933,class_942).
samebib(class_933,class_943).
samebib(class_933,class_944).
samebib(class_933,class_945).
samebib(class_933,class_946).
samebib(class_933,class_947).
samebib(class_933,class_948).
samebib(class_933,class_949).
samebib(class_933,class_950).
samebib(class_933,class_951).
samebib(class_933,class_952).
samebib(class_933,class_953).
samebib(class_933,class_954).
samebib(class_933,class_955).
samebib(class_933,class_956).
samebib(class_933,class_957).
samebib(class_933,class_958).
samebib(class_933,class_959).
samebib(class_933,class_960).
samebib(class_933,class_961).
samebib(class_933,class_962).
samebib(class_933,class_963).
samebib(class_933,class_964).
samebib(class_933,class_965).
samebib(class_933,class_966).
samebib(class_933,class_967).
samebib(class_933,class_968).
samebib(class_933,class_969).
samebib(class_933,class_970).
samebib(class_933,class_971).
samebib(class_933,class_972).
neg(samebib(class_933,class_1025)).
neg(samebib(class_933,class_1027)).
neg(samebib(class_933,class_1028)).
neg(samebib(class_933,class_1030)).
neg(samebib(class_933,class_1031)).
neg(samebib(class_933,class_1032)).
neg(samebib(class_933,class_1033)).
neg(samebib(class_933,class_1044)).
neg(samebib(class_933,class_1047)).
neg(samebib(class_934,class_911)).
neg(samebib(class_934,class_912)).
neg(samebib(class_934,class_913)).
neg(samebib(class_934,class_914)).
neg(samebib(class_934,class_917)).
neg(samebib(class_934,class_918)).
neg(samebib(class_934,class_919)).
neg(samebib(class_934,class_920)).
neg(samebib(class_934,class_921)).
neg(samebib(class_934,class_922)).
neg(samebib(class_934,class_923)).
neg(samebib(class_934,class_924)).
neg(samebib(class_934,class_925)).
neg(samebib(class_934,class_926)).
neg(samebib(class_934,class_927)).
neg(samebib(class_934,class_928)).
neg(samebib(class_934,class_929)).
samebib(class_934,class_930).
samebib(class_934,class_931).
samebib(class_934,class_932).
samebib(class_934,class_933).
samebib(class_934,class_934).
samebib(class_934,class_935).
samebib(class_934,class_936).
samebib(class_934,class_937).
samebib(class_934,class_938).
samebib(class_934,class_939).
samebib(class_934,class_940).
samebib(class_934,class_941).
samebib(class_934,class_942).
samebib(class_934,class_943).
samebib(class_934,class_944).
samebib(class_934,class_945).
samebib(class_934,class_946).
samebib(class_934,class_947).
samebib(class_934,class_948).
samebib(class_934,class_949).
samebib(class_934,class_950).
samebib(class_934,class_951).
samebib(class_934,class_952).
samebib(class_934,class_953).
samebib(class_934,class_954).
samebib(class_934,class_955).
samebib(class_934,class_956).
samebib(class_934,class_957).
samebib(class_934,class_958).
samebib(class_934,class_959).
samebib(class_934,class_960).
samebib(class_934,class_961).
samebib(class_934,class_962).
samebib(class_934,class_963).
samebib(class_934,class_964).
samebib(class_934,class_965).
samebib(class_934,class_966).
samebib(class_934,class_967).
samebib(class_934,class_968).
samebib(class_934,class_969).
samebib(class_934,class_970).
samebib(class_934,class_971).
samebib(class_934,class_972).
neg(samebib(class_934,class_1025)).
neg(samebib(class_934,class_1027)).
neg(samebib(class_934,class_1028)).
neg(samebib(class_934,class_1030)).
neg(samebib(class_934,class_1031)).
neg(samebib(class_934,class_1032)).
neg(samebib(class_934,class_1033)).
neg(samebib(class_934,class_1044)).
neg(samebib(class_934,class_1047)).
neg(samebib(class_935,class_911)).
neg(samebib(class_935,class_912)).
neg(samebib(class_935,class_913)).
neg(samebib(class_935,class_914)).
neg(samebib(class_935,class_917)).
neg(samebib(class_935,class_918)).
neg(samebib(class_935,class_919)).
neg(samebib(class_935,class_920)).
neg(samebib(class_935,class_921)).
neg(samebib(class_935,class_922)).
neg(samebib(class_935,class_923)).
neg(samebib(class_935,class_924)).
neg(samebib(class_935,class_925)).
neg(samebib(class_935,class_926)).
neg(samebib(class_935,class_927)).
neg(samebib(class_935,class_928)).
neg(samebib(class_935,class_929)).
samebib(class_935,class_930).
samebib(class_935,class_931).
samebib(class_935,class_932).
samebib(class_935,class_933).
samebib(class_935,class_934).
samebib(class_935,class_935).
samebib(class_935,class_936).
samebib(class_935,class_937).
samebib(class_935,class_938).
samebib(class_935,class_939).
samebib(class_935,class_940).
samebib(class_935,class_941).
samebib(class_935,class_942).
samebib(class_935,class_943).
samebib(class_935,class_944).
samebib(class_935,class_945).
samebib(class_935,class_946).
samebib(class_935,class_947).
samebib(class_935,class_948).
samebib(class_935,class_949).
samebib(class_935,class_950).
samebib(class_935,class_951).
samebib(class_935,class_952).
samebib(class_935,class_953).
samebib(class_935,class_954).
samebib(class_935,class_955).
samebib(class_935,class_956).
samebib(class_935,class_957).
samebib(class_935,class_958).
samebib(class_935,class_959).
samebib(class_935,class_960).
samebib(class_935,class_961).
samebib(class_935,class_962).
samebib(class_935,class_963).
samebib(class_935,class_964).
samebib(class_935,class_965).
samebib(class_935,class_966).
samebib(class_935,class_967).
samebib(class_935,class_968).
samebib(class_935,class_969).
samebib(class_935,class_970).
samebib(class_935,class_971).
samebib(class_935,class_972).
neg(samebib(class_935,class_1025)).
neg(samebib(class_935,class_1027)).
neg(samebib(class_935,class_1028)).
neg(samebib(class_935,class_1030)).
neg(samebib(class_935,class_1031)).
neg(samebib(class_935,class_1032)).
neg(samebib(class_935,class_1033)).
neg(samebib(class_935,class_1044)).
neg(samebib(class_935,class_1047)).
neg(samebib(class_936,class_911)).
neg(samebib(class_936,class_912)).
neg(samebib(class_936,class_913)).
neg(samebib(class_936,class_914)).
neg(samebib(class_936,class_917)).
neg(samebib(class_936,class_918)).
neg(samebib(class_936,class_919)).
neg(samebib(class_936,class_920)).
neg(samebib(class_936,class_921)).
neg(samebib(class_936,class_922)).
neg(samebib(class_936,class_923)).
neg(samebib(class_936,class_924)).
neg(samebib(class_936,class_925)).
neg(samebib(class_936,class_926)).
neg(samebib(class_936,class_927)).
neg(samebib(class_936,class_928)).
neg(samebib(class_936,class_929)).
samebib(class_936,class_930).
samebib(class_936,class_931).
samebib(class_936,class_932).
samebib(class_936,class_933).
samebib(class_936,class_934).
samebib(class_936,class_935).
samebib(class_936,class_936).
samebib(class_936,class_937).
samebib(class_936,class_938).
samebib(class_936,class_939).
samebib(class_936,class_940).
samebib(class_936,class_941).
samebib(class_936,class_942).
samebib(class_936,class_943).
samebib(class_936,class_944).
samebib(class_936,class_945).
samebib(class_936,class_946).
samebib(class_936,class_947).
samebib(class_936,class_948).
samebib(class_936,class_949).
samebib(class_936,class_950).
samebib(class_936,class_951).
samebib(class_936,class_952).
samebib(class_936,class_953).
samebib(class_936,class_954).
samebib(class_936,class_955).
samebib(class_936,class_956).
samebib(class_936,class_957).
samebib(class_936,class_958).
samebib(class_936,class_959).
samebib(class_936,class_960).
samebib(class_936,class_961).
samebib(class_936,class_962).
samebib(class_936,class_963).
samebib(class_936,class_964).
samebib(class_936,class_965).
samebib(class_936,class_966).
samebib(class_936,class_967).
samebib(class_936,class_968).
samebib(class_936,class_969).
samebib(class_936,class_970).
samebib(class_936,class_971).
samebib(class_936,class_972).
neg(samebib(class_936,class_1025)).
neg(samebib(class_936,class_1027)).
neg(samebib(class_936,class_1028)).
neg(samebib(class_936,class_1030)).
neg(samebib(class_936,class_1031)).
neg(samebib(class_936,class_1032)).
neg(samebib(class_936,class_1033)).
neg(samebib(class_936,class_1044)).
neg(samebib(class_936,class_1047)).
neg(samebib(class_937,class_911)).
neg(samebib(class_937,class_912)).
neg(samebib(class_937,class_913)).
neg(samebib(class_937,class_914)).
neg(samebib(class_937,class_917)).
neg(samebib(class_937,class_918)).
neg(samebib(class_937,class_919)).
neg(samebib(class_937,class_920)).
neg(samebib(class_937,class_921)).
neg(samebib(class_937,class_922)).
neg(samebib(class_937,class_923)).
neg(samebib(class_937,class_924)).
neg(samebib(class_937,class_925)).
neg(samebib(class_937,class_926)).
neg(samebib(class_937,class_927)).
neg(samebib(class_937,class_928)).
neg(samebib(class_937,class_929)).
samebib(class_937,class_930).
samebib(class_937,class_931).
samebib(class_937,class_932).
samebib(class_937,class_933).
samebib(class_937,class_934).
samebib(class_937,class_935).
samebib(class_937,class_936).
samebib(class_937,class_937).
samebib(class_937,class_938).
samebib(class_937,class_939).
samebib(class_937,class_940).
samebib(class_937,class_941).
samebib(class_937,class_942).
samebib(class_937,class_943).
samebib(class_937,class_944).
samebib(class_937,class_945).
samebib(class_937,class_946).
samebib(class_937,class_947).
samebib(class_937,class_948).
samebib(class_937,class_949).
samebib(class_937,class_950).
samebib(class_937,class_951).
samebib(class_937,class_952).
samebib(class_937,class_953).
samebib(class_937,class_954).
samebib(class_937,class_955).
samebib(class_937,class_956).
samebib(class_937,class_957).
samebib(class_937,class_958).
samebib(class_937,class_959).
samebib(class_937,class_960).
samebib(class_937,class_961).
samebib(class_937,class_962).
samebib(class_937,class_963).
samebib(class_937,class_964).
samebib(class_937,class_965).
samebib(class_937,class_966).
samebib(class_937,class_967).
samebib(class_937,class_968).
samebib(class_937,class_969).
samebib(class_937,class_970).
samebib(class_937,class_971).
samebib(class_937,class_972).
neg(samebib(class_937,class_1025)).
neg(samebib(class_937,class_1027)).
neg(samebib(class_937,class_1028)).
neg(samebib(class_937,class_1030)).
neg(samebib(class_937,class_1031)).
neg(samebib(class_937,class_1032)).
neg(samebib(class_937,class_1033)).
neg(samebib(class_937,class_1044)).
neg(samebib(class_937,class_1047)).
neg(samebib(class_938,class_911)).
neg(samebib(class_938,class_912)).
neg(samebib(class_938,class_913)).
neg(samebib(class_938,class_914)).
neg(samebib(class_938,class_917)).
neg(samebib(class_938,class_918)).
neg(samebib(class_938,class_919)).
neg(samebib(class_938,class_920)).
neg(samebib(class_938,class_921)).
neg(samebib(class_938,class_922)).
neg(samebib(class_938,class_923)).
neg(samebib(class_938,class_924)).
neg(samebib(class_938,class_925)).
neg(samebib(class_938,class_926)).
neg(samebib(class_938,class_927)).
neg(samebib(class_938,class_928)).
neg(samebib(class_938,class_929)).
samebib(class_938,class_930).
samebib(class_938,class_931).
samebib(class_938,class_932).
samebib(class_938,class_933).
samebib(class_938,class_934).
samebib(class_938,class_935).
samebib(class_938,class_936).
samebib(class_938,class_937).
samebib(class_938,class_938).
samebib(class_938,class_939).
samebib(class_938,class_940).
samebib(class_938,class_941).
samebib(class_938,class_942).
samebib(class_938,class_943).
samebib(class_938,class_944).
samebib(class_938,class_945).
samebib(class_938,class_946).
samebib(class_938,class_947).
samebib(class_938,class_948).
samebib(class_938,class_949).
samebib(class_938,class_950).
samebib(class_938,class_951).
samebib(class_938,class_952).
samebib(class_938,class_953).
samebib(class_938,class_954).
samebib(class_938,class_955).
samebib(class_938,class_956).
samebib(class_938,class_957).
samebib(class_938,class_958).
samebib(class_938,class_959).
samebib(class_938,class_960).
samebib(class_938,class_961).
samebib(class_938,class_962).
samebib(class_938,class_963).
samebib(class_938,class_964).
samebib(class_938,class_965).
samebib(class_938,class_966).
samebib(class_938,class_967).
samebib(class_938,class_968).
samebib(class_938,class_969).
samebib(class_938,class_970).
samebib(class_938,class_971).
samebib(class_938,class_972).
neg(samebib(class_938,class_1025)).
neg(samebib(class_938,class_1027)).
neg(samebib(class_938,class_1028)).
neg(samebib(class_938,class_1030)).
neg(samebib(class_938,class_1031)).
neg(samebib(class_938,class_1032)).
neg(samebib(class_938,class_1033)).
neg(samebib(class_938,class_1044)).
neg(samebib(class_938,class_1047)).
neg(samebib(class_939,class_911)).
neg(samebib(class_939,class_912)).
neg(samebib(class_939,class_913)).
neg(samebib(class_939,class_914)).
neg(samebib(class_939,class_917)).
neg(samebib(class_939,class_918)).
neg(samebib(class_939,class_919)).
neg(samebib(class_939,class_920)).
neg(samebib(class_939,class_921)).
neg(samebib(class_939,class_922)).
neg(samebib(class_939,class_923)).
neg(samebib(class_939,class_924)).
neg(samebib(class_939,class_925)).
neg(samebib(class_939,class_926)).
neg(samebib(class_939,class_927)).
neg(samebib(class_939,class_928)).
neg(samebib(class_939,class_929)).
samebib(class_939,class_930).
samebib(class_939,class_931).
samebib(class_939,class_932).
samebib(class_939,class_933).
samebib(class_939,class_934).
samebib(class_939,class_935).
samebib(class_939,class_936).
samebib(class_939,class_937).
samebib(class_939,class_938).
samebib(class_939,class_939).
samebib(class_939,class_940).
samebib(class_939,class_941).
samebib(class_939,class_942).
samebib(class_939,class_943).
samebib(class_939,class_944).
samebib(class_939,class_945).
samebib(class_939,class_946).
samebib(class_939,class_947).
samebib(class_939,class_948).
samebib(class_939,class_949).
samebib(class_939,class_950).
samebib(class_939,class_951).
samebib(class_939,class_952).
samebib(class_939,class_953).
samebib(class_939,class_954).
samebib(class_939,class_955).
samebib(class_939,class_956).
samebib(class_939,class_957).
samebib(class_939,class_958).
samebib(class_939,class_959).
samebib(class_939,class_960).
samebib(class_939,class_961).
samebib(class_939,class_962).
samebib(class_939,class_963).
samebib(class_939,class_964).
samebib(class_939,class_965).
samebib(class_939,class_966).
samebib(class_939,class_967).
samebib(class_939,class_968).
samebib(class_939,class_969).
samebib(class_939,class_970).
samebib(class_939,class_971).
samebib(class_939,class_972).
neg(samebib(class_939,class_1025)).
neg(samebib(class_939,class_1027)).
neg(samebib(class_939,class_1028)).
neg(samebib(class_939,class_1030)).
neg(samebib(class_939,class_1031)).
neg(samebib(class_939,class_1032)).
neg(samebib(class_939,class_1033)).
neg(samebib(class_939,class_1044)).
neg(samebib(class_939,class_1047)).
neg(samebib(class_940,class_911)).
neg(samebib(class_940,class_912)).
neg(samebib(class_940,class_913)).
neg(samebib(class_940,class_914)).
neg(samebib(class_940,class_917)).
neg(samebib(class_940,class_918)).
neg(samebib(class_940,class_919)).
neg(samebib(class_940,class_920)).
neg(samebib(class_940,class_921)).
neg(samebib(class_940,class_922)).
neg(samebib(class_940,class_923)).
neg(samebib(class_940,class_924)).
neg(samebib(class_940,class_925)).
neg(samebib(class_940,class_926)).
neg(samebib(class_940,class_927)).
neg(samebib(class_940,class_928)).
neg(samebib(class_940,class_929)).
samebib(class_940,class_930).
samebib(class_940,class_931).
samebib(class_940,class_932).
samebib(class_940,class_933).
samebib(class_940,class_934).
samebib(class_940,class_935).
samebib(class_940,class_936).
samebib(class_940,class_937).
samebib(class_940,class_938).
samebib(class_940,class_939).
samebib(class_940,class_940).
samebib(class_940,class_941).
samebib(class_940,class_942).
samebib(class_940,class_943).
samebib(class_940,class_944).
samebib(class_940,class_945).
samebib(class_940,class_946).
samebib(class_940,class_947).
samebib(class_940,class_948).
samebib(class_940,class_949).
samebib(class_940,class_950).
samebib(class_940,class_951).
samebib(class_940,class_952).
samebib(class_940,class_953).
samebib(class_940,class_954).
samebib(class_940,class_955).
samebib(class_940,class_956).
samebib(class_940,class_957).
samebib(class_940,class_958).
samebib(class_940,class_959).
samebib(class_940,class_960).
samebib(class_940,class_961).
samebib(class_940,class_962).
samebib(class_940,class_963).
samebib(class_940,class_964).
samebib(class_940,class_965).
samebib(class_940,class_966).
samebib(class_940,class_967).
samebib(class_940,class_968).
samebib(class_940,class_969).
samebib(class_940,class_970).
samebib(class_940,class_971).
samebib(class_940,class_972).
neg(samebib(class_940,class_1025)).
neg(samebib(class_940,class_1027)).
neg(samebib(class_940,class_1028)).
neg(samebib(class_940,class_1030)).
neg(samebib(class_940,class_1031)).
neg(samebib(class_940,class_1032)).
neg(samebib(class_940,class_1033)).
neg(samebib(class_940,class_1044)).
neg(samebib(class_940,class_1047)).
neg(samebib(class_941,class_911)).
neg(samebib(class_941,class_912)).
neg(samebib(class_941,class_913)).
neg(samebib(class_941,class_914)).
neg(samebib(class_941,class_917)).
neg(samebib(class_941,class_918)).
neg(samebib(class_941,class_919)).
neg(samebib(class_941,class_920)).
neg(samebib(class_941,class_921)).
neg(samebib(class_941,class_922)).
neg(samebib(class_941,class_923)).
neg(samebib(class_941,class_924)).
neg(samebib(class_941,class_925)).
neg(samebib(class_941,class_926)).
neg(samebib(class_941,class_927)).
neg(samebib(class_941,class_928)).
neg(samebib(class_941,class_929)).
samebib(class_941,class_930).
samebib(class_941,class_931).
samebib(class_941,class_932).
samebib(class_941,class_933).
samebib(class_941,class_934).
samebib(class_941,class_935).
samebib(class_941,class_936).
samebib(class_941,class_937).
samebib(class_941,class_938).
samebib(class_941,class_939).
samebib(class_941,class_940).
samebib(class_941,class_941).
samebib(class_941,class_942).
samebib(class_941,class_943).
samebib(class_941,class_944).
samebib(class_941,class_945).
samebib(class_941,class_946).
samebib(class_941,class_947).
samebib(class_941,class_948).
samebib(class_941,class_949).
samebib(class_941,class_950).
samebib(class_941,class_951).
samebib(class_941,class_952).
samebib(class_941,class_953).
samebib(class_941,class_954).
samebib(class_941,class_955).
samebib(class_941,class_956).
samebib(class_941,class_957).
samebib(class_941,class_958).
samebib(class_941,class_959).
samebib(class_941,class_960).
samebib(class_941,class_961).
samebib(class_941,class_962).
samebib(class_941,class_963).
samebib(class_941,class_964).
samebib(class_941,class_965).
samebib(class_941,class_966).
samebib(class_941,class_967).
samebib(class_941,class_968).
samebib(class_941,class_969).
samebib(class_941,class_970).
samebib(class_941,class_971).
samebib(class_941,class_972).
neg(samebib(class_941,class_1025)).
neg(samebib(class_941,class_1027)).
neg(samebib(class_941,class_1028)).
neg(samebib(class_941,class_1030)).
neg(samebib(class_941,class_1031)).
neg(samebib(class_941,class_1032)).
neg(samebib(class_941,class_1033)).
neg(samebib(class_941,class_1044)).
neg(samebib(class_941,class_1047)).
neg(samebib(class_942,class_911)).
neg(samebib(class_942,class_912)).
neg(samebib(class_942,class_913)).
neg(samebib(class_942,class_914)).
neg(samebib(class_942,class_917)).
neg(samebib(class_942,class_918)).
neg(samebib(class_942,class_919)).
neg(samebib(class_942,class_920)).
neg(samebib(class_942,class_921)).
neg(samebib(class_942,class_922)).
neg(samebib(class_942,class_923)).
neg(samebib(class_942,class_924)).
neg(samebib(class_942,class_925)).
neg(samebib(class_942,class_926)).
neg(samebib(class_942,class_927)).
neg(samebib(class_942,class_928)).
neg(samebib(class_942,class_929)).
samebib(class_942,class_930).
samebib(class_942,class_931).
samebib(class_942,class_932).
samebib(class_942,class_933).
samebib(class_942,class_934).
samebib(class_942,class_935).
samebib(class_942,class_936).
samebib(class_942,class_937).
samebib(class_942,class_938).
samebib(class_942,class_939).
samebib(class_942,class_940).
samebib(class_942,class_941).
samebib(class_942,class_942).
samebib(class_942,class_943).
samebib(class_942,class_944).
samebib(class_942,class_945).
samebib(class_942,class_946).
samebib(class_942,class_947).
samebib(class_942,class_948).
samebib(class_942,class_949).
samebib(class_942,class_950).
samebib(class_942,class_951).
samebib(class_942,class_952).
samebib(class_942,class_953).
samebib(class_942,class_954).
samebib(class_942,class_955).
samebib(class_942,class_956).
samebib(class_942,class_957).
samebib(class_942,class_958).
samebib(class_942,class_959).
samebib(class_942,class_960).
samebib(class_942,class_961).
samebib(class_942,class_962).
samebib(class_942,class_963).
samebib(class_942,class_964).
samebib(class_942,class_965).
samebib(class_942,class_966).
samebib(class_942,class_967).
samebib(class_942,class_968).
samebib(class_942,class_969).
samebib(class_942,class_970).
samebib(class_942,class_971).
samebib(class_942,class_972).
neg(samebib(class_942,class_1025)).
neg(samebib(class_942,class_1027)).
neg(samebib(class_942,class_1028)).
neg(samebib(class_942,class_1030)).
neg(samebib(class_942,class_1031)).
neg(samebib(class_942,class_1032)).
neg(samebib(class_942,class_1033)).
neg(samebib(class_942,class_1044)).
neg(samebib(class_942,class_1047)).
neg(samebib(class_943,class_911)).
neg(samebib(class_943,class_912)).
neg(samebib(class_943,class_913)).
neg(samebib(class_943,class_914)).
neg(samebib(class_943,class_917)).
neg(samebib(class_943,class_918)).
neg(samebib(class_943,class_919)).
neg(samebib(class_943,class_920)).
neg(samebib(class_943,class_921)).
neg(samebib(class_943,class_922)).
neg(samebib(class_943,class_923)).
neg(samebib(class_943,class_924)).
neg(samebib(class_943,class_925)).
neg(samebib(class_943,class_926)).
neg(samebib(class_943,class_927)).
neg(samebib(class_943,class_928)).
neg(samebib(class_943,class_929)).
samebib(class_943,class_930).
samebib(class_943,class_931).
samebib(class_943,class_932).
samebib(class_943,class_933).
samebib(class_943,class_934).
samebib(class_943,class_935).
samebib(class_943,class_936).
samebib(class_943,class_937).
samebib(class_943,class_938).
samebib(class_943,class_939).
samebib(class_943,class_940).
samebib(class_943,class_941).
samebib(class_943,class_942).
samebib(class_943,class_943).
samebib(class_943,class_944).
samebib(class_943,class_945).
samebib(class_943,class_946).
samebib(class_943,class_947).
samebib(class_943,class_948).
samebib(class_943,class_949).
samebib(class_943,class_950).
samebib(class_943,class_951).
samebib(class_943,class_952).
samebib(class_943,class_953).
samebib(class_943,class_954).
samebib(class_943,class_955).
samebib(class_943,class_956).
samebib(class_943,class_957).
samebib(class_943,class_958).
samebib(class_943,class_959).
samebib(class_943,class_960).
samebib(class_943,class_961).
samebib(class_943,class_962).
samebib(class_943,class_963).
samebib(class_943,class_964).
samebib(class_943,class_965).
samebib(class_943,class_966).
samebib(class_943,class_967).
samebib(class_943,class_968).
samebib(class_943,class_969).
samebib(class_943,class_970).
samebib(class_943,class_971).
samebib(class_943,class_972).
neg(samebib(class_943,class_1025)).
neg(samebib(class_943,class_1026)).
neg(samebib(class_943,class_1027)).
neg(samebib(class_943,class_1028)).
neg(samebib(class_943,class_1030)).
neg(samebib(class_943,class_1031)).
neg(samebib(class_943,class_1032)).
neg(samebib(class_943,class_1033)).
neg(samebib(class_943,class_1044)).
neg(samebib(class_943,class_1045)).
neg(samebib(class_943,class_1046)).
neg(samebib(class_943,class_1047)).
neg(samebib(class_943,class_1048)).
neg(samebib(class_944,class_911)).
neg(samebib(class_944,class_912)).
neg(samebib(class_944,class_913)).
neg(samebib(class_944,class_914)).
neg(samebib(class_944,class_917)).
neg(samebib(class_944,class_918)).
neg(samebib(class_944,class_919)).
neg(samebib(class_944,class_920)).
neg(samebib(class_944,class_921)).
neg(samebib(class_944,class_922)).
neg(samebib(class_944,class_923)).
neg(samebib(class_944,class_924)).
neg(samebib(class_944,class_925)).
neg(samebib(class_944,class_926)).
neg(samebib(class_944,class_927)).
neg(samebib(class_944,class_928)).
neg(samebib(class_944,class_929)).
samebib(class_944,class_930).
samebib(class_944,class_931).
samebib(class_944,class_932).
samebib(class_944,class_933).
samebib(class_944,class_934).
samebib(class_944,class_935).
samebib(class_944,class_936).
samebib(class_944,class_937).
samebib(class_944,class_938).
samebib(class_944,class_939).
samebib(class_944,class_940).
samebib(class_944,class_941).
samebib(class_944,class_942).
samebib(class_944,class_943).
samebib(class_944,class_944).
samebib(class_944,class_945).
samebib(class_944,class_946).
samebib(class_944,class_947).
samebib(class_944,class_948).
samebib(class_944,class_949).
samebib(class_944,class_950).
samebib(class_944,class_951).
samebib(class_944,class_952).
samebib(class_944,class_953).
samebib(class_944,class_954).
samebib(class_944,class_955).
samebib(class_944,class_956).
samebib(class_944,class_957).
samebib(class_944,class_958).
samebib(class_944,class_959).
samebib(class_944,class_960).
samebib(class_944,class_961).
samebib(class_944,class_962).
samebib(class_944,class_963).
samebib(class_944,class_964).
samebib(class_944,class_965).
samebib(class_944,class_966).
samebib(class_944,class_967).
samebib(class_944,class_968).
samebib(class_944,class_969).
samebib(class_944,class_970).
samebib(class_944,class_971).
samebib(class_944,class_972).
neg(samebib(class_944,class_1025)).
neg(samebib(class_944,class_1026)).
neg(samebib(class_944,class_1027)).
neg(samebib(class_944,class_1028)).
neg(samebib(class_944,class_1030)).
neg(samebib(class_944,class_1031)).
neg(samebib(class_944,class_1032)).
neg(samebib(class_944,class_1033)).
neg(samebib(class_944,class_1044)).
neg(samebib(class_944,class_1045)).
neg(samebib(class_944,class_1046)).
neg(samebib(class_944,class_1047)).
neg(samebib(class_944,class_1048)).
neg(samebib(class_945,class_911)).
neg(samebib(class_945,class_912)).
neg(samebib(class_945,class_913)).
neg(samebib(class_945,class_914)).
neg(samebib(class_945,class_917)).
neg(samebib(class_945,class_918)).
neg(samebib(class_945,class_919)).
neg(samebib(class_945,class_920)).
neg(samebib(class_945,class_921)).
neg(samebib(class_945,class_922)).
neg(samebib(class_945,class_923)).
neg(samebib(class_945,class_924)).
neg(samebib(class_945,class_925)).
neg(samebib(class_945,class_926)).
neg(samebib(class_945,class_927)).
neg(samebib(class_945,class_928)).
neg(samebib(class_945,class_929)).
samebib(class_945,class_930).
samebib(class_945,class_931).
samebib(class_945,class_932).
samebib(class_945,class_933).
samebib(class_945,class_934).
samebib(class_945,class_935).
samebib(class_945,class_936).
samebib(class_945,class_937).
samebib(class_945,class_938).
samebib(class_945,class_939).
samebib(class_945,class_940).
samebib(class_945,class_941).
samebib(class_945,class_942).
samebib(class_945,class_943).
samebib(class_945,class_944).
samebib(class_945,class_945).
samebib(class_945,class_946).
samebib(class_945,class_947).
samebib(class_945,class_948).
samebib(class_945,class_949).
samebib(class_945,class_950).
samebib(class_945,class_951).
samebib(class_945,class_952).
samebib(class_945,class_953).
samebib(class_945,class_954).
samebib(class_945,class_955).
samebib(class_945,class_956).
samebib(class_945,class_957).
samebib(class_945,class_958).
samebib(class_945,class_959).
samebib(class_945,class_960).
samebib(class_945,class_961).
samebib(class_945,class_962).
samebib(class_945,class_963).
samebib(class_945,class_964).
samebib(class_945,class_965).
samebib(class_945,class_966).
samebib(class_945,class_967).
samebib(class_945,class_968).
samebib(class_945,class_969).
samebib(class_945,class_970).
samebib(class_945,class_971).
samebib(class_945,class_972).
neg(samebib(class_945,class_1025)).
neg(samebib(class_945,class_1026)).
neg(samebib(class_945,class_1027)).
neg(samebib(class_945,class_1028)).
neg(samebib(class_945,class_1030)).
neg(samebib(class_945,class_1031)).
neg(samebib(class_945,class_1032)).
neg(samebib(class_945,class_1033)).
neg(samebib(class_945,class_1044)).
neg(samebib(class_945,class_1045)).
neg(samebib(class_945,class_1046)).
neg(samebib(class_945,class_1047)).
neg(samebib(class_945,class_1048)).
neg(samebib(class_946,class_911)).
neg(samebib(class_946,class_912)).
neg(samebib(class_946,class_913)).
neg(samebib(class_946,class_914)).
neg(samebib(class_946,class_917)).
neg(samebib(class_946,class_918)).
neg(samebib(class_946,class_919)).
neg(samebib(class_946,class_920)).
neg(samebib(class_946,class_921)).
neg(samebib(class_946,class_922)).
neg(samebib(class_946,class_923)).
neg(samebib(class_946,class_924)).
neg(samebib(class_946,class_925)).
neg(samebib(class_946,class_926)).
neg(samebib(class_946,class_927)).
neg(samebib(class_946,class_928)).
neg(samebib(class_946,class_929)).
samebib(class_946,class_930).
samebib(class_946,class_931).
samebib(class_946,class_932).
samebib(class_946,class_933).
samebib(class_946,class_934).
samebib(class_946,class_935).
samebib(class_946,class_936).
samebib(class_946,class_937).
samebib(class_946,class_938).
samebib(class_946,class_939).
samebib(class_946,class_940).
samebib(class_946,class_941).
samebib(class_946,class_942).
samebib(class_946,class_943).
samebib(class_946,class_944).
samebib(class_946,class_945).
samebib(class_946,class_946).
samebib(class_946,class_947).
samebib(class_946,class_948).
samebib(class_946,class_949).
samebib(class_946,class_950).
samebib(class_946,class_951).
samebib(class_946,class_952).
samebib(class_946,class_953).
samebib(class_946,class_954).
samebib(class_946,class_955).
samebib(class_946,class_956).
samebib(class_946,class_957).
samebib(class_946,class_958).
samebib(class_946,class_959).
samebib(class_946,class_960).
samebib(class_946,class_961).
samebib(class_946,class_962).
samebib(class_946,class_963).
samebib(class_946,class_964).
samebib(class_946,class_965).
samebib(class_946,class_966).
samebib(class_946,class_967).
samebib(class_946,class_968).
samebib(class_946,class_969).
samebib(class_946,class_970).
samebib(class_946,class_971).
samebib(class_946,class_972).
neg(samebib(class_946,class_1025)).
neg(samebib(class_946,class_1026)).
neg(samebib(class_946,class_1027)).
neg(samebib(class_946,class_1028)).
neg(samebib(class_946,class_1030)).
neg(samebib(class_946,class_1031)).
neg(samebib(class_946,class_1032)).
neg(samebib(class_946,class_1033)).
neg(samebib(class_946,class_1044)).
neg(samebib(class_946,class_1045)).
neg(samebib(class_946,class_1046)).
neg(samebib(class_946,class_1047)).
neg(samebib(class_946,class_1048)).
neg(samebib(class_947,class_911)).
neg(samebib(class_947,class_912)).
neg(samebib(class_947,class_913)).
neg(samebib(class_947,class_914)).
neg(samebib(class_947,class_917)).
neg(samebib(class_947,class_918)).
neg(samebib(class_947,class_919)).
neg(samebib(class_947,class_920)).
neg(samebib(class_947,class_921)).
neg(samebib(class_947,class_922)).
neg(samebib(class_947,class_923)).
neg(samebib(class_947,class_924)).
neg(samebib(class_947,class_925)).
neg(samebib(class_947,class_926)).
neg(samebib(class_947,class_927)).
neg(samebib(class_947,class_928)).
neg(samebib(class_947,class_929)).
samebib(class_947,class_930).
samebib(class_947,class_931).
samebib(class_947,class_932).
samebib(class_947,class_933).
samebib(class_947,class_934).
samebib(class_947,class_935).
samebib(class_947,class_936).
samebib(class_947,class_937).
samebib(class_947,class_938).
samebib(class_947,class_939).
samebib(class_947,class_940).
samebib(class_947,class_941).
samebib(class_947,class_942).
samebib(class_947,class_943).
samebib(class_947,class_944).
samebib(class_947,class_945).
samebib(class_947,class_946).
samebib(class_947,class_947).
samebib(class_947,class_948).
samebib(class_947,class_949).
samebib(class_947,class_950).
samebib(class_947,class_951).
samebib(class_947,class_952).
samebib(class_947,class_953).
samebib(class_947,class_954).
samebib(class_947,class_955).
samebib(class_947,class_956).
samebib(class_947,class_957).
samebib(class_947,class_958).
samebib(class_947,class_959).
samebib(class_947,class_960).
samebib(class_947,class_961).
samebib(class_947,class_962).
samebib(class_947,class_963).
samebib(class_947,class_964).
samebib(class_947,class_965).
samebib(class_947,class_966).
samebib(class_947,class_967).
samebib(class_947,class_968).
samebib(class_947,class_969).
samebib(class_947,class_970).
samebib(class_947,class_971).
samebib(class_947,class_972).
neg(samebib(class_947,class_1025)).
neg(samebib(class_947,class_1026)).
neg(samebib(class_947,class_1027)).
neg(samebib(class_947,class_1028)).
neg(samebib(class_947,class_1030)).
neg(samebib(class_947,class_1031)).
neg(samebib(class_947,class_1032)).
neg(samebib(class_947,class_1033)).
neg(samebib(class_947,class_1044)).
neg(samebib(class_947,class_1045)).
neg(samebib(class_947,class_1046)).
neg(samebib(class_947,class_1047)).
neg(samebib(class_947,class_1048)).
neg(samebib(class_948,class_911)).
neg(samebib(class_948,class_912)).
neg(samebib(class_948,class_913)).
neg(samebib(class_948,class_914)).
neg(samebib(class_948,class_917)).
neg(samebib(class_948,class_918)).
neg(samebib(class_948,class_919)).
neg(samebib(class_948,class_920)).
neg(samebib(class_948,class_921)).
neg(samebib(class_948,class_922)).
neg(samebib(class_948,class_923)).
neg(samebib(class_948,class_924)).
neg(samebib(class_948,class_925)).
neg(samebib(class_948,class_926)).
neg(samebib(class_948,class_927)).
neg(samebib(class_948,class_928)).
neg(samebib(class_948,class_929)).
samebib(class_948,class_930).
samebib(class_948,class_931).
samebib(class_948,class_932).
samebib(class_948,class_933).
samebib(class_948,class_934).
samebib(class_948,class_935).
samebib(class_948,class_936).
samebib(class_948,class_937).
samebib(class_948,class_938).
samebib(class_948,class_939).
samebib(class_948,class_940).
samebib(class_948,class_941).
samebib(class_948,class_942).
samebib(class_948,class_943).
samebib(class_948,class_944).
samebib(class_948,class_945).
samebib(class_948,class_946).
samebib(class_948,class_947).
samebib(class_948,class_948).
samebib(class_948,class_949).
samebib(class_948,class_950).
samebib(class_948,class_951).
samebib(class_948,class_952).
samebib(class_948,class_953).
samebib(class_948,class_954).
samebib(class_948,class_955).
samebib(class_948,class_956).
samebib(class_948,class_957).
samebib(class_948,class_958).
samebib(class_948,class_959).
samebib(class_948,class_960).
samebib(class_948,class_961).
samebib(class_948,class_962).
samebib(class_948,class_963).
samebib(class_948,class_964).
samebib(class_948,class_965).
samebib(class_948,class_966).
samebib(class_948,class_967).
samebib(class_948,class_968).
samebib(class_948,class_969).
samebib(class_948,class_970).
samebib(class_948,class_971).
samebib(class_948,class_972).
neg(samebib(class_948,class_1025)).
neg(samebib(class_948,class_1026)).
neg(samebib(class_948,class_1027)).
neg(samebib(class_948,class_1028)).
neg(samebib(class_948,class_1030)).
neg(samebib(class_948,class_1031)).
neg(samebib(class_948,class_1032)).
neg(samebib(class_948,class_1033)).
neg(samebib(class_948,class_1044)).
neg(samebib(class_948,class_1045)).
neg(samebib(class_948,class_1046)).
neg(samebib(class_948,class_1047)).
neg(samebib(class_948,class_1048)).
neg(samebib(class_949,class_911)).
neg(samebib(class_949,class_912)).
neg(samebib(class_949,class_913)).
neg(samebib(class_949,class_914)).
neg(samebib(class_949,class_917)).
neg(samebib(class_949,class_918)).
neg(samebib(class_949,class_919)).
neg(samebib(class_949,class_920)).
neg(samebib(class_949,class_921)).
neg(samebib(class_949,class_922)).
neg(samebib(class_949,class_923)).
neg(samebib(class_949,class_924)).
neg(samebib(class_949,class_925)).
neg(samebib(class_949,class_926)).
neg(samebib(class_949,class_927)).
neg(samebib(class_949,class_928)).
neg(samebib(class_949,class_929)).
samebib(class_949,class_930).
samebib(class_949,class_931).
samebib(class_949,class_932).
samebib(class_949,class_933).
samebib(class_949,class_934).
samebib(class_949,class_935).
samebib(class_949,class_936).
samebib(class_949,class_937).
samebib(class_949,class_938).
samebib(class_949,class_939).
samebib(class_949,class_940).
samebib(class_949,class_941).
samebib(class_949,class_942).
samebib(class_949,class_943).
samebib(class_949,class_944).
samebib(class_949,class_945).
samebib(class_949,class_946).
samebib(class_949,class_947).
samebib(class_949,class_948).
samebib(class_949,class_949).
samebib(class_949,class_950).
samebib(class_949,class_951).
samebib(class_949,class_952).
samebib(class_949,class_953).
samebib(class_949,class_954).
samebib(class_949,class_955).
samebib(class_949,class_956).
samebib(class_949,class_957).
samebib(class_949,class_958).
samebib(class_949,class_959).
samebib(class_949,class_960).
samebib(class_949,class_961).
samebib(class_949,class_962).
samebib(class_949,class_963).
samebib(class_949,class_964).
samebib(class_949,class_965).
samebib(class_949,class_966).
samebib(class_949,class_967).
samebib(class_949,class_968).
samebib(class_949,class_969).
samebib(class_949,class_970).
samebib(class_949,class_971).
samebib(class_949,class_972).
neg(samebib(class_949,class_1025)).
neg(samebib(class_949,class_1026)).
neg(samebib(class_949,class_1027)).
neg(samebib(class_949,class_1028)).
neg(samebib(class_949,class_1030)).
neg(samebib(class_949,class_1031)).
neg(samebib(class_949,class_1032)).
neg(samebib(class_949,class_1033)).
neg(samebib(class_949,class_1044)).
neg(samebib(class_949,class_1045)).
neg(samebib(class_949,class_1046)).
neg(samebib(class_949,class_1047)).
neg(samebib(class_949,class_1048)).
neg(samebib(class_950,class_911)).
neg(samebib(class_950,class_912)).
neg(samebib(class_950,class_913)).
neg(samebib(class_950,class_914)).
neg(samebib(class_950,class_917)).
neg(samebib(class_950,class_918)).
neg(samebib(class_950,class_919)).
neg(samebib(class_950,class_920)).
neg(samebib(class_950,class_921)).
neg(samebib(class_950,class_922)).
neg(samebib(class_950,class_923)).
neg(samebib(class_950,class_924)).
neg(samebib(class_950,class_925)).
neg(samebib(class_950,class_926)).
neg(samebib(class_950,class_927)).
neg(samebib(class_950,class_928)).
neg(samebib(class_950,class_929)).
samebib(class_950,class_930).
samebib(class_950,class_931).
samebib(class_950,class_932).
samebib(class_950,class_933).
samebib(class_950,class_934).
samebib(class_950,class_935).
samebib(class_950,class_936).
samebib(class_950,class_937).
samebib(class_950,class_938).
samebib(class_950,class_939).
samebib(class_950,class_940).
samebib(class_950,class_941).
samebib(class_950,class_942).
samebib(class_950,class_943).
samebib(class_950,class_944).
samebib(class_950,class_945).
samebib(class_950,class_946).
samebib(class_950,class_947).
samebib(class_950,class_948).
samebib(class_950,class_949).
samebib(class_950,class_950).
samebib(class_950,class_951).
samebib(class_950,class_952).
samebib(class_950,class_953).
samebib(class_950,class_954).
samebib(class_950,class_955).
samebib(class_950,class_956).
samebib(class_950,class_957).
samebib(class_950,class_958).
samebib(class_950,class_959).
samebib(class_950,class_960).
samebib(class_950,class_961).
samebib(class_950,class_962).
samebib(class_950,class_963).
samebib(class_950,class_964).
samebib(class_950,class_965).
samebib(class_950,class_966).
samebib(class_950,class_967).
samebib(class_950,class_968).
samebib(class_950,class_969).
samebib(class_950,class_970).
samebib(class_950,class_971).
samebib(class_950,class_972).
neg(samebib(class_950,class_1025)).
neg(samebib(class_950,class_1026)).
neg(samebib(class_950,class_1027)).
neg(samebib(class_950,class_1028)).
neg(samebib(class_950,class_1030)).
neg(samebib(class_950,class_1031)).
neg(samebib(class_950,class_1032)).
neg(samebib(class_950,class_1033)).
neg(samebib(class_950,class_1044)).
neg(samebib(class_950,class_1045)).
neg(samebib(class_950,class_1046)).
neg(samebib(class_950,class_1047)).
neg(samebib(class_950,class_1048)).
neg(samebib(class_951,class_911)).
neg(samebib(class_951,class_912)).
neg(samebib(class_951,class_913)).
neg(samebib(class_951,class_914)).
neg(samebib(class_951,class_917)).
neg(samebib(class_951,class_918)).
neg(samebib(class_951,class_919)).
neg(samebib(class_951,class_920)).
neg(samebib(class_951,class_921)).
neg(samebib(class_951,class_922)).
neg(samebib(class_951,class_923)).
neg(samebib(class_951,class_924)).
neg(samebib(class_951,class_925)).
neg(samebib(class_951,class_926)).
neg(samebib(class_951,class_927)).
neg(samebib(class_951,class_928)).
neg(samebib(class_951,class_929)).
samebib(class_951,class_930).
samebib(class_951,class_931).
samebib(class_951,class_932).
samebib(class_951,class_933).
samebib(class_951,class_934).
samebib(class_951,class_935).
samebib(class_951,class_936).
samebib(class_951,class_937).
samebib(class_951,class_938).
samebib(class_951,class_939).
samebib(class_951,class_940).
samebib(class_951,class_941).
samebib(class_951,class_942).
samebib(class_951,class_943).
samebib(class_951,class_944).
samebib(class_951,class_945).
samebib(class_951,class_946).
samebib(class_951,class_947).
samebib(class_951,class_948).
samebib(class_951,class_949).
samebib(class_951,class_950).
samebib(class_951,class_951).
samebib(class_951,class_952).
samebib(class_951,class_953).
samebib(class_951,class_954).
samebib(class_951,class_955).
samebib(class_951,class_956).
samebib(class_951,class_957).
samebib(class_951,class_958).
samebib(class_951,class_959).
samebib(class_951,class_960).
samebib(class_951,class_961).
samebib(class_951,class_962).
samebib(class_951,class_963).
samebib(class_951,class_964).
samebib(class_951,class_965).
samebib(class_951,class_966).
samebib(class_951,class_967).
samebib(class_951,class_968).
samebib(class_951,class_969).
samebib(class_951,class_970).
samebib(class_951,class_971).
samebib(class_951,class_972).
neg(samebib(class_951,class_1025)).
neg(samebib(class_951,class_1026)).
neg(samebib(class_951,class_1027)).
neg(samebib(class_951,class_1028)).
neg(samebib(class_951,class_1030)).
neg(samebib(class_951,class_1031)).
neg(samebib(class_951,class_1032)).
neg(samebib(class_951,class_1033)).
neg(samebib(class_951,class_1044)).
neg(samebib(class_951,class_1045)).
neg(samebib(class_951,class_1046)).
neg(samebib(class_951,class_1047)).
neg(samebib(class_951,class_1048)).
neg(samebib(class_952,class_911)).
neg(samebib(class_952,class_912)).
neg(samebib(class_952,class_913)).
neg(samebib(class_952,class_914)).
neg(samebib(class_952,class_917)).
neg(samebib(class_952,class_918)).
neg(samebib(class_952,class_919)).
neg(samebib(class_952,class_920)).
neg(samebib(class_952,class_921)).
neg(samebib(class_952,class_922)).
neg(samebib(class_952,class_923)).
neg(samebib(class_952,class_924)).
neg(samebib(class_952,class_925)).
neg(samebib(class_952,class_926)).
neg(samebib(class_952,class_927)).
neg(samebib(class_952,class_928)).
neg(samebib(class_952,class_929)).
samebib(class_952,class_930).
samebib(class_952,class_931).
samebib(class_952,class_932).
samebib(class_952,class_933).
samebib(class_952,class_934).
samebib(class_952,class_935).
samebib(class_952,class_936).
samebib(class_952,class_937).
samebib(class_952,class_938).
samebib(class_952,class_939).
samebib(class_952,class_940).
samebib(class_952,class_941).
samebib(class_952,class_942).
samebib(class_952,class_943).
samebib(class_952,class_944).
samebib(class_952,class_945).
samebib(class_952,class_946).
samebib(class_952,class_947).
samebib(class_952,class_948).
samebib(class_952,class_949).
samebib(class_952,class_950).
samebib(class_952,class_951).
samebib(class_952,class_952).
samebib(class_952,class_953).
samebib(class_952,class_954).
samebib(class_952,class_955).
samebib(class_952,class_956).
samebib(class_952,class_957).
samebib(class_952,class_958).
samebib(class_952,class_959).
samebib(class_952,class_960).
samebib(class_952,class_961).
samebib(class_952,class_962).
samebib(class_952,class_963).
samebib(class_952,class_964).
samebib(class_952,class_965).
samebib(class_952,class_966).
samebib(class_952,class_967).
samebib(class_952,class_968).
samebib(class_952,class_969).
samebib(class_952,class_970).
samebib(class_952,class_971).
samebib(class_952,class_972).
neg(samebib(class_952,class_1025)).
neg(samebib(class_952,class_1026)).
neg(samebib(class_952,class_1027)).
neg(samebib(class_952,class_1028)).
neg(samebib(class_952,class_1030)).
neg(samebib(class_952,class_1031)).
neg(samebib(class_952,class_1032)).
neg(samebib(class_952,class_1033)).
neg(samebib(class_952,class_1044)).
neg(samebib(class_952,class_1045)).
neg(samebib(class_952,class_1046)).
neg(samebib(class_952,class_1047)).
neg(samebib(class_952,class_1048)).
neg(samebib(class_953,class_911)).
neg(samebib(class_953,class_912)).
neg(samebib(class_953,class_913)).
neg(samebib(class_953,class_914)).
neg(samebib(class_953,class_917)).
neg(samebib(class_953,class_918)).
neg(samebib(class_953,class_919)).
neg(samebib(class_953,class_920)).
neg(samebib(class_953,class_921)).
neg(samebib(class_953,class_922)).
neg(samebib(class_953,class_923)).
neg(samebib(class_953,class_924)).
neg(samebib(class_953,class_925)).
neg(samebib(class_953,class_926)).
neg(samebib(class_953,class_927)).
neg(samebib(class_953,class_928)).
neg(samebib(class_953,class_929)).
samebib(class_953,class_930).
samebib(class_953,class_931).
samebib(class_953,class_932).
samebib(class_953,class_933).
samebib(class_953,class_934).
samebib(class_953,class_935).
samebib(class_953,class_936).
samebib(class_953,class_937).
samebib(class_953,class_938).
samebib(class_953,class_939).
samebib(class_953,class_940).
samebib(class_953,class_941).
samebib(class_953,class_942).
samebib(class_953,class_943).
samebib(class_953,class_944).
samebib(class_953,class_945).
samebib(class_953,class_946).
samebib(class_953,class_947).
samebib(class_953,class_948).
samebib(class_953,class_949).
samebib(class_953,class_950).
samebib(class_953,class_951).
samebib(class_953,class_952).
samebib(class_953,class_953).
samebib(class_953,class_954).
samebib(class_953,class_955).
samebib(class_953,class_956).
samebib(class_953,class_957).
samebib(class_953,class_958).
samebib(class_953,class_959).
samebib(class_953,class_960).
samebib(class_953,class_961).
samebib(class_953,class_962).
samebib(class_953,class_963).
samebib(class_953,class_964).
samebib(class_953,class_965).
samebib(class_953,class_966).
samebib(class_953,class_967).
samebib(class_953,class_968).
samebib(class_953,class_969).
samebib(class_953,class_970).
samebib(class_953,class_971).
samebib(class_953,class_972).
neg(samebib(class_953,class_1025)).
neg(samebib(class_953,class_1026)).
neg(samebib(class_953,class_1027)).
neg(samebib(class_953,class_1028)).
neg(samebib(class_953,class_1030)).
neg(samebib(class_953,class_1031)).
neg(samebib(class_953,class_1032)).
neg(samebib(class_953,class_1033)).
neg(samebib(class_953,class_1044)).
neg(samebib(class_953,class_1045)).
neg(samebib(class_953,class_1046)).
neg(samebib(class_953,class_1047)).
neg(samebib(class_953,class_1048)).
neg(samebib(class_954,class_911)).
neg(samebib(class_954,class_912)).
neg(samebib(class_954,class_913)).
neg(samebib(class_954,class_914)).
neg(samebib(class_954,class_917)).
neg(samebib(class_954,class_918)).
neg(samebib(class_954,class_919)).
neg(samebib(class_954,class_920)).
neg(samebib(class_954,class_921)).
neg(samebib(class_954,class_922)).
neg(samebib(class_954,class_923)).
neg(samebib(class_954,class_924)).
neg(samebib(class_954,class_925)).
neg(samebib(class_954,class_926)).
neg(samebib(class_954,class_927)).
neg(samebib(class_954,class_928)).
neg(samebib(class_954,class_929)).
samebib(class_954,class_930).
samebib(class_954,class_931).
samebib(class_954,class_932).
samebib(class_954,class_933).
samebib(class_954,class_934).
samebib(class_954,class_935).
samebib(class_954,class_936).
samebib(class_954,class_937).
samebib(class_954,class_938).
samebib(class_954,class_939).
samebib(class_954,class_940).
samebib(class_954,class_941).
samebib(class_954,class_942).
samebib(class_954,class_943).
samebib(class_954,class_944).
samebib(class_954,class_945).
samebib(class_954,class_946).
samebib(class_954,class_947).
samebib(class_954,class_948).
samebib(class_954,class_949).
samebib(class_954,class_950).
samebib(class_954,class_951).
samebib(class_954,class_952).
samebib(class_954,class_953).
samebib(class_954,class_954).
samebib(class_954,class_955).
samebib(class_954,class_956).
samebib(class_954,class_957).
samebib(class_954,class_958).
samebib(class_954,class_959).
samebib(class_954,class_960).
samebib(class_954,class_961).
samebib(class_954,class_962).
samebib(class_954,class_963).
samebib(class_954,class_964).
samebib(class_954,class_965).
samebib(class_954,class_966).
samebib(class_954,class_967).
samebib(class_954,class_968).
samebib(class_954,class_969).
samebib(class_954,class_970).
samebib(class_954,class_971).
samebib(class_954,class_972).
neg(samebib(class_954,class_1025)).
neg(samebib(class_954,class_1026)).
neg(samebib(class_954,class_1027)).
neg(samebib(class_954,class_1028)).
neg(samebib(class_954,class_1030)).
neg(samebib(class_954,class_1031)).
neg(samebib(class_954,class_1032)).
neg(samebib(class_954,class_1033)).
neg(samebib(class_954,class_1044)).
neg(samebib(class_954,class_1045)).
neg(samebib(class_954,class_1046)).
neg(samebib(class_954,class_1047)).
neg(samebib(class_954,class_1048)).
neg(samebib(class_955,class_911)).
neg(samebib(class_955,class_912)).
neg(samebib(class_955,class_913)).
neg(samebib(class_955,class_914)).
neg(samebib(class_955,class_917)).
neg(samebib(class_955,class_918)).
neg(samebib(class_955,class_919)).
neg(samebib(class_955,class_920)).
neg(samebib(class_955,class_921)).
neg(samebib(class_955,class_922)).
neg(samebib(class_955,class_923)).
neg(samebib(class_955,class_924)).
neg(samebib(class_955,class_925)).
neg(samebib(class_955,class_926)).
neg(samebib(class_955,class_927)).
neg(samebib(class_955,class_928)).
neg(samebib(class_955,class_929)).
samebib(class_955,class_930).
samebib(class_955,class_931).
samebib(class_955,class_932).
samebib(class_955,class_933).
samebib(class_955,class_934).
samebib(class_955,class_935).
samebib(class_955,class_936).
samebib(class_955,class_937).
samebib(class_955,class_938).
samebib(class_955,class_939).
samebib(class_955,class_940).
samebib(class_955,class_941).
samebib(class_955,class_942).
samebib(class_955,class_943).
samebib(class_955,class_944).
samebib(class_955,class_945).
samebib(class_955,class_946).
samebib(class_955,class_947).
samebib(class_955,class_948).
samebib(class_955,class_949).
samebib(class_955,class_950).
samebib(class_955,class_951).
samebib(class_955,class_952).
samebib(class_955,class_953).
samebib(class_955,class_954).
samebib(class_955,class_955).
samebib(class_955,class_956).
samebib(class_955,class_957).
samebib(class_955,class_958).
samebib(class_955,class_959).
samebib(class_955,class_960).
samebib(class_955,class_961).
samebib(class_955,class_962).
samebib(class_955,class_963).
samebib(class_955,class_964).
samebib(class_955,class_965).
samebib(class_955,class_966).
samebib(class_955,class_967).
samebib(class_955,class_968).
samebib(class_955,class_969).
samebib(class_955,class_970).
samebib(class_955,class_971).
samebib(class_955,class_972).
neg(samebib(class_955,class_1025)).
neg(samebib(class_955,class_1026)).
neg(samebib(class_955,class_1027)).
neg(samebib(class_955,class_1028)).
neg(samebib(class_955,class_1030)).
neg(samebib(class_955,class_1031)).
neg(samebib(class_955,class_1032)).
neg(samebib(class_955,class_1033)).
neg(samebib(class_955,class_1044)).
neg(samebib(class_955,class_1045)).
neg(samebib(class_955,class_1046)).
neg(samebib(class_955,class_1047)).
neg(samebib(class_955,class_1048)).
neg(samebib(class_956,class_911)).
neg(samebib(class_956,class_912)).
neg(samebib(class_956,class_913)).
neg(samebib(class_956,class_914)).
neg(samebib(class_956,class_917)).
neg(samebib(class_956,class_918)).
neg(samebib(class_956,class_919)).
neg(samebib(class_956,class_920)).
neg(samebib(class_956,class_921)).
neg(samebib(class_956,class_922)).
neg(samebib(class_956,class_923)).
neg(samebib(class_956,class_924)).
neg(samebib(class_956,class_925)).
neg(samebib(class_956,class_926)).
neg(samebib(class_956,class_927)).
neg(samebib(class_956,class_928)).
neg(samebib(class_956,class_929)).
samebib(class_956,class_930).
samebib(class_956,class_931).
samebib(class_956,class_932).
samebib(class_956,class_933).
samebib(class_956,class_934).
samebib(class_956,class_935).
samebib(class_956,class_936).
samebib(class_956,class_937).
samebib(class_956,class_938).
samebib(class_956,class_939).
samebib(class_956,class_940).
samebib(class_956,class_941).
samebib(class_956,class_942).
samebib(class_956,class_943).
samebib(class_956,class_944).
samebib(class_956,class_945).
samebib(class_956,class_946).
samebib(class_956,class_947).
samebib(class_956,class_948).
samebib(class_956,class_949).
samebib(class_956,class_950).
samebib(class_956,class_951).
samebib(class_956,class_952).
samebib(class_956,class_953).
samebib(class_956,class_954).
samebib(class_956,class_955).
samebib(class_956,class_956).
samebib(class_956,class_957).
samebib(class_956,class_958).
samebib(class_956,class_959).
samebib(class_956,class_960).
samebib(class_956,class_961).
samebib(class_956,class_962).
samebib(class_956,class_963).
samebib(class_956,class_964).
samebib(class_956,class_965).
samebib(class_956,class_966).
samebib(class_956,class_967).
samebib(class_956,class_968).
samebib(class_956,class_969).
samebib(class_956,class_970).
samebib(class_956,class_971).
samebib(class_956,class_972).
neg(samebib(class_956,class_1025)).
neg(samebib(class_956,class_1026)).
neg(samebib(class_956,class_1027)).
neg(samebib(class_956,class_1028)).
neg(samebib(class_956,class_1030)).
neg(samebib(class_956,class_1031)).
neg(samebib(class_956,class_1032)).
neg(samebib(class_956,class_1033)).
neg(samebib(class_956,class_1044)).
neg(samebib(class_956,class_1045)).
neg(samebib(class_956,class_1046)).
neg(samebib(class_956,class_1047)).
neg(samebib(class_956,class_1048)).
samebib(class_957,class_930).
samebib(class_957,class_931).
samebib(class_957,class_932).
samebib(class_957,class_933).
samebib(class_957,class_934).
samebib(class_957,class_935).
samebib(class_957,class_936).
samebib(class_957,class_937).
samebib(class_957,class_938).
samebib(class_957,class_939).
samebib(class_957,class_940).
samebib(class_957,class_941).
samebib(class_957,class_942).
samebib(class_957,class_943).
samebib(class_957,class_944).
samebib(class_957,class_945).
samebib(class_957,class_946).
samebib(class_957,class_947).
samebib(class_957,class_948).
samebib(class_957,class_949).
samebib(class_957,class_950).
samebib(class_957,class_951).
samebib(class_957,class_952).
samebib(class_957,class_953).
samebib(class_957,class_954).
samebib(class_957,class_955).
samebib(class_957,class_956).
samebib(class_957,class_957).
samebib(class_957,class_958).
samebib(class_957,class_959).
samebib(class_957,class_960).
samebib(class_957,class_961).
samebib(class_957,class_962).
samebib(class_957,class_963).
samebib(class_957,class_964).
samebib(class_957,class_965).
samebib(class_957,class_966).
samebib(class_957,class_967).
samebib(class_957,class_968).
samebib(class_957,class_969).
samebib(class_957,class_970).
samebib(class_957,class_971).
samebib(class_957,class_972).
neg(samebib(class_957,class_1025)).
neg(samebib(class_957,class_1026)).
neg(samebib(class_957,class_1027)).
neg(samebib(class_957,class_1028)).
neg(samebib(class_957,class_1030)).
neg(samebib(class_957,class_1031)).
neg(samebib(class_957,class_1032)).
neg(samebib(class_957,class_1033)).
neg(samebib(class_957,class_1044)).
neg(samebib(class_957,class_1045)).
neg(samebib(class_957,class_1046)).
neg(samebib(class_957,class_1047)).
neg(samebib(class_957,class_1048)).
neg(samebib(class_958,class_911)).
neg(samebib(class_958,class_912)).
neg(samebib(class_958,class_913)).
neg(samebib(class_958,class_914)).
neg(samebib(class_958,class_917)).
neg(samebib(class_958,class_918)).
neg(samebib(class_958,class_919)).
neg(samebib(class_958,class_920)).
neg(samebib(class_958,class_921)).
neg(samebib(class_958,class_922)).
neg(samebib(class_958,class_923)).
neg(samebib(class_958,class_924)).
neg(samebib(class_958,class_925)).
neg(samebib(class_958,class_926)).
neg(samebib(class_958,class_927)).
neg(samebib(class_958,class_928)).
neg(samebib(class_958,class_929)).
samebib(class_958,class_930).
samebib(class_958,class_931).
samebib(class_958,class_932).
samebib(class_958,class_933).
samebib(class_958,class_934).
samebib(class_958,class_935).
samebib(class_958,class_936).
samebib(class_958,class_937).
samebib(class_958,class_938).
samebib(class_958,class_939).
samebib(class_958,class_940).
samebib(class_958,class_941).
samebib(class_958,class_942).
samebib(class_958,class_943).
samebib(class_958,class_944).
samebib(class_958,class_945).
samebib(class_958,class_946).
samebib(class_958,class_947).
samebib(class_958,class_948).
samebib(class_958,class_949).
samebib(class_958,class_950).
samebib(class_958,class_951).
samebib(class_958,class_952).
samebib(class_958,class_953).
samebib(class_958,class_954).
samebib(class_958,class_955).
samebib(class_958,class_956).
samebib(class_958,class_957).
samebib(class_958,class_958).
samebib(class_958,class_959).
samebib(class_958,class_960).
samebib(class_958,class_961).
samebib(class_958,class_962).
samebib(class_958,class_963).
samebib(class_958,class_964).
samebib(class_958,class_965).
samebib(class_958,class_966).
samebib(class_958,class_967).
samebib(class_958,class_968).
samebib(class_958,class_969).
samebib(class_958,class_970).
samebib(class_958,class_971).
samebib(class_958,class_972).
neg(samebib(class_958,class_1025)).
neg(samebib(class_958,class_1026)).
neg(samebib(class_958,class_1027)).
neg(samebib(class_958,class_1028)).
neg(samebib(class_958,class_1030)).
neg(samebib(class_958,class_1031)).
neg(samebib(class_958,class_1032)).
neg(samebib(class_958,class_1033)).
neg(samebib(class_958,class_1044)).
neg(samebib(class_958,class_1045)).
neg(samebib(class_958,class_1046)).
neg(samebib(class_958,class_1047)).
neg(samebib(class_958,class_1048)).
neg(samebib(class_959,class_911)).
neg(samebib(class_959,class_912)).
neg(samebib(class_959,class_913)).
neg(samebib(class_959,class_914)).
neg(samebib(class_959,class_917)).
neg(samebib(class_959,class_918)).
neg(samebib(class_959,class_919)).
neg(samebib(class_959,class_920)).
neg(samebib(class_959,class_921)).
neg(samebib(class_959,class_922)).
neg(samebib(class_959,class_923)).
neg(samebib(class_959,class_924)).
neg(samebib(class_959,class_925)).
neg(samebib(class_959,class_926)).
neg(samebib(class_959,class_927)).
neg(samebib(class_959,class_928)).
neg(samebib(class_959,class_929)).
samebib(class_959,class_930).
samebib(class_959,class_931).
samebib(class_959,class_932).
samebib(class_959,class_933).
samebib(class_959,class_934).
samebib(class_959,class_935).
samebib(class_959,class_936).
samebib(class_959,class_937).
samebib(class_959,class_938).
samebib(class_959,class_939).
samebib(class_959,class_940).
samebib(class_959,class_941).
samebib(class_959,class_942).
samebib(class_959,class_943).
samebib(class_959,class_944).
samebib(class_959,class_945).
samebib(class_959,class_946).
samebib(class_959,class_947).
samebib(class_959,class_948).
samebib(class_959,class_949).
samebib(class_959,class_950).
samebib(class_959,class_951).
samebib(class_959,class_952).
samebib(class_959,class_953).
samebib(class_959,class_954).
samebib(class_959,class_955).
samebib(class_959,class_956).
samebib(class_959,class_957).
samebib(class_959,class_958).
samebib(class_959,class_959).
samebib(class_959,class_960).
samebib(class_959,class_961).
samebib(class_959,class_962).
samebib(class_959,class_963).
samebib(class_959,class_964).
samebib(class_959,class_965).
samebib(class_959,class_966).
samebib(class_959,class_967).
samebib(class_959,class_968).
samebib(class_959,class_969).
samebib(class_959,class_970).
samebib(class_959,class_971).
samebib(class_959,class_972).
neg(samebib(class_959,class_1025)).
neg(samebib(class_959,class_1026)).
neg(samebib(class_959,class_1027)).
neg(samebib(class_959,class_1028)).
neg(samebib(class_959,class_1030)).
neg(samebib(class_959,class_1031)).
neg(samebib(class_959,class_1032)).
neg(samebib(class_959,class_1033)).
neg(samebib(class_959,class_1044)).
neg(samebib(class_959,class_1045)).
neg(samebib(class_959,class_1046)).
neg(samebib(class_959,class_1047)).
neg(samebib(class_959,class_1048)).
neg(samebib(class_960,class_911)).
neg(samebib(class_960,class_912)).
neg(samebib(class_960,class_913)).
neg(samebib(class_960,class_914)).
neg(samebib(class_960,class_917)).
neg(samebib(class_960,class_918)).
neg(samebib(class_960,class_919)).
neg(samebib(class_960,class_920)).
neg(samebib(class_960,class_921)).
neg(samebib(class_960,class_922)).
neg(samebib(class_960,class_923)).
neg(samebib(class_960,class_924)).
neg(samebib(class_960,class_925)).
neg(samebib(class_960,class_926)).
neg(samebib(class_960,class_927)).
neg(samebib(class_960,class_928)).
neg(samebib(class_960,class_929)).
samebib(class_960,class_930).
samebib(class_960,class_931).
samebib(class_960,class_932).
samebib(class_960,class_933).
samebib(class_960,class_934).
samebib(class_960,class_935).
samebib(class_960,class_936).
samebib(class_960,class_937).
samebib(class_960,class_938).
samebib(class_960,class_939).
samebib(class_960,class_940).
samebib(class_960,class_941).
samebib(class_960,class_942).
samebib(class_960,class_943).
samebib(class_960,class_944).
samebib(class_960,class_945).
samebib(class_960,class_946).
samebib(class_960,class_947).
samebib(class_960,class_948).
samebib(class_960,class_949).
samebib(class_960,class_950).
samebib(class_960,class_951).
samebib(class_960,class_952).
samebib(class_960,class_953).
samebib(class_960,class_954).
samebib(class_960,class_955).
samebib(class_960,class_956).
samebib(class_960,class_957).
samebib(class_960,class_958).
samebib(class_960,class_959).
samebib(class_960,class_960).
samebib(class_960,class_961).
samebib(class_960,class_962).
samebib(class_960,class_963).
samebib(class_960,class_964).
samebib(class_960,class_965).
samebib(class_960,class_966).
samebib(class_960,class_967).
samebib(class_960,class_968).
samebib(class_960,class_969).
samebib(class_960,class_970).
samebib(class_960,class_971).
samebib(class_960,class_972).
neg(samebib(class_960,class_1025)).
neg(samebib(class_960,class_1026)).
neg(samebib(class_960,class_1027)).
neg(samebib(class_960,class_1028)).
neg(samebib(class_960,class_1030)).
neg(samebib(class_960,class_1031)).
neg(samebib(class_960,class_1032)).
neg(samebib(class_960,class_1033)).
neg(samebib(class_960,class_1044)).
neg(samebib(class_960,class_1045)).
neg(samebib(class_960,class_1046)).
neg(samebib(class_960,class_1047)).
neg(samebib(class_960,class_1048)).
neg(samebib(class_961,class_911)).
neg(samebib(class_961,class_912)).
neg(samebib(class_961,class_913)).
neg(samebib(class_961,class_914)).
neg(samebib(class_961,class_917)).
neg(samebib(class_961,class_918)).
neg(samebib(class_961,class_919)).
neg(samebib(class_961,class_920)).
neg(samebib(class_961,class_921)).
neg(samebib(class_961,class_922)).
neg(samebib(class_961,class_923)).
neg(samebib(class_961,class_924)).
neg(samebib(class_961,class_925)).
neg(samebib(class_961,class_926)).
neg(samebib(class_961,class_927)).
neg(samebib(class_961,class_928)).
neg(samebib(class_961,class_929)).
samebib(class_961,class_930).
samebib(class_961,class_931).
samebib(class_961,class_932).
samebib(class_961,class_933).
samebib(class_961,class_934).
samebib(class_961,class_935).
samebib(class_961,class_936).
samebib(class_961,class_937).
samebib(class_961,class_938).
samebib(class_961,class_939).
samebib(class_961,class_940).
samebib(class_961,class_941).
samebib(class_961,class_942).
samebib(class_961,class_943).
samebib(class_961,class_944).
samebib(class_961,class_945).
samebib(class_961,class_946).
samebib(class_961,class_947).
samebib(class_961,class_948).
samebib(class_961,class_949).
samebib(class_961,class_950).
samebib(class_961,class_951).
samebib(class_961,class_952).
samebib(class_961,class_953).
samebib(class_961,class_954).
samebib(class_961,class_955).
samebib(class_961,class_956).
samebib(class_961,class_957).
samebib(class_961,class_958).
samebib(class_961,class_959).
samebib(class_961,class_960).
samebib(class_961,class_961).
samebib(class_961,class_962).
samebib(class_961,class_963).
samebib(class_961,class_964).
samebib(class_961,class_965).
samebib(class_961,class_966).
samebib(class_961,class_967).
samebib(class_961,class_968).
samebib(class_961,class_969).
samebib(class_961,class_970).
samebib(class_961,class_971).
samebib(class_961,class_972).
neg(samebib(class_961,class_1025)).
neg(samebib(class_961,class_1026)).
neg(samebib(class_961,class_1027)).
neg(samebib(class_961,class_1028)).
neg(samebib(class_961,class_1030)).
neg(samebib(class_961,class_1031)).
neg(samebib(class_961,class_1032)).
neg(samebib(class_961,class_1033)).
neg(samebib(class_961,class_1044)).
neg(samebib(class_961,class_1045)).
neg(samebib(class_961,class_1046)).
neg(samebib(class_961,class_1047)).
neg(samebib(class_961,class_1048)).
neg(samebib(class_962,class_911)).
neg(samebib(class_962,class_912)).
neg(samebib(class_962,class_913)).
neg(samebib(class_962,class_914)).
neg(samebib(class_962,class_917)).
neg(samebib(class_962,class_918)).
neg(samebib(class_962,class_919)).
neg(samebib(class_962,class_920)).
neg(samebib(class_962,class_921)).
neg(samebib(class_962,class_922)).
neg(samebib(class_962,class_923)).
neg(samebib(class_962,class_924)).
neg(samebib(class_962,class_925)).
neg(samebib(class_962,class_926)).
neg(samebib(class_962,class_927)).
neg(samebib(class_962,class_928)).
neg(samebib(class_962,class_929)).
samebib(class_962,class_930).
samebib(class_962,class_931).
samebib(class_962,class_932).
samebib(class_962,class_933).
samebib(class_962,class_934).
samebib(class_962,class_935).
samebib(class_962,class_936).
samebib(class_962,class_937).
samebib(class_962,class_938).
samebib(class_962,class_939).
samebib(class_962,class_940).
samebib(class_962,class_941).
samebib(class_962,class_942).
samebib(class_962,class_943).
samebib(class_962,class_944).
samebib(class_962,class_945).
samebib(class_962,class_946).
samebib(class_962,class_947).
samebib(class_962,class_948).
samebib(class_962,class_949).
samebib(class_962,class_950).
samebib(class_962,class_951).
samebib(class_962,class_952).
samebib(class_962,class_953).
samebib(class_962,class_954).
samebib(class_962,class_955).
samebib(class_962,class_956).
samebib(class_962,class_957).
samebib(class_962,class_958).
samebib(class_962,class_959).
samebib(class_962,class_960).
samebib(class_962,class_961).
samebib(class_962,class_962).
samebib(class_962,class_963).
samebib(class_962,class_964).
samebib(class_962,class_965).
samebib(class_962,class_966).
samebib(class_962,class_967).
samebib(class_962,class_968).
samebib(class_962,class_969).
samebib(class_962,class_970).
samebib(class_962,class_971).
samebib(class_962,class_972).
neg(samebib(class_962,class_1025)).
neg(samebib(class_962,class_1026)).
neg(samebib(class_962,class_1027)).
neg(samebib(class_962,class_1028)).
neg(samebib(class_962,class_1030)).
neg(samebib(class_962,class_1031)).
neg(samebib(class_962,class_1032)).
neg(samebib(class_962,class_1033)).
neg(samebib(class_962,class_1044)).
neg(samebib(class_962,class_1045)).
neg(samebib(class_962,class_1046)).
neg(samebib(class_962,class_1047)).
neg(samebib(class_962,class_1048)).
neg(samebib(class_963,class_911)).
neg(samebib(class_963,class_912)).
neg(samebib(class_963,class_913)).
neg(samebib(class_963,class_914)).
neg(samebib(class_963,class_917)).
neg(samebib(class_963,class_918)).
neg(samebib(class_963,class_919)).
neg(samebib(class_963,class_920)).
neg(samebib(class_963,class_921)).
neg(samebib(class_963,class_922)).
neg(samebib(class_963,class_923)).
neg(samebib(class_963,class_924)).
neg(samebib(class_963,class_925)).
neg(samebib(class_963,class_926)).
neg(samebib(class_963,class_927)).
neg(samebib(class_963,class_928)).
neg(samebib(class_963,class_929)).
samebib(class_963,class_930).
samebib(class_963,class_931).
samebib(class_963,class_932).
samebib(class_963,class_933).
samebib(class_963,class_934).
samebib(class_963,class_935).
samebib(class_963,class_936).
samebib(class_963,class_937).
samebib(class_963,class_938).
samebib(class_963,class_939).
samebib(class_963,class_940).
samebib(class_963,class_941).
samebib(class_963,class_942).
samebib(class_963,class_943).
samebib(class_963,class_944).
samebib(class_963,class_945).
samebib(class_963,class_946).
samebib(class_963,class_947).
samebib(class_963,class_948).
samebib(class_963,class_949).
samebib(class_963,class_950).
samebib(class_963,class_951).
samebib(class_963,class_952).
samebib(class_963,class_953).
samebib(class_963,class_954).
samebib(class_963,class_955).
samebib(class_963,class_956).
samebib(class_963,class_957).
samebib(class_963,class_958).
samebib(class_963,class_959).
samebib(class_963,class_960).
samebib(class_963,class_961).
samebib(class_963,class_962).
samebib(class_963,class_963).
samebib(class_963,class_964).
samebib(class_963,class_965).
samebib(class_963,class_966).
samebib(class_963,class_967).
samebib(class_963,class_968).
samebib(class_963,class_969).
samebib(class_963,class_970).
samebib(class_963,class_971).
samebib(class_963,class_972).
neg(samebib(class_963,class_1025)).
neg(samebib(class_963,class_1026)).
neg(samebib(class_963,class_1027)).
neg(samebib(class_963,class_1028)).
neg(samebib(class_963,class_1030)).
neg(samebib(class_963,class_1031)).
neg(samebib(class_963,class_1032)).
neg(samebib(class_963,class_1033)).
neg(samebib(class_963,class_1044)).
neg(samebib(class_963,class_1045)).
neg(samebib(class_963,class_1046)).
neg(samebib(class_963,class_1047)).
neg(samebib(class_963,class_1048)).
neg(samebib(class_964,class_911)).
neg(samebib(class_964,class_912)).
neg(samebib(class_964,class_913)).
neg(samebib(class_964,class_914)).
neg(samebib(class_964,class_917)).
neg(samebib(class_964,class_918)).
neg(samebib(class_964,class_919)).
neg(samebib(class_964,class_920)).
neg(samebib(class_964,class_921)).
neg(samebib(class_964,class_922)).
neg(samebib(class_964,class_923)).
neg(samebib(class_964,class_924)).
neg(samebib(class_964,class_925)).
neg(samebib(class_964,class_926)).
neg(samebib(class_964,class_927)).
neg(samebib(class_964,class_928)).
neg(samebib(class_964,class_929)).
samebib(class_964,class_930).
samebib(class_964,class_931).
samebib(class_964,class_932).
samebib(class_964,class_933).
samebib(class_964,class_934).
samebib(class_964,class_935).
samebib(class_964,class_936).
samebib(class_964,class_937).
samebib(class_964,class_938).
samebib(class_964,class_939).
samebib(class_964,class_940).
samebib(class_964,class_941).
samebib(class_964,class_942).
samebib(class_964,class_943).
samebib(class_964,class_944).
samebib(class_964,class_945).
samebib(class_964,class_946).
samebib(class_964,class_947).
samebib(class_964,class_948).
samebib(class_964,class_949).
samebib(class_964,class_950).
samebib(class_964,class_951).
samebib(class_964,class_952).
samebib(class_964,class_953).
samebib(class_964,class_954).
samebib(class_964,class_955).
samebib(class_964,class_956).
samebib(class_964,class_957).
samebib(class_964,class_958).
samebib(class_964,class_959).
samebib(class_964,class_960).
samebib(class_964,class_961).
samebib(class_964,class_962).
samebib(class_964,class_963).
samebib(class_964,class_964).
samebib(class_964,class_965).
samebib(class_964,class_966).
samebib(class_964,class_967).
samebib(class_964,class_968).
samebib(class_964,class_969).
samebib(class_964,class_970).
samebib(class_964,class_971).
samebib(class_964,class_972).
neg(samebib(class_964,class_1025)).
neg(samebib(class_964,class_1026)).
neg(samebib(class_964,class_1027)).
neg(samebib(class_964,class_1028)).
neg(samebib(class_964,class_1030)).
neg(samebib(class_964,class_1031)).
neg(samebib(class_964,class_1032)).
neg(samebib(class_964,class_1033)).
neg(samebib(class_964,class_1044)).
neg(samebib(class_964,class_1045)).
neg(samebib(class_964,class_1046)).
neg(samebib(class_964,class_1047)).
neg(samebib(class_964,class_1048)).
neg(samebib(class_965,class_911)).
neg(samebib(class_965,class_912)).
neg(samebib(class_965,class_913)).
neg(samebib(class_965,class_914)).
neg(samebib(class_965,class_917)).
neg(samebib(class_965,class_918)).
neg(samebib(class_965,class_919)).
neg(samebib(class_965,class_920)).
neg(samebib(class_965,class_921)).
neg(samebib(class_965,class_922)).
neg(samebib(class_965,class_923)).
neg(samebib(class_965,class_924)).
neg(samebib(class_965,class_925)).
neg(samebib(class_965,class_926)).
neg(samebib(class_965,class_927)).
neg(samebib(class_965,class_928)).
neg(samebib(class_965,class_929)).
samebib(class_965,class_930).
samebib(class_965,class_931).
samebib(class_965,class_932).
samebib(class_965,class_933).
samebib(class_965,class_934).
samebib(class_965,class_935).
samebib(class_965,class_936).
samebib(class_965,class_937).
samebib(class_965,class_938).
samebib(class_965,class_939).
samebib(class_965,class_940).
samebib(class_965,class_941).
samebib(class_965,class_942).
samebib(class_965,class_943).
samebib(class_965,class_944).
samebib(class_965,class_945).
samebib(class_965,class_946).
samebib(class_965,class_947).
samebib(class_965,class_948).
samebib(class_965,class_949).
samebib(class_965,class_950).
samebib(class_965,class_951).
samebib(class_965,class_952).
samebib(class_965,class_953).
samebib(class_965,class_954).
samebib(class_965,class_955).
samebib(class_965,class_956).
samebib(class_965,class_957).
samebib(class_965,class_958).
samebib(class_965,class_959).
samebib(class_965,class_960).
samebib(class_965,class_961).
samebib(class_965,class_962).
samebib(class_965,class_963).
samebib(class_965,class_964).
samebib(class_965,class_965).
samebib(class_965,class_966).
samebib(class_965,class_967).
samebib(class_965,class_968).
samebib(class_965,class_969).
samebib(class_965,class_970).
samebib(class_965,class_971).
samebib(class_965,class_972).
neg(samebib(class_965,class_1025)).
neg(samebib(class_965,class_1026)).
neg(samebib(class_965,class_1027)).
neg(samebib(class_965,class_1028)).
neg(samebib(class_965,class_1030)).
neg(samebib(class_965,class_1031)).
neg(samebib(class_965,class_1032)).
neg(samebib(class_965,class_1033)).
neg(samebib(class_965,class_1044)).
neg(samebib(class_965,class_1045)).
neg(samebib(class_965,class_1046)).
neg(samebib(class_965,class_1047)).
neg(samebib(class_965,class_1048)).
neg(samebib(class_966,class_911)).
neg(samebib(class_966,class_912)).
neg(samebib(class_966,class_913)).
neg(samebib(class_966,class_914)).
neg(samebib(class_966,class_917)).
neg(samebib(class_966,class_918)).
neg(samebib(class_966,class_919)).
neg(samebib(class_966,class_920)).
neg(samebib(class_966,class_921)).
neg(samebib(class_966,class_922)).
neg(samebib(class_966,class_923)).
neg(samebib(class_966,class_924)).
neg(samebib(class_966,class_925)).
neg(samebib(class_966,class_926)).
neg(samebib(class_966,class_927)).
neg(samebib(class_966,class_928)).
neg(samebib(class_966,class_929)).
samebib(class_966,class_930).
samebib(class_966,class_931).
samebib(class_966,class_932).
samebib(class_966,class_933).
samebib(class_966,class_934).
samebib(class_966,class_935).
samebib(class_966,class_936).
samebib(class_966,class_937).
samebib(class_966,class_938).
samebib(class_966,class_939).
samebib(class_966,class_940).
samebib(class_966,class_941).
samebib(class_966,class_942).
samebib(class_966,class_943).
samebib(class_966,class_944).
samebib(class_966,class_945).
samebib(class_966,class_946).
samebib(class_966,class_947).
samebib(class_966,class_948).
samebib(class_966,class_949).
samebib(class_966,class_950).
samebib(class_966,class_951).
samebib(class_966,class_952).
samebib(class_966,class_953).
samebib(class_966,class_954).
samebib(class_966,class_955).
samebib(class_966,class_956).
samebib(class_966,class_957).
samebib(class_966,class_958).
samebib(class_966,class_959).
samebib(class_966,class_960).
samebib(class_966,class_961).
samebib(class_966,class_962).
samebib(class_966,class_963).
samebib(class_966,class_964).
samebib(class_966,class_965).
samebib(class_966,class_966).
samebib(class_966,class_967).
samebib(class_966,class_968).
samebib(class_966,class_969).
samebib(class_966,class_970).
samebib(class_966,class_971).
samebib(class_966,class_972).
neg(samebib(class_966,class_1025)).
neg(samebib(class_966,class_1026)).
neg(samebib(class_966,class_1027)).
neg(samebib(class_966,class_1028)).
neg(samebib(class_966,class_1030)).
neg(samebib(class_966,class_1031)).
neg(samebib(class_966,class_1032)).
neg(samebib(class_966,class_1033)).
neg(samebib(class_966,class_1044)).
neg(samebib(class_966,class_1045)).
neg(samebib(class_966,class_1046)).
neg(samebib(class_966,class_1047)).
neg(samebib(class_966,class_1048)).
neg(samebib(class_967,class_911)).
neg(samebib(class_967,class_912)).
neg(samebib(class_967,class_913)).
neg(samebib(class_967,class_914)).
neg(samebib(class_967,class_917)).
neg(samebib(class_967,class_918)).
neg(samebib(class_967,class_919)).
neg(samebib(class_967,class_920)).
neg(samebib(class_967,class_921)).
neg(samebib(class_967,class_922)).
neg(samebib(class_967,class_923)).
neg(samebib(class_967,class_924)).
neg(samebib(class_967,class_925)).
neg(samebib(class_967,class_926)).
neg(samebib(class_967,class_927)).
neg(samebib(class_967,class_928)).
neg(samebib(class_967,class_929)).
samebib(class_967,class_930).
samebib(class_967,class_931).
samebib(class_967,class_932).
samebib(class_967,class_933).
samebib(class_967,class_934).
samebib(class_967,class_935).
samebib(class_967,class_936).
samebib(class_967,class_937).
samebib(class_967,class_938).
samebib(class_967,class_939).
samebib(class_967,class_940).
samebib(class_967,class_941).
samebib(class_967,class_942).
samebib(class_967,class_943).
samebib(class_967,class_944).
samebib(class_967,class_945).
samebib(class_967,class_946).
samebib(class_967,class_947).
samebib(class_967,class_948).
samebib(class_967,class_949).
samebib(class_967,class_950).
samebib(class_967,class_951).
samebib(class_967,class_952).
samebib(class_967,class_953).
samebib(class_967,class_954).
samebib(class_967,class_955).
samebib(class_967,class_956).
samebib(class_967,class_957).
samebib(class_967,class_958).
samebib(class_967,class_959).
samebib(class_967,class_960).
samebib(class_967,class_961).
samebib(class_967,class_962).
samebib(class_967,class_963).
samebib(class_967,class_964).
samebib(class_967,class_965).
samebib(class_967,class_966).
samebib(class_967,class_967).
samebib(class_967,class_968).
samebib(class_967,class_969).
samebib(class_967,class_970).
samebib(class_967,class_971).
samebib(class_967,class_972).
neg(samebib(class_967,class_1025)).
neg(samebib(class_967,class_1026)).
neg(samebib(class_967,class_1027)).
neg(samebib(class_967,class_1028)).
neg(samebib(class_967,class_1030)).
neg(samebib(class_967,class_1031)).
neg(samebib(class_967,class_1032)).
neg(samebib(class_967,class_1033)).
neg(samebib(class_967,class_1044)).
neg(samebib(class_967,class_1045)).
neg(samebib(class_967,class_1046)).
neg(samebib(class_967,class_1047)).
neg(samebib(class_967,class_1048)).
neg(samebib(class_968,class_911)).
neg(samebib(class_968,class_912)).
neg(samebib(class_968,class_913)).
neg(samebib(class_968,class_914)).
neg(samebib(class_968,class_917)).
neg(samebib(class_968,class_918)).
neg(samebib(class_968,class_919)).
neg(samebib(class_968,class_920)).
neg(samebib(class_968,class_921)).
neg(samebib(class_968,class_922)).
neg(samebib(class_968,class_923)).
neg(samebib(class_968,class_924)).
neg(samebib(class_968,class_925)).
neg(samebib(class_968,class_926)).
neg(samebib(class_968,class_927)).
neg(samebib(class_968,class_928)).
neg(samebib(class_968,class_929)).
samebib(class_968,class_930).
samebib(class_968,class_931).
samebib(class_968,class_932).
samebib(class_968,class_933).
samebib(class_968,class_934).
samebib(class_968,class_935).
samebib(class_968,class_936).
samebib(class_968,class_937).
samebib(class_968,class_938).
samebib(class_968,class_939).
samebib(class_968,class_940).
samebib(class_968,class_941).
samebib(class_968,class_942).
samebib(class_968,class_943).
samebib(class_968,class_944).
samebib(class_968,class_945).
samebib(class_968,class_946).
samebib(class_968,class_947).
samebib(class_968,class_948).
samebib(class_968,class_949).
samebib(class_968,class_950).
samebib(class_968,class_951).
samebib(class_968,class_952).
samebib(class_968,class_953).
samebib(class_968,class_954).
samebib(class_968,class_955).
samebib(class_968,class_956).
samebib(class_968,class_957).
samebib(class_968,class_958).
samebib(class_968,class_959).
samebib(class_968,class_960).
samebib(class_968,class_961).
samebib(class_968,class_962).
samebib(class_968,class_963).
samebib(class_968,class_964).
samebib(class_968,class_965).
samebib(class_968,class_966).
samebib(class_968,class_967).
samebib(class_968,class_968).
samebib(class_968,class_969).
samebib(class_968,class_970).
samebib(class_968,class_971).
samebib(class_968,class_972).
neg(samebib(class_968,class_1025)).
neg(samebib(class_968,class_1026)).
neg(samebib(class_968,class_1027)).
neg(samebib(class_968,class_1028)).
neg(samebib(class_968,class_1030)).
neg(samebib(class_968,class_1031)).
neg(samebib(class_968,class_1032)).
neg(samebib(class_968,class_1033)).
neg(samebib(class_968,class_1044)).
neg(samebib(class_968,class_1045)).
neg(samebib(class_968,class_1046)).
neg(samebib(class_968,class_1047)).
neg(samebib(class_968,class_1048)).
neg(samebib(class_969,class_911)).
neg(samebib(class_969,class_912)).
neg(samebib(class_969,class_913)).
neg(samebib(class_969,class_914)).
neg(samebib(class_969,class_917)).
neg(samebib(class_969,class_918)).
neg(samebib(class_969,class_919)).
neg(samebib(class_969,class_920)).
neg(samebib(class_969,class_921)).
neg(samebib(class_969,class_922)).
neg(samebib(class_969,class_923)).
neg(samebib(class_969,class_924)).
neg(samebib(class_969,class_925)).
neg(samebib(class_969,class_926)).
neg(samebib(class_969,class_927)).
neg(samebib(class_969,class_928)).
neg(samebib(class_969,class_929)).
samebib(class_969,class_930).
samebib(class_969,class_931).
samebib(class_969,class_932).
samebib(class_969,class_933).
samebib(class_969,class_934).
samebib(class_969,class_935).
samebib(class_969,class_936).
samebib(class_969,class_937).
samebib(class_969,class_938).
samebib(class_969,class_939).
samebib(class_969,class_940).
samebib(class_969,class_941).
samebib(class_969,class_942).
samebib(class_969,class_943).
samebib(class_969,class_944).
samebib(class_969,class_945).
samebib(class_969,class_946).
samebib(class_969,class_947).
samebib(class_969,class_948).
samebib(class_969,class_949).
samebib(class_969,class_950).
samebib(class_969,class_951).
samebib(class_969,class_952).
samebib(class_969,class_953).
samebib(class_969,class_954).
samebib(class_969,class_955).
samebib(class_969,class_956).
samebib(class_969,class_957).
samebib(class_969,class_958).
samebib(class_969,class_959).
samebib(class_969,class_960).
samebib(class_969,class_961).
samebib(class_969,class_962).
samebib(class_969,class_963).
samebib(class_969,class_964).
samebib(class_969,class_965).
samebib(class_969,class_966).
samebib(class_969,class_967).
samebib(class_969,class_968).
samebib(class_969,class_969).
samebib(class_969,class_970).
samebib(class_969,class_971).
samebib(class_969,class_972).
neg(samebib(class_969,class_1025)).
neg(samebib(class_969,class_1026)).
neg(samebib(class_969,class_1027)).
neg(samebib(class_969,class_1028)).
neg(samebib(class_969,class_1030)).
neg(samebib(class_969,class_1031)).
neg(samebib(class_969,class_1032)).
neg(samebib(class_969,class_1033)).
neg(samebib(class_969,class_1044)).
neg(samebib(class_969,class_1045)).
neg(samebib(class_969,class_1046)).
neg(samebib(class_969,class_1047)).
neg(samebib(class_969,class_1048)).
neg(samebib(class_970,class_911)).
neg(samebib(class_970,class_912)).
neg(samebib(class_970,class_913)).
neg(samebib(class_970,class_914)).
neg(samebib(class_970,class_917)).
neg(samebib(class_970,class_918)).
neg(samebib(class_970,class_919)).
neg(samebib(class_970,class_920)).
neg(samebib(class_970,class_921)).
neg(samebib(class_970,class_922)).
neg(samebib(class_970,class_923)).
neg(samebib(class_970,class_924)).
neg(samebib(class_970,class_925)).
neg(samebib(class_970,class_926)).
neg(samebib(class_970,class_927)).
neg(samebib(class_970,class_928)).
neg(samebib(class_970,class_929)).
samebib(class_970,class_930).
samebib(class_970,class_931).
samebib(class_970,class_932).
samebib(class_970,class_933).
samebib(class_970,class_934).
samebib(class_970,class_935).
samebib(class_970,class_936).
samebib(class_970,class_937).
samebib(class_970,class_938).
samebib(class_970,class_939).
samebib(class_970,class_940).
samebib(class_970,class_941).
samebib(class_970,class_942).
samebib(class_970,class_943).
samebib(class_970,class_944).
samebib(class_970,class_945).
samebib(class_970,class_946).
samebib(class_970,class_947).
samebib(class_970,class_948).
samebib(class_970,class_949).
samebib(class_970,class_950).
samebib(class_970,class_951).
samebib(class_970,class_952).
samebib(class_970,class_953).
samebib(class_970,class_954).
samebib(class_970,class_955).
samebib(class_970,class_956).
samebib(class_970,class_957).
samebib(class_970,class_958).
samebib(class_970,class_959).
samebib(class_970,class_960).
samebib(class_970,class_961).
samebib(class_970,class_962).
samebib(class_970,class_963).
samebib(class_970,class_964).
samebib(class_970,class_965).
samebib(class_970,class_966).
samebib(class_970,class_967).
samebib(class_970,class_968).
samebib(class_970,class_969).
samebib(class_970,class_970).
samebib(class_970,class_971).
samebib(class_970,class_972).
neg(samebib(class_970,class_1025)).
neg(samebib(class_970,class_1026)).
neg(samebib(class_970,class_1027)).
neg(samebib(class_970,class_1028)).
neg(samebib(class_970,class_1030)).
neg(samebib(class_970,class_1031)).
neg(samebib(class_970,class_1032)).
neg(samebib(class_970,class_1033)).
neg(samebib(class_970,class_1044)).
neg(samebib(class_970,class_1045)).
neg(samebib(class_970,class_1046)).
neg(samebib(class_970,class_1047)).
neg(samebib(class_970,class_1048)).
samebib(class_971,class_930).
samebib(class_971,class_931).
samebib(class_971,class_932).
samebib(class_971,class_933).
samebib(class_971,class_934).
samebib(class_971,class_935).
samebib(class_971,class_936).
samebib(class_971,class_937).
samebib(class_971,class_938).
samebib(class_971,class_939).
samebib(class_971,class_940).
samebib(class_971,class_941).
samebib(class_971,class_942).
samebib(class_971,class_943).
samebib(class_971,class_944).
samebib(class_971,class_945).
samebib(class_971,class_946).
samebib(class_971,class_947).
samebib(class_971,class_948).
samebib(class_971,class_949).
samebib(class_971,class_950).
samebib(class_971,class_951).
samebib(class_971,class_952).
samebib(class_971,class_953).
samebib(class_971,class_954).
samebib(class_971,class_955).
samebib(class_971,class_956).
samebib(class_971,class_957).
samebib(class_971,class_958).
samebib(class_971,class_959).
samebib(class_971,class_960).
samebib(class_971,class_961).
samebib(class_971,class_962).
samebib(class_971,class_963).
samebib(class_971,class_964).
samebib(class_971,class_965).
samebib(class_971,class_966).
samebib(class_971,class_967).
samebib(class_971,class_968).
samebib(class_971,class_969).
samebib(class_971,class_970).
samebib(class_971,class_971).
samebib(class_971,class_972).
neg(samebib(class_971,class_1025)).
neg(samebib(class_971,class_1026)).
neg(samebib(class_971,class_1027)).
neg(samebib(class_971,class_1028)).
neg(samebib(class_971,class_1030)).
neg(samebib(class_971,class_1031)).
neg(samebib(class_971,class_1032)).
neg(samebib(class_971,class_1033)).
neg(samebib(class_971,class_1044)).
neg(samebib(class_971,class_1045)).
neg(samebib(class_971,class_1046)).
neg(samebib(class_971,class_1047)).
neg(samebib(class_971,class_1048)).
neg(samebib(class_972,class_911)).
neg(samebib(class_972,class_912)).
neg(samebib(class_972,class_913)).
neg(samebib(class_972,class_914)).
neg(samebib(class_972,class_917)).
neg(samebib(class_972,class_918)).
neg(samebib(class_972,class_919)).
neg(samebib(class_972,class_920)).
neg(samebib(class_972,class_921)).
neg(samebib(class_972,class_922)).
neg(samebib(class_972,class_923)).
neg(samebib(class_972,class_924)).
neg(samebib(class_972,class_925)).
neg(samebib(class_972,class_926)).
neg(samebib(class_972,class_927)).
neg(samebib(class_972,class_928)).
neg(samebib(class_972,class_929)).
samebib(class_972,class_930).
samebib(class_972,class_931).
samebib(class_972,class_932).
samebib(class_972,class_933).
samebib(class_972,class_934).
samebib(class_972,class_935).
samebib(class_972,class_936).
samebib(class_972,class_937).
samebib(class_972,class_938).
samebib(class_972,class_939).
samebib(class_972,class_940).
samebib(class_972,class_941).
samebib(class_972,class_942).
samebib(class_972,class_943).
samebib(class_972,class_944).
samebib(class_972,class_945).
samebib(class_972,class_946).
samebib(class_972,class_947).
samebib(class_972,class_948).
samebib(class_972,class_949).
samebib(class_972,class_950).
samebib(class_972,class_951).
samebib(class_972,class_952).
samebib(class_972,class_953).
samebib(class_972,class_954).
samebib(class_972,class_955).
samebib(class_972,class_956).
samebib(class_972,class_957).
samebib(class_972,class_958).
samebib(class_972,class_959).
samebib(class_972,class_960).
samebib(class_972,class_961).
samebib(class_972,class_962).
samebib(class_972,class_963).
samebib(class_972,class_964).
samebib(class_972,class_965).
samebib(class_972,class_966).
samebib(class_972,class_967).
samebib(class_972,class_968).
samebib(class_972,class_969).
samebib(class_972,class_970).
samebib(class_972,class_971).
samebib(class_972,class_972).
neg(samebib(class_972,class_1025)).
neg(samebib(class_972,class_1026)).
neg(samebib(class_972,class_1027)).
neg(samebib(class_972,class_1028)).
neg(samebib(class_972,class_1030)).
neg(samebib(class_972,class_1031)).
neg(samebib(class_972,class_1032)).
neg(samebib(class_972,class_1033)).
neg(samebib(class_972,class_1044)).
neg(samebib(class_972,class_1045)).
neg(samebib(class_972,class_1046)).
neg(samebib(class_972,class_1047)).
neg(samebib(class_972,class_1048)).
samebib(class_1024,class_1024).
samebib(class_1024,class_1025).
samebib(class_1024,class_1026).
samebib(class_1024,class_1027).
samebib(class_1024,class_1028).
samebib(class_1024,class_1029).
samebib(class_1024,class_1030).
samebib(class_1024,class_1031).
samebib(class_1024,class_1032).
samebib(class_1024,class_1033).
neg(samebib(class_1025,class_930)).
neg(samebib(class_1025,class_931)).
neg(samebib(class_1025,class_932)).
neg(samebib(class_1025,class_933)).
neg(samebib(class_1025,class_934)).
neg(samebib(class_1025,class_935)).
neg(samebib(class_1025,class_936)).
neg(samebib(class_1025,class_937)).
neg(samebib(class_1025,class_938)).
neg(samebib(class_1025,class_939)).
neg(samebib(class_1025,class_940)).
neg(samebib(class_1025,class_941)).
neg(samebib(class_1025,class_942)).
neg(samebib(class_1025,class_943)).
neg(samebib(class_1025,class_944)).
neg(samebib(class_1025,class_945)).
neg(samebib(class_1025,class_946)).
neg(samebib(class_1025,class_947)).
neg(samebib(class_1025,class_948)).
neg(samebib(class_1025,class_949)).
neg(samebib(class_1025,class_950)).
neg(samebib(class_1025,class_951)).
neg(samebib(class_1025,class_952)).
neg(samebib(class_1025,class_953)).
neg(samebib(class_1025,class_954)).
neg(samebib(class_1025,class_955)).
neg(samebib(class_1025,class_956)).
neg(samebib(class_1025,class_957)).
neg(samebib(class_1025,class_958)).
neg(samebib(class_1025,class_959)).
neg(samebib(class_1025,class_960)).
neg(samebib(class_1025,class_961)).
neg(samebib(class_1025,class_962)).
neg(samebib(class_1025,class_963)).
neg(samebib(class_1025,class_964)).
neg(samebib(class_1025,class_965)).
neg(samebib(class_1025,class_966)).
neg(samebib(class_1025,class_967)).
neg(samebib(class_1025,class_968)).
neg(samebib(class_1025,class_969)).
neg(samebib(class_1025,class_970)).
neg(samebib(class_1025,class_971)).
neg(samebib(class_1025,class_972)).
samebib(class_1025,class_1024).
samebib(class_1025,class_1025).
samebib(class_1025,class_1026).
samebib(class_1025,class_1027).
samebib(class_1025,class_1028).
samebib(class_1025,class_1029).
samebib(class_1025,class_1030).
samebib(class_1025,class_1031).
samebib(class_1025,class_1032).
samebib(class_1025,class_1033).
neg(samebib(class_1026,class_943)).
neg(samebib(class_1026,class_944)).
neg(samebib(class_1026,class_945)).
neg(samebib(class_1026,class_946)).
neg(samebib(class_1026,class_947)).
neg(samebib(class_1026,class_948)).
neg(samebib(class_1026,class_949)).
neg(samebib(class_1026,class_950)).
neg(samebib(class_1026,class_951)).
neg(samebib(class_1026,class_952)).
neg(samebib(class_1026,class_953)).
neg(samebib(class_1026,class_954)).
neg(samebib(class_1026,class_955)).
neg(samebib(class_1026,class_956)).
neg(samebib(class_1026,class_957)).
neg(samebib(class_1026,class_958)).
neg(samebib(class_1026,class_959)).
neg(samebib(class_1026,class_960)).
neg(samebib(class_1026,class_961)).
neg(samebib(class_1026,class_962)).
neg(samebib(class_1026,class_963)).
neg(samebib(class_1026,class_964)).
neg(samebib(class_1026,class_965)).
neg(samebib(class_1026,class_966)).
neg(samebib(class_1026,class_967)).
neg(samebib(class_1026,class_968)).
neg(samebib(class_1026,class_969)).
neg(samebib(class_1026,class_970)).
neg(samebib(class_1026,class_971)).
neg(samebib(class_1026,class_972)).
samebib(class_1026,class_1024).
samebib(class_1026,class_1025).
samebib(class_1026,class_1026).
samebib(class_1026,class_1027).
samebib(class_1026,class_1028).
samebib(class_1026,class_1029).
samebib(class_1026,class_1030).
samebib(class_1026,class_1031).
samebib(class_1026,class_1032).
samebib(class_1026,class_1033).
neg(samebib(class_1026,class_1045)).
neg(samebib(class_1026,class_1046)).
neg(samebib(class_1026,class_1048)).
neg(samebib(class_1027,class_932)).
neg(samebib(class_1027,class_933)).
neg(samebib(class_1027,class_934)).
neg(samebib(class_1027,class_935)).
neg(samebib(class_1027,class_936)).
neg(samebib(class_1027,class_937)).
neg(samebib(class_1027,class_938)).
neg(samebib(class_1027,class_939)).
neg(samebib(class_1027,class_940)).
neg(samebib(class_1027,class_941)).
neg(samebib(class_1027,class_942)).
neg(samebib(class_1027,class_943)).
neg(samebib(class_1027,class_944)).
neg(samebib(class_1027,class_945)).
neg(samebib(class_1027,class_946)).
neg(samebib(class_1027,class_947)).
neg(samebib(class_1027,class_948)).
neg(samebib(class_1027,class_949)).
neg(samebib(class_1027,class_950)).
neg(samebib(class_1027,class_951)).
neg(samebib(class_1027,class_952)).
neg(samebib(class_1027,class_953)).
neg(samebib(class_1027,class_954)).
neg(samebib(class_1027,class_955)).
neg(samebib(class_1027,class_956)).
neg(samebib(class_1027,class_957)).
neg(samebib(class_1027,class_958)).
neg(samebib(class_1027,class_959)).
neg(samebib(class_1027,class_960)).
neg(samebib(class_1027,class_961)).
neg(samebib(class_1027,class_962)).
neg(samebib(class_1027,class_963)).
neg(samebib(class_1027,class_964)).
neg(samebib(class_1027,class_965)).
neg(samebib(class_1027,class_966)).
neg(samebib(class_1027,class_967)).
neg(samebib(class_1027,class_968)).
neg(samebib(class_1027,class_969)).
neg(samebib(class_1027,class_970)).
neg(samebib(class_1027,class_971)).
neg(samebib(class_1027,class_972)).
samebib(class_1027,class_1024).
samebib(class_1027,class_1025).
samebib(class_1027,class_1026).
samebib(class_1027,class_1027).
samebib(class_1027,class_1028).
samebib(class_1027,class_1029).
samebib(class_1027,class_1030).
samebib(class_1027,class_1031).
samebib(class_1027,class_1032).
samebib(class_1027,class_1033).
neg(samebib(class_1027,class_1044)).
neg(samebib(class_1027,class_1045)).
neg(samebib(class_1027,class_1046)).
neg(samebib(class_1027,class_1047)).
neg(samebib(class_1028,class_932)).
neg(samebib(class_1028,class_933)).
neg(samebib(class_1028,class_934)).
neg(samebib(class_1028,class_935)).
neg(samebib(class_1028,class_936)).
neg(samebib(class_1028,class_937)).
neg(samebib(class_1028,class_938)).
neg(samebib(class_1028,class_939)).
neg(samebib(class_1028,class_940)).
neg(samebib(class_1028,class_941)).
neg(samebib(class_1028,class_942)).
neg(samebib(class_1028,class_943)).
neg(samebib(class_1028,class_944)).
neg(samebib(class_1028,class_945)).
neg(samebib(class_1028,class_946)).
neg(samebib(class_1028,class_947)).
neg(samebib(class_1028,class_948)).
neg(samebib(class_1028,class_949)).
neg(samebib(class_1028,class_950)).
neg(samebib(class_1028,class_951)).
neg(samebib(class_1028,class_952)).
neg(samebib(class_1028,class_953)).
neg(samebib(class_1028,class_954)).
neg(samebib(class_1028,class_955)).
neg(samebib(class_1028,class_956)).
neg(samebib(class_1028,class_957)).
neg(samebib(class_1028,class_958)).
neg(samebib(class_1028,class_959)).
neg(samebib(class_1028,class_960)).
neg(samebib(class_1028,class_961)).
neg(samebib(class_1028,class_962)).
neg(samebib(class_1028,class_963)).
neg(samebib(class_1028,class_964)).
neg(samebib(class_1028,class_965)).
neg(samebib(class_1028,class_966)).
neg(samebib(class_1028,class_967)).
neg(samebib(class_1028,class_968)).
neg(samebib(class_1028,class_969)).
neg(samebib(class_1028,class_970)).
neg(samebib(class_1028,class_971)).
neg(samebib(class_1028,class_972)).
samebib(class_1028,class_1024).
samebib(class_1028,class_1025).
samebib(class_1028,class_1026).
samebib(class_1028,class_1027).
samebib(class_1028,class_1028).
samebib(class_1028,class_1029).
samebib(class_1028,class_1030).
samebib(class_1028,class_1031).
samebib(class_1028,class_1032).
samebib(class_1028,class_1033).
neg(samebib(class_1028,class_1044)).
neg(samebib(class_1028,class_1045)).
neg(samebib(class_1028,class_1046)).
neg(samebib(class_1028,class_1047)).
samebib(class_1029,class_1024).
samebib(class_1029,class_1025).
samebib(class_1029,class_1026).
samebib(class_1029,class_1027).
samebib(class_1029,class_1028).
samebib(class_1029,class_1029).
samebib(class_1029,class_1030).
samebib(class_1029,class_1031).
samebib(class_1029,class_1032).
samebib(class_1029,class_1033).
neg(samebib(class_1029,class_1045)).
neg(samebib(class_1029,class_1046)).
neg(samebib(class_1030,class_932)).
neg(samebib(class_1030,class_933)).
neg(samebib(class_1030,class_934)).
neg(samebib(class_1030,class_935)).
neg(samebib(class_1030,class_936)).
neg(samebib(class_1030,class_937)).
neg(samebib(class_1030,class_938)).
neg(samebib(class_1030,class_939)).
neg(samebib(class_1030,class_940)).
neg(samebib(class_1030,class_941)).
neg(samebib(class_1030,class_942)).
neg(samebib(class_1030,class_943)).
neg(samebib(class_1030,class_944)).
neg(samebib(class_1030,class_945)).
neg(samebib(class_1030,class_946)).
neg(samebib(class_1030,class_947)).
neg(samebib(class_1030,class_948)).
neg(samebib(class_1030,class_949)).
neg(samebib(class_1030,class_950)).
neg(samebib(class_1030,class_951)).
neg(samebib(class_1030,class_952)).
neg(samebib(class_1030,class_953)).
neg(samebib(class_1030,class_954)).
neg(samebib(class_1030,class_955)).
neg(samebib(class_1030,class_956)).
neg(samebib(class_1030,class_957)).
neg(samebib(class_1030,class_958)).
neg(samebib(class_1030,class_959)).
neg(samebib(class_1030,class_960)).
neg(samebib(class_1030,class_961)).
neg(samebib(class_1030,class_962)).
neg(samebib(class_1030,class_963)).
neg(samebib(class_1030,class_964)).
neg(samebib(class_1030,class_965)).
neg(samebib(class_1030,class_966)).
neg(samebib(class_1030,class_967)).
neg(samebib(class_1030,class_968)).
neg(samebib(class_1030,class_969)).
neg(samebib(class_1030,class_970)).
neg(samebib(class_1030,class_971)).
neg(samebib(class_1030,class_972)).
samebib(class_1030,class_1024).
samebib(class_1030,class_1025).
samebib(class_1030,class_1026).
samebib(class_1030,class_1027).
samebib(class_1030,class_1028).
samebib(class_1030,class_1029).
samebib(class_1030,class_1030).
samebib(class_1030,class_1031).
samebib(class_1030,class_1032).
samebib(class_1030,class_1033).
neg(samebib(class_1030,class_1044)).
neg(samebib(class_1030,class_1045)).
neg(samebib(class_1030,class_1046)).
neg(samebib(class_1030,class_1047)).
neg(samebib(class_1031,class_930)).
neg(samebib(class_1031,class_931)).
neg(samebib(class_1031,class_932)).
neg(samebib(class_1031,class_933)).
neg(samebib(class_1031,class_934)).
neg(samebib(class_1031,class_935)).
neg(samebib(class_1031,class_936)).
neg(samebib(class_1031,class_937)).
neg(samebib(class_1031,class_938)).
neg(samebib(class_1031,class_939)).
neg(samebib(class_1031,class_940)).
neg(samebib(class_1031,class_941)).
neg(samebib(class_1031,class_942)).
neg(samebib(class_1031,class_943)).
neg(samebib(class_1031,class_944)).
neg(samebib(class_1031,class_945)).
neg(samebib(class_1031,class_946)).
neg(samebib(class_1031,class_947)).
neg(samebib(class_1031,class_948)).
neg(samebib(class_1031,class_949)).
neg(samebib(class_1031,class_950)).
neg(samebib(class_1031,class_951)).
neg(samebib(class_1031,class_952)).
neg(samebib(class_1031,class_953)).
neg(samebib(class_1031,class_954)).
neg(samebib(class_1031,class_955)).
neg(samebib(class_1031,class_956)).
neg(samebib(class_1031,class_957)).
neg(samebib(class_1031,class_958)).
neg(samebib(class_1031,class_959)).
neg(samebib(class_1031,class_960)).
neg(samebib(class_1031,class_961)).
neg(samebib(class_1031,class_962)).
neg(samebib(class_1031,class_963)).
neg(samebib(class_1031,class_964)).
neg(samebib(class_1031,class_965)).
neg(samebib(class_1031,class_966)).
neg(samebib(class_1031,class_967)).
neg(samebib(class_1031,class_968)).
neg(samebib(class_1031,class_969)).
neg(samebib(class_1031,class_970)).
neg(samebib(class_1031,class_971)).
neg(samebib(class_1031,class_972)).
samebib(class_1031,class_1024).
samebib(class_1031,class_1025).
samebib(class_1031,class_1026).
samebib(class_1031,class_1027).
samebib(class_1031,class_1028).
samebib(class_1031,class_1029).
samebib(class_1031,class_1030).
samebib(class_1031,class_1031).
samebib(class_1031,class_1032).
samebib(class_1031,class_1033).
neg(samebib(class_1031,class_1045)).
neg(samebib(class_1031,class_1046)).
neg(samebib(class_1032,class_932)).
neg(samebib(class_1032,class_933)).
neg(samebib(class_1032,class_934)).
neg(samebib(class_1032,class_935)).
neg(samebib(class_1032,class_936)).
neg(samebib(class_1032,class_937)).
neg(samebib(class_1032,class_938)).
neg(samebib(class_1032,class_939)).
neg(samebib(class_1032,class_940)).
neg(samebib(class_1032,class_941)).
neg(samebib(class_1032,class_942)).
neg(samebib(class_1032,class_943)).
neg(samebib(class_1032,class_944)).
neg(samebib(class_1032,class_945)).
neg(samebib(class_1032,class_946)).
neg(samebib(class_1032,class_947)).
neg(samebib(class_1032,class_948)).
neg(samebib(class_1032,class_949)).
neg(samebib(class_1032,class_950)).
neg(samebib(class_1032,class_951)).
neg(samebib(class_1032,class_952)).
neg(samebib(class_1032,class_953)).
neg(samebib(class_1032,class_954)).
neg(samebib(class_1032,class_955)).
neg(samebib(class_1032,class_956)).
neg(samebib(class_1032,class_957)).
neg(samebib(class_1032,class_958)).
neg(samebib(class_1032,class_959)).
neg(samebib(class_1032,class_960)).
neg(samebib(class_1032,class_961)).
neg(samebib(class_1032,class_962)).
neg(samebib(class_1032,class_963)).
neg(samebib(class_1032,class_964)).
neg(samebib(class_1032,class_965)).
neg(samebib(class_1032,class_966)).
neg(samebib(class_1032,class_967)).
neg(samebib(class_1032,class_968)).
neg(samebib(class_1032,class_969)).
neg(samebib(class_1032,class_970)).
neg(samebib(class_1032,class_971)).
neg(samebib(class_1032,class_972)).
samebib(class_1032,class_1024).
samebib(class_1032,class_1025).
samebib(class_1032,class_1026).
samebib(class_1032,class_1027).
samebib(class_1032,class_1028).
samebib(class_1032,class_1029).
samebib(class_1032,class_1030).
samebib(class_1032,class_1031).
samebib(class_1032,class_1032).
samebib(class_1032,class_1033).
neg(samebib(class_1032,class_1044)).
neg(samebib(class_1032,class_1045)).
neg(samebib(class_1032,class_1046)).
neg(samebib(class_1032,class_1047)).
neg(samebib(class_1033,class_930)).
neg(samebib(class_1033,class_931)).
neg(samebib(class_1033,class_932)).
neg(samebib(class_1033,class_933)).
neg(samebib(class_1033,class_934)).
neg(samebib(class_1033,class_935)).
neg(samebib(class_1033,class_936)).
neg(samebib(class_1033,class_937)).
neg(samebib(class_1033,class_938)).
neg(samebib(class_1033,class_939)).
neg(samebib(class_1033,class_940)).
neg(samebib(class_1033,class_941)).
neg(samebib(class_1033,class_942)).
neg(samebib(class_1033,class_943)).
neg(samebib(class_1033,class_944)).
neg(samebib(class_1033,class_945)).
neg(samebib(class_1033,class_946)).
neg(samebib(class_1033,class_947)).
neg(samebib(class_1033,class_948)).
neg(samebib(class_1033,class_949)).
neg(samebib(class_1033,class_950)).
neg(samebib(class_1033,class_951)).
neg(samebib(class_1033,class_952)).
neg(samebib(class_1033,class_953)).
neg(samebib(class_1033,class_954)).
neg(samebib(class_1033,class_955)).
neg(samebib(class_1033,class_956)).
neg(samebib(class_1033,class_957)).
neg(samebib(class_1033,class_958)).
neg(samebib(class_1033,class_959)).
neg(samebib(class_1033,class_960)).
neg(samebib(class_1033,class_961)).
neg(samebib(class_1033,class_962)).
neg(samebib(class_1033,class_963)).
neg(samebib(class_1033,class_964)).
neg(samebib(class_1033,class_965)).
neg(samebib(class_1033,class_966)).
neg(samebib(class_1033,class_967)).
neg(samebib(class_1033,class_968)).
neg(samebib(class_1033,class_969)).
neg(samebib(class_1033,class_970)).
neg(samebib(class_1033,class_971)).
neg(samebib(class_1033,class_972)).
samebib(class_1033,class_1024).
samebib(class_1033,class_1025).
samebib(class_1033,class_1026).
samebib(class_1033,class_1027).
samebib(class_1033,class_1028).
samebib(class_1033,class_1029).
samebib(class_1033,class_1030).
samebib(class_1033,class_1031).
samebib(class_1033,class_1032).
samebib(class_1033,class_1033).
neg(samebib(class_1033,class_1045)).
neg(samebib(class_1033,class_1046)).
neg(samebib(class_1044,class_911)).
neg(samebib(class_1044,class_912)).
neg(samebib(class_1044,class_913)).
neg(samebib(class_1044,class_914)).
neg(samebib(class_1044,class_917)).
neg(samebib(class_1044,class_918)).
neg(samebib(class_1044,class_919)).
neg(samebib(class_1044,class_920)).
neg(samebib(class_1044,class_921)).
neg(samebib(class_1044,class_922)).
neg(samebib(class_1044,class_923)).
neg(samebib(class_1044,class_924)).
neg(samebib(class_1044,class_925)).
neg(samebib(class_1044,class_926)).
neg(samebib(class_1044,class_927)).
neg(samebib(class_1044,class_928)).
neg(samebib(class_1044,class_929)).
neg(samebib(class_1044,class_930)).
neg(samebib(class_1044,class_931)).
neg(samebib(class_1044,class_932)).
neg(samebib(class_1044,class_933)).
neg(samebib(class_1044,class_934)).
neg(samebib(class_1044,class_935)).
neg(samebib(class_1044,class_936)).
neg(samebib(class_1044,class_937)).
neg(samebib(class_1044,class_938)).
neg(samebib(class_1044,class_939)).
neg(samebib(class_1044,class_940)).
neg(samebib(class_1044,class_941)).
neg(samebib(class_1044,class_942)).
neg(samebib(class_1044,class_943)).
neg(samebib(class_1044,class_944)).
neg(samebib(class_1044,class_945)).
neg(samebib(class_1044,class_946)).
neg(samebib(class_1044,class_947)).
neg(samebib(class_1044,class_948)).
neg(samebib(class_1044,class_949)).
neg(samebib(class_1044,class_950)).
neg(samebib(class_1044,class_951)).
neg(samebib(class_1044,class_952)).
neg(samebib(class_1044,class_953)).
neg(samebib(class_1044,class_954)).
neg(samebib(class_1044,class_955)).
neg(samebib(class_1044,class_956)).
neg(samebib(class_1044,class_957)).
neg(samebib(class_1044,class_958)).
neg(samebib(class_1044,class_959)).
neg(samebib(class_1044,class_960)).
neg(samebib(class_1044,class_961)).
neg(samebib(class_1044,class_962)).
neg(samebib(class_1044,class_963)).
neg(samebib(class_1044,class_964)).
neg(samebib(class_1044,class_965)).
neg(samebib(class_1044,class_966)).
neg(samebib(class_1044,class_967)).
neg(samebib(class_1044,class_968)).
neg(samebib(class_1044,class_969)).
neg(samebib(class_1044,class_970)).
neg(samebib(class_1044,class_971)).
neg(samebib(class_1044,class_972)).
neg(samebib(class_1044,class_1027)).
neg(samebib(class_1044,class_1028)).
neg(samebib(class_1044,class_1030)).
neg(samebib(class_1044,class_1032)).
samebib(class_1044,class_1044).
samebib(class_1044,class_1047).
samebib(class_1044,class_1048).
neg(samebib(class_1045,class_943)).
neg(samebib(class_1045,class_944)).
neg(samebib(class_1045,class_945)).
neg(samebib(class_1045,class_946)).
neg(samebib(class_1045,class_947)).
neg(samebib(class_1045,class_948)).
neg(samebib(class_1045,class_949)).
neg(samebib(class_1045,class_950)).
neg(samebib(class_1045,class_951)).
neg(samebib(class_1045,class_952)).
neg(samebib(class_1045,class_953)).
neg(samebib(class_1045,class_954)).
neg(samebib(class_1045,class_955)).
neg(samebib(class_1045,class_956)).
neg(samebib(class_1045,class_957)).
neg(samebib(class_1045,class_958)).
neg(samebib(class_1045,class_959)).
neg(samebib(class_1045,class_960)).
neg(samebib(class_1045,class_961)).
neg(samebib(class_1045,class_962)).
neg(samebib(class_1045,class_963)).
neg(samebib(class_1045,class_964)).
neg(samebib(class_1045,class_965)).
neg(samebib(class_1045,class_966)).
neg(samebib(class_1045,class_967)).
neg(samebib(class_1045,class_968)).
neg(samebib(class_1045,class_969)).
neg(samebib(class_1045,class_970)).
neg(samebib(class_1045,class_971)).
neg(samebib(class_1045,class_972)).
neg(samebib(class_1045,class_1026)).
neg(samebib(class_1045,class_1027)).
neg(samebib(class_1045,class_1028)).
neg(samebib(class_1045,class_1029)).
neg(samebib(class_1045,class_1030)).
neg(samebib(class_1045,class_1031)).
neg(samebib(class_1045,class_1032)).
neg(samebib(class_1045,class_1033)).
samebib(class_1045,class_1045).
samebib(class_1045,class_1046).
samebib(class_1045,class_1047).
samebib(class_1045,class_1048).
neg(samebib(class_1046,class_943)).
neg(samebib(class_1046,class_944)).
neg(samebib(class_1046,class_945)).
neg(samebib(class_1046,class_946)).
neg(samebib(class_1046,class_947)).
neg(samebib(class_1046,class_948)).
neg(samebib(class_1046,class_949)).
neg(samebib(class_1046,class_950)).
neg(samebib(class_1046,class_951)).
neg(samebib(class_1046,class_952)).
neg(samebib(class_1046,class_953)).
neg(samebib(class_1046,class_954)).
neg(samebib(class_1046,class_955)).
neg(samebib(class_1046,class_956)).
neg(samebib(class_1046,class_957)).
neg(samebib(class_1046,class_958)).
neg(samebib(class_1046,class_959)).
neg(samebib(class_1046,class_960)).
neg(samebib(class_1046,class_961)).
neg(samebib(class_1046,class_962)).
neg(samebib(class_1046,class_963)).
neg(samebib(class_1046,class_964)).
neg(samebib(class_1046,class_965)).
neg(samebib(class_1046,class_966)).
neg(samebib(class_1046,class_967)).
neg(samebib(class_1046,class_968)).
neg(samebib(class_1046,class_969)).
neg(samebib(class_1046,class_970)).
neg(samebib(class_1046,class_971)).
neg(samebib(class_1046,class_972)).
neg(samebib(class_1046,class_1026)).
neg(samebib(class_1046,class_1027)).
neg(samebib(class_1046,class_1028)).
neg(samebib(class_1046,class_1029)).
neg(samebib(class_1046,class_1030)).
neg(samebib(class_1046,class_1031)).
neg(samebib(class_1046,class_1032)).
neg(samebib(class_1046,class_1033)).
samebib(class_1046,class_1045).
samebib(class_1046,class_1046).
samebib(class_1046,class_1047).
samebib(class_1046,class_1048).
neg(samebib(class_1047,class_911)).
neg(samebib(class_1047,class_912)).
neg(samebib(class_1047,class_913)).
neg(samebib(class_1047,class_914)).
neg(samebib(class_1047,class_917)).
neg(samebib(class_1047,class_918)).
neg(samebib(class_1047,class_919)).
neg(samebib(class_1047,class_920)).
neg(samebib(class_1047,class_921)).
neg(samebib(class_1047,class_922)).
neg(samebib(class_1047,class_923)).
neg(samebib(class_1047,class_924)).
neg(samebib(class_1047,class_925)).
neg(samebib(class_1047,class_926)).
neg(samebib(class_1047,class_927)).
neg(samebib(class_1047,class_928)).
neg(samebib(class_1047,class_929)).
neg(samebib(class_1047,class_930)).
neg(samebib(class_1047,class_931)).
neg(samebib(class_1047,class_932)).
neg(samebib(class_1047,class_933)).
neg(samebib(class_1047,class_934)).
neg(samebib(class_1047,class_935)).
neg(samebib(class_1047,class_936)).
neg(samebib(class_1047,class_937)).
neg(samebib(class_1047,class_938)).
neg(samebib(class_1047,class_939)).
neg(samebib(class_1047,class_940)).
neg(samebib(class_1047,class_941)).
neg(samebib(class_1047,class_942)).
neg(samebib(class_1047,class_943)).
neg(samebib(class_1047,class_944)).
neg(samebib(class_1047,class_945)).
neg(samebib(class_1047,class_946)).
neg(samebib(class_1047,class_947)).
neg(samebib(class_1047,class_948)).
neg(samebib(class_1047,class_949)).
neg(samebib(class_1047,class_950)).
neg(samebib(class_1047,class_951)).
neg(samebib(class_1047,class_952)).
neg(samebib(class_1047,class_953)).
neg(samebib(class_1047,class_954)).
neg(samebib(class_1047,class_955)).
neg(samebib(class_1047,class_956)).
neg(samebib(class_1047,class_957)).
neg(samebib(class_1047,class_958)).
neg(samebib(class_1047,class_959)).
neg(samebib(class_1047,class_960)).
neg(samebib(class_1047,class_961)).
neg(samebib(class_1047,class_962)).
neg(samebib(class_1047,class_963)).
neg(samebib(class_1047,class_964)).
neg(samebib(class_1047,class_965)).
neg(samebib(class_1047,class_966)).
neg(samebib(class_1047,class_967)).
neg(samebib(class_1047,class_968)).
neg(samebib(class_1047,class_969)).
neg(samebib(class_1047,class_970)).
neg(samebib(class_1047,class_971)).
neg(samebib(class_1047,class_972)).
neg(samebib(class_1047,class_1027)).
neg(samebib(class_1047,class_1028)).
neg(samebib(class_1047,class_1030)).
neg(samebib(class_1047,class_1032)).
samebib(class_1047,class_1044).
samebib(class_1047,class_1045).
samebib(class_1047,class_1046).
samebib(class_1047,class_1047).
samebib(class_1047,class_1048).
neg(samebib(class_1048,class_943)).
neg(samebib(class_1048,class_944)).
neg(samebib(class_1048,class_945)).
neg(samebib(class_1048,class_946)).
neg(samebib(class_1048,class_947)).
neg(samebib(class_1048,class_948)).
neg(samebib(class_1048,class_949)).
neg(samebib(class_1048,class_950)).
neg(samebib(class_1048,class_951)).
neg(samebib(class_1048,class_952)).
neg(samebib(class_1048,class_953)).
neg(samebib(class_1048,class_954)).
neg(samebib(class_1048,class_955)).
neg(samebib(class_1048,class_956)).
neg(samebib(class_1048,class_957)).
neg(samebib(class_1048,class_958)).
neg(samebib(class_1048,class_959)).
neg(samebib(class_1048,class_960)).
neg(samebib(class_1048,class_961)).
neg(samebib(class_1048,class_962)).
neg(samebib(class_1048,class_963)).
neg(samebib(class_1048,class_964)).
neg(samebib(class_1048,class_965)).
neg(samebib(class_1048,class_966)).
neg(samebib(class_1048,class_967)).
neg(samebib(class_1048,class_968)).
neg(samebib(class_1048,class_969)).
neg(samebib(class_1048,class_970)).
neg(samebib(class_1048,class_971)).
neg(samebib(class_1048,class_972)).
neg(samebib(class_1048,class_1026)).
samebib(class_1048,class_1044).
samebib(class_1048,class_1045).
samebib(class_1048,class_1046).
samebib(class_1048,class_1047).
samebib(class_1048,class_1048).
samebib(class_1062,class_1062).
samebib(class_1124,class_1124).
samebib(class_1125,class_1125).
samebib(class_1125,class_1126).
samebib(class_1125,class_1127).
samebib(class_1125,class_1128).
samebib(class_1125,class_1129).
samebib(class_1125,class_1130).
samebib(class_1125,class_1131).
samebib(class_1125,class_1132).
samebib(class_1125,class_1133).
samebib(class_1125,class_1134).
samebib(class_1125,class_1136).
samebib(class_1125,class_1137).
samebib(class_1125,class_1138).
samebib(class_1125,class_1139).
samebib(class_1125,class_1140).
samebib(class_1125,class_1141).
samebib(class_1125,class_1142).
samebib(class_1125,class_1143).
samebib(class_1125,class_1144).
samebib(class_1125,class_1145).
samebib(class_1125,class_1146).
samebib(class_1125,class_1147).
samebib(class_1125,class_1148).
samebib(class_1125,class_1149).
samebib(class_1125,class_1150).
samebib(class_1125,class_1151).
samebib(class_1125,class_1152).
samebib(class_1125,class_1154).
samebib(class_1125,class_1155).
samebib(class_1125,class_1156).
samebib(class_1125,class_1157).
samebib(class_1125,class_1158).
samebib(class_1125,class_1159).
samebib(class_1125,class_1160).
samebib(class_1125,class_1161).
neg(samebib(class_1125,class_1162)).
neg(samebib(class_1125,class_1164)).
neg(samebib(class_1125,class_1165)).
samebib(class_1126,class_1125).
samebib(class_1126,class_1126).
samebib(class_1126,class_1127).
samebib(class_1126,class_1128).
samebib(class_1126,class_1129).
samebib(class_1126,class_1130).
samebib(class_1126,class_1131).
samebib(class_1126,class_1132).
samebib(class_1126,class_1133).
samebib(class_1126,class_1134).
samebib(class_1126,class_1136).
samebib(class_1126,class_1137).
samebib(class_1126,class_1138).
samebib(class_1126,class_1139).
samebib(class_1126,class_1140).
samebib(class_1126,class_1141).
samebib(class_1126,class_1142).
samebib(class_1126,class_1143).
samebib(class_1126,class_1144).
samebib(class_1126,class_1145).
samebib(class_1126,class_1146).
samebib(class_1126,class_1147).
samebib(class_1126,class_1148).
samebib(class_1126,class_1149).
samebib(class_1126,class_1150).
samebib(class_1126,class_1151).
samebib(class_1126,class_1152).
samebib(class_1126,class_1154).
samebib(class_1126,class_1155).
samebib(class_1126,class_1156).
samebib(class_1126,class_1157).
samebib(class_1126,class_1158).
samebib(class_1126,class_1159).
samebib(class_1126,class_1160).
samebib(class_1126,class_1161).
neg(samebib(class_1126,class_1162)).
neg(samebib(class_1126,class_1164)).
neg(samebib(class_1126,class_1165)).
samebib(class_1127,class_1125).
samebib(class_1127,class_1126).
samebib(class_1127,class_1127).
samebib(class_1127,class_1128).
samebib(class_1127,class_1129).
samebib(class_1127,class_1130).
samebib(class_1127,class_1131).
samebib(class_1127,class_1132).
samebib(class_1127,class_1133).
samebib(class_1127,class_1134).
samebib(class_1127,class_1136).
samebib(class_1127,class_1137).
samebib(class_1127,class_1138).
samebib(class_1127,class_1139).
samebib(class_1127,class_1140).
samebib(class_1127,class_1141).
samebib(class_1127,class_1142).
samebib(class_1127,class_1143).
samebib(class_1127,class_1144).
samebib(class_1127,class_1145).
samebib(class_1127,class_1146).
samebib(class_1127,class_1147).
samebib(class_1127,class_1148).
samebib(class_1127,class_1149).
samebib(class_1127,class_1150).
samebib(class_1127,class_1151).
samebib(class_1127,class_1152).
samebib(class_1127,class_1154).
samebib(class_1127,class_1155).
samebib(class_1127,class_1156).
samebib(class_1127,class_1157).
samebib(class_1127,class_1158).
samebib(class_1127,class_1159).
samebib(class_1127,class_1160).
samebib(class_1127,class_1161).
neg(samebib(class_1127,class_1162)).
neg(samebib(class_1127,class_1163)).
neg(samebib(class_1127,class_1164)).
neg(samebib(class_1127,class_1165)).
neg(samebib(class_1127,class_1166)).
samebib(class_1128,class_1125).
samebib(class_1128,class_1126).
samebib(class_1128,class_1127).
samebib(class_1128,class_1128).
samebib(class_1128,class_1129).
samebib(class_1128,class_1130).
samebib(class_1128,class_1131).
samebib(class_1128,class_1132).
samebib(class_1128,class_1133).
samebib(class_1128,class_1134).
samebib(class_1128,class_1136).
samebib(class_1128,class_1137).
samebib(class_1128,class_1138).
samebib(class_1128,class_1139).
samebib(class_1128,class_1140).
samebib(class_1128,class_1141).
samebib(class_1128,class_1142).
samebib(class_1128,class_1143).
samebib(class_1128,class_1144).
samebib(class_1128,class_1145).
samebib(class_1128,class_1146).
samebib(class_1128,class_1147).
samebib(class_1128,class_1148).
samebib(class_1128,class_1149).
samebib(class_1128,class_1150).
samebib(class_1128,class_1151).
samebib(class_1128,class_1152).
samebib(class_1128,class_1154).
samebib(class_1128,class_1155).
samebib(class_1128,class_1156).
samebib(class_1128,class_1157).
samebib(class_1128,class_1158).
samebib(class_1128,class_1159).
samebib(class_1128,class_1160).
samebib(class_1128,class_1161).
neg(samebib(class_1128,class_1162)).
neg(samebib(class_1128,class_1163)).
neg(samebib(class_1128,class_1164)).
neg(samebib(class_1128,class_1165)).
neg(samebib(class_1128,class_1166)).
samebib(class_1129,class_1125).
samebib(class_1129,class_1126).
samebib(class_1129,class_1127).
samebib(class_1129,class_1128).
samebib(class_1129,class_1129).
samebib(class_1129,class_1130).
samebib(class_1129,class_1131).
samebib(class_1129,class_1132).
samebib(class_1129,class_1133).
samebib(class_1129,class_1134).
samebib(class_1129,class_1136).
samebib(class_1129,class_1137).
samebib(class_1129,class_1138).
samebib(class_1129,class_1139).
samebib(class_1129,class_1140).
samebib(class_1129,class_1141).
samebib(class_1129,class_1142).
samebib(class_1129,class_1143).
samebib(class_1129,class_1144).
samebib(class_1129,class_1145).
samebib(class_1129,class_1146).
samebib(class_1129,class_1147).
samebib(class_1129,class_1148).
samebib(class_1129,class_1149).
samebib(class_1129,class_1150).
samebib(class_1129,class_1151).
samebib(class_1129,class_1152).
samebib(class_1129,class_1154).
samebib(class_1129,class_1155).
samebib(class_1129,class_1156).
samebib(class_1129,class_1157).
samebib(class_1129,class_1158).
samebib(class_1129,class_1159).
samebib(class_1129,class_1160).
samebib(class_1129,class_1161).
neg(samebib(class_1129,class_1162)).
neg(samebib(class_1129,class_1163)).
neg(samebib(class_1129,class_1164)).
neg(samebib(class_1129,class_1165)).
neg(samebib(class_1129,class_1166)).
samebib(class_1130,class_1125).
samebib(class_1130,class_1126).
samebib(class_1130,class_1127).
samebib(class_1130,class_1128).
samebib(class_1130,class_1129).
samebib(class_1130,class_1130).
samebib(class_1130,class_1131).
samebib(class_1130,class_1132).
samebib(class_1130,class_1133).
samebib(class_1130,class_1134).
samebib(class_1130,class_1136).
samebib(class_1130,class_1137).
samebib(class_1130,class_1138).
samebib(class_1130,class_1139).
samebib(class_1130,class_1140).
samebib(class_1130,class_1141).
samebib(class_1130,class_1142).
samebib(class_1130,class_1143).
samebib(class_1130,class_1144).
samebib(class_1130,class_1145).
samebib(class_1130,class_1146).
samebib(class_1130,class_1147).
samebib(class_1130,class_1148).
samebib(class_1130,class_1149).
samebib(class_1130,class_1150).
samebib(class_1130,class_1151).
samebib(class_1130,class_1152).
samebib(class_1130,class_1154).
samebib(class_1130,class_1155).
samebib(class_1130,class_1156).
samebib(class_1130,class_1157).
samebib(class_1130,class_1158).
samebib(class_1130,class_1159).
samebib(class_1130,class_1160).
samebib(class_1130,class_1161).
neg(samebib(class_1130,class_1162)).
neg(samebib(class_1130,class_1163)).
neg(samebib(class_1130,class_1164)).
neg(samebib(class_1130,class_1165)).
neg(samebib(class_1130,class_1166)).
samebib(class_1131,class_1125).
samebib(class_1131,class_1126).
samebib(class_1131,class_1127).
samebib(class_1131,class_1128).
samebib(class_1131,class_1129).
samebib(class_1131,class_1130).
samebib(class_1131,class_1131).
samebib(class_1131,class_1132).
samebib(class_1131,class_1133).
samebib(class_1131,class_1134).
samebib(class_1131,class_1136).
samebib(class_1131,class_1137).
samebib(class_1131,class_1138).
samebib(class_1131,class_1139).
samebib(class_1131,class_1140).
samebib(class_1131,class_1141).
samebib(class_1131,class_1142).
samebib(class_1131,class_1143).
samebib(class_1131,class_1144).
samebib(class_1131,class_1145).
samebib(class_1131,class_1146).
samebib(class_1131,class_1147).
samebib(class_1131,class_1148).
samebib(class_1131,class_1149).
samebib(class_1131,class_1150).
samebib(class_1131,class_1151).
samebib(class_1131,class_1152).
samebib(class_1131,class_1154).
samebib(class_1131,class_1155).
samebib(class_1131,class_1156).
samebib(class_1131,class_1157).
samebib(class_1131,class_1158).
samebib(class_1131,class_1159).
samebib(class_1131,class_1160).
samebib(class_1131,class_1161).
neg(samebib(class_1131,class_1162)).
neg(samebib(class_1131,class_1163)).
neg(samebib(class_1131,class_1164)).
neg(samebib(class_1131,class_1165)).
neg(samebib(class_1131,class_1166)).
samebib(class_1132,class_1125).
samebib(class_1132,class_1126).
samebib(class_1132,class_1127).
samebib(class_1132,class_1128).
samebib(class_1132,class_1129).
samebib(class_1132,class_1130).
samebib(class_1132,class_1131).
samebib(class_1132,class_1132).
samebib(class_1132,class_1133).
samebib(class_1132,class_1134).
samebib(class_1132,class_1136).
samebib(class_1132,class_1137).
samebib(class_1132,class_1138).
samebib(class_1132,class_1139).
samebib(class_1132,class_1140).
samebib(class_1132,class_1141).
samebib(class_1132,class_1142).
samebib(class_1132,class_1143).
samebib(class_1132,class_1144).
samebib(class_1132,class_1145).
samebib(class_1132,class_1146).
samebib(class_1132,class_1147).
samebib(class_1132,class_1148).
samebib(class_1132,class_1149).
samebib(class_1132,class_1150).
samebib(class_1132,class_1151).
samebib(class_1132,class_1152).
samebib(class_1132,class_1154).
samebib(class_1132,class_1155).
samebib(class_1132,class_1156).
samebib(class_1132,class_1157).
samebib(class_1132,class_1158).
samebib(class_1132,class_1159).
samebib(class_1132,class_1160).
samebib(class_1132,class_1161).
neg(samebib(class_1132,class_1162)).
neg(samebib(class_1132,class_1163)).
neg(samebib(class_1132,class_1164)).
neg(samebib(class_1132,class_1165)).
neg(samebib(class_1132,class_1166)).
samebib(class_1133,class_1125).
samebib(class_1133,class_1126).
samebib(class_1133,class_1127).
samebib(class_1133,class_1128).
samebib(class_1133,class_1129).
samebib(class_1133,class_1130).
samebib(class_1133,class_1131).
samebib(class_1133,class_1132).
samebib(class_1133,class_1133).
samebib(class_1133,class_1134).
samebib(class_1133,class_1136).
samebib(class_1133,class_1137).
samebib(class_1133,class_1138).
samebib(class_1133,class_1139).
samebib(class_1133,class_1140).
samebib(class_1133,class_1141).
samebib(class_1133,class_1142).
samebib(class_1133,class_1143).
samebib(class_1133,class_1144).
samebib(class_1133,class_1145).
samebib(class_1133,class_1146).
samebib(class_1133,class_1147).
samebib(class_1133,class_1148).
samebib(class_1133,class_1149).
samebib(class_1133,class_1150).
samebib(class_1133,class_1151).
samebib(class_1133,class_1152).
samebib(class_1133,class_1154).
samebib(class_1133,class_1155).
samebib(class_1133,class_1156).
samebib(class_1133,class_1157).
samebib(class_1133,class_1158).
samebib(class_1133,class_1159).
samebib(class_1133,class_1160).
samebib(class_1133,class_1161).
neg(samebib(class_1133,class_1162)).
neg(samebib(class_1133,class_1163)).
neg(samebib(class_1133,class_1164)).
neg(samebib(class_1133,class_1165)).
neg(samebib(class_1133,class_1166)).
samebib(class_1134,class_1125).
samebib(class_1134,class_1126).
samebib(class_1134,class_1127).
samebib(class_1134,class_1128).
samebib(class_1134,class_1129).
samebib(class_1134,class_1130).
samebib(class_1134,class_1131).
samebib(class_1134,class_1132).
samebib(class_1134,class_1133).
samebib(class_1134,class_1134).
samebib(class_1134,class_1136).
samebib(class_1134,class_1137).
samebib(class_1134,class_1138).
samebib(class_1134,class_1139).
samebib(class_1134,class_1140).
samebib(class_1134,class_1141).
samebib(class_1134,class_1142).
samebib(class_1134,class_1143).
samebib(class_1134,class_1144).
samebib(class_1134,class_1145).
samebib(class_1134,class_1146).
samebib(class_1134,class_1147).
samebib(class_1134,class_1148).
samebib(class_1134,class_1149).
samebib(class_1134,class_1150).
samebib(class_1134,class_1151).
samebib(class_1134,class_1152).
samebib(class_1134,class_1154).
samebib(class_1134,class_1155).
samebib(class_1134,class_1156).
samebib(class_1134,class_1157).
samebib(class_1134,class_1158).
samebib(class_1134,class_1159).
samebib(class_1134,class_1160).
samebib(class_1134,class_1161).
neg(samebib(class_1134,class_1162)).
neg(samebib(class_1134,class_1163)).
neg(samebib(class_1134,class_1164)).
neg(samebib(class_1134,class_1165)).
neg(samebib(class_1134,class_1166)).
samebib(class_1135,class_1135).
samebib(class_1135,class_1153).
neg(samebib(class_1135,class_1262)).
neg(samebib(class_1135,class_1263)).
neg(samebib(class_1135,class_1264)).
neg(samebib(class_1135,class_1265)).
neg(samebib(class_1135,class_1266)).
neg(samebib(class_1135,class_1270)).
neg(samebib(class_1135,class_1271)).
neg(samebib(class_1135,class_1273)).
neg(samebib(class_1135,class_1276)).
samebib(class_1136,class_1125).
samebib(class_1136,class_1126).
samebib(class_1136,class_1127).
samebib(class_1136,class_1128).
samebib(class_1136,class_1129).
samebib(class_1136,class_1130).
samebib(class_1136,class_1131).
samebib(class_1136,class_1132).
samebib(class_1136,class_1133).
samebib(class_1136,class_1134).
samebib(class_1136,class_1136).
samebib(class_1136,class_1137).
samebib(class_1136,class_1138).
samebib(class_1136,class_1139).
samebib(class_1136,class_1140).
samebib(class_1136,class_1141).
samebib(class_1136,class_1142).
samebib(class_1136,class_1143).
samebib(class_1136,class_1144).
samebib(class_1136,class_1145).
samebib(class_1136,class_1146).
samebib(class_1136,class_1147).
samebib(class_1136,class_1148).
samebib(class_1136,class_1149).
samebib(class_1136,class_1150).
samebib(class_1136,class_1151).
samebib(class_1136,class_1152).
samebib(class_1136,class_1154).
samebib(class_1136,class_1155).
samebib(class_1136,class_1156).
samebib(class_1136,class_1157).
samebib(class_1136,class_1158).
samebib(class_1136,class_1159).
samebib(class_1136,class_1160).
samebib(class_1136,class_1161).
neg(samebib(class_1136,class_1162)).
neg(samebib(class_1136,class_1163)).
neg(samebib(class_1136,class_1164)).
neg(samebib(class_1136,class_1165)).
neg(samebib(class_1136,class_1166)).
samebib(class_1137,class_1125).
samebib(class_1137,class_1126).
samebib(class_1137,class_1127).
samebib(class_1137,class_1128).
samebib(class_1137,class_1129).
samebib(class_1137,class_1130).
samebib(class_1137,class_1131).
samebib(class_1137,class_1132).
samebib(class_1137,class_1133).
samebib(class_1137,class_1134).
samebib(class_1137,class_1136).
samebib(class_1137,class_1137).
samebib(class_1137,class_1138).
samebib(class_1137,class_1139).
samebib(class_1137,class_1140).
samebib(class_1137,class_1141).
samebib(class_1137,class_1142).
samebib(class_1137,class_1143).
samebib(class_1137,class_1144).
samebib(class_1137,class_1145).
samebib(class_1137,class_1146).
samebib(class_1137,class_1147).
samebib(class_1137,class_1148).
samebib(class_1137,class_1149).
samebib(class_1137,class_1150).
samebib(class_1137,class_1151).
samebib(class_1137,class_1152).
samebib(class_1137,class_1154).
samebib(class_1137,class_1155).
samebib(class_1137,class_1156).
samebib(class_1137,class_1157).
samebib(class_1137,class_1158).
samebib(class_1137,class_1159).
samebib(class_1137,class_1160).
samebib(class_1137,class_1161).
neg(samebib(class_1137,class_1162)).
neg(samebib(class_1137,class_1163)).
neg(samebib(class_1137,class_1164)).
neg(samebib(class_1137,class_1165)).
neg(samebib(class_1137,class_1166)).
samebib(class_1138,class_1125).
samebib(class_1138,class_1126).
samebib(class_1138,class_1127).
samebib(class_1138,class_1128).
samebib(class_1138,class_1129).
samebib(class_1138,class_1130).
samebib(class_1138,class_1131).
samebib(class_1138,class_1132).
samebib(class_1138,class_1133).
samebib(class_1138,class_1134).
samebib(class_1138,class_1136).
samebib(class_1138,class_1137).
samebib(class_1138,class_1138).
samebib(class_1138,class_1139).
samebib(class_1138,class_1140).
samebib(class_1138,class_1141).
samebib(class_1138,class_1142).
samebib(class_1138,class_1143).
samebib(class_1138,class_1144).
samebib(class_1138,class_1145).
samebib(class_1138,class_1146).
samebib(class_1138,class_1147).
samebib(class_1138,class_1148).
samebib(class_1138,class_1149).
samebib(class_1138,class_1150).
samebib(class_1138,class_1151).
samebib(class_1138,class_1152).
samebib(class_1138,class_1154).
samebib(class_1138,class_1155).
samebib(class_1138,class_1156).
samebib(class_1138,class_1157).
samebib(class_1138,class_1158).
samebib(class_1138,class_1159).
samebib(class_1138,class_1160).
samebib(class_1138,class_1161).
neg(samebib(class_1138,class_1162)).
neg(samebib(class_1138,class_1163)).
neg(samebib(class_1138,class_1164)).
neg(samebib(class_1138,class_1165)).
neg(samebib(class_1138,class_1166)).
samebib(class_1139,class_1125).
samebib(class_1139,class_1126).
samebib(class_1139,class_1127).
samebib(class_1139,class_1128).
samebib(class_1139,class_1129).
samebib(class_1139,class_1130).
samebib(class_1139,class_1131).
samebib(class_1139,class_1132).
samebib(class_1139,class_1133).
samebib(class_1139,class_1134).
samebib(class_1139,class_1136).
samebib(class_1139,class_1137).
samebib(class_1139,class_1138).
samebib(class_1139,class_1139).
samebib(class_1139,class_1140).
samebib(class_1139,class_1141).
samebib(class_1139,class_1142).
samebib(class_1139,class_1143).
samebib(class_1139,class_1144).
samebib(class_1139,class_1145).
samebib(class_1139,class_1146).
samebib(class_1139,class_1147).
samebib(class_1139,class_1148).
samebib(class_1139,class_1149).
samebib(class_1139,class_1150).
samebib(class_1139,class_1151).
samebib(class_1139,class_1152).
samebib(class_1139,class_1154).
samebib(class_1139,class_1155).
samebib(class_1139,class_1156).
samebib(class_1139,class_1157).
samebib(class_1139,class_1158).
samebib(class_1139,class_1159).
samebib(class_1139,class_1160).
samebib(class_1139,class_1161).
neg(samebib(class_1139,class_1162)).
neg(samebib(class_1139,class_1163)).
neg(samebib(class_1139,class_1164)).
neg(samebib(class_1139,class_1165)).
neg(samebib(class_1139,class_1166)).
samebib(class_1140,class_1125).
samebib(class_1140,class_1126).
samebib(class_1140,class_1127).
samebib(class_1140,class_1128).
samebib(class_1140,class_1129).
samebib(class_1140,class_1130).
samebib(class_1140,class_1131).
samebib(class_1140,class_1132).
samebib(class_1140,class_1133).
samebib(class_1140,class_1134).
samebib(class_1140,class_1136).
samebib(class_1140,class_1137).
samebib(class_1140,class_1138).
samebib(class_1140,class_1139).
samebib(class_1140,class_1140).
samebib(class_1140,class_1141).
samebib(class_1140,class_1142).
samebib(class_1140,class_1143).
samebib(class_1140,class_1144).
samebib(class_1140,class_1145).
samebib(class_1140,class_1146).
samebib(class_1140,class_1147).
samebib(class_1140,class_1148).
samebib(class_1140,class_1149).
samebib(class_1140,class_1150).
samebib(class_1140,class_1151).
samebib(class_1140,class_1152).
samebib(class_1140,class_1154).
samebib(class_1140,class_1155).
samebib(class_1140,class_1156).
samebib(class_1140,class_1157).
samebib(class_1140,class_1158).
samebib(class_1140,class_1159).
samebib(class_1140,class_1160).
samebib(class_1140,class_1161).
neg(samebib(class_1140,class_1162)).
neg(samebib(class_1140,class_1163)).
neg(samebib(class_1140,class_1164)).
neg(samebib(class_1140,class_1165)).
neg(samebib(class_1140,class_1166)).
samebib(class_1141,class_1125).
samebib(class_1141,class_1126).
samebib(class_1141,class_1127).
samebib(class_1141,class_1128).
samebib(class_1141,class_1129).
samebib(class_1141,class_1130).
samebib(class_1141,class_1131).
samebib(class_1141,class_1132).
samebib(class_1141,class_1133).
samebib(class_1141,class_1134).
samebib(class_1141,class_1136).
samebib(class_1141,class_1137).
samebib(class_1141,class_1138).
samebib(class_1141,class_1139).
samebib(class_1141,class_1140).
samebib(class_1141,class_1141).
samebib(class_1141,class_1142).
samebib(class_1141,class_1143).
samebib(class_1141,class_1144).
samebib(class_1141,class_1145).
samebib(class_1141,class_1146).
samebib(class_1141,class_1147).
samebib(class_1141,class_1148).
samebib(class_1141,class_1149).
samebib(class_1141,class_1150).
samebib(class_1141,class_1151).
samebib(class_1141,class_1152).
samebib(class_1141,class_1154).
samebib(class_1141,class_1155).
samebib(class_1141,class_1156).
samebib(class_1141,class_1157).
samebib(class_1141,class_1158).
samebib(class_1141,class_1159).
samebib(class_1141,class_1160).
samebib(class_1141,class_1161).
neg(samebib(class_1141,class_1162)).
neg(samebib(class_1141,class_1163)).
neg(samebib(class_1141,class_1164)).
neg(samebib(class_1141,class_1165)).
neg(samebib(class_1141,class_1166)).
samebib(class_1142,class_1125).
samebib(class_1142,class_1126).
samebib(class_1142,class_1127).
samebib(class_1142,class_1128).
samebib(class_1142,class_1129).
samebib(class_1142,class_1130).
samebib(class_1142,class_1131).
samebib(class_1142,class_1132).
samebib(class_1142,class_1133).
samebib(class_1142,class_1134).
samebib(class_1142,class_1136).
samebib(class_1142,class_1137).
samebib(class_1142,class_1138).
samebib(class_1142,class_1139).
samebib(class_1142,class_1140).
samebib(class_1142,class_1141).
samebib(class_1142,class_1142).
samebib(class_1142,class_1143).
samebib(class_1142,class_1144).
samebib(class_1142,class_1145).
samebib(class_1142,class_1146).
samebib(class_1142,class_1147).
samebib(class_1142,class_1148).
samebib(class_1142,class_1149).
samebib(class_1142,class_1150).
samebib(class_1142,class_1151).
samebib(class_1142,class_1152).
samebib(class_1142,class_1154).
samebib(class_1142,class_1155).
samebib(class_1142,class_1156).
samebib(class_1142,class_1157).
samebib(class_1142,class_1158).
samebib(class_1142,class_1159).
samebib(class_1142,class_1160).
samebib(class_1142,class_1161).
neg(samebib(class_1142,class_1162)).
neg(samebib(class_1142,class_1163)).
neg(samebib(class_1142,class_1164)).
neg(samebib(class_1142,class_1165)).
neg(samebib(class_1142,class_1166)).
samebib(class_1143,class_1125).
samebib(class_1143,class_1126).
samebib(class_1143,class_1127).
samebib(class_1143,class_1128).
samebib(class_1143,class_1129).
samebib(class_1143,class_1130).
samebib(class_1143,class_1131).
samebib(class_1143,class_1132).
samebib(class_1143,class_1133).
samebib(class_1143,class_1134).
samebib(class_1143,class_1136).
samebib(class_1143,class_1137).
samebib(class_1143,class_1138).
samebib(class_1143,class_1139).
samebib(class_1143,class_1140).
samebib(class_1143,class_1141).
samebib(class_1143,class_1142).
samebib(class_1143,class_1143).
samebib(class_1143,class_1144).
samebib(class_1143,class_1145).
samebib(class_1143,class_1146).
samebib(class_1143,class_1147).
samebib(class_1143,class_1148).
samebib(class_1143,class_1149).
samebib(class_1143,class_1150).
samebib(class_1143,class_1151).
samebib(class_1143,class_1152).
samebib(class_1143,class_1154).
samebib(class_1143,class_1155).
samebib(class_1143,class_1156).
samebib(class_1143,class_1157).
samebib(class_1143,class_1158).
samebib(class_1143,class_1159).
samebib(class_1143,class_1160).
samebib(class_1143,class_1161).
neg(samebib(class_1143,class_1162)).
neg(samebib(class_1143,class_1163)).
neg(samebib(class_1143,class_1164)).
neg(samebib(class_1143,class_1165)).
neg(samebib(class_1143,class_1166)).
samebib(class_1144,class_1125).
samebib(class_1144,class_1126).
samebib(class_1144,class_1127).
samebib(class_1144,class_1128).
samebib(class_1144,class_1129).
samebib(class_1144,class_1130).
samebib(class_1144,class_1131).
samebib(class_1144,class_1132).
samebib(class_1144,class_1133).
samebib(class_1144,class_1134).
samebib(class_1144,class_1136).
samebib(class_1144,class_1137).
samebib(class_1144,class_1138).
samebib(class_1144,class_1139).
samebib(class_1144,class_1140).
samebib(class_1144,class_1141).
samebib(class_1144,class_1142).
samebib(class_1144,class_1143).
samebib(class_1144,class_1144).
samebib(class_1144,class_1145).
samebib(class_1144,class_1146).
samebib(class_1144,class_1147).
samebib(class_1144,class_1148).
samebib(class_1144,class_1149).
samebib(class_1144,class_1150).
samebib(class_1144,class_1151).
samebib(class_1144,class_1152).
samebib(class_1144,class_1154).
samebib(class_1144,class_1155).
samebib(class_1144,class_1156).
samebib(class_1144,class_1157).
samebib(class_1144,class_1158).
samebib(class_1144,class_1159).
samebib(class_1144,class_1160).
samebib(class_1144,class_1161).
neg(samebib(class_1144,class_1162)).
neg(samebib(class_1144,class_1163)).
neg(samebib(class_1144,class_1164)).
neg(samebib(class_1144,class_1165)).
neg(samebib(class_1144,class_1166)).
samebib(class_1145,class_1125).
samebib(class_1145,class_1126).
samebib(class_1145,class_1127).
samebib(class_1145,class_1128).
samebib(class_1145,class_1129).
samebib(class_1145,class_1130).
samebib(class_1145,class_1131).
samebib(class_1145,class_1132).
samebib(class_1145,class_1133).
samebib(class_1145,class_1134).
samebib(class_1145,class_1136).
samebib(class_1145,class_1137).
samebib(class_1145,class_1138).
samebib(class_1145,class_1139).
samebib(class_1145,class_1140).
samebib(class_1145,class_1141).
samebib(class_1145,class_1142).
samebib(class_1145,class_1143).
samebib(class_1145,class_1144).
samebib(class_1145,class_1145).
samebib(class_1145,class_1146).
samebib(class_1145,class_1147).
samebib(class_1145,class_1148).
samebib(class_1145,class_1149).
samebib(class_1145,class_1150).
samebib(class_1145,class_1151).
samebib(class_1145,class_1152).
samebib(class_1145,class_1154).
samebib(class_1145,class_1155).
samebib(class_1145,class_1156).
samebib(class_1145,class_1157).
samebib(class_1145,class_1158).
samebib(class_1145,class_1159).
samebib(class_1145,class_1160).
samebib(class_1145,class_1161).
neg(samebib(class_1145,class_1162)).
neg(samebib(class_1145,class_1163)).
neg(samebib(class_1145,class_1164)).
neg(samebib(class_1145,class_1165)).
neg(samebib(class_1145,class_1166)).
samebib(class_1146,class_1125).
samebib(class_1146,class_1126).
samebib(class_1146,class_1127).
samebib(class_1146,class_1128).
samebib(class_1146,class_1129).
samebib(class_1146,class_1130).
samebib(class_1146,class_1131).
samebib(class_1146,class_1132).
samebib(class_1146,class_1133).
samebib(class_1146,class_1134).
samebib(class_1146,class_1136).
samebib(class_1146,class_1137).
samebib(class_1146,class_1138).
samebib(class_1146,class_1139).
samebib(class_1146,class_1140).
samebib(class_1146,class_1141).
samebib(class_1146,class_1142).
samebib(class_1146,class_1143).
samebib(class_1146,class_1144).
samebib(class_1146,class_1145).
samebib(class_1146,class_1146).
samebib(class_1146,class_1147).
samebib(class_1146,class_1148).
samebib(class_1146,class_1149).
samebib(class_1146,class_1150).
samebib(class_1146,class_1151).
samebib(class_1146,class_1152).
samebib(class_1146,class_1154).
samebib(class_1146,class_1155).
samebib(class_1146,class_1156).
samebib(class_1146,class_1157).
samebib(class_1146,class_1158).
samebib(class_1146,class_1159).
samebib(class_1146,class_1160).
samebib(class_1146,class_1161).
neg(samebib(class_1146,class_1162)).
neg(samebib(class_1146,class_1163)).
neg(samebib(class_1146,class_1164)).
neg(samebib(class_1146,class_1165)).
neg(samebib(class_1146,class_1166)).
samebib(class_1147,class_1125).
samebib(class_1147,class_1126).
samebib(class_1147,class_1127).
samebib(class_1147,class_1128).
samebib(class_1147,class_1129).
samebib(class_1147,class_1130).
samebib(class_1147,class_1131).
samebib(class_1147,class_1132).
samebib(class_1147,class_1133).
samebib(class_1147,class_1134).
samebib(class_1147,class_1136).
samebib(class_1147,class_1137).
samebib(class_1147,class_1138).
samebib(class_1147,class_1139).
samebib(class_1147,class_1140).
samebib(class_1147,class_1141).
samebib(class_1147,class_1142).
samebib(class_1147,class_1143).
samebib(class_1147,class_1144).
samebib(class_1147,class_1145).
samebib(class_1147,class_1146).
samebib(class_1147,class_1147).
samebib(class_1147,class_1148).
samebib(class_1147,class_1149).
samebib(class_1147,class_1150).
samebib(class_1147,class_1151).
samebib(class_1147,class_1152).
samebib(class_1147,class_1154).
samebib(class_1147,class_1155).
samebib(class_1147,class_1156).
samebib(class_1147,class_1157).
samebib(class_1147,class_1158).
samebib(class_1147,class_1159).
samebib(class_1147,class_1160).
samebib(class_1147,class_1161).
neg(samebib(class_1147,class_1162)).
neg(samebib(class_1147,class_1163)).
neg(samebib(class_1147,class_1164)).
neg(samebib(class_1147,class_1165)).
neg(samebib(class_1147,class_1166)).
samebib(class_1148,class_1125).
samebib(class_1148,class_1126).
samebib(class_1148,class_1127).
samebib(class_1148,class_1128).
samebib(class_1148,class_1129).
samebib(class_1148,class_1130).
samebib(class_1148,class_1131).
samebib(class_1148,class_1132).
samebib(class_1148,class_1133).
samebib(class_1148,class_1134).
samebib(class_1148,class_1136).
samebib(class_1148,class_1137).
samebib(class_1148,class_1138).
samebib(class_1148,class_1139).
samebib(class_1148,class_1140).
samebib(class_1148,class_1141).
samebib(class_1148,class_1142).
samebib(class_1148,class_1143).
samebib(class_1148,class_1144).
samebib(class_1148,class_1145).
samebib(class_1148,class_1146).
samebib(class_1148,class_1147).
samebib(class_1148,class_1148).
samebib(class_1148,class_1149).
samebib(class_1148,class_1150).
samebib(class_1148,class_1151).
samebib(class_1148,class_1152).
samebib(class_1148,class_1154).
samebib(class_1148,class_1155).
samebib(class_1148,class_1156).
samebib(class_1148,class_1157).
samebib(class_1148,class_1158).
samebib(class_1148,class_1159).
samebib(class_1148,class_1160).
samebib(class_1148,class_1161).
neg(samebib(class_1148,class_1162)).
neg(samebib(class_1148,class_1163)).
neg(samebib(class_1148,class_1164)).
neg(samebib(class_1148,class_1165)).
neg(samebib(class_1148,class_1166)).
samebib(class_1149,class_1125).
samebib(class_1149,class_1126).
samebib(class_1149,class_1127).
samebib(class_1149,class_1128).
samebib(class_1149,class_1129).
samebib(class_1149,class_1130).
samebib(class_1149,class_1131).
samebib(class_1149,class_1132).
samebib(class_1149,class_1133).
samebib(class_1149,class_1134).
samebib(class_1149,class_1136).
samebib(class_1149,class_1137).
samebib(class_1149,class_1138).
samebib(class_1149,class_1139).
samebib(class_1149,class_1140).
samebib(class_1149,class_1141).
samebib(class_1149,class_1142).
samebib(class_1149,class_1143).
samebib(class_1149,class_1144).
samebib(class_1149,class_1145).
samebib(class_1149,class_1146).
samebib(class_1149,class_1147).
samebib(class_1149,class_1148).
samebib(class_1149,class_1149).
samebib(class_1149,class_1150).
samebib(class_1149,class_1151).
samebib(class_1149,class_1152).
samebib(class_1149,class_1154).
samebib(class_1149,class_1155).
samebib(class_1149,class_1156).
samebib(class_1149,class_1157).
samebib(class_1149,class_1158).
samebib(class_1149,class_1159).
samebib(class_1149,class_1160).
samebib(class_1149,class_1161).
neg(samebib(class_1149,class_1162)).
neg(samebib(class_1149,class_1163)).
neg(samebib(class_1149,class_1164)).
neg(samebib(class_1149,class_1165)).
neg(samebib(class_1149,class_1166)).
samebib(class_1150,class_1125).
samebib(class_1150,class_1126).
samebib(class_1150,class_1127).
samebib(class_1150,class_1128).
samebib(class_1150,class_1129).
samebib(class_1150,class_1130).
samebib(class_1150,class_1131).
samebib(class_1150,class_1132).
samebib(class_1150,class_1133).
samebib(class_1150,class_1134).
samebib(class_1150,class_1136).
samebib(class_1150,class_1137).
samebib(class_1150,class_1138).
samebib(class_1150,class_1139).
samebib(class_1150,class_1140).
samebib(class_1150,class_1141).
samebib(class_1150,class_1142).
samebib(class_1150,class_1143).
samebib(class_1150,class_1144).
samebib(class_1150,class_1145).
samebib(class_1150,class_1146).
samebib(class_1150,class_1147).
samebib(class_1150,class_1148).
samebib(class_1150,class_1149).
samebib(class_1150,class_1150).
samebib(class_1150,class_1151).
samebib(class_1150,class_1152).
samebib(class_1150,class_1154).
samebib(class_1150,class_1155).
samebib(class_1150,class_1156).
samebib(class_1150,class_1157).
samebib(class_1150,class_1158).
samebib(class_1150,class_1159).
samebib(class_1150,class_1160).
samebib(class_1150,class_1161).
neg(samebib(class_1150,class_1162)).
neg(samebib(class_1150,class_1163)).
neg(samebib(class_1150,class_1164)).
neg(samebib(class_1150,class_1165)).
neg(samebib(class_1150,class_1166)).
samebib(class_1151,class_1125).
samebib(class_1151,class_1126).
samebib(class_1151,class_1127).
samebib(class_1151,class_1128).
samebib(class_1151,class_1129).
samebib(class_1151,class_1130).
samebib(class_1151,class_1131).
samebib(class_1151,class_1132).
samebib(class_1151,class_1133).
samebib(class_1151,class_1134).
samebib(class_1151,class_1136).
samebib(class_1151,class_1137).
samebib(class_1151,class_1138).
samebib(class_1151,class_1139).
samebib(class_1151,class_1140).
samebib(class_1151,class_1141).
samebib(class_1151,class_1142).
samebib(class_1151,class_1143).
samebib(class_1151,class_1144).
samebib(class_1151,class_1145).
samebib(class_1151,class_1146).
samebib(class_1151,class_1147).
samebib(class_1151,class_1148).
samebib(class_1151,class_1149).
samebib(class_1151,class_1150).
samebib(class_1151,class_1151).
samebib(class_1151,class_1152).
samebib(class_1151,class_1154).
samebib(class_1151,class_1155).
samebib(class_1151,class_1156).
samebib(class_1151,class_1157).
samebib(class_1151,class_1158).
samebib(class_1151,class_1159).
samebib(class_1151,class_1160).
samebib(class_1151,class_1161).
neg(samebib(class_1151,class_1162)).
neg(samebib(class_1151,class_1163)).
neg(samebib(class_1151,class_1164)).
neg(samebib(class_1151,class_1165)).
neg(samebib(class_1151,class_1166)).
samebib(class_1152,class_1125).
samebib(class_1152,class_1126).
samebib(class_1152,class_1127).
samebib(class_1152,class_1128).
samebib(class_1152,class_1129).
samebib(class_1152,class_1130).
samebib(class_1152,class_1131).
samebib(class_1152,class_1132).
samebib(class_1152,class_1133).
samebib(class_1152,class_1134).
samebib(class_1152,class_1136).
samebib(class_1152,class_1137).
samebib(class_1152,class_1138).
samebib(class_1152,class_1139).
samebib(class_1152,class_1140).
samebib(class_1152,class_1141).
samebib(class_1152,class_1142).
samebib(class_1152,class_1143).
samebib(class_1152,class_1144).
samebib(class_1152,class_1145).
samebib(class_1152,class_1146).
samebib(class_1152,class_1147).
samebib(class_1152,class_1148).
samebib(class_1152,class_1149).
samebib(class_1152,class_1150).
samebib(class_1152,class_1151).
samebib(class_1152,class_1152).
samebib(class_1152,class_1154).
samebib(class_1152,class_1155).
samebib(class_1152,class_1156).
samebib(class_1152,class_1157).
samebib(class_1152,class_1158).
samebib(class_1152,class_1159).
samebib(class_1152,class_1160).
samebib(class_1152,class_1161).
neg(samebib(class_1152,class_1162)).
neg(samebib(class_1152,class_1163)).
neg(samebib(class_1152,class_1164)).
neg(samebib(class_1152,class_1165)).
neg(samebib(class_1152,class_1166)).
samebib(class_1153,class_1135).
samebib(class_1153,class_1153).
neg(samebib(class_1153,class_1262)).
neg(samebib(class_1153,class_1263)).
neg(samebib(class_1153,class_1264)).
neg(samebib(class_1153,class_1265)).
neg(samebib(class_1153,class_1266)).
neg(samebib(class_1153,class_1270)).
neg(samebib(class_1153,class_1271)).
neg(samebib(class_1153,class_1273)).
neg(samebib(class_1153,class_1276)).
samebib(class_1154,class_1125).
samebib(class_1154,class_1126).
samebib(class_1154,class_1127).
samebib(class_1154,class_1128).
samebib(class_1154,class_1129).
samebib(class_1154,class_1130).
samebib(class_1154,class_1131).
samebib(class_1154,class_1132).
samebib(class_1154,class_1133).
samebib(class_1154,class_1134).
samebib(class_1154,class_1136).
samebib(class_1154,class_1137).
samebib(class_1154,class_1138).
samebib(class_1154,class_1139).
samebib(class_1154,class_1140).
samebib(class_1154,class_1141).
samebib(class_1154,class_1142).
samebib(class_1154,class_1143).
samebib(class_1154,class_1144).
samebib(class_1154,class_1145).
samebib(class_1154,class_1146).
samebib(class_1154,class_1147).
samebib(class_1154,class_1148).
samebib(class_1154,class_1149).
samebib(class_1154,class_1150).
samebib(class_1154,class_1151).
samebib(class_1154,class_1152).
samebib(class_1154,class_1154).
samebib(class_1154,class_1155).
samebib(class_1154,class_1156).
samebib(class_1154,class_1157).
samebib(class_1154,class_1158).
samebib(class_1154,class_1159).
samebib(class_1154,class_1160).
samebib(class_1154,class_1161).
neg(samebib(class_1154,class_1162)).
neg(samebib(class_1154,class_1163)).
neg(samebib(class_1154,class_1164)).
neg(samebib(class_1154,class_1165)).
neg(samebib(class_1154,class_1166)).
samebib(class_1155,class_1125).
samebib(class_1155,class_1126).
samebib(class_1155,class_1127).
samebib(class_1155,class_1128).
samebib(class_1155,class_1129).
samebib(class_1155,class_1130).
samebib(class_1155,class_1131).
samebib(class_1155,class_1132).
samebib(class_1155,class_1133).
samebib(class_1155,class_1134).
samebib(class_1155,class_1136).
samebib(class_1155,class_1137).
samebib(class_1155,class_1138).
samebib(class_1155,class_1139).
samebib(class_1155,class_1140).
samebib(class_1155,class_1141).
samebib(class_1155,class_1142).
samebib(class_1155,class_1143).
samebib(class_1155,class_1144).
samebib(class_1155,class_1145).
samebib(class_1155,class_1146).
samebib(class_1155,class_1147).
samebib(class_1155,class_1148).
samebib(class_1155,class_1149).
samebib(class_1155,class_1150).
samebib(class_1155,class_1151).
samebib(class_1155,class_1152).
samebib(class_1155,class_1154).
samebib(class_1155,class_1155).
samebib(class_1155,class_1156).
samebib(class_1155,class_1157).
samebib(class_1155,class_1158).
samebib(class_1155,class_1159).
samebib(class_1155,class_1160).
samebib(class_1155,class_1161).
neg(samebib(class_1155,class_1162)).
neg(samebib(class_1155,class_1163)).
neg(samebib(class_1155,class_1164)).
neg(samebib(class_1155,class_1165)).
neg(samebib(class_1155,class_1166)).
samebib(class_1156,class_1125).
samebib(class_1156,class_1126).
samebib(class_1156,class_1127).
samebib(class_1156,class_1128).
samebib(class_1156,class_1129).
samebib(class_1156,class_1130).
samebib(class_1156,class_1131).
samebib(class_1156,class_1132).
samebib(class_1156,class_1133).
samebib(class_1156,class_1134).
samebib(class_1156,class_1136).
samebib(class_1156,class_1137).
samebib(class_1156,class_1138).
samebib(class_1156,class_1139).
samebib(class_1156,class_1140).
samebib(class_1156,class_1141).
samebib(class_1156,class_1142).
samebib(class_1156,class_1143).
samebib(class_1156,class_1144).
samebib(class_1156,class_1145).
samebib(class_1156,class_1146).
samebib(class_1156,class_1147).
samebib(class_1156,class_1148).
samebib(class_1156,class_1149).
samebib(class_1156,class_1150).
samebib(class_1156,class_1151).
samebib(class_1156,class_1152).
samebib(class_1156,class_1154).
samebib(class_1156,class_1155).
samebib(class_1156,class_1156).
samebib(class_1156,class_1157).
samebib(class_1156,class_1158).
samebib(class_1156,class_1159).
samebib(class_1156,class_1160).
samebib(class_1156,class_1161).
neg(samebib(class_1156,class_1162)).
neg(samebib(class_1156,class_1163)).
neg(samebib(class_1156,class_1164)).
neg(samebib(class_1156,class_1165)).
neg(samebib(class_1156,class_1166)).
samebib(class_1157,class_1125).
samebib(class_1157,class_1126).
samebib(class_1157,class_1127).
samebib(class_1157,class_1128).
samebib(class_1157,class_1129).
samebib(class_1157,class_1130).
samebib(class_1157,class_1131).
samebib(class_1157,class_1132).
samebib(class_1157,class_1133).
samebib(class_1157,class_1134).
samebib(class_1157,class_1136).
samebib(class_1157,class_1137).
samebib(class_1157,class_1138).
samebib(class_1157,class_1139).
samebib(class_1157,class_1140).
samebib(class_1157,class_1141).
samebib(class_1157,class_1142).
samebib(class_1157,class_1143).
samebib(class_1157,class_1144).
samebib(class_1157,class_1145).
samebib(class_1157,class_1146).
samebib(class_1157,class_1147).
samebib(class_1157,class_1148).
samebib(class_1157,class_1149).
samebib(class_1157,class_1150).
samebib(class_1157,class_1151).
samebib(class_1157,class_1152).
samebib(class_1157,class_1154).
samebib(class_1157,class_1155).
samebib(class_1157,class_1156).
samebib(class_1157,class_1157).
samebib(class_1157,class_1158).
samebib(class_1157,class_1159).
samebib(class_1157,class_1160).
samebib(class_1157,class_1161).
neg(samebib(class_1157,class_1162)).
neg(samebib(class_1157,class_1163)).
neg(samebib(class_1157,class_1164)).
neg(samebib(class_1157,class_1165)).
neg(samebib(class_1157,class_1166)).
samebib(class_1158,class_1125).
samebib(class_1158,class_1126).
samebib(class_1158,class_1127).
samebib(class_1158,class_1128).
samebib(class_1158,class_1129).
samebib(class_1158,class_1130).
samebib(class_1158,class_1131).
samebib(class_1158,class_1132).
samebib(class_1158,class_1133).
samebib(class_1158,class_1134).
samebib(class_1158,class_1136).
samebib(class_1158,class_1137).
samebib(class_1158,class_1138).
samebib(class_1158,class_1139).
samebib(class_1158,class_1140).
samebib(class_1158,class_1141).
samebib(class_1158,class_1142).
samebib(class_1158,class_1143).
samebib(class_1158,class_1144).
samebib(class_1158,class_1145).
samebib(class_1158,class_1146).
samebib(class_1158,class_1147).
samebib(class_1158,class_1148).
samebib(class_1158,class_1149).
samebib(class_1158,class_1150).
samebib(class_1158,class_1151).
samebib(class_1158,class_1152).
samebib(class_1158,class_1154).
samebib(class_1158,class_1155).
samebib(class_1158,class_1156).
samebib(class_1158,class_1157).
samebib(class_1158,class_1158).
samebib(class_1158,class_1159).
samebib(class_1158,class_1160).
samebib(class_1158,class_1161).
neg(samebib(class_1158,class_1162)).
neg(samebib(class_1158,class_1163)).
neg(samebib(class_1158,class_1164)).
neg(samebib(class_1158,class_1165)).
neg(samebib(class_1158,class_1166)).
samebib(class_1159,class_1125).
samebib(class_1159,class_1126).
samebib(class_1159,class_1127).
samebib(class_1159,class_1128).
samebib(class_1159,class_1129).
samebib(class_1159,class_1130).
samebib(class_1159,class_1131).
samebib(class_1159,class_1132).
samebib(class_1159,class_1133).
samebib(class_1159,class_1134).
samebib(class_1159,class_1136).
samebib(class_1159,class_1137).
samebib(class_1159,class_1138).
samebib(class_1159,class_1139).
samebib(class_1159,class_1140).
samebib(class_1159,class_1141).
samebib(class_1159,class_1142).
samebib(class_1159,class_1143).
samebib(class_1159,class_1144).
samebib(class_1159,class_1145).
samebib(class_1159,class_1146).
samebib(class_1159,class_1147).
samebib(class_1159,class_1148).
samebib(class_1159,class_1149).
samebib(class_1159,class_1150).
samebib(class_1159,class_1151).
samebib(class_1159,class_1152).
samebib(class_1159,class_1154).
samebib(class_1159,class_1155).
samebib(class_1159,class_1156).
samebib(class_1159,class_1157).
samebib(class_1159,class_1158).
samebib(class_1159,class_1159).
samebib(class_1159,class_1160).
samebib(class_1159,class_1161).
neg(samebib(class_1159,class_1162)).
neg(samebib(class_1159,class_1163)).
neg(samebib(class_1159,class_1164)).
neg(samebib(class_1159,class_1165)).
neg(samebib(class_1159,class_1166)).
samebib(class_1160,class_1125).
samebib(class_1160,class_1126).
samebib(class_1160,class_1127).
samebib(class_1160,class_1128).
samebib(class_1160,class_1129).
samebib(class_1160,class_1130).
samebib(class_1160,class_1131).
samebib(class_1160,class_1132).
samebib(class_1160,class_1133).
samebib(class_1160,class_1134).
samebib(class_1160,class_1136).
samebib(class_1160,class_1137).
samebib(class_1160,class_1138).
samebib(class_1160,class_1139).
samebib(class_1160,class_1140).
samebib(class_1160,class_1141).
samebib(class_1160,class_1142).
samebib(class_1160,class_1143).
samebib(class_1160,class_1144).
samebib(class_1160,class_1145).
samebib(class_1160,class_1146).
samebib(class_1160,class_1147).
samebib(class_1160,class_1148).
samebib(class_1160,class_1149).
samebib(class_1160,class_1150).
samebib(class_1160,class_1151).
samebib(class_1160,class_1152).
samebib(class_1160,class_1154).
samebib(class_1160,class_1155).
samebib(class_1160,class_1156).
samebib(class_1160,class_1157).
samebib(class_1160,class_1158).
samebib(class_1160,class_1159).
samebib(class_1160,class_1160).
samebib(class_1160,class_1161).
neg(samebib(class_1160,class_1162)).
neg(samebib(class_1160,class_1163)).
neg(samebib(class_1160,class_1164)).
neg(samebib(class_1160,class_1165)).
neg(samebib(class_1160,class_1166)).
samebib(class_1161,class_1125).
samebib(class_1161,class_1126).
samebib(class_1161,class_1127).
samebib(class_1161,class_1128).
samebib(class_1161,class_1129).
samebib(class_1161,class_1130).
samebib(class_1161,class_1131).
samebib(class_1161,class_1132).
samebib(class_1161,class_1133).
samebib(class_1161,class_1134).
samebib(class_1161,class_1136).
samebib(class_1161,class_1137).
samebib(class_1161,class_1138).
samebib(class_1161,class_1139).
samebib(class_1161,class_1140).
samebib(class_1161,class_1141).
samebib(class_1161,class_1142).
samebib(class_1161,class_1143).
samebib(class_1161,class_1144).
samebib(class_1161,class_1145).
samebib(class_1161,class_1146).
samebib(class_1161,class_1147).
samebib(class_1161,class_1148).
samebib(class_1161,class_1149).
samebib(class_1161,class_1150).
samebib(class_1161,class_1151).
samebib(class_1161,class_1152).
samebib(class_1161,class_1154).
samebib(class_1161,class_1155).
samebib(class_1161,class_1156).
samebib(class_1161,class_1157).
samebib(class_1161,class_1158).
samebib(class_1161,class_1159).
samebib(class_1161,class_1160).
samebib(class_1161,class_1161).
neg(samebib(class_1161,class_1162)).
neg(samebib(class_1161,class_1163)).
neg(samebib(class_1161,class_1164)).
neg(samebib(class_1161,class_1165)).
neg(samebib(class_1161,class_1166)).
neg(samebib(class_1162,class_1125)).
neg(samebib(class_1162,class_1126)).
neg(samebib(class_1162,class_1127)).
neg(samebib(class_1162,class_1128)).
neg(samebib(class_1162,class_1129)).
neg(samebib(class_1162,class_1130)).
neg(samebib(class_1162,class_1131)).
neg(samebib(class_1162,class_1132)).
neg(samebib(class_1162,class_1133)).
neg(samebib(class_1162,class_1134)).
neg(samebib(class_1162,class_1136)).
neg(samebib(class_1162,class_1137)).
neg(samebib(class_1162,class_1138)).
neg(samebib(class_1162,class_1139)).
neg(samebib(class_1162,class_1140)).
neg(samebib(class_1162,class_1141)).
neg(samebib(class_1162,class_1142)).
neg(samebib(class_1162,class_1143)).
neg(samebib(class_1162,class_1144)).
neg(samebib(class_1162,class_1145)).
neg(samebib(class_1162,class_1146)).
neg(samebib(class_1162,class_1147)).
neg(samebib(class_1162,class_1148)).
neg(samebib(class_1162,class_1149)).
neg(samebib(class_1162,class_1150)).
neg(samebib(class_1162,class_1151)).
neg(samebib(class_1162,class_1152)).
neg(samebib(class_1162,class_1154)).
neg(samebib(class_1162,class_1155)).
neg(samebib(class_1162,class_1156)).
neg(samebib(class_1162,class_1157)).
neg(samebib(class_1162,class_1158)).
neg(samebib(class_1162,class_1159)).
neg(samebib(class_1162,class_1160)).
neg(samebib(class_1162,class_1161)).
samebib(class_1162,class_1162).
samebib(class_1162,class_1163).
samebib(class_1162,class_1164).
samebib(class_1162,class_1165).
samebib(class_1162,class_1166).
neg(samebib(class_1163,class_1127)).
neg(samebib(class_1163,class_1128)).
neg(samebib(class_1163,class_1129)).
neg(samebib(class_1163,class_1130)).
neg(samebib(class_1163,class_1131)).
neg(samebib(class_1163,class_1132)).
neg(samebib(class_1163,class_1133)).
neg(samebib(class_1163,class_1134)).
neg(samebib(class_1163,class_1136)).
neg(samebib(class_1163,class_1137)).
neg(samebib(class_1163,class_1138)).
neg(samebib(class_1163,class_1139)).
neg(samebib(class_1163,class_1140)).
neg(samebib(class_1163,class_1141)).
neg(samebib(class_1163,class_1142)).
neg(samebib(class_1163,class_1143)).
neg(samebib(class_1163,class_1144)).
neg(samebib(class_1163,class_1145)).
neg(samebib(class_1163,class_1146)).
neg(samebib(class_1163,class_1147)).
neg(samebib(class_1163,class_1148)).
neg(samebib(class_1163,class_1149)).
neg(samebib(class_1163,class_1150)).
neg(samebib(class_1163,class_1151)).
neg(samebib(class_1163,class_1152)).
neg(samebib(class_1163,class_1154)).
neg(samebib(class_1163,class_1155)).
neg(samebib(class_1163,class_1156)).
neg(samebib(class_1163,class_1157)).
neg(samebib(class_1163,class_1158)).
neg(samebib(class_1163,class_1159)).
neg(samebib(class_1163,class_1160)).
neg(samebib(class_1163,class_1161)).
samebib(class_1163,class_1162).
samebib(class_1163,class_1163).
samebib(class_1163,class_1164).
samebib(class_1163,class_1165).
samebib(class_1163,class_1166).
neg(samebib(class_1164,class_1125)).
neg(samebib(class_1164,class_1126)).
neg(samebib(class_1164,class_1127)).
neg(samebib(class_1164,class_1128)).
neg(samebib(class_1164,class_1129)).
neg(samebib(class_1164,class_1130)).
neg(samebib(class_1164,class_1131)).
neg(samebib(class_1164,class_1132)).
neg(samebib(class_1164,class_1133)).
neg(samebib(class_1164,class_1134)).
neg(samebib(class_1164,class_1136)).
neg(samebib(class_1164,class_1137)).
neg(samebib(class_1164,class_1138)).
neg(samebib(class_1164,class_1139)).
neg(samebib(class_1164,class_1140)).
neg(samebib(class_1164,class_1141)).
neg(samebib(class_1164,class_1142)).
neg(samebib(class_1164,class_1143)).
neg(samebib(class_1164,class_1144)).
neg(samebib(class_1164,class_1145)).
neg(samebib(class_1164,class_1146)).
neg(samebib(class_1164,class_1147)).
neg(samebib(class_1164,class_1148)).
neg(samebib(class_1164,class_1149)).
neg(samebib(class_1164,class_1150)).
neg(samebib(class_1164,class_1151)).
neg(samebib(class_1164,class_1152)).
neg(samebib(class_1164,class_1154)).
neg(samebib(class_1164,class_1155)).
neg(samebib(class_1164,class_1156)).
neg(samebib(class_1164,class_1157)).
neg(samebib(class_1164,class_1158)).
neg(samebib(class_1164,class_1159)).
neg(samebib(class_1164,class_1160)).
neg(samebib(class_1164,class_1161)).
samebib(class_1164,class_1162).
samebib(class_1164,class_1163).
samebib(class_1164,class_1164).
samebib(class_1164,class_1165).
samebib(class_1164,class_1166).
neg(samebib(class_1165,class_1125)).
neg(samebib(class_1165,class_1126)).
neg(samebib(class_1165,class_1127)).
neg(samebib(class_1165,class_1128)).
neg(samebib(class_1165,class_1129)).
neg(samebib(class_1165,class_1130)).
neg(samebib(class_1165,class_1131)).
neg(samebib(class_1165,class_1132)).
neg(samebib(class_1165,class_1133)).
neg(samebib(class_1165,class_1134)).
neg(samebib(class_1165,class_1136)).
neg(samebib(class_1165,class_1137)).
neg(samebib(class_1165,class_1138)).
neg(samebib(class_1165,class_1139)).
neg(samebib(class_1165,class_1140)).
neg(samebib(class_1165,class_1141)).
neg(samebib(class_1165,class_1142)).
neg(samebib(class_1165,class_1143)).
neg(samebib(class_1165,class_1144)).
neg(samebib(class_1165,class_1145)).
neg(samebib(class_1165,class_1146)).
neg(samebib(class_1165,class_1147)).
neg(samebib(class_1165,class_1148)).
neg(samebib(class_1165,class_1149)).
neg(samebib(class_1165,class_1150)).
neg(samebib(class_1165,class_1151)).
neg(samebib(class_1165,class_1152)).
neg(samebib(class_1165,class_1154)).
neg(samebib(class_1165,class_1155)).
neg(samebib(class_1165,class_1156)).
neg(samebib(class_1165,class_1157)).
neg(samebib(class_1165,class_1158)).
neg(samebib(class_1165,class_1159)).
neg(samebib(class_1165,class_1160)).
neg(samebib(class_1165,class_1161)).
samebib(class_1165,class_1162).
samebib(class_1165,class_1163).
samebib(class_1165,class_1164).
samebib(class_1165,class_1165).
samebib(class_1165,class_1166).
neg(samebib(class_1166,class_1127)).
neg(samebib(class_1166,class_1128)).
neg(samebib(class_1166,class_1129)).
neg(samebib(class_1166,class_1130)).
neg(samebib(class_1166,class_1131)).
neg(samebib(class_1166,class_1132)).
neg(samebib(class_1166,class_1133)).
neg(samebib(class_1166,class_1134)).
neg(samebib(class_1166,class_1136)).
neg(samebib(class_1166,class_1137)).
neg(samebib(class_1166,class_1138)).
neg(samebib(class_1166,class_1139)).
neg(samebib(class_1166,class_1140)).
neg(samebib(class_1166,class_1141)).
neg(samebib(class_1166,class_1142)).
neg(samebib(class_1166,class_1143)).
neg(samebib(class_1166,class_1144)).
neg(samebib(class_1166,class_1145)).
neg(samebib(class_1166,class_1146)).
neg(samebib(class_1166,class_1147)).
neg(samebib(class_1166,class_1148)).
neg(samebib(class_1166,class_1149)).
neg(samebib(class_1166,class_1150)).
neg(samebib(class_1166,class_1151)).
neg(samebib(class_1166,class_1152)).
neg(samebib(class_1166,class_1154)).
neg(samebib(class_1166,class_1155)).
neg(samebib(class_1166,class_1156)).
neg(samebib(class_1166,class_1157)).
neg(samebib(class_1166,class_1158)).
neg(samebib(class_1166,class_1159)).
neg(samebib(class_1166,class_1160)).
neg(samebib(class_1166,class_1161)).
samebib(class_1166,class_1162).
samebib(class_1166,class_1163).
samebib(class_1166,class_1164).
samebib(class_1166,class_1165).
samebib(class_1166,class_1166).
samebib(class_1247,class_1247).
neg(samebib(class_1247,class_1256)).
neg(samebib(class_1247,class_1257)).
samebib(class_1248,class_1248).
neg(samebib(class_1248,class_1252)).
samebib(class_1249,class_1249).
samebib(class_1249,class_1250).
neg(samebib(class_1249,class_1251)).
neg(samebib(class_1249,class_1252)).
neg(samebib(class_1249,class_1291)).
neg(samebib(class_1249,class_1292)).
neg(samebib(class_1249,class_1293)).
neg(samebib(class_1249,class_1294)).
samebib(class_1250,class_1249).
samebib(class_1250,class_1250).
neg(samebib(class_1250,class_1251)).
neg(samebib(class_1250,class_1252)).
neg(samebib(class_1250,class_1291)).
neg(samebib(class_1250,class_1292)).
neg(samebib(class_1250,class_1293)).
neg(samebib(class_1250,class_1294)).
neg(samebib(class_1251,class_1249)).
neg(samebib(class_1251,class_1250)).
samebib(class_1251,class_1251).
neg(samebib(class_1251,class_1252)).
neg(samebib(class_1251,class_1253)).
neg(samebib(class_1251,class_1254)).
neg(samebib(class_1251,class_1255)).
neg(samebib(class_1251,class_1258)).
neg(samebib(class_1251,class_1259)).
neg(samebib(class_1251,class_1291)).
neg(samebib(class_1251,class_1292)).
neg(samebib(class_1251,class_1293)).
neg(samebib(class_1251,class_1294)).
neg(samebib(class_1252,class_1248)).
neg(samebib(class_1252,class_1249)).
neg(samebib(class_1252,class_1250)).
neg(samebib(class_1252,class_1251)).
samebib(class_1252,class_1252).
neg(samebib(class_1252,class_1253)).
neg(samebib(class_1252,class_1254)).
neg(samebib(class_1252,class_1255)).
neg(samebib(class_1252,class_1258)).
neg(samebib(class_1252,class_1259)).
neg(samebib(class_1252,class_1262)).
neg(samebib(class_1252,class_1291)).
neg(samebib(class_1252,class_1292)).
neg(samebib(class_1252,class_1293)).
neg(samebib(class_1252,class_1294)).
neg(samebib(class_1253,class_1251)).
neg(samebib(class_1253,class_1252)).
samebib(class_1253,class_1253).
neg(samebib(class_1253,class_1254)).
neg(samebib(class_1253,class_1255)).
neg(samebib(class_1253,class_1258)).
neg(samebib(class_1253,class_1259)).
neg(samebib(class_1253,class_1260)).
neg(samebib(class_1253,class_1261)).
neg(samebib(class_1253,class_1263)).
neg(samebib(class_1253,class_1264)).
neg(samebib(class_1253,class_1265)).
neg(samebib(class_1253,class_1266)).
neg(samebib(class_1253,class_1270)).
neg(samebib(class_1253,class_1276)).
neg(samebib(class_1253,class_1284)).
neg(samebib(class_1254,class_1251)).
neg(samebib(class_1254,class_1252)).
neg(samebib(class_1254,class_1253)).
samebib(class_1254,class_1254).
samebib(class_1254,class_1255).
samebib(class_1254,class_1256).
samebib(class_1254,class_1257).
samebib(class_1254,class_1258).
samebib(class_1254,class_1259).
neg(samebib(class_1254,class_1262)).
neg(samebib(class_1254,class_1263)).
neg(samebib(class_1254,class_1264)).
neg(samebib(class_1254,class_1265)).
neg(samebib(class_1254,class_1266)).
neg(samebib(class_1254,class_1267)).
neg(samebib(class_1254,class_1268)).
neg(samebib(class_1254,class_1269)).
neg(samebib(class_1254,class_1270)).
neg(samebib(class_1254,class_1271)).
neg(samebib(class_1254,class_1272)).
neg(samebib(class_1254,class_1273)).
neg(samebib(class_1254,class_1275)).
neg(samebib(class_1254,class_1276)).
neg(samebib(class_1254,class_1277)).
neg(samebib(class_1254,class_1278)).
neg(samebib(class_1254,class_1279)).
neg(samebib(class_1254,class_1280)).
neg(samebib(class_1254,class_1284)).
neg(samebib(class_1254,class_1285)).
neg(samebib(class_1254,class_1286)).
neg(samebib(class_1254,class_1287)).
neg(samebib(class_1254,class_1288)).
neg(samebib(class_1254,class_1290)).
neg(samebib(class_1254,class_1291)).
neg(samebib(class_1254,class_1292)).
neg(samebib(class_1254,class_1293)).
neg(samebib(class_1254,class_1294)).
neg(samebib(class_1255,class_1251)).
neg(samebib(class_1255,class_1252)).
neg(samebib(class_1255,class_1253)).
samebib(class_1255,class_1254).
samebib(class_1255,class_1255).
samebib(class_1255,class_1256).
samebib(class_1255,class_1257).
samebib(class_1255,class_1258).
samebib(class_1255,class_1259).
neg(samebib(class_1255,class_1262)).
neg(samebib(class_1255,class_1263)).
neg(samebib(class_1255,class_1264)).
neg(samebib(class_1255,class_1265)).
neg(samebib(class_1255,class_1266)).
neg(samebib(class_1255,class_1267)).
neg(samebib(class_1255,class_1268)).
neg(samebib(class_1255,class_1269)).
neg(samebib(class_1255,class_1270)).
neg(samebib(class_1255,class_1271)).
neg(samebib(class_1255,class_1272)).
neg(samebib(class_1255,class_1273)).
neg(samebib(class_1255,class_1275)).
neg(samebib(class_1255,class_1276)).
neg(samebib(class_1255,class_1277)).
neg(samebib(class_1255,class_1278)).
neg(samebib(class_1255,class_1279)).
neg(samebib(class_1255,class_1280)).
neg(samebib(class_1255,class_1284)).
neg(samebib(class_1255,class_1285)).
neg(samebib(class_1255,class_1286)).
neg(samebib(class_1255,class_1287)).
neg(samebib(class_1255,class_1288)).
neg(samebib(class_1255,class_1290)).
neg(samebib(class_1255,class_1291)).
neg(samebib(class_1255,class_1292)).
neg(samebib(class_1255,class_1293)).
neg(samebib(class_1255,class_1294)).
neg(samebib(class_1256,class_1247)).
samebib(class_1256,class_1254).
samebib(class_1256,class_1255).
samebib(class_1256,class_1256).
samebib(class_1256,class_1257).
samebib(class_1256,class_1258).
samebib(class_1256,class_1259).
neg(samebib(class_1256,class_1262)).
neg(samebib(class_1256,class_1263)).
neg(samebib(class_1256,class_1264)).
neg(samebib(class_1256,class_1265)).
neg(samebib(class_1256,class_1266)).
neg(samebib(class_1256,class_1267)).
neg(samebib(class_1256,class_1268)).
neg(samebib(class_1256,class_1269)).
neg(samebib(class_1256,class_1270)).
neg(samebib(class_1256,class_1271)).
neg(samebib(class_1256,class_1272)).
neg(samebib(class_1256,class_1273)).
neg(samebib(class_1256,class_1275)).
neg(samebib(class_1256,class_1276)).
neg(samebib(class_1256,class_1277)).
neg(samebib(class_1256,class_1278)).
neg(samebib(class_1256,class_1279)).
neg(samebib(class_1256,class_1280)).
neg(samebib(class_1256,class_1284)).
neg(samebib(class_1256,class_1285)).
neg(samebib(class_1256,class_1286)).
neg(samebib(class_1256,class_1287)).
neg(samebib(class_1256,class_1288)).
neg(samebib(class_1256,class_1290)).
neg(samebib(class_1256,class_1291)).
neg(samebib(class_1256,class_1292)).
neg(samebib(class_1256,class_1293)).
neg(samebib(class_1256,class_1294)).
neg(samebib(class_1257,class_1247)).
samebib(class_1257,class_1254).
samebib(class_1257,class_1255).
samebib(class_1257,class_1256).
samebib(class_1257,class_1257).
samebib(class_1257,class_1258).
samebib(class_1257,class_1259).
neg(samebib(class_1257,class_1262)).
neg(samebib(class_1257,class_1263)).
neg(samebib(class_1257,class_1264)).
neg(samebib(class_1257,class_1265)).
neg(samebib(class_1257,class_1266)).
neg(samebib(class_1257,class_1267)).
neg(samebib(class_1257,class_1268)).
neg(samebib(class_1257,class_1269)).
neg(samebib(class_1257,class_1270)).
neg(samebib(class_1257,class_1271)).
neg(samebib(class_1257,class_1272)).
neg(samebib(class_1257,class_1273)).
neg(samebib(class_1257,class_1275)).
neg(samebib(class_1257,class_1276)).
neg(samebib(class_1257,class_1277)).
neg(samebib(class_1257,class_1278)).
neg(samebib(class_1257,class_1279)).
neg(samebib(class_1257,class_1280)).
neg(samebib(class_1257,class_1284)).
neg(samebib(class_1257,class_1285)).
neg(samebib(class_1257,class_1286)).
neg(samebib(class_1257,class_1287)).
neg(samebib(class_1257,class_1288)).
neg(samebib(class_1257,class_1290)).
neg(samebib(class_1257,class_1291)).
neg(samebib(class_1257,class_1292)).
neg(samebib(class_1257,class_1293)).
neg(samebib(class_1257,class_1294)).
neg(samebib(class_1258,class_1251)).
neg(samebib(class_1258,class_1252)).
neg(samebib(class_1258,class_1253)).
samebib(class_1258,class_1254).
samebib(class_1258,class_1255).
samebib(class_1258,class_1256).
samebib(class_1258,class_1257).
samebib(class_1258,class_1258).
samebib(class_1258,class_1259).
neg(samebib(class_1258,class_1262)).
neg(samebib(class_1258,class_1263)).
neg(samebib(class_1258,class_1264)).
neg(samebib(class_1258,class_1265)).
neg(samebib(class_1258,class_1266)).
neg(samebib(class_1258,class_1267)).
neg(samebib(class_1258,class_1268)).
neg(samebib(class_1258,class_1269)).
neg(samebib(class_1258,class_1270)).
neg(samebib(class_1258,class_1271)).
neg(samebib(class_1258,class_1272)).
neg(samebib(class_1258,class_1273)).
neg(samebib(class_1258,class_1275)).
neg(samebib(class_1258,class_1276)).
neg(samebib(class_1258,class_1277)).
neg(samebib(class_1258,class_1278)).
neg(samebib(class_1258,class_1279)).
neg(samebib(class_1258,class_1280)).
neg(samebib(class_1258,class_1284)).
neg(samebib(class_1258,class_1285)).
neg(samebib(class_1258,class_1286)).
neg(samebib(class_1258,class_1287)).
neg(samebib(class_1258,class_1288)).
neg(samebib(class_1258,class_1290)).
neg(samebib(class_1258,class_1291)).
neg(samebib(class_1258,class_1292)).
neg(samebib(class_1258,class_1293)).
neg(samebib(class_1258,class_1294)).
neg(samebib(class_1259,class_1251)).
neg(samebib(class_1259,class_1252)).
neg(samebib(class_1259,class_1253)).
samebib(class_1259,class_1254).
samebib(class_1259,class_1255).
samebib(class_1259,class_1256).
samebib(class_1259,class_1257).
samebib(class_1259,class_1258).
samebib(class_1259,class_1259).
neg(samebib(class_1259,class_1262)).
neg(samebib(class_1259,class_1263)).
neg(samebib(class_1259,class_1264)).
neg(samebib(class_1259,class_1265)).
neg(samebib(class_1259,class_1266)).
neg(samebib(class_1259,class_1267)).
neg(samebib(class_1259,class_1268)).
neg(samebib(class_1259,class_1269)).
neg(samebib(class_1259,class_1270)).
neg(samebib(class_1259,class_1271)).
neg(samebib(class_1259,class_1272)).
neg(samebib(class_1259,class_1273)).
neg(samebib(class_1259,class_1275)).
neg(samebib(class_1259,class_1276)).
neg(samebib(class_1259,class_1277)).
neg(samebib(class_1259,class_1278)).
neg(samebib(class_1259,class_1279)).
neg(samebib(class_1259,class_1280)).
neg(samebib(class_1259,class_1284)).
neg(samebib(class_1259,class_1285)).
neg(samebib(class_1259,class_1286)).
neg(samebib(class_1259,class_1287)).
neg(samebib(class_1259,class_1288)).
neg(samebib(class_1259,class_1290)).
neg(samebib(class_1259,class_1291)).
neg(samebib(class_1259,class_1292)).
neg(samebib(class_1259,class_1293)).
neg(samebib(class_1259,class_1294)).
neg(samebib(class_1260,class_1253)).
samebib(class_1260,class_1260).
samebib(class_1260,class_1261).
neg(samebib(class_1260,class_1263)).
neg(samebib(class_1260,class_1264)).
neg(samebib(class_1260,class_1265)).
neg(samebib(class_1260,class_1266)).
neg(samebib(class_1260,class_1270)).
neg(samebib(class_1260,class_1276)).
neg(samebib(class_1260,class_1284)).
neg(samebib(class_1261,class_1253)).
samebib(class_1261,class_1260).
samebib(class_1261,class_1261).
neg(samebib(class_1261,class_1263)).
neg(samebib(class_1261,class_1264)).
neg(samebib(class_1261,class_1265)).
neg(samebib(class_1261,class_1266)).
neg(samebib(class_1261,class_1270)).
neg(samebib(class_1261,class_1276)).
neg(samebib(class_1261,class_1284)).
neg(samebib(class_1262,class_1135)).
neg(samebib(class_1262,class_1153)).
neg(samebib(class_1262,class_1252)).
neg(samebib(class_1262,class_1254)).
neg(samebib(class_1262,class_1255)).
neg(samebib(class_1262,class_1256)).
neg(samebib(class_1262,class_1257)).
neg(samebib(class_1262,class_1258)).
neg(samebib(class_1262,class_1259)).
samebib(class_1262,class_1262).
neg(samebib(class_1262,class_1263)).
neg(samebib(class_1262,class_1264)).
neg(samebib(class_1262,class_1265)).
neg(samebib(class_1262,class_1266)).
neg(samebib(class_1262,class_1267)).
neg(samebib(class_1262,class_1268)).
neg(samebib(class_1262,class_1269)).
neg(samebib(class_1262,class_1270)).
neg(samebib(class_1262,class_1271)).
neg(samebib(class_1262,class_1272)).
neg(samebib(class_1262,class_1273)).
neg(samebib(class_1262,class_1275)).
neg(samebib(class_1262,class_1276)).
neg(samebib(class_1262,class_1277)).
neg(samebib(class_1262,class_1278)).
neg(samebib(class_1262,class_1279)).
neg(samebib(class_1262,class_1280)).
neg(samebib(class_1262,class_1283)).
neg(samebib(class_1262,class_1284)).
neg(samebib(class_1262,class_1285)).
neg(samebib(class_1262,class_1286)).
neg(samebib(class_1262,class_1287)).
neg(samebib(class_1262,class_1288)).
neg(samebib(class_1262,class_1290)).
neg(samebib(class_1262,class_1291)).
neg(samebib(class_1262,class_1292)).
neg(samebib(class_1262,class_1293)).
neg(samebib(class_1262,class_1294)).
neg(samebib(class_1263,class_1135)).
neg(samebib(class_1263,class_1153)).
neg(samebib(class_1263,class_1253)).
neg(samebib(class_1263,class_1254)).
neg(samebib(class_1263,class_1255)).
neg(samebib(class_1263,class_1256)).
neg(samebib(class_1263,class_1257)).
neg(samebib(class_1263,class_1258)).
neg(samebib(class_1263,class_1259)).
neg(samebib(class_1263,class_1260)).
neg(samebib(class_1263,class_1261)).
neg(samebib(class_1263,class_1262)).
samebib(class_1263,class_1263).
samebib(class_1263,class_1264).
samebib(class_1263,class_1265).
samebib(class_1263,class_1266).
samebib(class_1263,class_1267).
samebib(class_1263,class_1268).
samebib(class_1263,class_1269).
samebib(class_1263,class_1270).
samebib(class_1263,class_1271).
samebib(class_1263,class_1272).
samebib(class_1263,class_1273).
samebib(class_1263,class_1274).
samebib(class_1263,class_1275).
samebib(class_1263,class_1276).
samebib(class_1263,class_1277).
samebib(class_1263,class_1278).
samebib(class_1263,class_1279).
samebib(class_1263,class_1280).
samebib(class_1263,class_1281).
samebib(class_1263,class_1282).
samebib(class_1263,class_1283).
neg(samebib(class_1263,class_1284)).
neg(samebib(class_1263,class_1285)).
neg(samebib(class_1263,class_1286)).
neg(samebib(class_1263,class_1287)).
neg(samebib(class_1263,class_1288)).
neg(samebib(class_1263,class_1290)).
neg(samebib(class_1263,class_1291)).
neg(samebib(class_1263,class_1292)).
neg(samebib(class_1263,class_1293)).
neg(samebib(class_1263,class_1294)).
neg(samebib(class_1264,class_1135)).
neg(samebib(class_1264,class_1153)).
neg(samebib(class_1264,class_1253)).
neg(samebib(class_1264,class_1254)).
neg(samebib(class_1264,class_1255)).
neg(samebib(class_1264,class_1256)).
neg(samebib(class_1264,class_1257)).
neg(samebib(class_1264,class_1258)).
neg(samebib(class_1264,class_1259)).
neg(samebib(class_1264,class_1260)).
neg(samebib(class_1264,class_1261)).
neg(samebib(class_1264,class_1262)).
samebib(class_1264,class_1263).
samebib(class_1264,class_1264).
samebib(class_1264,class_1265).
samebib(class_1264,class_1266).
samebib(class_1264,class_1267).
samebib(class_1264,class_1268).
samebib(class_1264,class_1269).
samebib(class_1264,class_1270).
samebib(class_1264,class_1271).
samebib(class_1264,class_1272).
samebib(class_1264,class_1273).
samebib(class_1264,class_1274).
samebib(class_1264,class_1275).
samebib(class_1264,class_1276).
samebib(class_1264,class_1277).
samebib(class_1264,class_1278).
samebib(class_1264,class_1279).
samebib(class_1264,class_1280).
samebib(class_1264,class_1281).
samebib(class_1264,class_1282).
samebib(class_1264,class_1283).
neg(samebib(class_1264,class_1284)).
neg(samebib(class_1264,class_1285)).
neg(samebib(class_1264,class_1286)).
neg(samebib(class_1264,class_1287)).
neg(samebib(class_1264,class_1288)).
neg(samebib(class_1264,class_1290)).
neg(samebib(class_1264,class_1291)).
neg(samebib(class_1264,class_1292)).
neg(samebib(class_1264,class_1293)).
neg(samebib(class_1264,class_1294)).
neg(samebib(class_1265,class_1135)).
neg(samebib(class_1265,class_1153)).
neg(samebib(class_1265,class_1253)).
neg(samebib(class_1265,class_1254)).
neg(samebib(class_1265,class_1255)).
neg(samebib(class_1265,class_1256)).
neg(samebib(class_1265,class_1257)).
neg(samebib(class_1265,class_1258)).
neg(samebib(class_1265,class_1259)).
neg(samebib(class_1265,class_1260)).
neg(samebib(class_1265,class_1261)).
neg(samebib(class_1265,class_1262)).
samebib(class_1265,class_1263).
samebib(class_1265,class_1264).
samebib(class_1265,class_1265).
samebib(class_1265,class_1266).
samebib(class_1265,class_1267).
samebib(class_1265,class_1268).
samebib(class_1265,class_1269).
samebib(class_1265,class_1270).
samebib(class_1265,class_1271).
samebib(class_1265,class_1272).
samebib(class_1265,class_1273).
samebib(class_1265,class_1274).
samebib(class_1265,class_1275).
samebib(class_1265,class_1276).
samebib(class_1265,class_1277).
samebib(class_1265,class_1278).
samebib(class_1265,class_1279).
samebib(class_1265,class_1280).
samebib(class_1265,class_1281).
samebib(class_1265,class_1282).
samebib(class_1265,class_1283).
neg(samebib(class_1265,class_1284)).
neg(samebib(class_1265,class_1285)).
neg(samebib(class_1265,class_1286)).
neg(samebib(class_1265,class_1287)).
neg(samebib(class_1265,class_1288)).
neg(samebib(class_1265,class_1290)).
neg(samebib(class_1265,class_1291)).
neg(samebib(class_1265,class_1292)).
neg(samebib(class_1265,class_1293)).
neg(samebib(class_1265,class_1294)).
neg(samebib(class_1266,class_1135)).
neg(samebib(class_1266,class_1153)).
neg(samebib(class_1266,class_1253)).
neg(samebib(class_1266,class_1254)).
neg(samebib(class_1266,class_1255)).
neg(samebib(class_1266,class_1256)).
neg(samebib(class_1266,class_1257)).
neg(samebib(class_1266,class_1258)).
neg(samebib(class_1266,class_1259)).
neg(samebib(class_1266,class_1260)).
neg(samebib(class_1266,class_1261)).
neg(samebib(class_1266,class_1262)).
samebib(class_1266,class_1263).
samebib(class_1266,class_1264).
samebib(class_1266,class_1265).
samebib(class_1266,class_1266).
samebib(class_1266,class_1267).
samebib(class_1266,class_1268).
samebib(class_1266,class_1269).
samebib(class_1266,class_1270).
samebib(class_1266,class_1271).
samebib(class_1266,class_1272).
samebib(class_1266,class_1273).
samebib(class_1266,class_1274).
samebib(class_1266,class_1275).
samebib(class_1266,class_1276).
samebib(class_1266,class_1277).
samebib(class_1266,class_1278).
samebib(class_1266,class_1279).
samebib(class_1266,class_1280).
samebib(class_1266,class_1281).
samebib(class_1266,class_1282).
samebib(class_1266,class_1283).
neg(samebib(class_1266,class_1284)).
neg(samebib(class_1266,class_1285)).
neg(samebib(class_1266,class_1286)).
neg(samebib(class_1266,class_1287)).
neg(samebib(class_1266,class_1288)).
neg(samebib(class_1266,class_1290)).
neg(samebib(class_1266,class_1291)).
neg(samebib(class_1266,class_1292)).
neg(samebib(class_1266,class_1293)).
neg(samebib(class_1266,class_1294)).
neg(samebib(class_1267,class_1254)).
neg(samebib(class_1267,class_1255)).
neg(samebib(class_1267,class_1256)).
neg(samebib(class_1267,class_1257)).
neg(samebib(class_1267,class_1258)).
neg(samebib(class_1267,class_1259)).
neg(samebib(class_1267,class_1262)).
samebib(class_1267,class_1263).
samebib(class_1267,class_1264).
samebib(class_1267,class_1265).
samebib(class_1267,class_1266).
samebib(class_1267,class_1267).
samebib(class_1267,class_1268).
samebib(class_1267,class_1269).
samebib(class_1267,class_1270).
samebib(class_1267,class_1271).
samebib(class_1267,class_1272).
samebib(class_1267,class_1273).
samebib(class_1267,class_1274).
samebib(class_1267,class_1275).
samebib(class_1267,class_1276).
samebib(class_1267,class_1277).
samebib(class_1267,class_1278).
samebib(class_1267,class_1279).
samebib(class_1267,class_1280).
samebib(class_1267,class_1281).
samebib(class_1267,class_1282).
samebib(class_1267,class_1283).
neg(samebib(class_1267,class_1284)).
neg(samebib(class_1267,class_1285)).
neg(samebib(class_1267,class_1286)).
neg(samebib(class_1267,class_1287)).
neg(samebib(class_1267,class_1288)).
neg(samebib(class_1267,class_1290)).
neg(samebib(class_1267,class_1291)).
neg(samebib(class_1267,class_1292)).
neg(samebib(class_1267,class_1293)).
neg(samebib(class_1267,class_1294)).
neg(samebib(class_1268,class_1254)).
neg(samebib(class_1268,class_1255)).
neg(samebib(class_1268,class_1256)).
neg(samebib(class_1268,class_1257)).
neg(samebib(class_1268,class_1258)).
neg(samebib(class_1268,class_1259)).
neg(samebib(class_1268,class_1262)).
samebib(class_1268,class_1263).
samebib(class_1268,class_1264).
samebib(class_1268,class_1265).
samebib(class_1268,class_1266).
samebib(class_1268,class_1267).
samebib(class_1268,class_1268).
samebib(class_1268,class_1269).
samebib(class_1268,class_1270).
samebib(class_1268,class_1271).
samebib(class_1268,class_1272).
samebib(class_1268,class_1273).
samebib(class_1268,class_1274).
samebib(class_1268,class_1275).
samebib(class_1268,class_1276).
samebib(class_1268,class_1277).
samebib(class_1268,class_1278).
samebib(class_1268,class_1279).
samebib(class_1268,class_1280).
samebib(class_1268,class_1281).
samebib(class_1268,class_1282).
samebib(class_1268,class_1283).
neg(samebib(class_1268,class_1284)).
neg(samebib(class_1268,class_1285)).
neg(samebib(class_1268,class_1286)).
neg(samebib(class_1268,class_1287)).
neg(samebib(class_1268,class_1288)).
neg(samebib(class_1268,class_1290)).
neg(samebib(class_1268,class_1291)).
neg(samebib(class_1268,class_1292)).
neg(samebib(class_1268,class_1293)).
neg(samebib(class_1268,class_1294)).
neg(samebib(class_1269,class_1254)).
neg(samebib(class_1269,class_1255)).
neg(samebib(class_1269,class_1256)).
neg(samebib(class_1269,class_1257)).
neg(samebib(class_1269,class_1258)).
neg(samebib(class_1269,class_1259)).
neg(samebib(class_1269,class_1262)).
samebib(class_1269,class_1263).
samebib(class_1269,class_1264).
samebib(class_1269,class_1265).
samebib(class_1269,class_1266).
samebib(class_1269,class_1267).
samebib(class_1269,class_1268).
samebib(class_1269,class_1269).
samebib(class_1269,class_1270).
samebib(class_1269,class_1271).
samebib(class_1269,class_1272).
samebib(class_1269,class_1273).
samebib(class_1269,class_1274).
samebib(class_1269,class_1275).
samebib(class_1269,class_1276).
samebib(class_1269,class_1277).
samebib(class_1269,class_1278).
samebib(class_1269,class_1279).
samebib(class_1269,class_1280).
samebib(class_1269,class_1281).
samebib(class_1269,class_1282).
samebib(class_1269,class_1283).
neg(samebib(class_1269,class_1284)).
neg(samebib(class_1269,class_1285)).
neg(samebib(class_1269,class_1286)).
neg(samebib(class_1269,class_1287)).
neg(samebib(class_1269,class_1288)).
neg(samebib(class_1269,class_1290)).
neg(samebib(class_1269,class_1291)).
neg(samebib(class_1269,class_1292)).
neg(samebib(class_1269,class_1293)).
neg(samebib(class_1269,class_1294)).
neg(samebib(class_1270,class_1135)).
neg(samebib(class_1270,class_1153)).
neg(samebib(class_1270,class_1253)).
neg(samebib(class_1270,class_1254)).
neg(samebib(class_1270,class_1255)).
neg(samebib(class_1270,class_1256)).
neg(samebib(class_1270,class_1257)).
neg(samebib(class_1270,class_1258)).
neg(samebib(class_1270,class_1259)).
neg(samebib(class_1270,class_1260)).
neg(samebib(class_1270,class_1261)).
neg(samebib(class_1270,class_1262)).
samebib(class_1270,class_1263).
samebib(class_1270,class_1264).
samebib(class_1270,class_1265).
samebib(class_1270,class_1266).
samebib(class_1270,class_1267).
samebib(class_1270,class_1268).
samebib(class_1270,class_1269).
samebib(class_1270,class_1270).
samebib(class_1270,class_1271).
samebib(class_1270,class_1272).
samebib(class_1270,class_1273).
samebib(class_1270,class_1274).
samebib(class_1270,class_1275).
samebib(class_1270,class_1276).
samebib(class_1270,class_1277).
samebib(class_1270,class_1278).
samebib(class_1270,class_1279).
samebib(class_1270,class_1280).
samebib(class_1270,class_1281).
samebib(class_1270,class_1282).
samebib(class_1270,class_1283).
neg(samebib(class_1270,class_1284)).
neg(samebib(class_1270,class_1285)).
neg(samebib(class_1270,class_1286)).
neg(samebib(class_1270,class_1287)).
neg(samebib(class_1270,class_1288)).
neg(samebib(class_1270,class_1290)).
neg(samebib(class_1270,class_1291)).
neg(samebib(class_1270,class_1292)).
neg(samebib(class_1270,class_1293)).
neg(samebib(class_1270,class_1294)).
neg(samebib(class_1271,class_1135)).
neg(samebib(class_1271,class_1153)).
neg(samebib(class_1271,class_1254)).
neg(samebib(class_1271,class_1255)).
neg(samebib(class_1271,class_1256)).
neg(samebib(class_1271,class_1257)).
neg(samebib(class_1271,class_1258)).
neg(samebib(class_1271,class_1259)).
neg(samebib(class_1271,class_1262)).
samebib(class_1271,class_1263).
samebib(class_1271,class_1264).
samebib(class_1271,class_1265).
samebib(class_1271,class_1266).
samebib(class_1271,class_1267).
samebib(class_1271,class_1268).
samebib(class_1271,class_1269).
samebib(class_1271,class_1270).
samebib(class_1271,class_1271).
samebib(class_1271,class_1272).
samebib(class_1271,class_1273).
samebib(class_1271,class_1274).
samebib(class_1271,class_1275).
samebib(class_1271,class_1276).
samebib(class_1271,class_1277).
samebib(class_1271,class_1278).
samebib(class_1271,class_1279).
samebib(class_1271,class_1280).
samebib(class_1271,class_1281).
samebib(class_1271,class_1282).
samebib(class_1271,class_1283).
neg(samebib(class_1271,class_1284)).
neg(samebib(class_1271,class_1285)).
neg(samebib(class_1271,class_1286)).
neg(samebib(class_1271,class_1287)).
neg(samebib(class_1271,class_1288)).
neg(samebib(class_1271,class_1290)).
neg(samebib(class_1271,class_1291)).
neg(samebib(class_1271,class_1292)).
neg(samebib(class_1271,class_1293)).
neg(samebib(class_1271,class_1294)).
neg(samebib(class_1272,class_1254)).
neg(samebib(class_1272,class_1255)).
neg(samebib(class_1272,class_1256)).
neg(samebib(class_1272,class_1257)).
neg(samebib(class_1272,class_1258)).
neg(samebib(class_1272,class_1259)).
neg(samebib(class_1272,class_1262)).
samebib(class_1272,class_1263).
samebib(class_1272,class_1264).
samebib(class_1272,class_1265).
samebib(class_1272,class_1266).
samebib(class_1272,class_1267).
samebib(class_1272,class_1268).
samebib(class_1272,class_1269).
samebib(class_1272,class_1270).
samebib(class_1272,class_1271).
samebib(class_1272,class_1272).
samebib(class_1272,class_1273).
samebib(class_1272,class_1274).
samebib(class_1272,class_1275).
samebib(class_1272,class_1276).
samebib(class_1272,class_1277).
samebib(class_1272,class_1278).
samebib(class_1272,class_1279).
samebib(class_1272,class_1280).
samebib(class_1272,class_1281).
samebib(class_1272,class_1282).
samebib(class_1272,class_1283).
neg(samebib(class_1272,class_1284)).
neg(samebib(class_1272,class_1285)).
neg(samebib(class_1272,class_1286)).
neg(samebib(class_1272,class_1287)).
neg(samebib(class_1272,class_1288)).
neg(samebib(class_1272,class_1290)).
neg(samebib(class_1272,class_1291)).
neg(samebib(class_1272,class_1292)).
neg(samebib(class_1272,class_1293)).
neg(samebib(class_1272,class_1294)).
neg(samebib(class_1273,class_1135)).
neg(samebib(class_1273,class_1153)).
neg(samebib(class_1273,class_1254)).
neg(samebib(class_1273,class_1255)).
neg(samebib(class_1273,class_1256)).
neg(samebib(class_1273,class_1257)).
neg(samebib(class_1273,class_1258)).
neg(samebib(class_1273,class_1259)).
neg(samebib(class_1273,class_1262)).
samebib(class_1273,class_1263).
samebib(class_1273,class_1264).
samebib(class_1273,class_1265).
samebib(class_1273,class_1266).
samebib(class_1273,class_1267).
samebib(class_1273,class_1268).
samebib(class_1273,class_1269).
samebib(class_1273,class_1270).
samebib(class_1273,class_1271).
samebib(class_1273,class_1272).
samebib(class_1273,class_1273).
samebib(class_1273,class_1274).
samebib(class_1273,class_1275).
samebib(class_1273,class_1276).
samebib(class_1273,class_1277).
samebib(class_1273,class_1278).
samebib(class_1273,class_1279).
samebib(class_1273,class_1280).
samebib(class_1273,class_1281).
samebib(class_1273,class_1282).
samebib(class_1273,class_1283).
neg(samebib(class_1273,class_1284)).
neg(samebib(class_1273,class_1285)).
neg(samebib(class_1273,class_1286)).
neg(samebib(class_1273,class_1287)).
neg(samebib(class_1273,class_1288)).
neg(samebib(class_1273,class_1290)).
neg(samebib(class_1273,class_1291)).
neg(samebib(class_1273,class_1292)).
neg(samebib(class_1273,class_1293)).
neg(samebib(class_1273,class_1294)).
samebib(class_1274,class_1263).
samebib(class_1274,class_1264).
samebib(class_1274,class_1265).
samebib(class_1274,class_1266).
samebib(class_1274,class_1267).
samebib(class_1274,class_1268).
samebib(class_1274,class_1269).
samebib(class_1274,class_1270).
samebib(class_1274,class_1271).
samebib(class_1274,class_1272).
samebib(class_1274,class_1273).
samebib(class_1274,class_1274).
samebib(class_1274,class_1275).
samebib(class_1274,class_1276).
samebib(class_1274,class_1277).
samebib(class_1274,class_1278).
samebib(class_1274,class_1279).
samebib(class_1274,class_1280).
samebib(class_1274,class_1281).
samebib(class_1274,class_1282).
samebib(class_1274,class_1283).
neg(samebib(class_1274,class_1284)).
neg(samebib(class_1274,class_1285)).
neg(samebib(class_1274,class_1286)).
neg(samebib(class_1274,class_1287)).
neg(samebib(class_1274,class_1288)).
neg(samebib(class_1275,class_1254)).
neg(samebib(class_1275,class_1255)).
neg(samebib(class_1275,class_1256)).
neg(samebib(class_1275,class_1257)).
neg(samebib(class_1275,class_1258)).
neg(samebib(class_1275,class_1259)).
neg(samebib(class_1275,class_1262)).
samebib(class_1275,class_1263).
samebib(class_1275,class_1264).
samebib(class_1275,class_1265).
samebib(class_1275,class_1266).
samebib(class_1275,class_1267).
samebib(class_1275,class_1268).
samebib(class_1275,class_1269).
samebib(class_1275,class_1270).
samebib(class_1275,class_1271).
samebib(class_1275,class_1272).
samebib(class_1275,class_1273).
samebib(class_1275,class_1274).
samebib(class_1275,class_1275).
samebib(class_1275,class_1276).
samebib(class_1275,class_1277).
samebib(class_1275,class_1278).
samebib(class_1275,class_1279).
samebib(class_1275,class_1280).
samebib(class_1275,class_1281).
samebib(class_1275,class_1282).
samebib(class_1275,class_1283).
neg(samebib(class_1275,class_1284)).
neg(samebib(class_1275,class_1285)).
neg(samebib(class_1275,class_1286)).
neg(samebib(class_1275,class_1287)).
neg(samebib(class_1275,class_1288)).
neg(samebib(class_1275,class_1290)).
neg(samebib(class_1275,class_1291)).
neg(samebib(class_1275,class_1292)).
neg(samebib(class_1275,class_1293)).
neg(samebib(class_1275,class_1294)).
neg(samebib(class_1276,class_1135)).
neg(samebib(class_1276,class_1153)).
neg(samebib(class_1276,class_1253)).
neg(samebib(class_1276,class_1254)).
neg(samebib(class_1276,class_1255)).
neg(samebib(class_1276,class_1256)).
neg(samebib(class_1276,class_1257)).
neg(samebib(class_1276,class_1258)).
neg(samebib(class_1276,class_1259)).
neg(samebib(class_1276,class_1260)).
neg(samebib(class_1276,class_1261)).
neg(samebib(class_1276,class_1262)).
samebib(class_1276,class_1263).
samebib(class_1276,class_1264).
samebib(class_1276,class_1265).
samebib(class_1276,class_1266).
samebib(class_1276,class_1267).
samebib(class_1276,class_1268).
samebib(class_1276,class_1269).
samebib(class_1276,class_1270).
samebib(class_1276,class_1271).
samebib(class_1276,class_1272).
samebib(class_1276,class_1273).
samebib(class_1276,class_1274).
samebib(class_1276,class_1275).
samebib(class_1276,class_1276).
samebib(class_1276,class_1277).
samebib(class_1276,class_1278).
samebib(class_1276,class_1279).
samebib(class_1276,class_1280).
samebib(class_1276,class_1281).
samebib(class_1276,class_1282).
samebib(class_1276,class_1283).
neg(samebib(class_1276,class_1284)).
neg(samebib(class_1276,class_1285)).
neg(samebib(class_1276,class_1286)).
neg(samebib(class_1276,class_1287)).
neg(samebib(class_1276,class_1288)).
neg(samebib(class_1276,class_1290)).
neg(samebib(class_1276,class_1291)).
neg(samebib(class_1276,class_1292)).
neg(samebib(class_1276,class_1293)).
neg(samebib(class_1276,class_1294)).
neg(samebib(class_1277,class_1254)).
neg(samebib(class_1277,class_1255)).
neg(samebib(class_1277,class_1256)).
neg(samebib(class_1277,class_1257)).
neg(samebib(class_1277,class_1258)).
neg(samebib(class_1277,class_1259)).
neg(samebib(class_1277,class_1262)).
samebib(class_1277,class_1263).
samebib(class_1277,class_1264).
samebib(class_1277,class_1265).
samebib(class_1277,class_1266).
samebib(class_1277,class_1267).
samebib(class_1277,class_1268).
samebib(class_1277,class_1269).
samebib(class_1277,class_1270).
samebib(class_1277,class_1271).
samebib(class_1277,class_1272).
samebib(class_1277,class_1273).
samebib(class_1277,class_1274).
samebib(class_1277,class_1275).
samebib(class_1277,class_1276).
samebib(class_1277,class_1277).
samebib(class_1277,class_1278).
samebib(class_1277,class_1279).
samebib(class_1277,class_1280).
samebib(class_1277,class_1281).
samebib(class_1277,class_1282).
samebib(class_1277,class_1283).
neg(samebib(class_1277,class_1284)).
neg(samebib(class_1277,class_1285)).
neg(samebib(class_1277,class_1286)).
neg(samebib(class_1277,class_1287)).
neg(samebib(class_1277,class_1288)).
neg(samebib(class_1277,class_1290)).
neg(samebib(class_1277,class_1291)).
neg(samebib(class_1277,class_1292)).
neg(samebib(class_1277,class_1293)).
neg(samebib(class_1277,class_1294)).
neg(samebib(class_1278,class_1254)).
neg(samebib(class_1278,class_1255)).
neg(samebib(class_1278,class_1256)).
neg(samebib(class_1278,class_1257)).
neg(samebib(class_1278,class_1258)).
neg(samebib(class_1278,class_1259)).
neg(samebib(class_1278,class_1262)).
samebib(class_1278,class_1263).
samebib(class_1278,class_1264).
samebib(class_1278,class_1265).
samebib(class_1278,class_1266).
samebib(class_1278,class_1267).
samebib(class_1278,class_1268).
samebib(class_1278,class_1269).
samebib(class_1278,class_1270).
samebib(class_1278,class_1271).
samebib(class_1278,class_1272).
samebib(class_1278,class_1273).
samebib(class_1278,class_1274).
samebib(class_1278,class_1275).
samebib(class_1278,class_1276).
samebib(class_1278,class_1277).
samebib(class_1278,class_1278).
samebib(class_1278,class_1279).
samebib(class_1278,class_1280).
samebib(class_1278,class_1281).
samebib(class_1278,class_1282).
samebib(class_1278,class_1283).
neg(samebib(class_1278,class_1284)).
neg(samebib(class_1278,class_1285)).
neg(samebib(class_1278,class_1286)).
neg(samebib(class_1278,class_1287)).
neg(samebib(class_1278,class_1288)).
neg(samebib(class_1278,class_1290)).
neg(samebib(class_1278,class_1291)).
neg(samebib(class_1278,class_1292)).
neg(samebib(class_1278,class_1293)).
neg(samebib(class_1278,class_1294)).
neg(samebib(class_1279,class_1254)).
neg(samebib(class_1279,class_1255)).
neg(samebib(class_1279,class_1256)).
neg(samebib(class_1279,class_1257)).
neg(samebib(class_1279,class_1258)).
neg(samebib(class_1279,class_1259)).
neg(samebib(class_1279,class_1262)).
samebib(class_1279,class_1263).
samebib(class_1279,class_1264).
samebib(class_1279,class_1265).
samebib(class_1279,class_1266).
samebib(class_1279,class_1267).
samebib(class_1279,class_1268).
samebib(class_1279,class_1269).
samebib(class_1279,class_1270).
samebib(class_1279,class_1271).
samebib(class_1279,class_1272).
samebib(class_1279,class_1273).
samebib(class_1279,class_1274).
samebib(class_1279,class_1275).
samebib(class_1279,class_1276).
samebib(class_1279,class_1277).
samebib(class_1279,class_1278).
samebib(class_1279,class_1279).
samebib(class_1279,class_1280).
samebib(class_1279,class_1281).
samebib(class_1279,class_1282).
samebib(class_1279,class_1283).
neg(samebib(class_1279,class_1284)).
neg(samebib(class_1279,class_1285)).
neg(samebib(class_1279,class_1286)).
neg(samebib(class_1279,class_1287)).
neg(samebib(class_1279,class_1288)).
neg(samebib(class_1279,class_1290)).
neg(samebib(class_1279,class_1291)).
neg(samebib(class_1279,class_1292)).
neg(samebib(class_1279,class_1293)).
neg(samebib(class_1279,class_1294)).
neg(samebib(class_1280,class_1254)).
neg(samebib(class_1280,class_1255)).
neg(samebib(class_1280,class_1256)).
neg(samebib(class_1280,class_1257)).
neg(samebib(class_1280,class_1258)).
neg(samebib(class_1280,class_1259)).
neg(samebib(class_1280,class_1262)).
samebib(class_1280,class_1263).
samebib(class_1280,class_1264).
samebib(class_1280,class_1265).
samebib(class_1280,class_1266).
samebib(class_1280,class_1267).
samebib(class_1280,class_1268).
samebib(class_1280,class_1269).
samebib(class_1280,class_1270).
samebib(class_1280,class_1271).
samebib(class_1280,class_1272).
samebib(class_1280,class_1273).
samebib(class_1280,class_1274).
samebib(class_1280,class_1275).
samebib(class_1280,class_1276).
samebib(class_1280,class_1277).
samebib(class_1280,class_1278).
samebib(class_1280,class_1279).
samebib(class_1280,class_1280).
samebib(class_1280,class_1281).
samebib(class_1280,class_1282).
samebib(class_1280,class_1283).
neg(samebib(class_1280,class_1284)).
neg(samebib(class_1280,class_1285)).
neg(samebib(class_1280,class_1286)).
neg(samebib(class_1280,class_1287)).
neg(samebib(class_1280,class_1288)).
neg(samebib(class_1280,class_1290)).
neg(samebib(class_1280,class_1291)).
neg(samebib(class_1280,class_1292)).
neg(samebib(class_1280,class_1293)).
neg(samebib(class_1280,class_1294)).
samebib(class_1281,class_1263).
samebib(class_1281,class_1264).
samebib(class_1281,class_1265).
samebib(class_1281,class_1266).
samebib(class_1281,class_1267).
samebib(class_1281,class_1268).
samebib(class_1281,class_1269).
samebib(class_1281,class_1270).
samebib(class_1281,class_1271).
samebib(class_1281,class_1272).
samebib(class_1281,class_1273).
samebib(class_1281,class_1274).
samebib(class_1281,class_1275).
samebib(class_1281,class_1276).
samebib(class_1281,class_1277).
samebib(class_1281,class_1278).
samebib(class_1281,class_1279).
samebib(class_1281,class_1280).
samebib(class_1281,class_1281).
samebib(class_1281,class_1282).
samebib(class_1281,class_1283).
neg(samebib(class_1281,class_1284)).
neg(samebib(class_1281,class_1285)).
neg(samebib(class_1281,class_1286)).
neg(samebib(class_1281,class_1287)).
neg(samebib(class_1281,class_1288)).
neg(samebib(class_1281,class_1290)).
samebib(class_1282,class_1263).
samebib(class_1282,class_1264).
samebib(class_1282,class_1265).
samebib(class_1282,class_1266).
samebib(class_1282,class_1267).
samebib(class_1282,class_1268).
samebib(class_1282,class_1269).
samebib(class_1282,class_1270).
samebib(class_1282,class_1271).
samebib(class_1282,class_1272).
samebib(class_1282,class_1273).
samebib(class_1282,class_1274).
samebib(class_1282,class_1275).
samebib(class_1282,class_1276).
samebib(class_1282,class_1277).
samebib(class_1282,class_1278).
samebib(class_1282,class_1279).
samebib(class_1282,class_1280).
samebib(class_1282,class_1281).
samebib(class_1282,class_1282).
samebib(class_1282,class_1283).
neg(samebib(class_1282,class_1284)).
neg(samebib(class_1282,class_1285)).
neg(samebib(class_1282,class_1286)).
neg(samebib(class_1282,class_1287)).
neg(samebib(class_1282,class_1288)).
neg(samebib(class_1282,class_1290)).
neg(samebib(class_1282,class_1291)).
neg(samebib(class_1282,class_1292)).
neg(samebib(class_1282,class_1293)).
neg(samebib(class_1282,class_1294)).
neg(samebib(class_1283,class_1262)).
samebib(class_1283,class_1263).
samebib(class_1283,class_1264).
samebib(class_1283,class_1265).
samebib(class_1283,class_1266).
samebib(class_1283,class_1267).
samebib(class_1283,class_1268).
samebib(class_1283,class_1269).
samebib(class_1283,class_1270).
samebib(class_1283,class_1271).
samebib(class_1283,class_1272).
samebib(class_1283,class_1273).
samebib(class_1283,class_1274).
samebib(class_1283,class_1275).
samebib(class_1283,class_1276).
samebib(class_1283,class_1277).
samebib(class_1283,class_1278).
samebib(class_1283,class_1279).
samebib(class_1283,class_1280).
samebib(class_1283,class_1281).
samebib(class_1283,class_1282).
samebib(class_1283,class_1283).
neg(samebib(class_1283,class_1284)).
neg(samebib(class_1283,class_1285)).
neg(samebib(class_1283,class_1286)).
neg(samebib(class_1283,class_1287)).
neg(samebib(class_1283,class_1288)).
neg(samebib(class_1283,class_1290)).
neg(samebib(class_1283,class_1291)).
neg(samebib(class_1283,class_1292)).
neg(samebib(class_1283,class_1293)).
neg(samebib(class_1283,class_1294)).
neg(samebib(class_1284,class_1253)).
neg(samebib(class_1284,class_1254)).
neg(samebib(class_1284,class_1255)).
neg(samebib(class_1284,class_1256)).
neg(samebib(class_1284,class_1257)).
neg(samebib(class_1284,class_1258)).
neg(samebib(class_1284,class_1259)).
neg(samebib(class_1284,class_1260)).
neg(samebib(class_1284,class_1261)).
neg(samebib(class_1284,class_1262)).
neg(samebib(class_1284,class_1263)).
neg(samebib(class_1284,class_1264)).
neg(samebib(class_1284,class_1265)).
neg(samebib(class_1284,class_1266)).
neg(samebib(class_1284,class_1267)).
neg(samebib(class_1284,class_1268)).
neg(samebib(class_1284,class_1269)).
neg(samebib(class_1284,class_1270)).
neg(samebib(class_1284,class_1271)).
neg(samebib(class_1284,class_1272)).
neg(samebib(class_1284,class_1273)).
neg(samebib(class_1284,class_1274)).
neg(samebib(class_1284,class_1275)).
neg(samebib(class_1284,class_1276)).
neg(samebib(class_1284,class_1277)).
neg(samebib(class_1284,class_1278)).
neg(samebib(class_1284,class_1279)).
neg(samebib(class_1284,class_1280)).
neg(samebib(class_1284,class_1281)).
neg(samebib(class_1284,class_1282)).
neg(samebib(class_1284,class_1283)).
samebib(class_1284,class_1284).
neg(samebib(class_1284,class_1285)).
neg(samebib(class_1284,class_1286)).
neg(samebib(class_1284,class_1287)).
neg(samebib(class_1284,class_1288)).
neg(samebib(class_1284,class_1290)).
neg(samebib(class_1284,class_1291)).
neg(samebib(class_1284,class_1292)).
neg(samebib(class_1284,class_1293)).
neg(samebib(class_1284,class_1294)).
neg(samebib(class_1285,class_1254)).
neg(samebib(class_1285,class_1255)).
neg(samebib(class_1285,class_1256)).
neg(samebib(class_1285,class_1257)).
neg(samebib(class_1285,class_1258)).
neg(samebib(class_1285,class_1259)).
neg(samebib(class_1285,class_1262)).
neg(samebib(class_1285,class_1263)).
neg(samebib(class_1285,class_1264)).
neg(samebib(class_1285,class_1265)).
neg(samebib(class_1285,class_1266)).
neg(samebib(class_1285,class_1267)).
neg(samebib(class_1285,class_1268)).
neg(samebib(class_1285,class_1269)).
neg(samebib(class_1285,class_1270)).
neg(samebib(class_1285,class_1271)).
neg(samebib(class_1285,class_1272)).
neg(samebib(class_1285,class_1273)).
neg(samebib(class_1285,class_1274)).
neg(samebib(class_1285,class_1275)).
neg(samebib(class_1285,class_1276)).
neg(samebib(class_1285,class_1277)).
neg(samebib(class_1285,class_1278)).
neg(samebib(class_1285,class_1279)).
neg(samebib(class_1285,class_1280)).
neg(samebib(class_1285,class_1281)).
neg(samebib(class_1285,class_1282)).
neg(samebib(class_1285,class_1283)).
neg(samebib(class_1285,class_1284)).
samebib(class_1285,class_1285).
samebib(class_1285,class_1286).
samebib(class_1285,class_1287).
samebib(class_1285,class_1288).
neg(samebib(class_1285,class_1289)).
neg(samebib(class_1285,class_1290)).
neg(samebib(class_1285,class_1291)).
neg(samebib(class_1285,class_1292)).
neg(samebib(class_1285,class_1293)).
neg(samebib(class_1285,class_1294)).
neg(samebib(class_1285,class_1295)).
neg(samebib(class_1286,class_1254)).
neg(samebib(class_1286,class_1255)).
neg(samebib(class_1286,class_1256)).
neg(samebib(class_1286,class_1257)).
neg(samebib(class_1286,class_1258)).
neg(samebib(class_1286,class_1259)).
neg(samebib(class_1286,class_1262)).
neg(samebib(class_1286,class_1263)).
neg(samebib(class_1286,class_1264)).
neg(samebib(class_1286,class_1265)).
neg(samebib(class_1286,class_1266)).
neg(samebib(class_1286,class_1267)).
neg(samebib(class_1286,class_1268)).
neg(samebib(class_1286,class_1269)).
neg(samebib(class_1286,class_1270)).
neg(samebib(class_1286,class_1271)).
neg(samebib(class_1286,class_1272)).
neg(samebib(class_1286,class_1273)).
neg(samebib(class_1286,class_1274)).
neg(samebib(class_1286,class_1275)).
neg(samebib(class_1286,class_1276)).
neg(samebib(class_1286,class_1277)).
neg(samebib(class_1286,class_1278)).
neg(samebib(class_1286,class_1279)).
neg(samebib(class_1286,class_1280)).
neg(samebib(class_1286,class_1281)).
neg(samebib(class_1286,class_1282)).
neg(samebib(class_1286,class_1283)).
neg(samebib(class_1286,class_1284)).
samebib(class_1286,class_1285).
samebib(class_1286,class_1286).
samebib(class_1286,class_1287).
samebib(class_1286,class_1288).
neg(samebib(class_1286,class_1289)).
neg(samebib(class_1286,class_1290)).
neg(samebib(class_1286,class_1291)).
neg(samebib(class_1286,class_1292)).
neg(samebib(class_1286,class_1293)).
neg(samebib(class_1286,class_1294)).
neg(samebib(class_1286,class_1295)).
neg(samebib(class_1287,class_1254)).
neg(samebib(class_1287,class_1255)).
neg(samebib(class_1287,class_1256)).
neg(samebib(class_1287,class_1257)).
neg(samebib(class_1287,class_1258)).
neg(samebib(class_1287,class_1259)).
neg(samebib(class_1287,class_1262)).
neg(samebib(class_1287,class_1263)).
neg(samebib(class_1287,class_1264)).
neg(samebib(class_1287,class_1265)).
neg(samebib(class_1287,class_1266)).
neg(samebib(class_1287,class_1267)).
neg(samebib(class_1287,class_1268)).
neg(samebib(class_1287,class_1269)).
neg(samebib(class_1287,class_1270)).
neg(samebib(class_1287,class_1271)).
neg(samebib(class_1287,class_1272)).
neg(samebib(class_1287,class_1273)).
neg(samebib(class_1287,class_1274)).
neg(samebib(class_1287,class_1275)).
neg(samebib(class_1287,class_1276)).
neg(samebib(class_1287,class_1277)).
neg(samebib(class_1287,class_1278)).
neg(samebib(class_1287,class_1279)).
neg(samebib(class_1287,class_1280)).
neg(samebib(class_1287,class_1281)).
neg(samebib(class_1287,class_1282)).
neg(samebib(class_1287,class_1283)).
neg(samebib(class_1287,class_1284)).
samebib(class_1287,class_1285).
samebib(class_1287,class_1286).
samebib(class_1287,class_1287).
samebib(class_1287,class_1288).
neg(samebib(class_1287,class_1289)).
neg(samebib(class_1287,class_1290)).
neg(samebib(class_1287,class_1291)).
neg(samebib(class_1287,class_1292)).
neg(samebib(class_1287,class_1293)).
neg(samebib(class_1287,class_1294)).
neg(samebib(class_1287,class_1295)).
neg(samebib(class_1288,class_1254)).
neg(samebib(class_1288,class_1255)).
neg(samebib(class_1288,class_1256)).
neg(samebib(class_1288,class_1257)).
neg(samebib(class_1288,class_1258)).
neg(samebib(class_1288,class_1259)).
neg(samebib(class_1288,class_1262)).
neg(samebib(class_1288,class_1263)).
neg(samebib(class_1288,class_1264)).
neg(samebib(class_1288,class_1265)).
neg(samebib(class_1288,class_1266)).
neg(samebib(class_1288,class_1267)).
neg(samebib(class_1288,class_1268)).
neg(samebib(class_1288,class_1269)).
neg(samebib(class_1288,class_1270)).
neg(samebib(class_1288,class_1271)).
neg(samebib(class_1288,class_1272)).
neg(samebib(class_1288,class_1273)).
neg(samebib(class_1288,class_1274)).
neg(samebib(class_1288,class_1275)).
neg(samebib(class_1288,class_1276)).
neg(samebib(class_1288,class_1277)).
neg(samebib(class_1288,class_1278)).
neg(samebib(class_1288,class_1279)).
neg(samebib(class_1288,class_1280)).
neg(samebib(class_1288,class_1281)).
neg(samebib(class_1288,class_1282)).
neg(samebib(class_1288,class_1283)).
neg(samebib(class_1288,class_1284)).
samebib(class_1288,class_1285).
samebib(class_1288,class_1286).
samebib(class_1288,class_1287).
samebib(class_1288,class_1288).
neg(samebib(class_1288,class_1289)).
neg(samebib(class_1288,class_1290)).
neg(samebib(class_1288,class_1291)).
neg(samebib(class_1288,class_1292)).
neg(samebib(class_1288,class_1293)).
neg(samebib(class_1288,class_1294)).
neg(samebib(class_1288,class_1295)).
neg(samebib(class_1289,class_1285)).
neg(samebib(class_1289,class_1286)).
neg(samebib(class_1289,class_1287)).
neg(samebib(class_1289,class_1288)).
samebib(class_1289,class_1289).
neg(samebib(class_1289,class_1291)).
neg(samebib(class_1289,class_1292)).
neg(samebib(class_1289,class_1293)).
neg(samebib(class_1289,class_1294)).
neg(samebib(class_1289,class_1295)).
neg(samebib(class_1290,class_1254)).
neg(samebib(class_1290,class_1255)).
neg(samebib(class_1290,class_1256)).
neg(samebib(class_1290,class_1257)).
neg(samebib(class_1290,class_1258)).
neg(samebib(class_1290,class_1259)).
neg(samebib(class_1290,class_1262)).
neg(samebib(class_1290,class_1263)).
neg(samebib(class_1290,class_1264)).
neg(samebib(class_1290,class_1265)).
neg(samebib(class_1290,class_1266)).
neg(samebib(class_1290,class_1267)).
neg(samebib(class_1290,class_1268)).
neg(samebib(class_1290,class_1269)).
neg(samebib(class_1290,class_1270)).
neg(samebib(class_1290,class_1271)).
neg(samebib(class_1290,class_1272)).
neg(samebib(class_1290,class_1273)).
neg(samebib(class_1290,class_1275)).
neg(samebib(class_1290,class_1276)).
neg(samebib(class_1290,class_1277)).
neg(samebib(class_1290,class_1278)).
neg(samebib(class_1290,class_1279)).
neg(samebib(class_1290,class_1280)).
neg(samebib(class_1290,class_1281)).
neg(samebib(class_1290,class_1282)).
neg(samebib(class_1290,class_1283)).
neg(samebib(class_1290,class_1284)).
neg(samebib(class_1290,class_1285)).
neg(samebib(class_1290,class_1286)).
neg(samebib(class_1290,class_1287)).
neg(samebib(class_1290,class_1288)).
samebib(class_1290,class_1290).
samebib(class_1290,class_1291).
samebib(class_1290,class_1292).
samebib(class_1290,class_1293).
samebib(class_1290,class_1294).
neg(samebib(class_1291,class_1249)).
neg(samebib(class_1291,class_1250)).
neg(samebib(class_1291,class_1251)).
neg(samebib(class_1291,class_1252)).
neg(samebib(class_1291,class_1254)).
neg(samebib(class_1291,class_1255)).
neg(samebib(class_1291,class_1256)).
neg(samebib(class_1291,class_1257)).
neg(samebib(class_1291,class_1258)).
neg(samebib(class_1291,class_1259)).
neg(samebib(class_1291,class_1262)).
neg(samebib(class_1291,class_1263)).
neg(samebib(class_1291,class_1264)).
neg(samebib(class_1291,class_1265)).
neg(samebib(class_1291,class_1266)).
neg(samebib(class_1291,class_1267)).
neg(samebib(class_1291,class_1268)).
neg(samebib(class_1291,class_1269)).
neg(samebib(class_1291,class_1270)).
neg(samebib(class_1291,class_1271)).
neg(samebib(class_1291,class_1272)).
neg(samebib(class_1291,class_1273)).
neg(samebib(class_1291,class_1275)).
neg(samebib(class_1291,class_1276)).
neg(samebib(class_1291,class_1277)).
neg(samebib(class_1291,class_1278)).
neg(samebib(class_1291,class_1279)).
neg(samebib(class_1291,class_1280)).
neg(samebib(class_1291,class_1282)).
neg(samebib(class_1291,class_1283)).
neg(samebib(class_1291,class_1284)).
neg(samebib(class_1291,class_1285)).
neg(samebib(class_1291,class_1286)).
neg(samebib(class_1291,class_1287)).
neg(samebib(class_1291,class_1288)).
neg(samebib(class_1291,class_1289)).
samebib(class_1291,class_1290).
samebib(class_1291,class_1291).
samebib(class_1291,class_1292).
samebib(class_1291,class_1293).
samebib(class_1291,class_1294).
neg(samebib(class_1291,class_1295)).
neg(samebib(class_1292,class_1249)).
neg(samebib(class_1292,class_1250)).
neg(samebib(class_1292,class_1251)).
neg(samebib(class_1292,class_1252)).
neg(samebib(class_1292,class_1254)).
neg(samebib(class_1292,class_1255)).
neg(samebib(class_1292,class_1256)).
neg(samebib(class_1292,class_1257)).
neg(samebib(class_1292,class_1258)).
neg(samebib(class_1292,class_1259)).
neg(samebib(class_1292,class_1262)).
neg(samebib(class_1292,class_1263)).
neg(samebib(class_1292,class_1264)).
neg(samebib(class_1292,class_1265)).
neg(samebib(class_1292,class_1266)).
neg(samebib(class_1292,class_1267)).
neg(samebib(class_1292,class_1268)).
neg(samebib(class_1292,class_1269)).
neg(samebib(class_1292,class_1270)).
neg(samebib(class_1292,class_1271)).
neg(samebib(class_1292,class_1272)).
neg(samebib(class_1292,class_1273)).
neg(samebib(class_1292,class_1275)).
neg(samebib(class_1292,class_1276)).
neg(samebib(class_1292,class_1277)).
neg(samebib(class_1292,class_1278)).
neg(samebib(class_1292,class_1279)).
neg(samebib(class_1292,class_1280)).
neg(samebib(class_1292,class_1282)).
neg(samebib(class_1292,class_1283)).
neg(samebib(class_1292,class_1284)).
neg(samebib(class_1292,class_1285)).
neg(samebib(class_1292,class_1286)).
neg(samebib(class_1292,class_1287)).
neg(samebib(class_1292,class_1288)).
neg(samebib(class_1292,class_1289)).
samebib(class_1292,class_1290).
samebib(class_1292,class_1291).
samebib(class_1292,class_1292).
samebib(class_1292,class_1293).
samebib(class_1292,class_1294).
neg(samebib(class_1292,class_1295)).
neg(samebib(class_1293,class_1249)).
neg(samebib(class_1293,class_1250)).
neg(samebib(class_1293,class_1251)).
neg(samebib(class_1293,class_1252)).
neg(samebib(class_1293,class_1254)).
neg(samebib(class_1293,class_1255)).
neg(samebib(class_1293,class_1256)).
neg(samebib(class_1293,class_1257)).
neg(samebib(class_1293,class_1258)).
neg(samebib(class_1293,class_1259)).
neg(samebib(class_1293,class_1262)).
neg(samebib(class_1293,class_1263)).
neg(samebib(class_1293,class_1264)).
neg(samebib(class_1293,class_1265)).
neg(samebib(class_1293,class_1266)).
neg(samebib(class_1293,class_1267)).
neg(samebib(class_1293,class_1268)).
neg(samebib(class_1293,class_1269)).
neg(samebib(class_1293,class_1270)).
neg(samebib(class_1293,class_1271)).
neg(samebib(class_1293,class_1272)).
neg(samebib(class_1293,class_1273)).
neg(samebib(class_1293,class_1275)).
neg(samebib(class_1293,class_1276)).
neg(samebib(class_1293,class_1277)).
neg(samebib(class_1293,class_1278)).
neg(samebib(class_1293,class_1279)).
neg(samebib(class_1293,class_1280)).
neg(samebib(class_1293,class_1282)).
neg(samebib(class_1293,class_1283)).
neg(samebib(class_1293,class_1284)).
neg(samebib(class_1293,class_1285)).
neg(samebib(class_1293,class_1286)).
neg(samebib(class_1293,class_1287)).
neg(samebib(class_1293,class_1288)).
neg(samebib(class_1293,class_1289)).
samebib(class_1293,class_1290).
samebib(class_1293,class_1291).
samebib(class_1293,class_1292).
samebib(class_1293,class_1293).
samebib(class_1293,class_1294).
neg(samebib(class_1293,class_1295)).
neg(samebib(class_1294,class_1249)).
neg(samebib(class_1294,class_1250)).
neg(samebib(class_1294,class_1251)).
neg(samebib(class_1294,class_1252)).
neg(samebib(class_1294,class_1254)).
neg(samebib(class_1294,class_1255)).
neg(samebib(class_1294,class_1256)).
neg(samebib(class_1294,class_1257)).
neg(samebib(class_1294,class_1258)).
neg(samebib(class_1294,class_1259)).
neg(samebib(class_1294,class_1262)).
neg(samebib(class_1294,class_1263)).
neg(samebib(class_1294,class_1264)).
neg(samebib(class_1294,class_1265)).
neg(samebib(class_1294,class_1266)).
neg(samebib(class_1294,class_1267)).
neg(samebib(class_1294,class_1268)).
neg(samebib(class_1294,class_1269)).
neg(samebib(class_1294,class_1270)).
neg(samebib(class_1294,class_1271)).
neg(samebib(class_1294,class_1272)).
neg(samebib(class_1294,class_1273)).
neg(samebib(class_1294,class_1275)).
neg(samebib(class_1294,class_1276)).
neg(samebib(class_1294,class_1277)).
neg(samebib(class_1294,class_1278)).
neg(samebib(class_1294,class_1279)).
neg(samebib(class_1294,class_1280)).
neg(samebib(class_1294,class_1282)).
neg(samebib(class_1294,class_1283)).
neg(samebib(class_1294,class_1284)).
neg(samebib(class_1294,class_1285)).
neg(samebib(class_1294,class_1286)).
neg(samebib(class_1294,class_1287)).
neg(samebib(class_1294,class_1288)).
neg(samebib(class_1294,class_1289)).
samebib(class_1294,class_1290).
samebib(class_1294,class_1291).
samebib(class_1294,class_1292).
samebib(class_1294,class_1293).
samebib(class_1294,class_1294).
neg(samebib(class_1294,class_1295)).
neg(samebib(class_1295,class_1285)).
neg(samebib(class_1295,class_1286)).
neg(samebib(class_1295,class_1287)).
neg(samebib(class_1295,class_1288)).
neg(samebib(class_1295,class_1289)).
neg(samebib(class_1295,class_1291)).
neg(samebib(class_1295,class_1292)).
neg(samebib(class_1295,class_1293)).
neg(samebib(class_1295,class_1294)).
samebib(class_1295,class_1295).

sameauthor(author_blum_a_,author_blum_a_).
sameauthor(author_blum_a_,author_a_blum_).
sameauthor(author_blum_a_,author_avrim_blum_).
sameauthor(author_a_blum_,author_blum_a_).
sameauthor(author_a_blum_,author_a_blum_).
sameauthor(author_a_blum_,author_avrim_blum_).
sameauthor(author_avrim_blum_,author_blum_a_).
sameauthor(author_avrim_blum_,author_a_blum_).
sameauthor(author_avrim_blum_,author_avrim_blum_).
sameauthor(author_william_w_cohen_,author_william_w_cohen_).
sameauthor(author_william_w_cohen_,author_w_w_cohen_).
sameauthor(author_w_w_cohen_,author_william_w_cohen_).
sameauthor(author_w_w_cohen_,author_w_w_cohen_).
sameauthor(author_h_drucker_,author_h_drucker_).
sameauthor(author_h_drucker_,author_drucker_h_).
sameauthor(author_h_drucker_,author_drucker_harris_).
sameauthor(author_h_drucker_,author_drucker_).
sameauthor(author_h_drucker_,author_harris_drucker_).
sameauthor(author_h_drucker_,author_h_druker_).
sameauthor(author_drucker_h_,author_h_drucker_).
sameauthor(author_drucker_h_,author_drucker_h_).
sameauthor(author_drucker_h_,author_drucker_harris_).
sameauthor(author_drucker_h_,author_drucker_).
sameauthor(author_drucker_h_,author_harris_drucker_).
sameauthor(author_drucker_h_,author_h_druker_).
sameauthor(author_drucker_harris_,author_h_drucker_).
sameauthor(author_drucker_harris_,author_drucker_h_).
sameauthor(author_drucker_harris_,author_drucker_harris_).
sameauthor(author_drucker_harris_,author_drucker_).
sameauthor(author_drucker_harris_,author_harris_drucker_).
sameauthor(author_drucker_harris_,author_h_druker_).
sameauthor(author_drucker_,author_h_drucker_).
sameauthor(author_drucker_,author_drucker_h_).
sameauthor(author_drucker_,author_drucker_harris_).
sameauthor(author_drucker_,author_drucker_).
sameauthor(author_drucker_,author_harris_drucker_).
sameauthor(author_drucker_,author_h_druker_).
sameauthor(author_harris_drucker_,author_h_drucker_).
sameauthor(author_harris_drucker_,author_drucker_h_).
sameauthor(author_harris_drucker_,author_drucker_harris_).
sameauthor(author_harris_drucker_,author_drucker_).
sameauthor(author_harris_drucker_,author_harris_drucker_).
sameauthor(author_harris_drucker_,author_h_druker_).
sameauthor(author_h_druker_,author_h_drucker_).
sameauthor(author_h_druker_,author_drucker_h_).
sameauthor(author_h_druker_,author_drucker_harris_).
sameauthor(author_h_druker_,author_drucker_).
sameauthor(author_h_druker_,author_harris_drucker_).
sameauthor(author_h_druker_,author_h_druker_).
sameauthor(author_a_ehrenfeucht_,author_a_ehrenfeucht_).
sameauthor(author_yoav_freund_,author_yoav_freund_).
sameauthor(author_yoav_freund_,author_freund_y_).
sameauthor(author_yoav_freund_,author_y_freund_).
sameauthor(author_freund_y_,author_yoav_freund_).
sameauthor(author_freund_y_,author_freund_y_).
sameauthor(author_freund_y_,author_y_freund_).
sameauthor(author_y_freund_,author_yoav_freund_).
sameauthor(author_y_freund_,author_freund_y_).
sameauthor(author_y_freund_,author_y_freund_).
sameauthor(author_freud_y_,author_freud_y_).
sameauthor(author_freud_y_,author_y_freund_).
sameauthor(author_freud_y_,author_yoav_freund_).
sameauthor(author_freud_y_,author_freund_y_).
sameauthor(author_y_freund_,author_freud_y_).
sameauthor(author_yoav_freund_,author_freud_y_).
sameauthor(author_freund_y_,author_freud_y_).
sameauthor(author_goldman_s_,author_goldman_s_).
sameauthor(author_goldman_s_,author_sally_a_goldman_).
sameauthor(author_goldman_s_,author_s_a_goldman_).
sameauthor(author_sally_a_goldman_,author_goldman_s_).
sameauthor(author_sally_a_goldman_,author_sally_a_goldman_).
sameauthor(author_sally_a_goldman_,author_s_a_goldman_).
sameauthor(author_s_a_goldman_,author_goldman_s_).
sameauthor(author_s_a_goldman_,author_sally_a_goldman_).
sameauthor(author_s_a_goldman_,author_s_a_goldman_).
sameauthor(author_d_haussler_,author_d_haussler_).
sameauthor(author_d_haussler_,author_haussler_d_).
sameauthor(author_haussler_d_,author_d_haussler_).
sameauthor(author_haussler_d_,author_haussler_d_).
sameauthor(author_haussler_d_,author_david_haussler_).
sameauthor(author_d_haussler_,author_david_haussler_).
sameauthor(author_david_haussler_,author_haussler_d_).
sameauthor(author_david_haussler_,author_d_haussler_).
sameauthor(author_david_haussler_,author_david_haussler_).
sameauthor(author_kautz_henry_,author_kautz_henry_).
sameauthor(author_j_p_kearns_,author_j_p_kearns_).
sameauthor(author_michael_j_kearns_,author_michael_j_kearns_).
sameauthor(author_michael_j_kearns_,author_kearns_m_j_).
sameauthor(author_michael_j_kearns_,author_michael_kearns_).
sameauthor(author_michael_j_kearns_,author_m_j_kearns_).
sameauthor(author_michael_j_kearns_,author_kearns_michael_).
sameauthor(author_m_kearns_,author_m_kearns_).
sameauthor(author_m_kearns_,author_kearns_m_j_).
sameauthor(author_m_kearns_,author_m_j_kearns_).
sameauthor(author_m_kearns_,author_kearns_m_).
sameauthor(author_m_kearns_,author_kearns_michael_).
sameauthor(author_m_kearns_,author_michael_kearns_).
sameauthor(author_m_kearns_,author_michael_j_kearns_).
sameauthor(author_kearns_m_j_,author_michael_j_kearns_).
sameauthor(author_kearns_m_j_,author_m_kearns_).
sameauthor(author_kearns_m_j_,author_kearns_m_j_).
sameauthor(author_kearns_m_j_,author_michael_kearns_).
sameauthor(author_kearns_m_j_,author_m_j_kearns_).
sameauthor(author_kearns_m_j_,author_kearns_m_).
sameauthor(author_kearns_m_j_,author_kearns_michael_).
sameauthor(author_michael_kearns_,author_michael_j_kearns_).
sameauthor(author_michael_kearns_,author_kearns_m_j_).
sameauthor(author_michael_kearns_,author_michael_kearns_).
sameauthor(author_michael_kearns_,author_m_j_kearns_).
sameauthor(author_michael_kearns_,author_m_kearns_).
sameauthor(author_michael_kearns_,author_kearns_michael_).
sameauthor(author_m_j_kearns_,author_michael_j_kearns_).
sameauthor(author_m_j_kearns_,author_m_kearns_).
sameauthor(author_m_j_kearns_,author_kearns_m_j_).
sameauthor(author_m_j_kearns_,author_michael_kearns_).
sameauthor(author_m_j_kearns_,author_m_j_kearns_).
sameauthor(author_m_j_kearns_,author_kearns_m_).
sameauthor(author_m_j_kearns_,author_kearns_michael_).
sameauthor(author_kearns_m_,author_m_kearns_).
sameauthor(author_kearns_m_,author_kearns_m_j_).
sameauthor(author_kearns_m_,author_m_j_kearns_).
sameauthor(author_kearns_m_,author_kearns_m_).
sameauthor(author_kearns_m_,author_kearns_michael_).
sameauthor(author_kearns_m_,author_michael_kearns_).
sameauthor(author_kearns_m_,author_michael_j_kearns_).
sameauthor(author_kearns_michael_,author_michael_j_kearns_).
sameauthor(author_kearns_michael_,author_m_kearns_).
sameauthor(author_kearns_michael_,author_kearns_m_j_).
sameauthor(author_kearns_michael_,author_michael_kearns_).
sameauthor(author_kearns_michael_,author_m_j_kearns_).
sameauthor(author_kearns_michael_,author_kearns_m_).
sameauthor(author_kearns_michael_,author_kearns_michael_).
sameauthor(author_michael_kearns_,author_kearns_m_).
sameauthor(author_michael_j_kearns_,author_m_kearns_).
sameauthor(author_michael_j_kearns_,author_kearns_m_).
sameauthor(author_r_rivest_,author_r_rivest_).
sameauthor(author_r_rivest_,author_rivest_r_).
sameauthor(author_r_rivest_,author_rivest_r_l_).
sameauthor(author_r_rivest_,author_r_l_rivest_).
neg(sameauthor(author_r_rivest_,author_schapire_r_)).
neg(sameauthor(author_r_rivest_,author_r_schapire_)).
sameauthor(author_r_rivest_,author_ronald_l_rivest_).
sameauthor(author_rivest_r_,author_r_rivest_).
sameauthor(author_rivest_r_,author_rivest_r_).
sameauthor(author_rivest_r_,author_rivest_r_l_).
sameauthor(author_rivest_r_,author_ronald_l_rivest_).
sameauthor(author_rivest_r_,author_r_l_rivest_).
neg(sameauthor(author_rivest_r_,author_schapire_r_)).
neg(sameauthor(author_rivest_r_,author_r_schapire_)).
sameauthor(author_rivest_r_l_,author_r_rivest_).
sameauthor(author_rivest_r_l_,author_rivest_r_).
sameauthor(author_rivest_r_l_,author_rivest_r_l_).
sameauthor(author_rivest_r_l_,author_ronald_l_rivest_).
sameauthor(author_rivest_r_l_,author_r_l_rivest_).
neg(sameauthor(author_rivest_r_l_,author_schapire_r_)).
neg(sameauthor(author_rivest_r_l_,author_r_schapire_)).
sameauthor(author_ronald_l_rivest_,author_r_rivest_).
sameauthor(author_ronald_l_rivest_,author_rivest_r_).
sameauthor(author_ronald_l_rivest_,author_rivest_r_l_).
sameauthor(author_ronald_l_rivest_,author_ronald_l_rivest_).
sameauthor(author_ronald_l_rivest_,author_r_l_rivest_).
sameauthor(author_r_e_schapire_,author_r_e_schapire_).
sameauthor(author_r_e_schapire_,author_robert_e_schapire_).
sameauthor(author_r_e_schapire_,author_schapire_r_e_).
sameauthor(author_r_e_schapire_,author_schapire_r_).
sameauthor(author_r_e_schapire_,author_r_schapire_).
sameauthor(author_r_e_schapire_,author_schapire_).
sameauthor(author_r_l_rivest_,author_r_rivest_).
sameauthor(author_r_l_rivest_,author_rivest_r_).
sameauthor(author_r_l_rivest_,author_rivest_r_l_).
sameauthor(author_r_l_rivest_,author_ronald_l_rivest_).
sameauthor(author_r_l_rivest_,author_r_l_rivest_).
neg(sameauthor(author_r_l_rivest_,author_schapire_r_)).
neg(sameauthor(author_r_l_rivest_,author_r_schapire_)).
sameauthor(author_robert_e_schapire_,author_r_e_schapire_).
sameauthor(author_robert_e_schapire_,author_robert_e_schapire_).
sameauthor(author_robert_e_schapire_,author_schapire_r_e_).
sameauthor(author_robert_e_schapire_,author_schapire_r_).
sameauthor(author_robert_e_schapire_,author_r_schapire_).
sameauthor(author_robert_e_schapire_,author_schapire_).
sameauthor(author_schapire_r_e_,author_r_e_schapire_).
sameauthor(author_schapire_r_e_,author_robert_e_schapire_).
sameauthor(author_schapire_r_e_,author_schapire_r_e_).
sameauthor(author_schapire_r_e_,author_schapire_r_).
sameauthor(author_schapire_r_e_,author_r_schapire_).
sameauthor(author_schapire_r_e_,author_schapire_).
sameauthor(author_schapire_r_e_,author_shapire_r_).
sameauthor(author_r_e_schapire_,author_shapire_r_).
neg(sameauthor(author_schapire_r_,author_r_rivest_)).
neg(sameauthor(author_schapire_r_,author_rivest_r_)).
neg(sameauthor(author_schapire_r_,author_rivest_r_l_)).
sameauthor(author_schapire_r_,author_r_e_schapire_).
neg(sameauthor(author_schapire_r_,author_r_l_rivest_)).
sameauthor(author_schapire_r_,author_robert_e_schapire_).
sameauthor(author_schapire_r_,author_schapire_r_e_).
sameauthor(author_schapire_r_,author_schapire_r_).
sameauthor(author_schapire_r_,author_shapire_r_).
sameauthor(author_schapire_r_,author_r_schapire_).
sameauthor(author_schapire_r_,author_schapire_).
sameauthor(author_shapire_r_,author_schapire_r_e_).
sameauthor(author_shapire_r_,author_r_e_schapire_).
sameauthor(author_shapire_r_,author_schapire_r_).
sameauthor(author_shapire_r_,author_shapire_r_).
sameauthor(author_shapire_r_,author_r_schapire_).
sameauthor(author_shapire_r_,author_robert_e_schapire_).
sameauthor(author_shapire_r_,author_schapire_).
neg(sameauthor(author_r_schapire_,author_r_rivest_)).
neg(sameauthor(author_r_schapire_,author_rivest_r_)).
neg(sameauthor(author_r_schapire_,author_rivest_r_l_)).
sameauthor(author_r_schapire_,author_r_e_schapire_).
neg(sameauthor(author_r_schapire_,author_r_l_rivest_)).
sameauthor(author_r_schapire_,author_robert_e_schapire_).
sameauthor(author_r_schapire_,author_schapire_r_e_).
sameauthor(author_r_schapire_,author_schapire_r_).
sameauthor(author_r_schapire_,author_shapire_r_).
sameauthor(author_r_schapire_,author_r_schapire_).
sameauthor(author_r_schapire_,author_schapire_).
sameauthor(author_robert_e_schapire_,author_shapire_r_).
sameauthor(author_schapire_,author_r_e_schapire_).
sameauthor(author_schapire_,author_robert_e_schapire_).
sameauthor(author_schapire_,author_schapire_r_e_).
sameauthor(author_schapire_,author_schapire_r_).
sameauthor(author_schapire_,author_shapire_r_).
sameauthor(author_schapire_,author_r_schapire_).
sameauthor(author_schapire_,author_schapire_).

sametitle(title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_,title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_).
sametitle(title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_,title_weakly_learning_learning_dnf_10_characterizing_statistical_query_using_fourier_analysis_).
sametitle(title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_,title_weakly_8_learning_dnf_characterizing_statistical_query_learn_ing_using_fourier_analysis_).
sametitle(title_weakly_learning_learning_dnf_10_characterizing_statistical_query_using_fourier_analysis_,title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_).
sametitle(title_weakly_learning_learning_dnf_10_characterizing_statistical_query_using_fourier_analysis_,title_weakly_learning_learning_dnf_10_characterizing_statistical_query_using_fourier_analysis_).
sametitle(title_weakly_learning_learning_dnf_10_characterizing_statistical_query_using_fourier_analysis_,title_weakly_8_learning_dnf_characterizing_statistical_query_learn_ing_using_fourier_analysis_).
sametitle(title_weakly_8_learning_dnf_characterizing_statistical_query_learn_ing_using_fourier_analysis_,title_weakly_learning_learning_dnf_characterizing_statistical_query_using_fourier_analysis_).
sametitle(title_weakly_8_learning_dnf_characterizing_statistical_query_learn_ing_using_fourier_analysis_,title_weakly_learning_learning_dnf_10_characterizing_statistical_query_using_fourier_analysis_).
sametitle(title_weakly_8_learning_dnf_characterizing_statistical_query_learn_ing_using_fourier_analysis_,title_weakly_8_learning_dnf_characterizing_statistical_query_learn_ing_using_fourier_analysis_).
sametitle(title_learning_to_order_things_,title_learning_to_order_things_).
sametitle(title_learning_to_order_things_,title_learning_ro_order_things_).
sametitle(title_learning_ro_order_things_,title_learning_to_order_things_).
sametitle(title_learning_ro_order_things_,title_learning_ro_order_things_).
sametitle(title_improving_performance_neural_networks_using_a_boosting_algorithm_,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
sametitle(title_improving_performance_neural_networks_using_a_boosting_algorithm_,title_patrice_1993_improving_performance_neural_networks_using_a_boosting_algorithm_).
sametitle(title_improving_performance_neural_networks_using_a_boosting_algorithm_,title_1993_improving_performance_neural_networks_using_a_boosting_algorithm_).
sametitle(title_patrice_1993_improving_performance_neural_networks_using_a_boosting_algorithm_,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
sametitle(title_patrice_1993_improving_performance_neural_networks_using_a_boosting_algorithm_,title_patrice_1993_improving_performance_neural_networks_using_a_boosting_algorithm_).
sametitle(title_patrice_1993_improving_performance_neural_networks_using_a_boosting_algorithm_,title_1993_improving_performance_neural_networks_using_a_boosting_algorithm_).
sametitle(title_1993_improving_performance_neural_networks_using_a_boosting_algorithm_,title_improving_performance_neural_networks_using_a_boosting_algorithm_).
sametitle(title_1993_improving_performance_neural_networks_using_a_boosting_algorithm_,title_patrice_1993_improving_performance_neural_networks_using_a_boosting_algorithm_).
sametitle(title_1993_improving_performance_neural_networks_using_a_boosting_algorithm_,title_1993_improving_performance_neural_networks_using_a_boosting_algorithm_).
neg(sametitle(title_1993_improving_performance_neural_networks_using_a_boosting_algorithm_,title_1993_inference_finite_automata_using_homing_sequences_)).
neg(sametitle(title_improving_performance_neural_networks_using_a_boosting_algorithm_,title_1993_inference_finite_automata_using_homing_sequences_)).
sametitle(title_a_general_lower_bound_number_examples_needed_learning_,title_a_general_lower_bound_number_examples_needed_learning_).
sametitle(title_data_filtering_distribution_modeling_algorithms_machine_learning_,title_data_filtering_distribution_modeling_algorithms_machine_learning_).
sametitle(title_accelerating_learning_using_query_by_committee_,title_accelerating_learning_using_query_by_committee_).
neg(sametitle(title_accelerating_learning_using_query_by_committee_,title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_)).
neg(sametitle(title_accelerating_learning_using_query_by_committee_,title_query_by_committee_)).
sametitle(title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_,title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_).
sametitle(title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_,title_1995_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_).
neg(sametitle(title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_,title_accelerating_learning_using_query_by_committee_)).
sametitle(title_1995_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_,title_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_).
sametitle(title_1995_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_,title_1995_efficient_algorithms_learning_to_play_repeated_games_against_computationally_bounded_adversaries_).
sametitle(title_predicting_a_binary_sequence_almost_as_as_well_optimal_biased_coin_,title_predicting_a_binary_sequence_almost_as_as_well_optimal_biased_coin_).
neg(sametitle(title_predicting_a_binary_sequence_almost_as_as_well_optimal_biased_coin_,title_predicting_bits_almost_as_as_well_optimal_biased_coin_)).
sametitle(title_predicting_a_binary_sequence_almost_as_as_well_optimal_biased_coin_,title_predicting_a_binary_sequence_as_as_well_optimal_biased_coin_).
neg(sametitle(title_predicting_bits_almost_as_as_well_optimal_biased_coin_,title_predicting_a_binary_sequence_almost_as_as_well_optimal_biased_coin_)).
sametitle(title_predicting_bits_almost_as_as_well_optimal_biased_coin_,title_predicting_bits_almost_as_as_well_optimal_biased_coin_).
neg(sametitle(title_predicting_bits_almost_as_as_well_optimal_biased_coin_,title_predicting_a_binary_sequence_as_as_well_optimal_biased_coin_)).
sametitle(title_predicting_a_binary_sequence_as_as_well_optimal_biased_coin_,title_predicting_a_binary_sequence_almost_as_as_well_optimal_biased_coin_).
neg(sametitle(title_predicting_a_binary_sequence_as_as_well_optimal_biased_coin_,title_predicting_bits_almost_as_as_well_optimal_biased_coin_)).
sametitle(title_predicting_a_binary_sequence_as_as_well_optimal_biased_coin_,title_predicting_a_binary_sequence_as_as_well_optimal_biased_coin_).
sametitle(title_learning_binary_relations_total_orders_,title_learning_binary_relations_total_orders_).
sametitle(title_equivalence_models_polynomial_learnability_,title_equivalence_models_polynomial_learnability_).
sametitle(title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
sametitle(title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_,title_unifying_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_).
sametitle(title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_,title_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_).
neg(sametitle(title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_,title_rigourous_learning_curve_bounds_from_statistical_mechanics_)).
neg(sametitle(title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_,title_rigorous_learning_curve_bounds_from_statistical_mechanics_)).
sametitle(title_unifying_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
sametitle(title_unifying_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_,title_unifying_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_).
sametitle(title_unifying_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_,title_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_).
neg(sametitle(title_unifying_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_,title_rigourous_learning_curve_bounds_from_statistical_mechanics_)).
neg(sametitle(title_unifying_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_,title_rigorous_learning_curve_bounds_from_statistical_mechanics_)).
sametitle(title_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_).
sametitle(title_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_,title_unifying_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_).
sametitle(title_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_,title_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_).
neg(sametitle(title_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_,title_rigourous_learning_curve_bounds_from_statistical_mechanics_)).
neg(sametitle(title_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_,title_rigorous_learning_curve_bounds_from_statistical_mechanics_)).
neg(sametitle(title_rigourous_learning_curve_bounds_from_statistical_mechanics_,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_)).
neg(sametitle(title_rigourous_learning_curve_bounds_from_statistical_mechanics_,title_unifying_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_)).
neg(sametitle(title_rigourous_learning_curve_bounds_from_statistical_mechanics_,title_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_)).
sametitle(title_rigourous_learning_curve_bounds_from_statistical_mechanics_,title_rigourous_learning_curve_bounds_from_statistical_mechanics_).
sametitle(title_rigourous_learning_curve_bounds_from_statistical_mechanics_,title_rigorous_learning_curve_bounds_from_statistical_mechanics_).
neg(sametitle(title_rigorous_learning_curve_bounds_from_statistical_mechanics_,title_bounds_sample_complexity_bayesian_learning_using_information_theory_vc_dimension_)).
neg(sametitle(title_rigorous_learning_curve_bounds_from_statistical_mechanics_,title_unifying_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_)).
neg(sametitle(title_rigorous_learning_curve_bounds_from_statistical_mechanics_,title_bounds_sample_complexity_bayesian_learning_theory_theory_using_information_vc_dimension_)).
sametitle(title_rigorous_learning_curve_bounds_from_statistical_mechanics_,title_rigourous_learning_curve_bounds_from_statistical_mechanics_).
sametitle(title_rigorous_learning_curve_bounds_from_statistical_mechanics_,title_rigorous_learning_curve_bounds_from_statistical_mechanics_).
sametitle(title_bart_1993_reasoning_with_characteristic_models_,title_bart_1993_reasoning_with_characteristic_models_).
sametitle(title_performance_evaluation_protocols_distributed_systems_an_alternative_to_message_counting_,title_performance_evaluation_protocols_distributed_systems_an_alternative_to_message_counting_).
sametitle(title_learning_from_a_population_hypotheses_,title_learning_from_a_population_hypotheses_).
sametitle(title_learnabil_ity_discrete_distributions_,title_learnabil_ity_discrete_distributions_).
sametitle(title_learnabil_ity_discrete_distributions_,title_learnability_discrete_distributions_).
neg(sametitle(title_learnabil_ity_discrete_distributions_,title_an_introduction_to_computational_learning_theory_)).
neg(sametitle(title_learnabil_ity_discrete_distributions_,title_introduction_to_computational_learning_theory_)).
neg(sametitle(title_learnabil_ity_discrete_distributions_,title_vazirani_1995_introduction_to_computational_learning_theory_)).
neg(sametitle(title_learnabil_ity_discrete_distributions_,title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_)).
neg(sametitle(title_learnabil_ity_discrete_distributions_,title_boosting_ability_top_down_decision_tree_learning_algorithms_)).
sametitle(title_learnability_discrete_distributions_,title_learnabil_ity_discrete_distributions_).
sametitle(title_learnability_discrete_distributions_,title_learnability_discrete_distributions_).
sametitle(title_learnability_discrete_distributions_,title_learnability_discrete_distributions_extended_abstract_).
neg(sametitle(title_learnability_discrete_distributions_,title_an_introduction_to_computational_learning_theory_)).
neg(sametitle(title_learnability_discrete_distributions_,title_introduction_to_computational_learning_theory_)).
neg(sametitle(title_learnability_discrete_distributions_,title_vazirani_1995_introduction_to_computational_learning_theory_)).
neg(sametitle(title_learnability_discrete_distributions_,title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_)).
neg(sametitle(title_learnability_discrete_distributions_,title_boosting_ability_top_down_decision_tree_learning_algorithms_)).
sametitle(title_learnability_discrete_distributions_extended_abstract_,title_learnability_discrete_distributions_).
sametitle(title_learnability_discrete_distributions_extended_abstract_,title_learnability_discrete_distributions_extended_abstract_).
sametitle(title_learnability_discrete_distributions_extended_abstract_,title_learnabil_ity_discrete_distributions_).
sametitle(title_learnabil_ity_discrete_distributions_,title_learnability_discrete_distributions_extended_abstract_).
neg(sametitle(title_an_introduction_to_computational_learning_theory_,title_learnabil_ity_discrete_distributions_)).
neg(sametitle(title_an_introduction_to_computational_learning_theory_,title_learnability_discrete_distributions_)).
sametitle(title_an_introduction_to_computational_learning_theory_,title_an_introduction_to_computational_learning_theory_).
sametitle(title_an_introduction_to_computational_learning_theory_,title_introduction_to_computational_learning_theory_).
sametitle(title_an_introduction_to_computational_learning_theory_,title_vazirani_1995_introduction_to_computational_learning_theory_).
neg(sametitle(title_an_introduction_to_computational_learning_theory_,title_an_experimental_theoretical_comparison_model_selection_methods_)).
neg(sametitle(title_an_introduction_to_computational_learning_theory_,title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_)).
neg(sametitle(title_an_introduction_to_computational_learning_theory_,title_boosting_ability_top_down_decision_tree_learning_algorithms_)).
neg(sametitle(title_introduction_to_computational_learning_theory_,title_learnabil_ity_discrete_distributions_)).
neg(sametitle(title_introduction_to_computational_learning_theory_,title_learnability_discrete_distributions_)).
sametitle(title_introduction_to_computational_learning_theory_,title_an_introduction_to_computational_learning_theory_).
sametitle(title_introduction_to_computational_learning_theory_,title_introduction_to_computational_learning_theory_).
sametitle(title_introduction_to_computational_learning_theory_,title_vazirani_1995_introduction_to_computational_learning_theory_).
neg(sametitle(title_introduction_to_computational_learning_theory_,title_an_experimental_theoretical_comparison_model_selection_methods_)).
neg(sametitle(title_introduction_to_computational_learning_theory_,title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_)).
neg(sametitle(title_introduction_to_computational_learning_theory_,title_boosting_ability_top_down_decision_tree_learning_algorithms_)).
neg(sametitle(title_vazirani_1995_introduction_to_computational_learning_theory_,title_learnabil_ity_discrete_distributions_)).
neg(sametitle(title_vazirani_1995_introduction_to_computational_learning_theory_,title_learnability_discrete_distributions_)).
sametitle(title_vazirani_1995_introduction_to_computational_learning_theory_,title_an_introduction_to_computational_learning_theory_).
sametitle(title_vazirani_1995_introduction_to_computational_learning_theory_,title_introduction_to_computational_learning_theory_).
sametitle(title_vazirani_1995_introduction_to_computational_learning_theory_,title_vazirani_1995_introduction_to_computational_learning_theory_).
neg(sametitle(title_vazirani_1995_introduction_to_computational_learning_theory_,title_an_experimental_theoretical_comparison_model_selection_methods_)).
neg(sametitle(title_vazirani_1995_introduction_to_computational_learning_theory_,title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_)).
neg(sametitle(title_vazirani_1995_introduction_to_computational_learning_theory_,title_boosting_ability_top_down_decision_tree_learning_algorithms_)).
sametitle(title_an_experimental_theoretical_comparison_model_selection_methods_,title_an_experimental_theoretical_comparison_model_selection_methods_).
neg(sametitle(title_an_experimental_theoretical_comparison_model_selection_methods_,title_an_introduction_to_computational_learning_theory_)).
neg(sametitle(title_an_experimental_theoretical_comparison_model_selection_methods_,title_introduction_to_computational_learning_theory_)).
neg(sametitle(title_an_experimental_theoretical_comparison_model_selection_methods_,title_vazirani_1995_introduction_to_computational_learning_theory_)).
neg(sametitle(title_an_experimental_theoretical_comparison_model_selection_methods_,title_boosting_ability_top_down_decision_tree_learning_algorithms_)).
neg(sametitle(title_an_experimental_theoretical_comparison_model_selection_methods_,title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_)).
neg(sametitle(title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_,title_learnabil_ity_discrete_distributions_)).
neg(sametitle(title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_,title_learnability_discrete_distributions_)).
neg(sametitle(title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_,title_an_introduction_to_computational_learning_theory_)).
neg(sametitle(title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_,title_introduction_to_computational_learning_theory_)).
neg(sametitle(title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_,title_vazirani_1995_introduction_to_computational_learning_theory_)).
neg(sametitle(title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_,title_an_experimental_theoretical_comparison_model_selection_methods_)).
sametitle(title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_,title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_).
sametitle(title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_,title_boosting_ability_top_down_decision_tree_learning_algorithms_).
neg(sametitle(title_boosting_ability_top_down_decision_tree_learning_algorithms_,title_an_introduction_to_computational_learning_theory_)).
neg(sametitle(title_boosting_ability_top_down_decision_tree_learning_algorithms_,title_introduction_to_computational_learning_theory_)).
neg(sametitle(title_boosting_ability_top_down_decision_tree_learning_algorithms_,title_vazirani_1995_introduction_to_computational_learning_theory_)).
neg(sametitle(title_boosting_ability_top_down_decision_tree_learning_algorithms_,title_an_experimental_theoretical_comparison_model_selection_methods_)).
sametitle(title_boosting_ability_top_down_decision_tree_learning_algorithms_,title_mansour_1996_boosting_ability_top_down_decision_tree_learning_algorithms_).
sametitle(title_boosting_ability_top_down_decision_tree_learning_algorithms_,title_boosting_ability_top_down_decision_tree_learning_algorithms_).
neg(sametitle(title_boosting_ability_top_down_decision_tree_learning_algorithms_,title_learnabil_ity_discrete_distributions_)).
neg(sametitle(title_boosting_ability_top_down_decision_tree_learning_algorithms_,title_learnability_discrete_distributions_)).
sametitle(title_a_sparse_sampling_algorithm_near_optimal_planning_large_markov_decision_processes_1999_,title_a_sparse_sampling_algorithm_near_optimal_planning_large_markov_decision_processes_1999_).
sametitle(title_md5_message_digest_algorithm_request_comments_informational_rfc_1321_,title_md5_message_digest_algorithm_request_comments_informational_rfc_1321_).
sametitle(title_inference_finite_automata_using_homing_sequences_,title_inference_finite_automata_using_homing_sequences_).
sametitle(title_inference_finite_automata_using_homing_sequences_,title_1993_inference_finite_automata_using_homing_sequences_).
sametitle(title_inference_finite_automata_using_homing_sequences_,title_inference_finite_automata_using_homing_sequences_r_l_).
sametitle(title_inference_finite_automata_using_homing_sequences_,title_inference_finite_automata_using_homing_sequences_information_computation_).
neg(sametitle(title_inference_finite_automata_using_homing_sequences_,title_diversity_based_inference_finite_automata_)).
neg(sametitle(title_1993_inference_finite_automata_using_homing_sequences_,title_1993_improving_performance_neural_networks_using_a_boosting_algorithm_)).
neg(sametitle(title_1993_inference_finite_automata_using_homing_sequences_,title_improving_performance_neural_networks_using_a_boosting_algorithm_)).
sametitle(title_1993_inference_finite_automata_using_homing_sequences_,title_inference_finite_automata_using_homing_sequences_).
sametitle(title_1993_inference_finite_automata_using_homing_sequences_,title_1993_inference_finite_automata_using_homing_sequences_).
sametitle(title_1993_inference_finite_automata_using_homing_sequences_,title_inference_finite_automata_using_homing_sequences_r_l_).
sametitle(title_1993_inference_finite_automata_using_homing_sequences_,title_inference_finite_automata_using_homing_sequences_information_computation_).
neg(sametitle(title_1993_inference_finite_automata_using_homing_sequences_,title_diversity_based_inference_finite_automata_)).
sametitle(title_inference_finite_automata_using_homing_sequences_r_l_,title_inference_finite_automata_using_homing_sequences_).
sametitle(title_inference_finite_automata_using_homing_sequences_r_l_,title_1993_inference_finite_automata_using_homing_sequences_).
sametitle(title_inference_finite_automata_using_homing_sequences_r_l_,title_inference_finite_automata_using_homing_sequences_r_l_).
sametitle(title_inference_finite_automata_using_homing_sequences_r_l_,title_inference_finite_automata_using_homing_sequences_information_computation_).
neg(sametitle(title_inference_finite_automata_using_homing_sequences_r_l_,title_private_communication_)).
neg(sametitle(title_inference_finite_automata_using_homing_sequences_r_l_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_inference_finite_automata_using_homing_sequences_r_l_,title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_inference_finite_automata_using_homing_sequences_r_l_,title_boosting_margin_)).
neg(sametitle(title_inference_finite_automata_using_homing_sequences_r_l_,title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
sametitle(title_inference_finite_automata_using_homing_sequences_information_computation_,title_inference_finite_automata_using_homing_sequences_).
sametitle(title_inference_finite_automata_using_homing_sequences_information_computation_,title_1993_inference_finite_automata_using_homing_sequences_).
sametitle(title_inference_finite_automata_using_homing_sequences_information_computation_,title_inference_finite_automata_using_homing_sequences_r_l_).
sametitle(title_inference_finite_automata_using_homing_sequences_information_computation_,title_inference_finite_automata_using_homing_sequences_information_computation_).
neg(sametitle(title_inference_finite_automata_using_homing_sequences_information_computation_,title_diversity_based_inference_finite_automata_)).
neg(sametitle(title_diversity_based_inference_finite_automata_,title_inference_finite_automata_using_homing_sequences_)).
neg(sametitle(title_diversity_based_inference_finite_automata_,title_1993_inference_finite_automata_using_homing_sequences_)).
neg(sametitle(title_diversity_based_inference_finite_automata_,title_inference_finite_automata_using_homing_sequences_information_computation_)).
sametitle(title_diversity_based_inference_finite_automata_,title_diversity_based_inference_finite_automata_).
sametitle(title_design_analysis_efficient_learning_algorithms_,title_design_analysis_efficient_learning_algorithms_).
sametitle(title_design_analysis_efficient_learning_algorithms_,title_design_analysis_ef_ficient_learning_algorithms_).
neg(sametitle(title_design_analysis_efficient_learning_algorithms_,title_worst_case_analysis_temporal_difference_learning_algorithms_)).
sametitle(title_private_correspondence_,title_private_correspondence_).
neg(sametitle(title_private_correspondence_,title_private_communication_)).
sametitle(title_learning_probabilistic_read_once_formulas_product_distributions_,title_learning_probabilistic_read_once_formulas_product_distributions_).
neg(sametitle(title_learning_probabilistic_read_once_formulas_product_distributions_,title_design_analysis_ef_ficient_learning_algorithms_)).
neg(sametitle(title_learning_probabilistic_read_once_formulas_product_distributions_,title_private_correspondence_)).
neg(sametitle(title_learning_probabilistic_read_once_formulas_product_distributions_,title_improved_boosting_algorithms_using_confidence_rated_predictions_)).
sametitle(title_design_analysis_ef_ficient_learning_algorithms_,title_design_analysis_efficient_learning_algorithms_).
neg(sametitle(title_design_analysis_ef_ficient_learning_algorithms_,title_learning_probabilistic_read_once_formulas_product_distributions_)).
sametitle(title_design_analysis_ef_ficient_learning_algorithms_,title_design_analysis_ef_ficient_learning_algorithms_).
neg(sametitle(title_design_analysis_ef_ficient_learning_algorithms_,title_private_correspondence_)).
neg(sametitle(title_design_analysis_ef_ficient_learning_algorithms_,title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_)).
neg(sametitle(title_design_analysis_ef_ficient_learning_algorithms_,title_worst_case_analysis_temporal_difference_learning_algorithms_)).
neg(sametitle(title_design_analysis_ef_ficient_learning_algorithms_,title_worst_case_analysis_temporal_difference_learing_algorithms_)).
neg(sametitle(title_design_analysis_ef_ficient_learning_algorithms_,title_improved_boosting_algorithms_using_confidence_rated_predictions_)).
neg(sametitle(title_private_correspondence_,title_learning_probabilistic_read_once_formulas_product_distributions_)).
neg(sametitle(title_private_correspondence_,title_design_analysis_ef_ficient_learning_algorithms_)).
neg(sametitle(title_private_correspondence_,title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_)).
neg(sametitle(title_private_correspondence_,title_worst_case_analysis_temporal_difference_learning_algorithms_)).
neg(sametitle(title_private_correspondence_,title_worst_case_analysis_temporal_difference_learing_algorithms_)).
neg(sametitle(title_private_correspondence_,title_improved_boosting_algorithms_using_confidence_rated_predictions_)).
neg(sametitle(title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_,title_design_analysis_ef_ficient_learning_algorithms_)).
neg(sametitle(title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_,title_private_correspondence_)).
sametitle(title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_,title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_).
neg(sametitle(title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_,title_worst_case_analysis_temporal_difference_learning_algorithms_)).
neg(sametitle(title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_,title_worst_case_analysis_temporal_difference_learing_algorithms_)).
sametitle(title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_,title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_).
neg(sametitle(title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_,title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_,title_query_by_committee_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learning_algorithms_,title_design_analysis_ef_ficient_learning_algorithms_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learning_algorithms_,title_private_correspondence_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learning_algorithms_,title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_)).
sametitle(title_worst_case_analysis_temporal_difference_learning_algorithms_,title_worst_case_analysis_temporal_difference_learning_algorithms_).
sametitle(title_worst_case_analysis_temporal_difference_learning_algorithms_,title_worst_case_analysis_temporal_difference_learing_algorithms_).
neg(sametitle(title_worst_case_analysis_temporal_difference_learning_algorithms_,title_private_communication_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learning_algorithms_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learning_algorithms_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learning_algorithms_,title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learning_algorithms_,title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learning_algorithms_,title_boosting_margin_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learning_algorithms_,title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learning_algorithms_,title_query_by_committee_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learning_algorithms_,title_using_output_codes_to_boost_multiclass_learning_problems_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learning_algorithms_,title_improved_boosting_algorithms_using_confidence_rated_predictions_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learing_algorithms_,title_design_analysis_ef_ficient_learning_algorithms_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learing_algorithms_,title_private_correspondence_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learing_algorithms_,title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_)).
sametitle(title_worst_case_analysis_temporal_difference_learing_algorithms_,title_worst_case_analysis_temporal_difference_learning_algorithms_).
sametitle(title_worst_case_analysis_temporal_difference_learing_algorithms_,title_worst_case_analysis_temporal_difference_learing_algorithms_).
neg(sametitle(title_worst_case_analysis_temporal_difference_learing_algorithms_,title_private_communication_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learing_algorithms_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learing_algorithms_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learing_algorithms_,title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learing_algorithms_,title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learing_algorithms_,title_boosting_margin_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learing_algorithms_,title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learing_algorithms_,title_query_by_committee_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learing_algorithms_,title_using_output_codes_to_boost_multiclass_learning_problems_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learing_algorithms_,title_improved_boosting_algorithms_using_confidence_rated_predictions_)).
neg(sametitle(title_worst_case_analysis_temporal_difference_learning_algorithms_,title_design_analysis_efficient_learning_algorithms_)).
sametitle(title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_,title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_).
sametitle(title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_,title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_).
neg(sametitle(title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_,title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_,title_query_by_committee_)).
neg(sametitle(title_private_communication_,title_inference_finite_automata_using_homing_sequences_r_l_)).
neg(sametitle(title_private_communication_,title_private_correspondence_)).
neg(sametitle(title_private_communication_,title_worst_case_analysis_temporal_difference_learning_algorithms_)).
neg(sametitle(title_private_communication_,title_worst_case_analysis_temporal_difference_learing_algorithms_)).
sametitle(title_private_communication_,title_private_communication_).
neg(sametitle(title_private_communication_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_private_communication_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_)).
neg(sametitle(title_private_communication_,title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_private_communication_,title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_private_communication_,title_boosting_margin_)).
neg(sametitle(title_private_communication_,title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_private_communication_,title_query_by_committee_)).
neg(sametitle(title_private_communication_,title_using_output_codes_to_boost_multiclass_learning_problems_)).
neg(sametitle(title_private_communication_,title_improved_boosting_algorithms_using_confidence_rated_predictions_)).
neg(sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_inference_finite_automata_using_homing_sequences_r_l_)).
neg(sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_)).
neg(sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_worst_case_analysis_temporal_difference_learning_algorithms_)).
neg(sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_worst_case_analysis_temporal_difference_learing_algorithms_)).
neg(sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_)).
neg(sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_private_communication_)).
sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_).
sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_boosting_margin_).
sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_).
neg(sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_query_by_committee_)).
neg(sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_using_output_codes_to_boost_multiclass_learning_problems_)).
neg(sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_improved_boosting_algorithms_using_confidence_rated_predictions_)).
neg(sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_worst_case_analysis_temporal_difference_learning_algorithms_)).
neg(sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_worst_case_analysis_temporal_difference_learing_algorithms_)).
neg(sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_private_communication_)).
sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_).
sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_boosting_margin_).
sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_).
neg(sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_query_by_committee_)).
neg(sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_using_output_codes_to_boost_multiclass_learning_problems_)).
neg(sametitle(title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_improved_boosting_algorithms_using_confidence_rated_predictions_)).
neg(sametitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_worst_case_analysis_temporal_difference_learning_algorithms_)).
neg(sametitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_worst_case_analysis_temporal_difference_learing_algorithms_)).
neg(sametitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_private_communication_)).
sametitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_).
sametitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_boosting_margin_).
sametitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_).
neg(sametitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_query_by_committee_)).
neg(sametitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_using_output_codes_to_boost_multiclass_learning_problems_)).
neg(sametitle(title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_improved_boosting_algorithms_using_confidence_rated_predictions_)).
neg(sametitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_inference_finite_automata_using_homing_sequences_r_l_)).
neg(sametitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_)).
neg(sametitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_worst_case_analysis_temporal_difference_learning_algorithms_)).
neg(sametitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_worst_case_analysis_temporal_difference_learing_algorithms_)).
neg(sametitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_)).
neg(sametitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_private_communication_)).
sametitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_).
sametitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_boosting_margin_).
sametitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_).
neg(sametitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_query_by_committee_)).
neg(sametitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_using_output_codes_to_boost_multiclass_learning_problems_)).
neg(sametitle(title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_improved_boosting_algorithms_using_confidence_rated_predictions_)).
neg(sametitle(title_boosting_margin_,title_inference_finite_automata_using_homing_sequences_r_l_)).
neg(sametitle(title_boosting_margin_,title_worst_case_analysis_temporal_difference_learning_algorithms_)).
neg(sametitle(title_boosting_margin_,title_worst_case_analysis_temporal_difference_learing_algorithms_)).
neg(sametitle(title_boosting_margin_,title_private_communication_)).
sametitle(title_boosting_margin_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_boosting_margin_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_).
sametitle(title_boosting_margin_,title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_boosting_margin_,title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_boosting_margin_,title_boosting_margin_).
sametitle(title_boosting_margin_,title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_boosting_margin_,title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_).
neg(sametitle(title_boosting_margin_,title_query_by_committee_)).
neg(sametitle(title_boosting_margin_,title_using_output_codes_to_boost_multiclass_learning_problems_)).
neg(sametitle(title_boosting_margin_,title_improved_boosting_algorithms_using_confidence_rated_predictions_)).
neg(sametitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_inference_finite_automata_using_homing_sequences_r_l_)).
neg(sametitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_worst_case_analysis_temporal_difference_learning_algorithms_)).
neg(sametitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_worst_case_analysis_temporal_difference_learing_algorithms_)).
neg(sametitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_private_communication_)).
sametitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_).
sametitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_boosting_margin_).
sametitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_).
neg(sametitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_query_by_committee_)).
neg(sametitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_using_output_codes_to_boost_multiclass_learning_problems_)).
neg(sametitle(title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_,title_improved_boosting_algorithms_using_confidence_rated_predictions_)).
sametitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_).
sametitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_boosting_margin_).
sametitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_).
sametitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_).
neg(sametitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_query_by_committee_)).
neg(sametitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_using_output_codes_to_boost_multiclass_learning_problems_)).
neg(sametitle(title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_,title_improved_boosting_algorithms_using_confidence_rated_predictions_)).
neg(sametitle(title_query_by_committee_,title_accelerating_learning_using_query_by_committee_)).
neg(sametitle(title_query_by_committee_,title_learning_sparse_multivariate_polynomial_over_a_field_with_queries_counterexamples_)).
neg(sametitle(title_query_by_committee_,title_worst_case_analysis_temporal_difference_learning_algorithms_)).
neg(sametitle(title_query_by_committee_,title_worst_case_analysis_temporal_difference_learing_algorithms_)).
neg(sametitle(title_query_by_committee_,title_learning_sparse_multivariate_polynomials_over_a_field_with_queries_counterexamples_)).
neg(sametitle(title_query_by_committee_,title_private_communication_)).
neg(sametitle(title_query_by_committee_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_query_by_committee_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_)).
neg(sametitle(title_query_by_committee_,title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_query_by_committee_,title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_query_by_committee_,title_boosting_margin_)).
neg(sametitle(title_query_by_committee_,title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_query_by_committee_,title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_)).
sametitle(title_query_by_committee_,title_query_by_committee_).
neg(sametitle(title_query_by_committee_,title_using_output_codes_to_boost_multiclass_learning_problems_)).
neg(sametitle(title_query_by_committee_,title_improved_boosting_algorithms_using_confidence_rated_predictions_)).
neg(sametitle(title_using_output_codes_to_boost_multiclass_learning_problems_,title_worst_case_analysis_temporal_difference_learning_algorithms_)).
neg(sametitle(title_using_output_codes_to_boost_multiclass_learning_problems_,title_worst_case_analysis_temporal_difference_learing_algorithms_)).
neg(sametitle(title_using_output_codes_to_boost_multiclass_learning_problems_,title_private_communication_)).
neg(sametitle(title_using_output_codes_to_boost_multiclass_learning_problems_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_using_output_codes_to_boost_multiclass_learning_problems_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_)).
neg(sametitle(title_using_output_codes_to_boost_multiclass_learning_problems_,title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_using_output_codes_to_boost_multiclass_learning_problems_,title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_using_output_codes_to_boost_multiclass_learning_problems_,title_boosting_margin_)).
neg(sametitle(title_using_output_codes_to_boost_multiclass_learning_problems_,title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_using_output_codes_to_boost_multiclass_learning_problems_,title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_)).
neg(sametitle(title_using_output_codes_to_boost_multiclass_learning_problems_,title_query_by_committee_)).
sametitle(title_using_output_codes_to_boost_multiclass_learning_problems_,title_using_output_codes_to_boost_multiclass_learning_problems_).
neg(sametitle(title_using_output_codes_to_boost_multiclass_learning_problems_,title_improved_boosting_algorithms_using_confidence_rated_predictions_)).
neg(sametitle(title_using_output_codes_to_boost_multiclass_learning_problems_,title_a_system_multiclass_multi_label_text_categorization_)).
neg(sametitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_,title_worst_case_analysis_temporal_difference_learning_algorithms_)).
neg(sametitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_,title_worst_case_analysis_temporal_difference_learing_algorithms_)).
neg(sametitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_,title_private_communication_)).
neg(sametitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_,title_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_)).
neg(sametitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_,title_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_,title_w_s_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_,title_boosting_margin_)).
neg(sametitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_,title_bartlett_w_s_lee_1997_boosting_margin_a_new_explanation_effectiveness_voting_methods_)).
neg(sametitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_,title_w_s_leee_boosting_margin_a_new_explanation_effectiveness_voting_methods_machines_that_learn_snowbird_)).
neg(sametitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_,title_query_by_committee_)).
neg(sametitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_,title_using_output_codes_to_boost_multiclass_learning_problems_)).
sametitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_,title_improved_boosting_algorithms_using_confidence_rated_predictions_).
neg(sametitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_,title_learning_probabilistic_read_once_formulas_product_distributions_)).
neg(sametitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_,title_design_analysis_ef_ficient_learning_algorithms_)).
neg(sametitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_,title_private_correspondence_)).
neg(sametitle(title_improved_boosting_algorithms_using_confidence_rated_predictions_,title_a_system_multiclass_multi_label_text_categorization_)).
neg(sametitle(title_a_system_multiclass_multi_label_text_categorization_,title_using_output_codes_to_boost_multiclass_learning_problems_)).
neg(sametitle(title_a_system_multiclass_multi_label_text_categorization_,title_improved_boosting_algorithms_using_confidence_rated_predictions_)).
sametitle(title_a_system_multiclass_multi_label_text_categorization_,title_a_system_multiclass_multi_label_text_categorization_).

samevenue(venue_proceedings_26th_symposium_theory_computing_,venue_proceedings_26th_symposium_theory_computing_).
samevenue(venue_proceedings_26th_symposium_theory_computing_,venue_proceedings_twenty_sixth_symposium_theory_computing_).
samevenue(venue_proceedings_26th_symposium_theory_computing_,venue_proc_26th_sympos_theory_comput_).
samevenue(venue_proceedings_26th_symposium_theory_computing_,venue_t_t_he_26_th_symposium_heory_computing_).
samevenue(venue_proceedings_26th_symposium_theory_computing_,venue_proceedings_26_th_symposium_theory_computing_).
samevenue(venue_proceedings_26th_symposium_theory_computing_,venue_proc_26th_symp_theory_computing_).
neg(samevenue(venue_proceedings_26th_symposium_theory_computing_,venue_symposium_theory_computing_)).
samevenue(venue_proceedings_twenty_sixth_symposium_theory_computing_,venue_proceedings_26th_symposium_theory_computing_).
samevenue(venue_proceedings_twenty_sixth_symposium_theory_computing_,venue_proceedings_twenty_sixth_symposium_theory_computing_).
samevenue(venue_proceedings_twenty_sixth_symposium_theory_computing_,venue_proc_26th_sympos_theory_comput_).
samevenue(venue_proceedings_twenty_sixth_symposium_theory_computing_,venue_t_t_he_26_th_symposium_heory_computing_).
samevenue(venue_proceedings_twenty_sixth_symposium_theory_computing_,venue_proceedings_26_th_symposium_theory_computing_).
neg(samevenue(venue_proceedings_twenty_sixth_symposium_theory_computing_,venue_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_twenty_sixth_symposium_theory_computing_,venue_proceedings_twenty_eighth_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_twenty_sixth_symposium_theory_computing_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
samevenue(venue_proc_26th_sympos_theory_comput_,venue_proceedings_26th_symposium_theory_computing_).
samevenue(venue_proc_26th_sympos_theory_comput_,venue_proceedings_twenty_sixth_symposium_theory_computing_).
samevenue(venue_proc_26th_sympos_theory_comput_,venue_proc_26th_sympos_theory_comput_).
samevenue(venue_proc_26th_sympos_theory_comput_,venue_t_t_he_26_th_symposium_heory_computing_).
samevenue(venue_proc_26th_sympos_theory_comput_,venue_proceedings_26_th_symposium_theory_computing_).
samevenue(venue_proc_26th_sympos_theory_comput_,venue_proc_26th_symp_theory_computing_).
neg(samevenue(venue_proc_26th_sympos_theory_comput_,venue_symposium_theory_computing_)).
samevenue(venue_t_t_he_26_th_symposium_heory_computing_,venue_proceedings_26th_symposium_theory_computing_).
samevenue(venue_t_t_he_26_th_symposium_heory_computing_,venue_proceedings_twenty_sixth_symposium_theory_computing_).
samevenue(venue_t_t_he_26_th_symposium_heory_computing_,venue_proc_26th_sympos_theory_comput_).
samevenue(venue_t_t_he_26_th_symposium_heory_computing_,venue_t_t_he_26_th_symposium_heory_computing_).
samevenue(venue_t_t_he_26_th_symposium_heory_computing_,venue_proceedings_26_th_symposium_theory_computing_).
samevenue(venue_proceedings_26_th_symposium_theory_computing_,venue_proceedings_26th_symposium_theory_computing_).
samevenue(venue_proceedings_26_th_symposium_theory_computing_,venue_proceedings_twenty_sixth_symposium_theory_computing_).
samevenue(venue_proceedings_26_th_symposium_theory_computing_,venue_proc_26th_sympos_theory_comput_).
samevenue(venue_proceedings_26_th_symposium_theory_computing_,venue_t_t_he_26_th_symposium_heory_computing_).
samevenue(venue_proceedings_26_th_symposium_theory_computing_,venue_proceedings_26_th_symposium_theory_computing_).
neg(samevenue(venue_proceedings_26_th_symposium_theory_computing_,venue_proceedings_36_th_symposium_foundations_computer_science_)).
samevenue(venue_advances_neural_information_processing_systems_10_,venue_advances_neural_information_processing_systems_10_).
neg(samevenue(venue_advances_neural_information_processing_systems_10_,venue_advances_neural_information_processing_systems_)).
neg(samevenue(venue_advances_neural_information_processing_systems_10_,venue_advances_neural_information_processing_systems_vol_4_)).
neg(samevenue(venue_advances_neural_information_processing_systems_10_,venue_advances_neural_information_processing_systems_volume_5_)).
neg(samevenue(venue_advances_neural_information_processing_systems_10_,venue_advances_neural_information_processing_systems_5_)).
neg(samevenue(venue_advances_neural_information_processing_systems_10_,venue_advances_neural_informations_processing_systems_5_)).
neg(samevenue(venue_advances_neural_information_processing_systems_10_,venue_neural_information_processing_systems_)).
neg(samevenue(venue_advances_neural_information_processing_systems_,venue_advances_neural_information_processing_systems_10_)).
samevenue(venue_advances_neural_information_processing_systems_,venue_advances_neural_information_processing_systems_).
neg(samevenue(venue_advances_neural_information_processing_systems_,venue_advances_neural_information_processing_systems_vol_4_)).
neg(samevenue(venue_advances_neural_information_processing_systems_,venue_advances_neural_information_processing_systems_volume_5_)).
neg(samevenue(venue_advances_neural_information_processing_systems_,venue_advances_neural_information_processing_systems_5_)).
neg(samevenue(venue_advances_neural_information_processing_systems_,venue_advances_neural_informations_processing_systems_5_)).
neg(samevenue(venue_advances_neural_information_processing_systems_,venue_proceedings_fifth_conf_neural_informations_processing_systems_)).
samevenue(venue_advances_neural_information_processing_systems_,venue_neural_information_processing_systems_).
neg(samevenue(venue_advances_neural_information_processing_systems_vol_4_,venue_advances_neural_information_processing_systems_10_)).
neg(samevenue(venue_advances_neural_information_processing_systems_vol_4_,venue_advances_neural_information_processing_systems_)).
samevenue(venue_advances_neural_information_processing_systems_vol_4_,venue_advances_neural_information_processing_systems_vol_4_).
neg(samevenue(venue_advances_neural_information_processing_systems_vol_4_,venue_advances_neural_information_processing_systems_volume_5_)).
neg(samevenue(venue_advances_neural_information_processing_systems_vol_4_,venue_advances_neural_information_processing_systems_5_)).
neg(samevenue(venue_advances_neural_information_processing_systems_vol_4_,venue_advances_neural_informations_processing_systems_5_)).
samevenue(venue_advances_neural_information_processing_systems_vol_4_,venue_proceedings_fifth_conf_neural_informations_processing_systems_).
neg(samevenue(venue_advances_neural_information_processing_systems_vol_4_,venue_neural_information_processing_systems_)).
neg(samevenue(venue_advances_neural_information_processing_systems_volume_5_,venue_advances_neural_information_processing_systems_10_)).
neg(samevenue(venue_advances_neural_information_processing_systems_volume_5_,venue_advances_neural_information_processing_systems_)).
neg(samevenue(venue_advances_neural_information_processing_systems_volume_5_,venue_advances_neural_information_processing_systems_vol_4_)).
samevenue(venue_advances_neural_information_processing_systems_volume_5_,venue_advances_neural_information_processing_systems_volume_5_).
samevenue(venue_advances_neural_information_processing_systems_volume_5_,venue_advances_neural_information_processing_systems_5_).
samevenue(venue_advances_neural_information_processing_systems_volume_5_,venue_advances_neural_informations_processing_systems_5_).
neg(samevenue(venue_advances_neural_information_processing_systems_volume_5_,venue_proceedings_fifth_conf_neural_informations_processing_systems_)).
neg(samevenue(venue_advances_neural_information_processing_systems_volume_5_,venue_neural_information_processing_systems_)).
neg(samevenue(venue_advances_neural_information_processing_systems_5_,venue_advances_neural_information_processing_systems_10_)).
neg(samevenue(venue_advances_neural_information_processing_systems_5_,venue_advances_neural_information_processing_systems_)).
neg(samevenue(venue_advances_neural_information_processing_systems_5_,venue_advances_neural_information_processing_systems_vol_4_)).
samevenue(venue_advances_neural_information_processing_systems_5_,venue_advances_neural_information_processing_systems_volume_5_).
samevenue(venue_advances_neural_information_processing_systems_5_,venue_advances_neural_information_processing_systems_5_).
samevenue(venue_advances_neural_information_processing_systems_5_,venue_advances_neural_informations_processing_systems_5_).
neg(samevenue(venue_advances_neural_information_processing_systems_5_,venue_proceedings_fifth_conf_neural_informations_processing_systems_)).
neg(samevenue(venue_advances_neural_information_processing_systems_5_,venue_neural_information_processing_systems_)).
neg(samevenue(venue_advances_neural_informations_processing_systems_5_,venue_advances_neural_information_processing_systems_10_)).
neg(samevenue(venue_advances_neural_informations_processing_systems_5_,venue_advances_neural_information_processing_systems_)).
neg(samevenue(venue_advances_neural_informations_processing_systems_5_,venue_advances_neural_information_processing_systems_vol_4_)).
samevenue(venue_advances_neural_informations_processing_systems_5_,venue_advances_neural_information_processing_systems_volume_5_).
samevenue(venue_advances_neural_informations_processing_systems_5_,venue_advances_neural_information_processing_systems_5_).
samevenue(venue_advances_neural_informations_processing_systems_5_,venue_advances_neural_informations_processing_systems_5_).
neg(samevenue(venue_advances_neural_informations_processing_systems_5_,venue_proceedings_fifth_conf_neural_informations_processing_systems_)).
neg(samevenue(venue_advances_neural_informations_processing_systems_5_,venue_neural_information_processing_systems_)).
neg(samevenue(venue_proceedings_fifth_conf_neural_informations_processing_systems_,venue_advances_neural_information_processing_systems_)).
samevenue(venue_proceedings_fifth_conf_neural_informations_processing_systems_,venue_advances_neural_information_processing_systems_vol_4_).
neg(samevenue(venue_proceedings_fifth_conf_neural_informations_processing_systems_,venue_advances_neural_information_processing_systems_volume_5_)).
neg(samevenue(venue_proceedings_fifth_conf_neural_informations_processing_systems_,venue_advances_neural_information_processing_systems_5_)).
neg(samevenue(venue_proceedings_fifth_conf_neural_informations_processing_systems_,venue_advances_neural_informations_processing_systems_5_)).
samevenue(venue_proceedings_fifth_conf_neural_informations_processing_systems_,venue_proceedings_fifth_conf_neural_informations_processing_systems_).
neg(samevenue(venue_proceedings_fifth_conf_neural_informations_processing_systems_,venue_neural_information_processing_systems_)).
neg(samevenue(venue_proceedings_fifth_conf_neural_informations_processing_systems_,venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_)).
neg(samevenue(venue_advances_neural_information_processing_systems_,venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_)).
neg(samevenue(venue_neural_information_processing_systems_,venue_advances_neural_information_processing_systems_10_)).
samevenue(venue_neural_information_processing_systems_,venue_advances_neural_information_processing_systems_).
neg(samevenue(venue_neural_information_processing_systems_,venue_advances_neural_information_processing_systems_vol_4_)).
neg(samevenue(venue_neural_information_processing_systems_,venue_advances_neural_information_processing_systems_volume_5_)).
neg(samevenue(venue_neural_information_processing_systems_,venue_advances_neural_information_processing_systems_5_)).
neg(samevenue(venue_neural_information_processing_systems_,venue_advances_neural_informations_processing_systems_5_)).
neg(samevenue(venue_neural_information_processing_systems_,venue_proceedings_fifth_conf_neural_informations_processing_systems_)).
samevenue(venue_neural_information_processing_systems_,venue_neural_information_processing_systems_).
neg(samevenue(venue_neural_information_processing_systems_,venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_)).
samevenue(venue_technical_report_ucsc_crl_87_26_,venue_technical_report_ucsc_crl_87_26_).
neg(samevenue(venue_technical_report_ucsc_crl_87_26_,venue_technical_report_number_ucsc_crl_88_06_)).
neg(samevenue(venue_technical_report_ucsc_crl_87_26_,venue_technical_report_)).
samevenue(venue_phd_thesis_,venue_phd_thesis_).
neg(samevenue(venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_,venue_proceedings_fifth_conf_neural_informations_processing_systems_)).
neg(samevenue(venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_,venue_advances_neural_information_processing_systems_)).
neg(samevenue(venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_,venue_neural_information_processing_systems_)).
samevenue(venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_,venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_).
neg(samevenue(venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_,venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_,venue_25th_symposium_theory_computing_to_appear_)).
neg(samevenue(venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_36_th_symposium_foundations_computer_science_,venue_proceedings_26_th_symposium_theory_computing_)).
samevenue(venue_proceedings_36_th_symposium_foundations_computer_science_,venue_proceedings_36_th_symposium_foundations_computer_science_).
samevenue(venue_proceedings_36_th_symposium_foundations_computer_science_,venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_).
neg(samevenue(venue_proceedings_36_th_symposium_foundations_computer_science_,venue_proceedings_thirty_seventh_symposium_foundations_computer_science_)).
samevenue(venue_proceedings_36_th_symposium_foundations_computer_science_,venue_proceedings_thirty_sixth_symposium_foundations_computer_science_).
samevenue(venue_proceedings_36_th_symposium_foundations_computer_science_,venue_focs_36_).
neg(samevenue(venue_proceedings_36_th_symposium_foundations_computer_science_,venue_ieee_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_36_th_symposium_foundations_computer_science_,venue_proceedings_30th_symposium_foundation_computer_science_)).
neg(samevenue(venue_proceedings_36_th_symposium_foundations_computer_science_,venue_30th_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_36_th_symposium_foundations_computer_science_,venue_proceedings_thirtieth_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_36_th_symposium_foundations_computer_science_,venue_proceedings_30th_symposium_foundations_computer_science_)).
neg(samevenue(venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_)).
samevenue(venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_proceedings_36_th_symposium_foundations_computer_science_).
samevenue(venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_).
neg(samevenue(venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_proceedings_thirty_seventh_symposium_foundations_computer_science_)).
samevenue(venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_proceedings_thirty_sixth_symposium_foundations_computer_science_).
samevenue(venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_focs_36_).
neg(samevenue(venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_ieee_symposium_foundations_computer_science_)).
neg(samevenue(venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_proceedings_30th_symposium_foundation_computer_science_)).
neg(samevenue(venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_proceedings_30th_symposium_foundations_computer_science_)).
neg(samevenue(venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_25th_symposium_theory_computing_to_appear_)).
neg(samevenue(venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_thirty_seventh_symposium_foundations_computer_science_,venue_proceedings_36_th_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_thirty_seventh_symposium_foundations_computer_science_,venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
samevenue(venue_proceedings_thirty_seventh_symposium_foundations_computer_science_,venue_proceedings_thirty_seventh_symposium_foundations_computer_science_).
neg(samevenue(venue_proceedings_thirty_seventh_symposium_foundations_computer_science_,venue_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_thirty_seventh_symposium_foundations_computer_science_,venue_focs_36_)).
neg(samevenue(venue_proceedings_thirty_seventh_symposium_foundations_computer_science_,venue_ieee_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_thirty_seventh_symposium_foundations_computer_science_,venue_proceedings_30th_symposium_foundation_computer_science_)).
neg(samevenue(venue_proceedings_thirty_seventh_symposium_foundations_computer_science_,venue_30th_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_thirty_seventh_symposium_foundations_computer_science_,venue_proceedings_thirtieth_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_thirty_seventh_symposium_foundations_computer_science_,venue_proceedings_30th_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_thirty_seventh_symposium_foundations_computer_science_,venue_25th_symposium_theory_computing_to_appear_)).
neg(samevenue(venue_proceedings_thirty_seventh_symposium_foundations_computer_science_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_thirty_seventh_symposium_foundations_computer_science_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_proceedings_twenty_sixth_symposium_theory_computing_)).
samevenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_proceedings_36_th_symposium_foundations_computer_science_).
samevenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_).
neg(samevenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_proceedings_thirty_seventh_symposium_foundations_computer_science_)).
samevenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_proceedings_thirty_sixth_symposium_foundations_computer_science_).
samevenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_focs_36_).
neg(samevenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_ieee_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_proceedings_30th_symposium_foundation_computer_science_)).
neg(samevenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_30th_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_proceedings_thirtieth_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_proceedings_30th_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_25th_symposium_theory_computing_to_appear_)).
neg(samevenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_proceedings_twenty_eighth_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_thirty_sixth_symposium_foundations_computer_science_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
samevenue(venue_focs_36_,venue_proceedings_36_th_symposium_foundations_computer_science_).
samevenue(venue_focs_36_,venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_).
neg(samevenue(venue_focs_36_,venue_proceedings_thirty_seventh_symposium_foundations_computer_science_)).
samevenue(venue_focs_36_,venue_proceedings_thirty_sixth_symposium_foundations_computer_science_).
samevenue(venue_focs_36_,venue_focs_36_).
neg(samevenue(venue_focs_36_,venue_ieee_symposium_foundations_computer_science_)).
neg(samevenue(venue_focs_36_,venue_proceedings_30th_symposium_foundation_computer_science_)).
neg(samevenue(venue_focs_36_,venue_30th_symposium_foundations_computer_science_)).
neg(samevenue(venue_focs_36_,venue_proceedings_thirtieth_symposium_foundations_computer_science_)).
neg(samevenue(venue_focs_36_,venue_proceedings_30th_symposium_foundations_computer_science_)).
neg(samevenue(venue_ieee_symposium_foundations_computer_science_,venue_proceedings_36_th_symposium_foundations_computer_science_)).
neg(samevenue(venue_ieee_symposium_foundations_computer_science_,venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
neg(samevenue(venue_ieee_symposium_foundations_computer_science_,venue_proceedings_thirty_seventh_symposium_foundations_computer_science_)).
neg(samevenue(venue_ieee_symposium_foundations_computer_science_,venue_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
neg(samevenue(venue_ieee_symposium_foundations_computer_science_,venue_focs_36_)).
samevenue(venue_ieee_symposium_foundations_computer_science_,venue_ieee_symposium_foundations_computer_science_).
neg(samevenue(venue_ieee_symposium_foundations_computer_science_,venue_proceedings_30th_symposium_foundation_computer_science_)).
neg(samevenue(venue_ieee_symposium_foundations_computer_science_,venue_30th_symposium_foundations_computer_science_)).
neg(samevenue(venue_ieee_symposium_foundations_computer_science_,venue_proceedings_thirtieth_symposium_foundations_computer_science_)).
neg(samevenue(venue_ieee_symposium_foundations_computer_science_,venue_proceedings_30th_symposium_foundations_computer_science_)).
neg(samevenue(venue_ieee_symposium_foundations_computer_science_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
neg(samevenue(venue_ieee_symposium_foundations_computer_science_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
samevenue(venue_proceedings_9th_conference_computational_learning_theory_,venue_proceedings_9th_conference_computational_learning_theory_).
samevenue(venue_proceedings_9th_conference_computational_learning_theory_,venue_proc_9th_conference_computational_learning_theory_).
samevenue(venue_proceedings_9th_conference_computational_learning_theory_,venue_proceedings_ninth_conference_computational_learning_theory_).
samevenue(venue_proc_9th_conference_computational_learning_theory_,venue_proceedings_9th_conference_computational_learning_theory_).
samevenue(venue_proc_9th_conference_computational_learning_theory_,venue_proc_9th_conference_computational_learning_theory_).
samevenue(venue_proc_9th_conference_computational_learning_theory_,venue_proceedings_ninth_conference_computational_learning_theory_).
neg(samevenue(venue_proc_9th_conference_computational_learning_theory_,venue_proc_workshop_computational_learning_theory_)).
samevenue(venue_proceedings_ninth_conference_computational_learning_theory_,venue_proceedings_9th_conference_computational_learning_theory_).
samevenue(venue_proceedings_ninth_conference_computational_learning_theory_,venue_proc_9th_conference_computational_learning_theory_).
samevenue(venue_proceedings_ninth_conference_computational_learning_theory_,venue_proceedings_ninth_conference_computational_learning_theory_).
neg(samevenue(venue_proceedings_30th_symposium_foundation_computer_science_,venue_proceedings_36_th_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_30th_symposium_foundation_computer_science_,venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_30th_symposium_foundation_computer_science_,venue_proceedings_thirty_seventh_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_30th_symposium_foundation_computer_science_,venue_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_30th_symposium_foundation_computer_science_,venue_focs_36_)).
neg(samevenue(venue_proceedings_30th_symposium_foundation_computer_science_,venue_ieee_symposium_foundations_computer_science_)).
samevenue(venue_proceedings_30th_symposium_foundation_computer_science_,venue_proceedings_30th_symposium_foundation_computer_science_).
samevenue(venue_proceedings_30th_symposium_foundation_computer_science_,venue_30th_symposium_foundations_computer_science_).
samevenue(venue_proceedings_30th_symposium_foundation_computer_science_,venue_proceedings_thirtieth_symposium_foundations_computer_science_).
samevenue(venue_proceedings_30th_symposium_foundation_computer_science_,venue_proceedings_30th_symposium_foundations_computer_science_).
neg(samevenue(venue_proceedings_30th_symposium_foundation_computer_science_,venue_technical_report_mit_lcs_tm_413_)).
neg(samevenue(venue_proceedings_30th_symposium_foundation_computer_science_,venue_25th_symposium_theory_computing_to_appear_)).
neg(samevenue(venue_proceedings_30th_symposium_foundation_computer_science_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_30th_symposium_foundation_computer_science_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_30th_symposium_foundation_computer_science_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
neg(samevenue(venue_30th_symposium_foundations_computer_science_,venue_proceedings_36_th_symposium_foundations_computer_science_)).
neg(samevenue(venue_30th_symposium_foundations_computer_science_,venue_proceedings_thirty_seventh_symposium_foundations_computer_science_)).
neg(samevenue(venue_30th_symposium_foundations_computer_science_,venue_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
neg(samevenue(venue_30th_symposium_foundations_computer_science_,venue_focs_36_)).
neg(samevenue(venue_30th_symposium_foundations_computer_science_,venue_ieee_symposium_foundations_computer_science_)).
samevenue(venue_30th_symposium_foundations_computer_science_,venue_proceedings_30th_symposium_foundation_computer_science_).
samevenue(venue_30th_symposium_foundations_computer_science_,venue_30th_symposium_foundations_computer_science_).
samevenue(venue_30th_symposium_foundations_computer_science_,venue_proceedings_thirtieth_symposium_foundations_computer_science_).
samevenue(venue_30th_symposium_foundations_computer_science_,venue_proceedings_30th_symposium_foundations_computer_science_).
neg(samevenue(venue_30th_symposium_foundations_computer_science_,venue_technical_report_mit_lcs_tm_413_)).
neg(samevenue(venue_30th_symposium_foundations_computer_science_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
neg(samevenue(venue_30th_symposium_foundations_computer_science_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_thirtieth_symposium_foundations_computer_science_,venue_proceedings_36_th_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_thirtieth_symposium_foundations_computer_science_,venue_proceedings_thirty_seventh_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_thirtieth_symposium_foundations_computer_science_,venue_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_thirtieth_symposium_foundations_computer_science_,venue_focs_36_)).
neg(samevenue(venue_proceedings_thirtieth_symposium_foundations_computer_science_,venue_ieee_symposium_foundations_computer_science_)).
samevenue(venue_proceedings_thirtieth_symposium_foundations_computer_science_,venue_proceedings_30th_symposium_foundation_computer_science_).
samevenue(venue_proceedings_thirtieth_symposium_foundations_computer_science_,venue_30th_symposium_foundations_computer_science_).
samevenue(venue_proceedings_thirtieth_symposium_foundations_computer_science_,venue_proceedings_thirtieth_symposium_foundations_computer_science_).
samevenue(venue_proceedings_thirtieth_symposium_foundations_computer_science_,venue_proceedings_30th_symposium_foundations_computer_science_).
neg(samevenue(venue_proceedings_thirtieth_symposium_foundations_computer_science_,venue_technical_report_mit_lcs_tm_413_)).
neg(samevenue(venue_proceedings_thirtieth_symposium_foundations_computer_science_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_thirtieth_symposium_foundations_computer_science_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_30th_symposium_foundations_computer_science_,venue_proceedings_36_th_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_30th_symposium_foundations_computer_science_,venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_30th_symposium_foundations_computer_science_,venue_proceedings_thirty_seventh_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_30th_symposium_foundations_computer_science_,venue_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_30th_symposium_foundations_computer_science_,venue_focs_36_)).
neg(samevenue(venue_proceedings_30th_symposium_foundations_computer_science_,venue_ieee_symposium_foundations_computer_science_)).
samevenue(venue_proceedings_30th_symposium_foundations_computer_science_,venue_proceedings_30th_symposium_foundation_computer_science_).
samevenue(venue_proceedings_30th_symposium_foundations_computer_science_,venue_30th_symposium_foundations_computer_science_).
samevenue(venue_proceedings_30th_symposium_foundations_computer_science_,venue_proceedings_thirtieth_symposium_foundations_computer_science_).
samevenue(venue_proceedings_30th_symposium_foundations_computer_science_,venue_proceedings_30th_symposium_foundations_computer_science_).
neg(samevenue(venue_proceedings_30th_symposium_foundations_computer_science_,venue_technical_report_mit_lcs_tm_413_)).
neg(samevenue(venue_proceedings_30th_symposium_foundations_computer_science_,venue_25th_symposium_theory_computing_to_appear_)).
neg(samevenue(venue_proceedings_30th_symposium_foundations_computer_science_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_30th_symposium_foundations_computer_science_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_30th_symposium_foundations_computer_science_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
neg(samevenue(venue_technical_report_mit_lcs_tm_413_,venue_proceedings_30th_symposium_foundation_computer_science_)).
neg(samevenue(venue_technical_report_mit_lcs_tm_413_,venue_30th_symposium_foundations_computer_science_)).
neg(samevenue(venue_technical_report_mit_lcs_tm_413_,venue_proceedings_thirtieth_symposium_foundations_computer_science_)).
neg(samevenue(venue_technical_report_mit_lcs_tm_413_,venue_proceedings_30th_symposium_foundations_computer_science_)).
samevenue(venue_technical_report_mit_lcs_tm_413_,venue_technical_report_mit_lcs_tm_413_).
neg(samevenue(venue_technical_report_mit_lcs_tm_413_,venue_technical_report_)).
neg(samevenue(venue_technical_report_mit_lcs_tm_413_,venue_technical_report_mit_lcs_tr_493_)).
neg(samevenue(venue_technical_report_number_ucsc_crl_88_06_,venue_technical_report_ucsc_crl_87_26_)).
samevenue(venue_technical_report_number_ucsc_crl_88_06_,venue_technical_report_number_ucsc_crl_88_06_).
neg(samevenue(venue_technical_report_number_ucsc_crl_88_06_,venue_technical_report_)).
samevenue(venue_proceedings_fourth_conference_computational_learning_theory_colt_91_,venue_proceedings_fourth_conference_computational_learning_theory_colt_91_).
samevenue(venue_proceedings_fourth_conference_computational_learning_theory_colt_91_,venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_).
samevenue(venue_proceedings_fourth_conference_computational_learning_theory_colt_91_,venue_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_proceedings_fourth_conference_computational_learning_theory_colt_91_,venue_colt_91_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_proceedings_fourth_conference_computational_learning_theory_colt_91_,venue_computational_learning_theory_proceedings_fourth_workshop_).
samevenue(venue_proceedings_fourth_conference_computational_learning_theory_colt_91_,venue_proc_4th_workshop_comput_learning_theory_).
samevenue(venue_proceedings_fourth_conference_computational_learning_theory_colt_91_,venue_proc_4th_workshop_computational_learning_theory_).
samevenue(venue_proceedings_fourth_conference_computational_learning_theory_colt_91_,venue_proceedings_fourth_workshop_computational_learning_theory_).
samevenue(venue_proceedings_fourth_conference_computational_learning_theory_colt_91_,venue_1991_workshop_computational_learning_theory_).
samevenue(venue_proceedings_fourth_conference_computational_learning_theory_colt_91_,venue_proceedings_1991_workshop_computational_learning_theory_).
neg(samevenue(venue_proceedings_fourth_conference_computational_learning_theory_colt_91_,venue_machine_learning_)).
samevenue(venue_proceedings_fourth_conference_computational_learning_theory_colt_91_,venue_computation_learning_theory_proceedings_fourth_workshop_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_,venue_proceedings_fourth_conference_computational_learning_theory_colt_91_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_,venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_,venue_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_,venue_colt_91_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_,venue_computational_learning_theory_proceedings_fourth_workshop_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_,venue_proc_4th_workshop_comput_learning_theory_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_,venue_proc_4th_workshop_computational_learning_theory_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_,venue_proceedings_fourth_workshop_computational_learning_theory_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_,venue_1991_workshop_computational_learning_theory_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_,venue_proceedings_1991_workshop_computational_learning_theory_).
neg(samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_,venue_machine_learning_)).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_,venue_computation_learning_theory_proceedings_fourth_workshop_).
samevenue(venue_proceedings_4th_workshop_computational_learning_theory_,venue_proceedings_fourth_conference_computational_learning_theory_colt_91_).
samevenue(venue_proceedings_4th_workshop_computational_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_).
samevenue(venue_proceedings_4th_workshop_computational_learning_theory_,venue_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_proceedings_4th_workshop_computational_learning_theory_,venue_colt_91_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_proceedings_4th_workshop_computational_learning_theory_,venue_computational_learning_theory_proceedings_fourth_workshop_).
samevenue(venue_proceedings_4th_workshop_computational_learning_theory_,venue_proc_4th_workshop_comput_learning_theory_).
samevenue(venue_proceedings_4th_workshop_computational_learning_theory_,venue_proc_4th_workshop_computational_learning_theory_).
samevenue(venue_proceedings_4th_workshop_computational_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_).
samevenue(venue_proceedings_4th_workshop_computational_learning_theory_,venue_1991_workshop_computational_learning_theory_).
samevenue(venue_proceedings_4th_workshop_computational_learning_theory_,venue_proceedings_1991_workshop_computational_learning_theory_).
neg(samevenue(venue_proceedings_4th_workshop_computational_learning_theory_,venue_machine_learning_)).
neg(samevenue(venue_proceedings_4th_workshop_computational_learning_theory_,venue_proceedings_eighth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_4th_workshop_computational_learning_theory_,venue_proc_workshop_computational_learning_theory_)).
samevenue(venue_proceedings_4th_workshop_computational_learning_theory_,venue_computation_learning_theory_proceedings_fourth_workshop_).
neg(samevenue(venue_proceedings_4th_workshop_computational_learning_theory_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
samevenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_,venue_proceedings_fourth_conference_computational_learning_theory_colt_91_).
samevenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_).
samevenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_,venue_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_,venue_colt_91_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_,venue_computational_learning_theory_proceedings_fourth_workshop_).
samevenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_,venue_proc_4th_workshop_comput_learning_theory_).
samevenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_,venue_proc_4th_workshop_computational_learning_theory_).
samevenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_).
samevenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_,venue_1991_workshop_computational_learning_theory_).
samevenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_,venue_proceedings_1991_workshop_computational_learning_theory_).
neg(samevenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_,venue_machine_learning_)).
neg(samevenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_,venue_proceedings_eighth_workshop_computational_learning_theory_)).
neg(samevenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_,venue_proc_workshop_computational_learning_theory_)).
samevenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_,venue_computation_learning_theory_proceedings_fourth_workshop_).
neg(samevenue(venue_colt_91_proceedings_4th_workshop_computational_learning_theory_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
samevenue(venue_computational_learning_theory_proceedings_fourth_workshop_,venue_proceedings_fourth_conference_computational_learning_theory_colt_91_).
samevenue(venue_computational_learning_theory_proceedings_fourth_workshop_,venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_).
samevenue(venue_computational_learning_theory_proceedings_fourth_workshop_,venue_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_computational_learning_theory_proceedings_fourth_workshop_,venue_colt_91_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_computational_learning_theory_proceedings_fourth_workshop_,venue_computational_learning_theory_proceedings_fourth_workshop_).
samevenue(venue_computational_learning_theory_proceedings_fourth_workshop_,venue_proc_4th_workshop_comput_learning_theory_).
samevenue(venue_computational_learning_theory_proceedings_fourth_workshop_,venue_proc_4th_workshop_computational_learning_theory_).
samevenue(venue_computational_learning_theory_proceedings_fourth_workshop_,venue_proceedings_fourth_workshop_computational_learning_theory_).
samevenue(venue_computational_learning_theory_proceedings_fourth_workshop_,venue_1991_workshop_computational_learning_theory_).
samevenue(venue_computational_learning_theory_proceedings_fourth_workshop_,venue_proceedings_1991_workshop_computational_learning_theory_).
neg(samevenue(venue_computational_learning_theory_proceedings_fourth_workshop_,venue_machine_learning_)).
neg(samevenue(venue_computational_learning_theory_proceedings_fourth_workshop_,venue_proceedings_eighth_workshop_computational_learning_theory_)).
neg(samevenue(venue_computational_learning_theory_proceedings_fourth_workshop_,venue_proc_workshop_computational_learning_theory_)).
samevenue(venue_computational_learning_theory_proceedings_fourth_workshop_,venue_computation_learning_theory_proceedings_fourth_workshop_).
neg(samevenue(venue_computational_learning_theory_proceedings_fourth_workshop_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
samevenue(venue_proc_4th_workshop_comput_learning_theory_,venue_proceedings_fourth_conference_computational_learning_theory_colt_91_).
samevenue(venue_proc_4th_workshop_comput_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_).
samevenue(venue_proc_4th_workshop_comput_learning_theory_,venue_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_proc_4th_workshop_comput_learning_theory_,venue_colt_91_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_proc_4th_workshop_comput_learning_theory_,venue_computational_learning_theory_proceedings_fourth_workshop_).
samevenue(venue_proc_4th_workshop_comput_learning_theory_,venue_proc_4th_workshop_comput_learning_theory_).
samevenue(venue_proc_4th_workshop_comput_learning_theory_,venue_proc_4th_workshop_computational_learning_theory_).
samevenue(venue_proc_4th_workshop_comput_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_).
samevenue(venue_proc_4th_workshop_comput_learning_theory_,venue_1991_workshop_computational_learning_theory_).
samevenue(venue_proc_4th_workshop_comput_learning_theory_,venue_proceedings_1991_workshop_computational_learning_theory_).
neg(samevenue(venue_proc_4th_workshop_comput_learning_theory_,venue_machine_learning_)).
neg(samevenue(venue_proc_4th_workshop_comput_learning_theory_,venue_proceedings_eighth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proc_4th_workshop_comput_learning_theory_,venue_proc_workshop_computational_learning_theory_)).
samevenue(venue_proc_4th_workshop_comput_learning_theory_,venue_computation_learning_theory_proceedings_fourth_workshop_).
neg(samevenue(venue_proc_4th_workshop_comput_learning_theory_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
samevenue(venue_proc_4th_workshop_computational_learning_theory_,venue_proceedings_fourth_conference_computational_learning_theory_colt_91_).
samevenue(venue_proc_4th_workshop_computational_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_).
samevenue(venue_proc_4th_workshop_computational_learning_theory_,venue_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_proc_4th_workshop_computational_learning_theory_,venue_colt_91_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_proc_4th_workshop_computational_learning_theory_,venue_computational_learning_theory_proceedings_fourth_workshop_).
samevenue(venue_proc_4th_workshop_computational_learning_theory_,venue_proc_4th_workshop_comput_learning_theory_).
samevenue(venue_proc_4th_workshop_computational_learning_theory_,venue_proc_4th_workshop_computational_learning_theory_).
samevenue(venue_proc_4th_workshop_computational_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_).
samevenue(venue_proc_4th_workshop_computational_learning_theory_,venue_1991_workshop_computational_learning_theory_).
samevenue(venue_proc_4th_workshop_computational_learning_theory_,venue_proceedings_1991_workshop_computational_learning_theory_).
neg(samevenue(venue_proc_4th_workshop_computational_learning_theory_,venue_machine_learning_)).
neg(samevenue(venue_proc_4th_workshop_computational_learning_theory_,venue_proceedings_eighth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proc_4th_workshop_computational_learning_theory_,venue_proc_workshop_computational_learning_theory_)).
samevenue(venue_proc_4th_workshop_computational_learning_theory_,venue_computation_learning_theory_proceedings_fourth_workshop_).
neg(samevenue(venue_proc_4th_workshop_computational_learning_theory_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_proceedings_fourth_conference_computational_learning_theory_colt_91_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_colt_91_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_computational_learning_theory_proceedings_fourth_workshop_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_proc_4th_workshop_comput_learning_theory_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_proc_4th_workshop_computational_learning_theory_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_1991_workshop_computational_learning_theory_).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_proceedings_1991_workshop_computational_learning_theory_).
neg(samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_machine_learning_)).
neg(samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_proceedings_eighth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_proc_workshop_computational_learning_theory_)).
samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_computation_learning_theory_proceedings_fourth_workshop_).
neg(samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
samevenue(venue_1991_workshop_computational_learning_theory_,venue_proceedings_fourth_conference_computational_learning_theory_colt_91_).
samevenue(venue_1991_workshop_computational_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_).
samevenue(venue_1991_workshop_computational_learning_theory_,venue_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_1991_workshop_computational_learning_theory_,venue_colt_91_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_1991_workshop_computational_learning_theory_,venue_computational_learning_theory_proceedings_fourth_workshop_).
samevenue(venue_1991_workshop_computational_learning_theory_,venue_proc_4th_workshop_comput_learning_theory_).
samevenue(venue_1991_workshop_computational_learning_theory_,venue_proc_4th_workshop_computational_learning_theory_).
samevenue(venue_1991_workshop_computational_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_).
samevenue(venue_1991_workshop_computational_learning_theory_,venue_1991_workshop_computational_learning_theory_).
samevenue(venue_1991_workshop_computational_learning_theory_,venue_proceedings_1991_workshop_computational_learning_theory_).
neg(samevenue(venue_1991_workshop_computational_learning_theory_,venue_machine_learning_)).
samevenue(venue_proceedings_1991_workshop_computational_learning_theory_,venue_proceedings_fourth_conference_computational_learning_theory_colt_91_).
samevenue(venue_proceedings_1991_workshop_computational_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_).
samevenue(venue_proceedings_1991_workshop_computational_learning_theory_,venue_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_proceedings_1991_workshop_computational_learning_theory_,venue_colt_91_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_proceedings_1991_workshop_computational_learning_theory_,venue_computational_learning_theory_proceedings_fourth_workshop_).
samevenue(venue_proceedings_1991_workshop_computational_learning_theory_,venue_proc_4th_workshop_comput_learning_theory_).
samevenue(venue_proceedings_1991_workshop_computational_learning_theory_,venue_proc_4th_workshop_computational_learning_theory_).
samevenue(venue_proceedings_1991_workshop_computational_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_).
samevenue(venue_proceedings_1991_workshop_computational_learning_theory_,venue_1991_workshop_computational_learning_theory_).
samevenue(venue_proceedings_1991_workshop_computational_learning_theory_,venue_proceedings_1991_workshop_computational_learning_theory_).
neg(samevenue(venue_proceedings_1991_workshop_computational_learning_theory_,venue_machine_learning_)).
neg(samevenue(venue_proceedings_1991_workshop_computational_learning_theory_,venue_proceedings_eighth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_1991_workshop_computational_learning_theory_,venue_proc_workshop_computational_learning_theory_)).
samevenue(venue_proceedings_1991_workshop_computational_learning_theory_,venue_computation_learning_theory_proceedings_fourth_workshop_).
neg(samevenue(venue_proceedings_1991_workshop_computational_learning_theory_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
neg(samevenue(venue_machine_learning_,venue_proceedings_fourth_conference_computational_learning_theory_colt_91_)).
neg(samevenue(venue_machine_learning_,venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_)).
neg(samevenue(venue_machine_learning_,venue_proceedings_4th_workshop_computational_learning_theory_)).
neg(samevenue(venue_machine_learning_,venue_colt_91_proceedings_4th_workshop_computational_learning_theory_)).
neg(samevenue(venue_machine_learning_,venue_computational_learning_theory_proceedings_fourth_workshop_)).
neg(samevenue(venue_machine_learning_,venue_proc_4th_workshop_comput_learning_theory_)).
neg(samevenue(venue_machine_learning_,venue_proc_4th_workshop_computational_learning_theory_)).
neg(samevenue(venue_machine_learning_,venue_proceedings_fourth_workshop_computational_learning_theory_)).
neg(samevenue(venue_machine_learning_,venue_1991_workshop_computational_learning_theory_)).
neg(samevenue(venue_machine_learning_,venue_proceedings_1991_workshop_computational_learning_theory_)).
samevenue(venue_machine_learning_,venue_machine_learning_).
neg(samevenue(venue_machine_learning_,venue_machine_learning_proceedings_eleventh_international_conference_)).
neg(samevenue(venue_machine_learning_,venue_proceedings_eleventh_international_conference_machine_learning_)).
neg(samevenue(venue_machine_learning_,venue_proc_11th_international_conference_machine_learning_)).
neg(samevenue(venue_machine_learning_,venue_proceedings_fourteenth_international_conference_machine_learning_)).
neg(samevenue(venue_machine_learning_,venue_machine_learning_proceedings_fourteenth_international_conference_)).
neg(samevenue(venue_machine_learning_,venue_proceedings_14th_international_conference_machine_learning_)).
neg(samevenue(venue_machine_learning_,venue_proceedings_11th_international_conference_machine_learning_)).
neg(samevenue(venue_technical_report_,venue_technical_report_ucsc_crl_87_26_)).
neg(samevenue(venue_technical_report_,venue_technical_report_mit_lcs_tm_413_)).
neg(samevenue(venue_technical_report_,venue_technical_report_number_ucsc_crl_88_06_)).
samevenue(venue_technical_report_,venue_technical_report_).
neg(samevenue(venue_technical_report_,venue_technical_report_mit_lcs_tr_493_)).
samevenue(venue_proceedings_winter_simulation_conference_,venue_proceedings_winter_simulation_conference_).
samevenue(venue_proceedings_colt_93_,venue_proceedings_colt_93_).
samevenue(venue_25th_symposium_theory_computing_,venue_25th_symposium_theory_computing_).
samevenue(venue_25th_symposium_theory_computing_,venue_25th_symposium_theory_computing_1994_).
neg(samevenue(venue_25th_symposium_theory_computing_,venue_stoc_)).
neg(samevenue(venue_25th_symposium_theory_computing_,venue_stoc_94_)).
neg(samevenue(venue_25th_symposium_theory_computing_,venue_proceedings_26th_symposium_theory_computing_)).
samevenue(venue_25th_symposium_theory_computing_,venue_25th_symp_theory_computing_).
samevenue(venue_25th_symposium_theory_computing_,venue_25th_symposium_theory_computing_to_appear_).
neg(samevenue(venue_25th_symposium_theory_computing_,venue_stoc_p_)).
neg(samevenue(venue_25th_symposium_theory_computing_,venue_symposium_theory_computing_)).
neg(samevenue(venue_25th_symposium_theory_computing_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_25th_symposium_theory_computing_,venue_proceedings_28th_symposium_theory_computing_)).
samevenue(venue_25th_symposium_theory_computing_1994_,venue_25th_symposium_theory_computing_).
samevenue(venue_25th_symposium_theory_computing_1994_,venue_25th_symposium_theory_computing_1994_).
neg(samevenue(venue_25th_symposium_theory_computing_1994_,venue_stoc_)).
neg(samevenue(venue_25th_symposium_theory_computing_1994_,venue_stoc_94_)).
neg(samevenue(venue_25th_symposium_theory_computing_1994_,venue_proceedings_26th_symposium_theory_computing_)).
samevenue(venue_25th_symposium_theory_computing_1994_,venue_25th_symp_theory_computing_).
samevenue(venue_25th_symposium_theory_computing_1994_,venue_25th_symposium_theory_computing_to_appear_).
neg(samevenue(venue_25th_symposium_theory_computing_1994_,venue_stoc_p_)).
neg(samevenue(venue_25th_symposium_theory_computing_1994_,venue_symposium_theory_computing_)).
neg(samevenue(venue_25th_symposium_theory_computing_1994_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_25th_symposium_theory_computing_1994_,venue_proceedings_28th_symposium_theory_computing_)).
samevenue(venue_proc_26th_symp_theory_computing_,venue_proceedings_26th_symposium_theory_computing_).
samevenue(venue_proc_26th_symp_theory_computing_,venue_proc_26th_sympos_theory_comput_).
samevenue(venue_proc_26th_symp_theory_computing_,venue_proc_26th_symp_theory_computing_).
neg(samevenue(venue_proc_26th_symp_theory_computing_,venue_25th_symp_theory_computing_)).
neg(samevenue(venue_proc_26th_symp_theory_computing_,venue_symposium_theory_computing_)).
neg(samevenue(venue_proc_26th_symp_theory_computing_,venue_proc_workshop_computational_learning_theory_)).
neg(samevenue(venue_proc_26th_symp_theory_computing_,venue_proc_21st_symp_theoretical_computing_)).
neg(samevenue(venue_stoc_,venue_25th_symposium_theory_computing_)).
neg(samevenue(venue_stoc_,venue_25th_symposium_theory_computing_1994_)).
samevenue(venue_stoc_,venue_stoc_).
neg(samevenue(venue_stoc_,venue_stoc_94_)).
neg(samevenue(venue_stoc_,venue_proceedings_26th_symposium_theory_computing_)).
neg(samevenue(venue_stoc_,venue_25th_symp_theory_computing_)).
neg(samevenue(venue_stoc_,venue_25th_symposium_theory_computing_to_appear_)).
samevenue(venue_stoc_,venue_stoc_p_).
samevenue(venue_stoc_,venue_symposium_theory_computing_).
neg(samevenue(venue_stoc_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_stoc_,venue_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_stoc_94_,venue_25th_symposium_theory_computing_)).
neg(samevenue(venue_stoc_94_,venue_25th_symposium_theory_computing_1994_)).
neg(samevenue(venue_stoc_94_,venue_stoc_)).
samevenue(venue_stoc_94_,venue_stoc_94_).
samevenue(venue_stoc_94_,venue_proceedings_26th_symposium_theory_computing_).
neg(samevenue(venue_stoc_94_,venue_25th_symp_theory_computing_)).
neg(samevenue(venue_stoc_94_,venue_25th_symposium_theory_computing_to_appear_)).
neg(samevenue(venue_stoc_94_,venue_stoc_p_)).
neg(samevenue(venue_stoc_94_,venue_symposium_theory_computing_)).
neg(samevenue(venue_stoc_94_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_stoc_94_,venue_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_26th_symposium_theory_computing_,venue_25th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_26th_symposium_theory_computing_,venue_25th_symposium_theory_computing_1994_)).
neg(samevenue(venue_proceedings_26th_symposium_theory_computing_,venue_stoc_)).
samevenue(venue_proceedings_26th_symposium_theory_computing_,venue_stoc_94_).
neg(samevenue(venue_proceedings_26th_symposium_theory_computing_,venue_25th_symp_theory_computing_)).
neg(samevenue(venue_proceedings_26th_symposium_theory_computing_,venue_25th_symposium_theory_computing_to_appear_)).
neg(samevenue(venue_proceedings_26th_symposium_theory_computing_,venue_stoc_p_)).
neg(samevenue(venue_proceedings_26th_symposium_theory_computing_,venue_proc_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_26th_symposium_theory_computing_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_26th_symposium_theory_computing_,venue_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_26th_symposium_theory_computing_,venue_proc_21st_symp_theoretical_computing_)).
samevenue(venue_25th_symp_theory_computing_,venue_25th_symposium_theory_computing_).
samevenue(venue_25th_symp_theory_computing_,venue_25th_symposium_theory_computing_1994_).
neg(samevenue(venue_25th_symp_theory_computing_,venue_proc_26th_symp_theory_computing_)).
neg(samevenue(venue_25th_symp_theory_computing_,venue_stoc_)).
neg(samevenue(venue_25th_symp_theory_computing_,venue_stoc_94_)).
neg(samevenue(venue_25th_symp_theory_computing_,venue_proceedings_26th_symposium_theory_computing_)).
samevenue(venue_25th_symp_theory_computing_,venue_25th_symp_theory_computing_).
samevenue(venue_25th_symp_theory_computing_,venue_25th_symposium_theory_computing_to_appear_).
neg(samevenue(venue_25th_symp_theory_computing_,venue_stoc_p_)).
neg(samevenue(venue_25th_symp_theory_computing_,venue_symposium_theory_computing_)).
neg(samevenue(venue_25th_symp_theory_computing_,venue_proc_workshop_computational_learning_theory_)).
neg(samevenue(venue_25th_symp_theory_computing_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_25th_symp_theory_computing_,venue_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_25th_symp_theory_computing_,venue_proc_21st_symp_theoretical_computing_)).
neg(samevenue(venue_25th_symposium_theory_computing_to_appear_,venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_)).
neg(samevenue(venue_25th_symposium_theory_computing_to_appear_,venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
neg(samevenue(venue_25th_symposium_theory_computing_to_appear_,venue_proceedings_thirty_seventh_symposium_foundations_computer_science_)).
neg(samevenue(venue_25th_symposium_theory_computing_to_appear_,venue_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
neg(samevenue(venue_25th_symposium_theory_computing_to_appear_,venue_proceedings_30th_symposium_foundation_computer_science_)).
neg(samevenue(venue_25th_symposium_theory_computing_to_appear_,venue_proceedings_30th_symposium_foundations_computer_science_)).
samevenue(venue_25th_symposium_theory_computing_to_appear_,venue_25th_symposium_theory_computing_).
samevenue(venue_25th_symposium_theory_computing_to_appear_,venue_25th_symposium_theory_computing_1994_).
neg(samevenue(venue_25th_symposium_theory_computing_to_appear_,venue_stoc_)).
neg(samevenue(venue_25th_symposium_theory_computing_to_appear_,venue_stoc_94_)).
neg(samevenue(venue_25th_symposium_theory_computing_to_appear_,venue_proceedings_26th_symposium_theory_computing_)).
samevenue(venue_25th_symposium_theory_computing_to_appear_,venue_25th_symp_theory_computing_).
samevenue(venue_25th_symposium_theory_computing_to_appear_,venue_25th_symposium_theory_computing_to_appear_).
neg(samevenue(venue_25th_symposium_theory_computing_to_appear_,venue_stoc_p_)).
neg(samevenue(venue_25th_symposium_theory_computing_to_appear_,venue_symposium_theory_computing_)).
neg(samevenue(venue_25th_symposium_theory_computing_to_appear_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_25th_symposium_theory_computing_to_appear_,venue_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_stoc_p_,venue_25th_symposium_theory_computing_)).
neg(samevenue(venue_stoc_p_,venue_25th_symposium_theory_computing_1994_)).
samevenue(venue_stoc_p_,venue_stoc_).
neg(samevenue(venue_stoc_p_,venue_stoc_94_)).
neg(samevenue(venue_stoc_p_,venue_proceedings_26th_symposium_theory_computing_)).
neg(samevenue(venue_stoc_p_,venue_25th_symp_theory_computing_)).
neg(samevenue(venue_stoc_p_,venue_25th_symposium_theory_computing_to_appear_)).
samevenue(venue_stoc_p_,venue_stoc_p_).
samevenue(venue_stoc_p_,venue_symposium_theory_computing_).
neg(samevenue(venue_stoc_p_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_stoc_p_,venue_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_symposium_theory_computing_,venue_proceedings_26th_symposium_theory_computing_)).
neg(samevenue(venue_symposium_theory_computing_,venue_proc_26th_sympos_theory_comput_)).
neg(samevenue(venue_symposium_theory_computing_,venue_25th_symposium_theory_computing_)).
neg(samevenue(venue_symposium_theory_computing_,venue_25th_symposium_theory_computing_1994_)).
neg(samevenue(venue_symposium_theory_computing_,venue_proc_26th_symp_theory_computing_)).
samevenue(venue_symposium_theory_computing_,venue_stoc_).
neg(samevenue(venue_symposium_theory_computing_,venue_stoc_94_)).
neg(samevenue(venue_symposium_theory_computing_,venue_25th_symp_theory_computing_)).
neg(samevenue(venue_symposium_theory_computing_,venue_25th_symposium_theory_computing_to_appear_)).
samevenue(venue_symposium_theory_computing_,venue_stoc_p_).
samevenue(venue_symposium_theory_computing_,venue_symposium_theory_computing_).
neg(samevenue(venue_symposium_theory_computing_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_symposium_theory_computing_,venue_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_25th_symposium_theory_computing_,venue_57_62_)).
neg(samevenue(venue_25th_symposium_theory_computing_,venue_chp_)).
neg(samevenue(venue_25th_symposium_theory_computing_,venue_proceedings_eighth_workshop_computational_learning_theory_)).
neg(samevenue(venue_25th_symposium_theory_computing_,venue_proceedings_eighth_conference_computational_learning_theory_)).
neg(samevenue(venue_25th_symposium_theory_computing_,venue_proceedings_eigth_conference_computational_learning_theory_)).
neg(samevenue(venue_25th_symposium_theory_computing_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
neg(samevenue(venue_25th_symposium_theory_computing_,venue_proc_workshop_computational_learning_theory_)).
neg(samevenue(venue_25th_symposium_theory_computing_,venue_proceedings_twenty_eighth_symposium_theory_computing_)).
neg(samevenue(venue_57_62_,venue_25th_symposium_theory_computing_)).
samevenue(venue_57_62_,venue_57_62_).
neg(samevenue(venue_57_62_,venue_chp_)).
neg(samevenue(venue_57_62_,venue_proceedings_eighth_workshop_computational_learning_theory_)).
neg(samevenue(venue_57_62_,venue_proceedings_eighth_conference_computational_learning_theory_)).
neg(samevenue(venue_57_62_,venue_proceedings_eigth_conference_computational_learning_theory_)).
neg(samevenue(venue_57_62_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
neg(samevenue(venue_57_62_,venue_proc_workshop_computational_learning_theory_)).
neg(samevenue(venue_57_62_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_57_62_,venue_proceedings_twenty_eighth_symposium_theory_computing_)).
neg(samevenue(venue_57_62_,venue_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_chp_,venue_25th_symposium_theory_computing_)).
neg(samevenue(venue_chp_,venue_57_62_)).
samevenue(venue_chp_,venue_chp_).
neg(samevenue(venue_chp_,venue_proceedings_eighth_workshop_computational_learning_theory_)).
neg(samevenue(venue_chp_,venue_proceedings_eighth_conference_computational_learning_theory_)).
neg(samevenue(venue_chp_,venue_proceedings_eigth_conference_computational_learning_theory_)).
neg(samevenue(venue_chp_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
neg(samevenue(venue_chp_,venue_proc_workshop_computational_learning_theory_)).
neg(samevenue(venue_chp_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_chp_,venue_proceedings_twenty_eighth_symposium_theory_computing_)).
neg(samevenue(venue_chp_,venue_proceedings_28th_symposium_theory_computing_)).
samevenue(venue_proceedings_colt_95_,venue_proceedings_colt_95_).
samevenue(venue_proceedings_colt_95_,venue_proceedings_eighth_workshop_computational_learning_theory_).
samevenue(venue_proceedings_colt_95_,venue_proceedings_eighth_conference_computational_learning_theory_).
samevenue(venue_proceedings_colt_95_,venue_proceedings_eigth_conference_computational_learning_theory_).
samevenue(venue_proceedings_colt_95_,venue_colt_95_).
neg(samevenue(venue_proceedings_colt_95_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_colt_95_,venue_proc_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_proceedings_4th_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_colt_91_proceedings_4th_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_computational_learning_theory_proceedings_fourth_workshop_)).
neg(samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_proc_4th_workshop_comput_learning_theory_)).
neg(samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_proc_4th_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_proceedings_1991_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_25th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_57_62_)).
neg(samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_chp_)).
samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_proceedings_colt_95_).
samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_proceedings_eighth_workshop_computational_learning_theory_).
samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_proceedings_eighth_conference_computational_learning_theory_).
samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_proceedings_eigth_conference_computational_learning_theory_).
samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_colt_95_).
neg(samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_proc_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_proceedings_twenty_eighth_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_computation_learning_theory_proceedings_fourth_workshop_)).
neg(samevenue(venue_proceedings_eighth_workshop_computational_learning_theory_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_eighth_conference_computational_learning_theory_,venue_25th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_eighth_conference_computational_learning_theory_,venue_57_62_)).
neg(samevenue(venue_proceedings_eighth_conference_computational_learning_theory_,venue_chp_)).
samevenue(venue_proceedings_eighth_conference_computational_learning_theory_,venue_proceedings_colt_95_).
samevenue(venue_proceedings_eighth_conference_computational_learning_theory_,venue_proceedings_eighth_workshop_computational_learning_theory_).
samevenue(venue_proceedings_eighth_conference_computational_learning_theory_,venue_proceedings_eighth_conference_computational_learning_theory_).
samevenue(venue_proceedings_eighth_conference_computational_learning_theory_,venue_proceedings_eigth_conference_computational_learning_theory_).
samevenue(venue_proceedings_eighth_conference_computational_learning_theory_,venue_colt_95_).
neg(samevenue(venue_proceedings_eighth_conference_computational_learning_theory_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_eighth_conference_computational_learning_theory_,venue_proc_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_eighth_conference_computational_learning_theory_,venue_proceedings_twenty_eighth_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_eighth_conference_computational_learning_theory_,venue_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_eighth_conference_computational_learning_theory_,venue_computation_learning_theory_proceedings_fourth_workshop_)).
neg(samevenue(venue_proceedings_eighth_conference_computational_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_eighth_conference_computational_learning_theory_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_eighth_conference_computational_learning_theory_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_eigth_conference_computational_learning_theory_,venue_25th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_eigth_conference_computational_learning_theory_,venue_57_62_)).
neg(samevenue(venue_proceedings_eigth_conference_computational_learning_theory_,venue_chp_)).
samevenue(venue_proceedings_eigth_conference_computational_learning_theory_,venue_proceedings_colt_95_).
samevenue(venue_proceedings_eigth_conference_computational_learning_theory_,venue_proceedings_eighth_workshop_computational_learning_theory_).
samevenue(venue_proceedings_eigth_conference_computational_learning_theory_,venue_proceedings_eighth_conference_computational_learning_theory_).
samevenue(venue_proceedings_eigth_conference_computational_learning_theory_,venue_proceedings_eigth_conference_computational_learning_theory_).
samevenue(venue_proceedings_eigth_conference_computational_learning_theory_,venue_colt_95_).
neg(samevenue(venue_proceedings_eigth_conference_computational_learning_theory_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_eigth_conference_computational_learning_theory_,venue_proc_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_eigth_conference_computational_learning_theory_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_eigth_conference_computational_learning_theory_,venue_proceedings_twenty_eighth_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_eigth_conference_computational_learning_theory_,venue_proceedings_28th_symposium_theory_computing_)).
samevenue(venue_colt_95_,venue_proceedings_colt_95_).
samevenue(venue_colt_95_,venue_proceedings_eighth_workshop_computational_learning_theory_).
samevenue(venue_colt_95_,venue_proceedings_eighth_conference_computational_learning_theory_).
samevenue(venue_colt_95_,venue_proceedings_eigth_conference_computational_learning_theory_).
samevenue(venue_colt_95_,venue_colt_95_).
neg(samevenue(venue_colt_95_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
neg(samevenue(venue_colt_95_,venue_proc_workshop_computational_learning_theory_)).
neg(samevenue(venue_colt_95_,venue_proceedings_twenty_eighth_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_proceedings_thirty_seventh_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_ieee_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_proceedings_30th_symposium_foundation_computer_science_)).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_30th_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_proceedings_thirtieth_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_proceedings_30th_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_25th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_57_62_)).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_chp_)).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_proceedings_colt_95_)).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_proceedings_eighth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_proceedings_eighth_conference_computational_learning_theory_)).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_proceedings_eigth_conference_computational_learning_theory_)).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_colt_95_)).
samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_proceedings_seventh_workshop_computational_learning_theory_).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_proc_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_proceedings_twenty_eighth_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_seventh_workshop_computational_learning_theory_,venue_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_proc_9th_conference_computational_learning_theory_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_proceedings_4th_workshop_computational_learning_theory_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_colt_91_proceedings_4th_workshop_computational_learning_theory_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_computational_learning_theory_proceedings_fourth_workshop_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_proc_4th_workshop_comput_learning_theory_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_proc_4th_workshop_computational_learning_theory_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_proceedings_1991_workshop_computational_learning_theory_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_proc_26th_symp_theory_computing_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_proceedings_26th_symposium_theory_computing_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_25th_symp_theory_computing_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_25th_symposium_theory_computing_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_57_62_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_chp_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_proceedings_colt_95_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_proceedings_eighth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_proceedings_eighth_conference_computational_learning_theory_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_proceedings_eigth_conference_computational_learning_theory_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_colt_95_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
samevenue(venue_proc_workshop_computational_learning_theory_,venue_proc_workshop_computational_learning_theory_).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_to_appear_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_proceedings_twenty_eighth_symposium_theory_computing_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_proceedings_28th_symposium_theory_computing_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_proc_21st_symp_theoretical_computing_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_computation_learning_theory_proceedings_fourth_workshop_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proc_workshop_computational_learning_theory_,venue_proc_11th_international_conference_machine_learning_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_proceedings_1992_conf_neural_informations_processing_systems_to_appear_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_to_appear_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_proceedings_thirty_seventh_symposium_foundations_computer_science_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_proceedings_30th_symposium_foundation_computer_science_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_proceedings_30th_symposium_foundations_computer_science_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_25th_symposium_theory_computing_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_25th_symposium_theory_computing_1994_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_stoc_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_stoc_94_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_proceedings_26th_symposium_theory_computing_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_25th_symp_theory_computing_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_25th_symposium_theory_computing_to_appear_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_stoc_p_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_symposium_theory_computing_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_57_62_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_chp_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_proceedings_eighth_conference_computational_learning_theory_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_proceedings_eigth_conference_computational_learning_theory_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
neg(samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_proc_workshop_computational_learning_theory_)).
samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_to_appear_proceedings_28th_symposium_theory_computing_).
samevenue(venue_to_appear_proceedings_28th_symposium_theory_computing_,venue_proceedings_28th_symposium_theory_computing_).
neg(samevenue(venue_proceedings_twenty_eighth_symposium_theory_computing_,venue_proceedings_twenty_sixth_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_twenty_eighth_symposium_theory_computing_,venue_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_twenty_eighth_symposium_theory_computing_,venue_25th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_twenty_eighth_symposium_theory_computing_,venue_57_62_)).
neg(samevenue(venue_proceedings_twenty_eighth_symposium_theory_computing_,venue_chp_)).
neg(samevenue(venue_proceedings_twenty_eighth_symposium_theory_computing_,venue_proceedings_eighth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_twenty_eighth_symposium_theory_computing_,venue_proceedings_eighth_conference_computational_learning_theory_)).
neg(samevenue(venue_proceedings_twenty_eighth_symposium_theory_computing_,venue_proceedings_eigth_conference_computational_learning_theory_)).
neg(samevenue(venue_proceedings_twenty_eighth_symposium_theory_computing_,venue_colt_95_)).
neg(samevenue(venue_proceedings_twenty_eighth_symposium_theory_computing_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_twenty_eighth_symposium_theory_computing_,venue_proc_workshop_computational_learning_theory_)).
samevenue(venue_proceedings_twenty_eighth_symposium_theory_computing_,venue_proceedings_twenty_eighth_symposium_theory_computing_).
samevenue(venue_proceedings_twenty_eighth_symposium_theory_computing_,venue_proceedings_28th_symposium_theory_computing_).
neg(samevenue(venue_proceedings_twenty_eighth_symposium_theory_computing_,venue_computation_learning_theory_proceedings_fourth_workshop_)).
neg(samevenue(venue_proceedings_twenty_eighth_symposium_theory_computing_,venue_proceedings_fourth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_twenty_eighth_symposium_theory_computing_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_28th_symposium_theory_computing_,venue_25th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_28th_symposium_theory_computing_,venue_25th_symposium_theory_computing_1994_)).
neg(samevenue(venue_proceedings_28th_symposium_theory_computing_,venue_stoc_)).
neg(samevenue(venue_proceedings_28th_symposium_theory_computing_,venue_stoc_94_)).
neg(samevenue(venue_proceedings_28th_symposium_theory_computing_,venue_proceedings_26th_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_28th_symposium_theory_computing_,venue_25th_symp_theory_computing_)).
neg(samevenue(venue_proceedings_28th_symposium_theory_computing_,venue_25th_symposium_theory_computing_to_appear_)).
neg(samevenue(venue_proceedings_28th_symposium_theory_computing_,venue_stoc_p_)).
neg(samevenue(venue_proceedings_28th_symposium_theory_computing_,venue_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_28th_symposium_theory_computing_,venue_57_62_)).
neg(samevenue(venue_proceedings_28th_symposium_theory_computing_,venue_chp_)).
neg(samevenue(venue_proceedings_28th_symposium_theory_computing_,venue_proceedings_eighth_conference_computational_learning_theory_)).
neg(samevenue(venue_proceedings_28th_symposium_theory_computing_,venue_proceedings_eigth_conference_computational_learning_theory_)).
neg(samevenue(venue_proceedings_28th_symposium_theory_computing_,venue_proceedings_seventh_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_28th_symposium_theory_computing_,venue_proc_workshop_computational_learning_theory_)).
samevenue(venue_proceedings_28th_symposium_theory_computing_,venue_to_appear_proceedings_28th_symposium_theory_computing_).
samevenue(venue_proceedings_28th_symposium_theory_computing_,venue_proceedings_twenty_eighth_symposium_theory_computing_).
samevenue(venue_proceedings_28th_symposium_theory_computing_,venue_proceedings_28th_symposium_theory_computing_).
samevenue(venue_unpublished_manuscript_,venue_unpublished_manuscript_).
neg(samevenue(venue_proc_21st_symp_theoretical_computing_,venue_proc_26th_symp_theory_computing_)).
neg(samevenue(venue_proc_21st_symp_theoretical_computing_,venue_proceedings_26th_symposium_theory_computing_)).
neg(samevenue(venue_proc_21st_symp_theoretical_computing_,venue_25th_symp_theory_computing_)).
neg(samevenue(venue_proc_21st_symp_theoretical_computing_,venue_proc_workshop_computational_learning_theory_)).
samevenue(venue_proc_21st_symp_theoretical_computing_,venue_proc_21st_symp_theoretical_computing_).
samevenue(venue_information_computation_,venue_information_computation_).
neg(samevenue(venue_information_computation_,venue_information_computation_103_2_)).
neg(samevenue(venue_information_computation_,venue_formation_computation_103_2_1993_)).
samevenue(venue_information_computation_,venue_information_compuation_).
neg(samevenue(venue_information_computation_,venue_information_computation_103_)).
neg(samevenue(venue_information_computation_,venue_journal_association_computing_machinery_)).
neg(samevenue(venue_information_computation_103_2_,venue_information_computation_)).
samevenue(venue_information_computation_103_2_,venue_information_computation_103_2_).
samevenue(venue_information_computation_103_2_,venue_formation_computation_103_2_1993_).
neg(samevenue(venue_information_computation_103_2_,venue_information_compuation_)).
samevenue(venue_information_computation_103_2_,venue_information_computation_103_).
neg(samevenue(venue_information_computation_103_2_,venue_journal_association_computing_machinery_)).
samevenue(venue_machine_learning_from_theory_to_applications_,venue_machine_learning_from_theory_to_applications_).
neg(samevenue(venue_machine_learning_from_theory_to_applications_,venue_proceedings_fourteenth_international_conference_machine_learning_)).
neg(samevenue(venue_machine_learning_from_theory_to_applications_,venue_machine_learning_proceedings_fourteenth_international_conference_)).
neg(samevenue(venue_formation_computation_103_2_1993_,venue_information_computation_)).
samevenue(venue_formation_computation_103_2_1993_,venue_information_computation_103_2_).
samevenue(venue_formation_computation_103_2_1993_,venue_formation_computation_103_2_1993_).
neg(samevenue(venue_formation_computation_103_2_1993_,venue_information_compuation_)).
samevenue(venue_formation_computation_103_2_1993_,venue_information_computation_103_).
neg(samevenue(venue_formation_computation_103_2_1993_,venue_journal_association_computing_machinery_)).
samevenue(venue_information_compuation_,venue_information_computation_).
neg(samevenue(venue_information_compuation_,venue_information_computation_103_2_)).
neg(samevenue(venue_information_compuation_,venue_formation_computation_103_2_1993_)).
samevenue(venue_information_compuation_,venue_information_compuation_).
neg(samevenue(venue_information_compuation_,venue_information_computation_103_)).
neg(samevenue(venue_information_compuation_,venue_journal_association_computing_machinery_)).
neg(samevenue(venue_information_computation_103_,venue_information_computation_)).
samevenue(venue_information_computation_103_,venue_information_computation_103_2_).
samevenue(venue_information_computation_103_,venue_formation_computation_103_2_1993_).
neg(samevenue(venue_information_computation_103_,venue_information_compuation_)).
samevenue(venue_information_computation_103_,venue_information_computation_103_).
neg(samevenue(venue_information_computation_103_,venue_journal_association_computing_machinery_)).
neg(samevenue(venue_journal_association_computing_machinery_,venue_information_computation_)).
neg(samevenue(venue_journal_association_computing_machinery_,venue_information_computation_103_2_)).
neg(samevenue(venue_journal_association_computing_machinery_,venue_formation_computation_103_2_1993_)).
neg(samevenue(venue_journal_association_computing_machinery_,venue_information_compuation_)).
neg(samevenue(venue_journal_association_computing_machinery_,venue_information_computation_103_)).
samevenue(venue_journal_association_computing_machinery_,venue_journal_association_computing_machinery_).
neg(samevenue(venue_technical_report_mit_lcs_tr_493_,venue_technical_report_mit_lcs_tm_413_)).
neg(samevenue(venue_technical_report_mit_lcs_tr_493_,venue_technical_report_)).
samevenue(venue_technical_report_mit_lcs_tr_493_,venue_technical_report_mit_lcs_tr_493_).
neg(samevenue(venue_technical_report_mit_lcs_tr_493_,venue_proceedings_eleventh_international_conference_machine_learning_)).
neg(samevenue(venue_technical_report_mit_lcs_tr_493_,venue_proc_11th_international_conference_machine_learning_)).
neg(samevenue(venue_phd_thesis_,venue_proceedings_fourth_workshop_computational_learning_theory_)).
samevenue(venue_computation_learning_theory_proceedings_fourth_workshop_,venue_proceedings_fourth_conference_computational_learning_theory_colt_91_).
samevenue(venue_computation_learning_theory_proceedings_fourth_workshop_,venue_proceedings_fourth_workshop_computational_learning_theory_pages_61_74_).
samevenue(venue_computation_learning_theory_proceedings_fourth_workshop_,venue_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_computation_learning_theory_proceedings_fourth_workshop_,venue_colt_91_proceedings_4th_workshop_computational_learning_theory_).
samevenue(venue_computation_learning_theory_proceedings_fourth_workshop_,venue_computational_learning_theory_proceedings_fourth_workshop_).
samevenue(venue_computation_learning_theory_proceedings_fourth_workshop_,venue_proc_4th_workshop_comput_learning_theory_).
samevenue(venue_computation_learning_theory_proceedings_fourth_workshop_,venue_proc_4th_workshop_computational_learning_theory_).
samevenue(venue_computation_learning_theory_proceedings_fourth_workshop_,venue_proceedings_fourth_workshop_computational_learning_theory_).
samevenue(venue_computation_learning_theory_proceedings_fourth_workshop_,venue_proceedings_1991_workshop_computational_learning_theory_).
neg(samevenue(venue_computation_learning_theory_proceedings_fourth_workshop_,venue_proceedings_eighth_workshop_computational_learning_theory_)).
neg(samevenue(venue_computation_learning_theory_proceedings_fourth_workshop_,venue_proceedings_eighth_conference_computational_learning_theory_)).
neg(samevenue(venue_computation_learning_theory_proceedings_fourth_workshop_,venue_proc_workshop_computational_learning_theory_)).
neg(samevenue(venue_computation_learning_theory_proceedings_fourth_workshop_,venue_proceedings_twenty_eighth_symposium_theory_computing_)).
samevenue(venue_computation_learning_theory_proceedings_fourth_workshop_,venue_computation_learning_theory_proceedings_fourth_workshop_).
neg(samevenue(venue_computation_learning_theory_proceedings_fourth_workshop_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
neg(samevenue(venue_computation_learning_theory_proceedings_fourth_workshop_,venue_proc_11th_international_conference_machine_learning_)).
neg(samevenue(venue_computation_learning_theory_proceedings_fourth_workshop_,venue_proceedings_eleventh_conference_computational_learning_theory_)).
neg(samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_proceedings_eighth_conference_computational_learning_theory_)).
neg(samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_proceedings_twenty_eighth_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_proc_11th_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_proceedings_eleventh_conference_computational_learning_theory_)).
neg(samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_machine_learning_proceedings_eleventh_international_conference_)).
neg(samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_proceddings_11th_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_proc_11th_international_conf_machine_learning_)).
neg(samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_phd_thesis_)).
neg(samevenue(venue_proceedings_fourth_workshop_computational_learning_theory_,venue_proceedings_fourteenth_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proceedings_twenty_sixth_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proceedings_thirty_sixth_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_ieee_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proceedings_30th_symposium_foundation_computer_science_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_30th_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proceedings_thirtieth_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proceedings_30th_symposium_foundations_computer_science_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proceedings_4th_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_colt_91_proceedings_4th_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_computational_learning_theory_proceedings_fourth_workshop_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proc_4th_workshop_comput_learning_theory_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proc_4th_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proceedings_1991_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proceedings_eighth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proceedings_eighth_conference_computational_learning_theory_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proc_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proceedings_twenty_eighth_symposium_theory_computing_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_computation_learning_theory_proceedings_fourth_workshop_)).
samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proceedings_sixth_workshop_computational_learning_theory_).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_machine_learning_proceedings_eleventh_international_conference_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proceddings_11th_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proc_11th_international_conf_machine_learning_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proc_11th_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_j_computer_system_sciences_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proceedings_fourteenth_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_machine_learning_proceedings_fourteenth_international_conference_)).
neg(samevenue(venue_proceedings_sixth_workshop_computational_learning_theory_,venue_proceedings_14th_international_conference_machine_learning_)).
neg(samevenue(venue_machine_learning_proceedings_eleventh_international_conference_,venue_machine_learning_)).
neg(samevenue(venue_machine_learning_proceedings_eleventh_international_conference_,venue_proceedings_fourth_workshop_computational_learning_theory_)).
neg(samevenue(venue_machine_learning_proceedings_eleventh_international_conference_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
samevenue(venue_machine_learning_proceedings_eleventh_international_conference_,venue_machine_learning_proceedings_eleventh_international_conference_).
samevenue(venue_machine_learning_proceedings_eleventh_international_conference_,venue_proceddings_11th_international_conference_machine_learning_).
samevenue(venue_machine_learning_proceedings_eleventh_international_conference_,venue_proceedings_eleventh_international_conference_machine_learning_).
samevenue(venue_machine_learning_proceedings_eleventh_international_conference_,venue_proc_11th_international_conference_machine_learning_).
samevenue(venue_machine_learning_proceedings_eleventh_international_conference_,venue_proc_11th_international_conf_machine_learning_).
neg(samevenue(venue_machine_learning_proceedings_eleventh_international_conference_,venue_proceedings_fourteenth_international_conference_machine_learning_)).
neg(samevenue(venue_machine_learning_proceedings_eleventh_international_conference_,venue_machine_learning_proceedings_fourteenth_international_conference_)).
neg(samevenue(venue_machine_learning_proceedings_eleventh_international_conference_,venue_proceedings_14th_international_conference_machine_learning_)).
samevenue(venue_machine_learning_proceedings_eleventh_international_conference_,venue_proceedings_11th_international_conference_machine_learning_).
neg(samevenue(venue_machine_learning_proceedings_eleventh_international_conference_,venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_)).
neg(samevenue(venue_machine_learning_proceedings_eleventh_international_conference_,venue_proceedings_eleventh_conference_computational_learning_theory_)).
neg(samevenue(venue_proceddings_11th_international_conference_machine_learning_,venue_proceedings_fourth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceddings_11th_international_conference_machine_learning_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
samevenue(venue_proceddings_11th_international_conference_machine_learning_,venue_machine_learning_proceedings_eleventh_international_conference_).
samevenue(venue_proceddings_11th_international_conference_machine_learning_,venue_proceddings_11th_international_conference_machine_learning_).
samevenue(venue_proceddings_11th_international_conference_machine_learning_,venue_proceedings_eleventh_international_conference_machine_learning_).
samevenue(venue_proceddings_11th_international_conference_machine_learning_,venue_proc_11th_international_conference_machine_learning_).
samevenue(venue_proceddings_11th_international_conference_machine_learning_,venue_proc_11th_international_conf_machine_learning_).
neg(samevenue(venue_proceddings_11th_international_conference_machine_learning_,venue_proceedings_fourteenth_international_conference_machine_learning_)).
neg(samevenue(venue_proceddings_11th_international_conference_machine_learning_,venue_machine_learning_proceedings_fourteenth_international_conference_)).
neg(samevenue(venue_proceddings_11th_international_conference_machine_learning_,venue_proceedings_14th_international_conference_machine_learning_)).
samevenue(venue_proceddings_11th_international_conference_machine_learning_,venue_proceedings_11th_international_conference_machine_learning_).
neg(samevenue(venue_proceddings_11th_international_conference_machine_learning_,venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_)).
neg(samevenue(venue_proceddings_11th_international_conference_machine_learning_,venue_proceedings_eleventh_conference_computational_learning_theory_)).
neg(samevenue(venue_proceedings_eleventh_international_conference_machine_learning_,venue_machine_learning_)).
neg(samevenue(venue_proceedings_eleventh_international_conference_machine_learning_,venue_technical_report_mit_lcs_tr_493_)).
samevenue(venue_proceedings_eleventh_international_conference_machine_learning_,venue_machine_learning_proceedings_eleventh_international_conference_).
samevenue(venue_proceedings_eleventh_international_conference_machine_learning_,venue_proceddings_11th_international_conference_machine_learning_).
samevenue(venue_proceedings_eleventh_international_conference_machine_learning_,venue_proceedings_eleventh_international_conference_machine_learning_).
samevenue(venue_proceedings_eleventh_international_conference_machine_learning_,venue_proc_11th_international_conference_machine_learning_).
samevenue(venue_proceedings_eleventh_international_conference_machine_learning_,venue_proc_11th_international_conf_machine_learning_).
neg(samevenue(venue_proceedings_eleventh_international_conference_machine_learning_,venue_proceedings_fourteenth_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_eleventh_international_conference_machine_learning_,venue_machine_learning_proceedings_fourteenth_international_conference_)).
neg(samevenue(venue_proceedings_eleventh_international_conference_machine_learning_,venue_proceedings_14th_international_conference_machine_learning_)).
samevenue(venue_proceedings_eleventh_international_conference_machine_learning_,venue_proceedings_11th_international_conference_machine_learning_).
neg(samevenue(venue_proceedings_eleventh_international_conference_machine_learning_,venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_)).
neg(samevenue(venue_proceedings_eleventh_international_conference_machine_learning_,venue_proceedings_eleventh_conference_computational_learning_theory_)).
neg(samevenue(venue_proc_11th_international_conference_machine_learning_,venue_machine_learning_)).
neg(samevenue(venue_proc_11th_international_conference_machine_learning_,venue_proc_workshop_computational_learning_theory_)).
neg(samevenue(venue_proc_11th_international_conference_machine_learning_,venue_technical_report_mit_lcs_tr_493_)).
neg(samevenue(venue_proc_11th_international_conference_machine_learning_,venue_computation_learning_theory_proceedings_fourth_workshop_)).
neg(samevenue(venue_proc_11th_international_conference_machine_learning_,venue_proceedings_fourth_workshop_computational_learning_theory_)).
samevenue(venue_proc_11th_international_conference_machine_learning_,venue_machine_learning_proceedings_eleventh_international_conference_).
samevenue(venue_proc_11th_international_conference_machine_learning_,venue_proceddings_11th_international_conference_machine_learning_).
samevenue(venue_proc_11th_international_conference_machine_learning_,venue_proceedings_eleventh_international_conference_machine_learning_).
samevenue(venue_proc_11th_international_conference_machine_learning_,venue_proc_11th_international_conference_machine_learning_).
samevenue(venue_proc_11th_international_conference_machine_learning_,venue_proc_11th_international_conf_machine_learning_).
neg(samevenue(venue_proc_11th_international_conference_machine_learning_,venue_proceedings_fourteenth_international_conference_machine_learning_)).
neg(samevenue(venue_proc_11th_international_conference_machine_learning_,venue_machine_learning_proceedings_fourteenth_international_conference_)).
neg(samevenue(venue_proc_11th_international_conference_machine_learning_,venue_proceedings_14th_international_conference_machine_learning_)).
samevenue(venue_proc_11th_international_conference_machine_learning_,venue_proceedings_11th_international_conference_machine_learning_).
neg(samevenue(venue_proc_11th_international_conference_machine_learning_,venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_)).
neg(samevenue(venue_proc_11th_international_conference_machine_learning_,venue_proceedings_eleventh_conference_computational_learning_theory_)).
neg(samevenue(venue_proc_11th_international_conf_machine_learning_,venue_proceedings_fourth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proc_11th_international_conf_machine_learning_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
samevenue(venue_proc_11th_international_conf_machine_learning_,venue_machine_learning_proceedings_eleventh_international_conference_).
samevenue(venue_proc_11th_international_conf_machine_learning_,venue_proceddings_11th_international_conference_machine_learning_).
samevenue(venue_proc_11th_international_conf_machine_learning_,venue_proceedings_eleventh_international_conference_machine_learning_).
samevenue(venue_proc_11th_international_conf_machine_learning_,venue_proc_11th_international_conference_machine_learning_).
samevenue(venue_proc_11th_international_conf_machine_learning_,venue_proc_11th_international_conf_machine_learning_).
neg(samevenue(venue_proc_11th_international_conf_machine_learning_,venue_proceedings_fourteenth_international_conference_machine_learning_)).
neg(samevenue(venue_proc_11th_international_conf_machine_learning_,venue_machine_learning_proceedings_fourteenth_international_conference_)).
neg(samevenue(venue_proc_11th_international_conf_machine_learning_,venue_proceedings_14th_international_conference_machine_learning_)).
samevenue(venue_proc_11th_international_conf_machine_learning_,venue_proceedings_11th_international_conference_machine_learning_).
neg(samevenue(venue_proc_11th_international_conf_machine_learning_,venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_)).
neg(samevenue(venue_proc_11th_international_conf_machine_learning_,venue_proceedings_eleventh_conference_computational_learning_theory_)).
neg(samevenue(venue_proc_11th_international_conference_machine_learning_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
neg(samevenue(venue_j_computer_system_sciences_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
samevenue(venue_j_computer_system_sciences_,venue_j_computer_system_sciences_).
neg(samevenue(venue_j_computer_system_sciences_,venue_proceedings_fourteenth_international_conference_machine_learning_)).
neg(samevenue(venue_j_computer_system_sciences_,venue_machine_learning_proceedings_fourteenth_international_conference_)).
neg(samevenue(venue_j_computer_system_sciences_,venue_proceedings_14th_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_machine_learning_)).
neg(samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_machine_learning_from_theory_to_applications_)).
neg(samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_proceedings_fourth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_machine_learning_proceedings_eleventh_international_conference_)).
neg(samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_proceddings_11th_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_proceedings_eleventh_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_proc_11th_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_proc_11th_international_conf_machine_learning_)).
samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_proceedings_fourteenth_international_conference_machine_learning_).
samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_machine_learning_proceedings_fourteenth_international_conference_).
samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_proceedings_14th_international_conference_machine_learning_).
neg(samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_proceedings_11th_international_conference_machine_learning_)).
samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_).
neg(samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_proceedings_eleventh_conference_computational_learning_theory_)).
neg(samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_j_computer_system_sciences_)).
neg(samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_snowbird_1997_conference_)).
neg(samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_1997_)).
samevenue(venue_proceedings_fourteenth_international_conference_machine_learning_,venue_icml_97_).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_machine_learning_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_machine_learning_from_theory_to_applications_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_machine_learning_proceedings_eleventh_international_conference_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_proceddings_11th_international_conference_machine_learning_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_proceedings_eleventh_international_conference_machine_learning_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_proc_11th_international_conference_machine_learning_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_proc_11th_international_conf_machine_learning_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_j_computer_system_sciences_)).
samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_proceedings_fourteenth_international_conference_machine_learning_).
samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_machine_learning_proceedings_fourteenth_international_conference_).
samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_proceedings_14th_international_conference_machine_learning_).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_proceedings_11th_international_conference_machine_learning_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_snowbird_1997_conference_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_1997_)).
samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_icml_97_).
samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_proceedings_eleventh_conference_computational_learning_theory_)).
neg(samevenue(venue_proceedings_14th_international_conference_machine_learning_,venue_machine_learning_)).
neg(samevenue(venue_proceedings_14th_international_conference_machine_learning_,venue_machine_learning_proceedings_eleventh_international_conference_)).
neg(samevenue(venue_proceedings_14th_international_conference_machine_learning_,venue_proceddings_11th_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_14th_international_conference_machine_learning_,venue_proceedings_eleventh_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_14th_international_conference_machine_learning_,venue_proc_11th_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_14th_international_conference_machine_learning_,venue_proc_11th_international_conf_machine_learning_)).
samevenue(venue_proceedings_14th_international_conference_machine_learning_,venue_proceedings_fourteenth_international_conference_machine_learning_).
samevenue(venue_proceedings_14th_international_conference_machine_learning_,venue_machine_learning_proceedings_fourteenth_international_conference_).
samevenue(venue_proceedings_14th_international_conference_machine_learning_,venue_proceedings_14th_international_conference_machine_learning_).
neg(samevenue(venue_proceedings_14th_international_conference_machine_learning_,venue_proceedings_11th_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_14th_international_conference_machine_learning_,venue_snowbird_1997_conference_)).
neg(samevenue(venue_proceedings_14th_international_conference_machine_learning_,venue_1997_)).
samevenue(venue_proceedings_14th_international_conference_machine_learning_,venue_icml_97_).
samevenue(venue_proceedings_14th_international_conference_machine_learning_,venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_).
neg(samevenue(venue_proceedings_14th_international_conference_machine_learning_,venue_proceedings_eleventh_conference_computational_learning_theory_)).
neg(samevenue(venue_proceedings_11th_international_conference_machine_learning_,venue_machine_learning_)).
samevenue(venue_proceedings_11th_international_conference_machine_learning_,venue_machine_learning_proceedings_eleventh_international_conference_).
samevenue(venue_proceedings_11th_international_conference_machine_learning_,venue_proceddings_11th_international_conference_machine_learning_).
samevenue(venue_proceedings_11th_international_conference_machine_learning_,venue_proceedings_eleventh_international_conference_machine_learning_).
samevenue(venue_proceedings_11th_international_conference_machine_learning_,venue_proc_11th_international_conference_machine_learning_).
samevenue(venue_proceedings_11th_international_conference_machine_learning_,venue_proc_11th_international_conf_machine_learning_).
neg(samevenue(venue_proceedings_11th_international_conference_machine_learning_,venue_proceedings_fourteenth_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_11th_international_conference_machine_learning_,venue_machine_learning_proceedings_fourteenth_international_conference_)).
neg(samevenue(venue_proceedings_11th_international_conference_machine_learning_,venue_proceedings_14th_international_conference_machine_learning_)).
samevenue(venue_proceedings_11th_international_conference_machine_learning_,venue_proceedings_11th_international_conference_machine_learning_).
neg(samevenue(venue_proceedings_11th_international_conference_machine_learning_,venue_snowbird_1997_conference_)).
neg(samevenue(venue_proceedings_11th_international_conference_machine_learning_,venue_1997_)).
neg(samevenue(venue_proceedings_11th_international_conference_machine_learning_,venue_icml_97_)).
neg(samevenue(venue_proceedings_11th_international_conference_machine_learning_,venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_)).
neg(samevenue(venue_proceedings_11th_international_conference_machine_learning_,venue_proceedings_eleventh_conference_computational_learning_theory_)).
neg(samevenue(venue_snowbird_1997_conference_,venue_proceedings_fourteenth_international_conference_machine_learning_)).
neg(samevenue(venue_snowbird_1997_conference_,venue_machine_learning_proceedings_fourteenth_international_conference_)).
neg(samevenue(venue_snowbird_1997_conference_,venue_proceedings_14th_international_conference_machine_learning_)).
neg(samevenue(venue_snowbird_1997_conference_,venue_proceedings_11th_international_conference_machine_learning_)).
samevenue(venue_snowbird_1997_conference_,venue_snowbird_1997_conference_).
neg(samevenue(venue_snowbird_1997_conference_,venue_1997_)).
neg(samevenue(venue_snowbird_1997_conference_,venue_icml_97_)).
neg(samevenue(venue_snowbird_1997_conference_,venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_)).
neg(samevenue(venue_1997_,venue_proceedings_fourteenth_international_conference_machine_learning_)).
neg(samevenue(venue_1997_,venue_machine_learning_proceedings_fourteenth_international_conference_)).
neg(samevenue(venue_1997_,venue_proceedings_14th_international_conference_machine_learning_)).
neg(samevenue(venue_1997_,venue_proceedings_11th_international_conference_machine_learning_)).
neg(samevenue(venue_1997_,venue_snowbird_1997_conference_)).
samevenue(venue_1997_,venue_1997_).
neg(samevenue(venue_1997_,venue_icml_97_)).
neg(samevenue(venue_1997_,venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_)).
neg(samevenue(venue_1997_,venue_proceedings_eleventh_conference_computational_learning_theory_)).
samevenue(venue_icml_97_,venue_proceedings_fourteenth_international_conference_machine_learning_).
samevenue(venue_icml_97_,venue_machine_learning_proceedings_fourteenth_international_conference_).
samevenue(venue_icml_97_,venue_proceedings_14th_international_conference_machine_learning_).
neg(samevenue(venue_icml_97_,venue_proceedings_11th_international_conference_machine_learning_)).
neg(samevenue(venue_icml_97_,venue_snowbird_1997_conference_)).
neg(samevenue(venue_icml_97_,venue_1997_)).
samevenue(venue_icml_97_,venue_icml_97_).
samevenue(venue_icml_97_,venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_).
neg(samevenue(venue_icml_97_,venue_proceedings_eleventh_conference_computational_learning_theory_)).
neg(samevenue(venue_proceedings_14th_international_conference_machine_learning_,venue_proceedings_sixth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_14th_international_conference_machine_learning_,venue_j_computer_system_sciences_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_tech_rep_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_,venue_unpublished_manuscript_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_,venue_machine_learning_proceedings_eleventh_international_conference_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_,venue_proceddings_11th_international_conference_machine_learning_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_,venue_proceedings_eleventh_international_conference_machine_learning_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_,venue_proc_11th_international_conference_machine_learning_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_,venue_proc_11th_international_conf_machine_learning_)).
samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_,venue_proceedings_fourteenth_international_conference_machine_learning_).
samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_,venue_machine_learning_proceedings_fourteenth_international_conference_).
samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_,venue_proceedings_14th_international_conference_machine_learning_).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_,venue_proceedings_11th_international_conference_machine_learning_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_,venue_snowbird_1997_conference_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_,venue_1997_)).
samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_,venue_icml_97_).
samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_,venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_,venue_tech_rep_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_,venue_proceedings_eleventh_conference_computational_learning_theory_)).
neg(samevenue(venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_,venue_unpublished_manuscript_)).
neg(samevenue(venue_tech_rep_,venue_machine_learning_proceedings_fourteenth_international_conference_)).
neg(samevenue(venue_tech_rep_,venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_)).
samevenue(venue_tech_rep_,venue_tech_rep_).
neg(samevenue(venue_tech_rep_,venue_proceedings_eleventh_conference_computational_learning_theory_)).
neg(samevenue(venue_tech_rep_,venue_unpublished_manuscript_)).
neg(samevenue(venue_proceedings_eleventh_conference_computational_learning_theory_,venue_machine_learning_proceedings_eleventh_international_conference_)).
neg(samevenue(venue_proceedings_eleventh_conference_computational_learning_theory_,venue_proceddings_11th_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_eleventh_conference_computational_learning_theory_,venue_proceedings_eleventh_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_eleventh_conference_computational_learning_theory_,venue_proc_11th_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_eleventh_conference_computational_learning_theory_,venue_proc_11th_international_conf_machine_learning_)).
neg(samevenue(venue_proceedings_eleventh_conference_computational_learning_theory_,venue_proceedings_fourteenth_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_eleventh_conference_computational_learning_theory_,venue_machine_learning_proceedings_fourteenth_international_conference_)).
neg(samevenue(venue_proceedings_eleventh_conference_computational_learning_theory_,venue_proceedings_14th_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_eleventh_conference_computational_learning_theory_,venue_proceedings_11th_international_conference_machine_learning_)).
neg(samevenue(venue_proceedings_eleventh_conference_computational_learning_theory_,venue_1997_)).
neg(samevenue(venue_proceedings_eleventh_conference_computational_learning_theory_,venue_icml_97_)).
neg(samevenue(venue_proceedings_eleventh_conference_computational_learning_theory_,venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_)).
samevenue(venue_proceedings_eleventh_conference_computational_learning_theory_,venue_proceedings_eleventh_conference_computational_learning_theory_).
neg(samevenue(venue_proceedings_eleventh_conference_computational_learning_theory_,venue_computation_learning_theory_proceedings_fourth_workshop_)).
neg(samevenue(venue_proceedings_eleventh_conference_computational_learning_theory_,venue_proceedings_fourth_workshop_computational_learning_theory_)).
neg(samevenue(venue_proceedings_eleventh_conference_computational_learning_theory_,venue_tech_rep_)).
neg(samevenue(venue_proceedings_eleventh_conference_computational_learning_theory_,venue_unpublished_manuscript_)).
neg(samevenue(venue_unpublished_manuscript_,venue_machine_learning_proceedings_fourteenth_international_conference_)).
neg(samevenue(venue_unpublished_manuscript_,venue_machine_learning_proceedings_fourteenth_international_conference_icml_97_)).
neg(samevenue(venue_unpublished_manuscript_,venue_tech_rep_)).
neg(samevenue(venue_unpublished_manuscript_,venue_proceedings_eleventh_conference_computational_learning_theory_)).
end(model(m1)).
