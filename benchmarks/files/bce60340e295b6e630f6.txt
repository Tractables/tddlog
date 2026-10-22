0.5::route.
0.0001::drop.
result :- \+route, \+drop.
result_1 :- route.
result_1 :- result.
0.5::route_1.
s2_1 :- route_1, result_1.
s3_1 :- \+route_1, result_1.
0.0001::drop_1.
result_2 :- s3_1, \+drop_1.
result_3 :- s2_1.
result_3 :- result_2.
0.5::route_2.
s2_2 :- route_2, result_3.
s3_2 :- \+route_2, result_3.
0.0001::drop_2.
result_4 :- s3_2, \+drop_2.
result_5 :- s2_2.
result_5 :- result_4.
0.5::route_3.
s2_3 :- route_3, result_5.
s3_3 :- \+route_3, result_5.
0.0001::drop_3.
result_6 :- s3_3, \+drop_3.
result_7 :- s2_3.
result_7 :- result_6.
0.5::route_4.
s2_4 :- route_4, result_7.
s3_4 :- \+route_4, result_7.
0.0001::drop_4.
result_8 :- s3_4, \+drop_4.
result_9 :- s2_4.
result_9 :- result_8.
0.5::route_5.
s2_5 :- route_5, result_9.
s3_5 :- \+route_5, result_9.
0.0001::drop_5.
result_10 :- s3_5, \+drop_5.
result_11 :- s2_5.
result_11 :- result_10.
0.5::route_6.
s2_6 :- route_6, result_11.
s3_6 :- \+route_6, result_11.
0.0001::drop_6.
result_12 :- s3_6, \+drop_6.
result_13 :- s2_6.
result_13 :- result_12.
0.5::route_7.
s2_7 :- route_7, result_13.
s3_7 :- \+route_7, result_13.
0.0001::drop_7.
result_14 :- s3_7, \+drop_7.
result_15 :- s2_7.
result_15 :- result_14.
0.5::route_8.
s2_8 :- route_8, result_15.
s3_8 :- \+route_8, result_15.
0.0001::drop_8.
result_16 :- s3_8, \+drop_8.
result_17 :- s2_8.
result_17 :- result_16.
0.5::route_9.
s2_9 :- route_9, result_17.
s3_9 :- \+route_9, result_17.
0.0001::drop_9.
result_18 :- s3_9, \+drop_9.
result_19 :- s2_9.
result_19 :- result_18.
0.5::route_10.
s2_10 :- route_10, result_19.
s3_10 :- \+route_10, result_19.
0.0001::drop_10.
result_20 :- s3_10, \+drop_10.
result_21 :- s2_10.
result_21 :- result_20.
0.5::route_11.
s2_11 :- route_11, result_21.
s3_11 :- \+route_11, result_21.
0.0001::drop_11.
result_22 :- s3_11, \+drop_11.
result_23 :- s2_11.
result_23 :- result_22.
0.5::route_12.
s2_12 :- route_12, result_23.
s3_12 :- \+route_12, result_23.
0.0001::drop_12.
result_24 :- s3_12, \+drop_12.
result_25 :- s2_12.
result_25 :- result_24.
0.5::route_13.
s2_13 :- route_13, result_25.
s3_13 :- \+route_13, result_25.
0.0001::drop_13.
result_26 :- s3_13, \+drop_13.
result_27 :- s2_13.
result_27 :- result_26.
0.5::route_14.
s2_14 :- route_14, result_27.
s3_14 :- \+route_14, result_27.
0.0001::drop_14.
result_28 :- s3_14, \+drop_14.
result_29 :- s2_14.
result_29 :- result_28.
0.5::route_15.
s2_15 :- route_15, result_29.
s3_15 :- \+route_15, result_29.
0.0001::drop_15.
result_30 :- s3_15, \+drop_15.
result_31 :- s2_15.
result_31 :- result_30.
0.5::route_16.
s2_16 :- route_16, result_31.
s3_16 :- \+route_16, result_31.
0.0001::drop_16.
result_32 :- s3_16, \+drop_16.
result_33 :- s2_16.
result_33 :- result_32.
0.5::route_17.
s2_17 :- route_17, result_33.
s3_17 :- \+route_17, result_33.
0.0001::drop_17.
result_34 :- s3_17, \+drop_17.
result_35 :- s2_17.
result_35 :- result_34.
0.5::route_18.
s2_18 :- route_18, result_35.
s3_18 :- \+route_18, result_35.
0.0001::drop_18.
result_36 :- s3_18, \+drop_18.
result_37 :- s2_18.
result_37 :- result_36.
0.5::route_19.
s2_19 :- route_19, result_37.
s3_19 :- \+route_19, result_37.
0.0001::drop_19.
result_38 :- s3_19, \+drop_19.
result_39 :- s2_19.
result_39 :- result_38.
0.5::route_20.
s2_20 :- route_20, result_39.
s3_20 :- \+route_20, result_39.
0.0001::drop_20.
result_40 :- s3_20, \+drop_20.
result_41 :- s2_20.
result_41 :- result_40.
0.5::route_21.
s2_21 :- route_21, result_41.
s3_21 :- \+route_21, result_41.
0.0001::drop_21.
result_42 :- s3_21, \+drop_21.
result_43 :- s2_21.
result_43 :- result_42.
0.5::route_22.
s2_22 :- route_22, result_43.
s3_22 :- \+route_22, result_43.
0.0001::drop_22.
result_44 :- s3_22, \+drop_22.
result_45 :- s2_22.
result_45 :- result_44.
0.5::route_23.
s2_23 :- route_23, result_45.
s3_23 :- \+route_23, result_45.
0.0001::drop_23.
result_46 :- s3_23, \+drop_23.
result_47 :- s2_23.
result_47 :- result_46.
0.5::route_24.
s2_24 :- route_24, result_47.
s3_24 :- \+route_24, result_47.
0.0001::drop_24.
result_48 :- s3_24, \+drop_24.
result_49 :- s2_24.
result_49 :- result_48.
0.5::route_25.
s2_25 :- route_25, result_49.
s3_25 :- \+route_25, result_49.
0.0001::drop_25.
result_50 :- s3_25, \+drop_25.
result_51 :- s2_25.
result_51 :- result_50.
0.5::route_26.
s2_26 :- route_26, result_51.
s3_26 :- \+route_26, result_51.
0.0001::drop_26.
result_52 :- s3_26, \+drop_26.
result_53 :- s2_26.
result_53 :- result_52.
0.5::route_27.
s2_27 :- route_27, result_53.
s3_27 :- \+route_27, result_53.
0.0001::drop_27.
result_54 :- s3_27, \+drop_27.
result_55 :- s2_27.
result_55 :- result_54.
0.5::route_28.
s2_28 :- route_28, result_55.
s3_28 :- \+route_28, result_55.
0.0001::drop_28.
result_56 :- s3_28, \+drop_28.
result_57 :- s2_28.
result_57 :- result_56.
0.5::route_29.
s2_29 :- route_29, result_57.
s3_29 :- \+route_29, result_57.
0.0001::drop_29.
result_58 :- s3_29, \+drop_29.
result_59 :- s2_29.
result_59 :- result_58.
0.5::route_30.
s2_30 :- route_30, result_59.
s3_30 :- \+route_30, result_59.
0.0001::drop_30.
result_60 :- s3_30, \+drop_30.
result_61 :- s2_30.
result_61 :- result_60.
0.5::route_31.
s2_31 :- route_31, result_61.
s3_31 :- \+route_31, result_61.
0.0001::drop_31.
result_62 :- s3_31, \+drop_31.
result_63 :- s2_31.
result_63 :- result_62.
0.5::route_32.
s2_32 :- route_32, result_63.
s3_32 :- \+route_32, result_63.
0.0001::drop_32.
result_64 :- s3_32, \+drop_32.
result_65 :- s2_32.
result_65 :- result_64.
0.5::route_33.
s2_33 :- route_33, result_65.
s3_33 :- \+route_33, result_65.
0.0001::drop_33.
result_66 :- s3_33, \+drop_33.
result_67 :- s2_33.
result_67 :- result_66.
0.5::route_34.
s2_34 :- route_34, result_67.
s3_34 :- \+route_34, result_67.
0.0001::drop_34.
result_68 :- s3_34, \+drop_34.
result_69 :- s2_34.
result_69 :- result_68.
0.5::route_35.
s2_35 :- route_35, result_69.
s3_35 :- \+route_35, result_69.
0.0001::drop_35.
result_70 :- s3_35, \+drop_35.
result_71 :- s2_35.
result_71 :- result_70.
0.5::route_36.
s2_36 :- route_36, result_71.
s3_36 :- \+route_36, result_71.
0.0001::drop_36.
result_72 :- s3_36, \+drop_36.
result_73 :- s2_36.
result_73 :- result_72.
0.5::route_37.
s2_37 :- route_37, result_73.
s3_37 :- \+route_37, result_73.
0.0001::drop_37.
result_74 :- s3_37, \+drop_37.
result_75 :- s2_37.
result_75 :- result_74.
0.5::route_38.
s2_38 :- route_38, result_75.
s3_38 :- \+route_38, result_75.
0.0001::drop_38.
result_76 :- s3_38, \+drop_38.
result_77 :- s2_38.
result_77 :- result_76.
0.5::route_39.
s2_39 :- route_39, result_77.
s3_39 :- \+route_39, result_77.
0.0001::drop_39.
result_78 :- s3_39, \+drop_39.
result_79 :- s2_39.
result_79 :- result_78.
0.5::route_40.
s2_40 :- route_40, result_79.
s3_40 :- \+route_40, result_79.
0.0001::drop_40.
result_80 :- s3_40, \+drop_40.
result_81 :- s2_40.
result_81 :- result_80.
0.5::route_41.
s2_41 :- route_41, result_81.
s3_41 :- \+route_41, result_81.
0.0001::drop_41.
result_82 :- s3_41, \+drop_41.
result_83 :- s2_41.
result_83 :- result_82.
0.5::route_42.
s2_42 :- route_42, result_83.
s3_42 :- \+route_42, result_83.
0.0001::drop_42.
result_84 :- s3_42, \+drop_42.
result_85 :- s2_42.
result_85 :- result_84.
0.5::route_43.
s2_43 :- route_43, result_85.
s3_43 :- \+route_43, result_85.
0.0001::drop_43.
result_86 :- s3_43, \+drop_43.
result_87 :- s2_43.
result_87 :- result_86.
0.5::route_44.
s2_44 :- route_44, result_87.
s3_44 :- \+route_44, result_87.
0.0001::drop_44.
result_88 :- s3_44, \+drop_44.
result_89 :- s2_44.
result_89 :- result_88.
0.5::route_45.
s2_45 :- route_45, result_89.
s3_45 :- \+route_45, result_89.
0.0001::drop_45.
result_90 :- s3_45, \+drop_45.
result_91 :- s2_45.
result_91 :- result_90.
0.5::route_46.
s2_46 :- route_46, result_91.
s3_46 :- \+route_46, result_91.
0.0001::drop_46.
result_92 :- s3_46, \+drop_46.
result_93 :- s2_46.
result_93 :- result_92.
0.5::route_47.
s2_47 :- route_47, result_93.
s3_47 :- \+route_47, result_93.
0.0001::drop_47.
result_94 :- s3_47, \+drop_47.
result_95 :- s2_47.
result_95 :- result_94.
0.5::route_48.
s2_48 :- route_48, result_95.
s3_48 :- \+route_48, result_95.
0.0001::drop_48.
result_96 :- s3_48, \+drop_48.
result_97 :- s2_48.
result_97 :- result_96.
0.5::route_49.
s2_49 :- route_49, result_97.
s3_49 :- \+route_49, result_97.
0.0001::drop_49.
result_98 :- s3_49, \+drop_49.
result_99 :- s2_49.
result_99 :- result_98.
0.5::route_50.
s2_50 :- route_50, result_99.
s3_50 :- \+route_50, result_99.
0.0001::drop_50.
result_100 :- s3_50, \+drop_50.
result_101 :- s2_50.
result_101 :- result_100.
0.5::route_51.
s2_51 :- route_51, result_101.
s3_51 :- \+route_51, result_101.
0.0001::drop_51.
result_102 :- s3_51, \+drop_51.
result_103 :- s2_51.
result_103 :- result_102.
0.5::route_52.
s2_52 :- route_52, result_103.
s3_52 :- \+route_52, result_103.
0.0001::drop_52.
result_104 :- s3_52, \+drop_52.
result_105 :- s2_52.
result_105 :- result_104.
0.5::route_53.
s2_53 :- route_53, result_105.
s3_53 :- \+route_53, result_105.
0.0001::drop_53.
result_106 :- s3_53, \+drop_53.
result_107 :- s2_53.
result_107 :- result_106.
0.5::route_54.
s2_54 :- route_54, result_107.
s3_54 :- \+route_54, result_107.
0.0001::drop_54.
result_108 :- s3_54, \+drop_54.
result_109 :- s2_54.
result_109 :- result_108.
0.5::route_55.
s2_55 :- route_55, result_109.
s3_55 :- \+route_55, result_109.
0.0001::drop_55.
result_110 :- s3_55, \+drop_55.
result_111 :- s2_55.
result_111 :- result_110.
0.5::route_56.
s2_56 :- route_56, result_111.
s3_56 :- \+route_56, result_111.
0.0001::drop_56.
result_112 :- s3_56, \+drop_56.
result_113 :- s2_56.
result_113 :- result_112.
0.5::route_57.
s2_57 :- route_57, result_113.
s3_57 :- \+route_57, result_113.
0.0001::drop_57.
result_114 :- s3_57, \+drop_57.
result_115 :- s2_57.
result_115 :- result_114.
0.5::route_58.
s2_58 :- route_58, result_115.
s3_58 :- \+route_58, result_115.
0.0001::drop_58.
result_116 :- s3_58, \+drop_58.
result_117 :- s2_58.
result_117 :- result_116.
0.5::route_59.
s2_59 :- route_59, result_117.
s3_59 :- \+route_59, result_117.
0.0001::drop_59.
result_118 :- s3_59, \+drop_59.
result_119 :- s2_59.
result_119 :- result_118.
0.5::route_60.
s2_60 :- route_60, result_119.
s3_60 :- \+route_60, result_119.
0.0001::drop_60.
result_120 :- s3_60, \+drop_60.
result_121 :- s2_60.
result_121 :- result_120.
0.5::route_61.
s2_61 :- route_61, result_121.
s3_61 :- \+route_61, result_121.
0.0001::drop_61.
result_122 :- s3_61, \+drop_61.
result_123 :- s2_61.
result_123 :- result_122.
0.5::route_62.
s2_62 :- route_62, result_123.
s3_62 :- \+route_62, result_123.
0.0001::drop_62.
result_124 :- s3_62, \+drop_62.
result_125 :- s2_62.
result_125 :- result_124.
0.5::route_63.
s2_63 :- route_63, result_125.
s3_63 :- \+route_63, result_125.
0.0001::drop_63.
result_126 :- s3_63, \+drop_63.
result_127 :- s2_63.
result_127 :- result_126.
0.5::route_64.
s2_64 :- route_64, result_127.
s3_64 :- \+route_64, result_127.
0.0001::drop_64.
result_128 :- s3_64, \+drop_64.
result_129 :- s2_64.
result_129 :- result_128.
0.5::route_65.
s2_65 :- route_65, result_129.
s3_65 :- \+route_65, result_129.
0.0001::drop_65.
result_130 :- s3_65, \+drop_65.
result_131 :- s2_65.
result_131 :- result_130.
0.5::route_66.
s2_66 :- route_66, result_131.
s3_66 :- \+route_66, result_131.
0.0001::drop_66.
result_132 :- s3_66, \+drop_66.
result_133 :- s2_66.
result_133 :- result_132.
0.5::route_67.
s2_67 :- route_67, result_133.
s3_67 :- \+route_67, result_133.
0.0001::drop_67.
result_134 :- s3_67, \+drop_67.
result_135 :- s2_67.
result_135 :- result_134.
0.5::route_68.
s2_68 :- route_68, result_135.
s3_68 :- \+route_68, result_135.
0.0001::drop_68.
result_136 :- s3_68, \+drop_68.
result_137 :- s2_68.
result_137 :- result_136.
0.5::route_69.
s2_69 :- route_69, result_137.
s3_69 :- \+route_69, result_137.
0.0001::drop_69.
result_138 :- s3_69, \+drop_69.
result_139 :- s2_69.
result_139 :- result_138.
0.5::route_70.
s2_70 :- route_70, result_139.
s3_70 :- \+route_70, result_139.
0.0001::drop_70.
result_140 :- s3_70, \+drop_70.
result_141 :- s2_70.
result_141 :- result_140.
0.5::route_71.
s2_71 :- route_71, result_141.
s3_71 :- \+route_71, result_141.
0.0001::drop_71.
result_142 :- s3_71, \+drop_71.
result_143 :- s2_71.
result_143 :- result_142.
0.5::route_72.
s2_72 :- route_72, result_143.
s3_72 :- \+route_72, result_143.
0.0001::drop_72.
result_144 :- s3_72, \+drop_72.
result_145 :- s2_72.
result_145 :- result_144.
0.5::route_73.
s2_73 :- route_73, result_145.
s3_73 :- \+route_73, result_145.
0.0001::drop_73.
result_146 :- s3_73, \+drop_73.
result_147 :- s2_73.
result_147 :- result_146.
0.5::route_74.
s2_74 :- route_74, result_147.
s3_74 :- \+route_74, result_147.
0.0001::drop_74.
result_148 :- s3_74, \+drop_74.
result_149 :- s2_74.
result_149 :- result_148.
0.5::route_75.
s2_75 :- route_75, result_149.
s3_75 :- \+route_75, result_149.
0.0001::drop_75.
result_150 :- s3_75, \+drop_75.
result_151 :- s2_75.
result_151 :- result_150.
0.5::route_76.
s2_76 :- route_76, result_151.
s3_76 :- \+route_76, result_151.
0.0001::drop_76.
result_152 :- s3_76, \+drop_76.
result_153 :- s2_76.
result_153 :- result_152.
0.5::route_77.
s2_77 :- route_77, result_153.
s3_77 :- \+route_77, result_153.
0.0001::drop_77.
result_154 :- s3_77, \+drop_77.
result_155 :- s2_77.
result_155 :- result_154.
0.5::route_78.
s2_78 :- route_78, result_155.
s3_78 :- \+route_78, result_155.
0.0001::drop_78.
result_156 :- s3_78, \+drop_78.
result_157 :- s2_78.
result_157 :- result_156.
0.5::route_79.
s2_79 :- route_79, result_157.
s3_79 :- \+route_79, result_157.
0.0001::drop_79.
result_158 :- s3_79, \+drop_79.
result_159 :- s2_79.
result_159 :- result_158.
0.5::route_80.
s2_80 :- route_80, result_159.
s3_80 :- \+route_80, result_159.
0.0001::drop_80.
result_160 :- s3_80, \+drop_80.
result_161 :- s2_80.
result_161 :- result_160.
0.5::route_81.
s2_81 :- route_81, result_161.
s3_81 :- \+route_81, result_161.
0.0001::drop_81.
result_162 :- s3_81, \+drop_81.
result_163 :- s2_81.
result_163 :- result_162.
0.5::route_82.
s2_82 :- route_82, result_163.
s3_82 :- \+route_82, result_163.
0.0001::drop_82.
result_164 :- s3_82, \+drop_82.
result_165 :- s2_82.
result_165 :- result_164.
0.5::route_83.
s2_83 :- route_83, result_165.
s3_83 :- \+route_83, result_165.
0.0001::drop_83.
result_166 :- s3_83, \+drop_83.
result_167 :- s2_83.
result_167 :- result_166.
0.5::route_84.
s2_84 :- route_84, result_167.
s3_84 :- \+route_84, result_167.
0.0001::drop_84.
result_168 :- s3_84, \+drop_84.
result_169 :- s2_84.
result_169 :- result_168.
0.5::route_85.
s2_85 :- route_85, result_169.
s3_85 :- \+route_85, result_169.
0.0001::drop_85.
result_170 :- s3_85, \+drop_85.
result_171 :- s2_85.
result_171 :- result_170.
0.5::route_86.
s2_86 :- route_86, result_171.
s3_86 :- \+route_86, result_171.
0.0001::drop_86.
result_172 :- s3_86, \+drop_86.
result_173 :- s2_86.
result_173 :- result_172.
0.5::route_87.
s2_87 :- route_87, result_173.
s3_87 :- \+route_87, result_173.
0.0001::drop_87.
result_174 :- s3_87, \+drop_87.
result_175 :- s2_87.
result_175 :- result_174.
0.5::route_88.
s2_88 :- route_88, result_175.
s3_88 :- \+route_88, result_175.
0.0001::drop_88.
result_176 :- s3_88, \+drop_88.
result_177 :- s2_88.
result_177 :- result_176.
0.5::route_89.
s2_89 :- route_89, result_177.
s3_89 :- \+route_89, result_177.
0.0001::drop_89.
result_178 :- s3_89, \+drop_89.
result_179 :- s2_89.
result_179 :- result_178.
0.5::route_90.
s2_90 :- route_90, result_179.
s3_90 :- \+route_90, result_179.
0.0001::drop_90.
result_180 :- s3_90, \+drop_90.
result_181 :- s2_90.
result_181 :- result_180.
0.5::route_91.
s2_91 :- route_91, result_181.
s3_91 :- \+route_91, result_181.
0.0001::drop_91.
result_182 :- s3_91, \+drop_91.
result_183 :- s2_91.
result_183 :- result_182.
0.5::route_92.
s2_92 :- route_92, result_183.
s3_92 :- \+route_92, result_183.
0.0001::drop_92.
result_184 :- s3_92, \+drop_92.
result_185 :- s2_92.
result_185 :- result_184.
0.5::route_93.
s2_93 :- route_93, result_185.
s3_93 :- \+route_93, result_185.
0.0001::drop_93.
result_186 :- s3_93, \+drop_93.
result_187 :- s2_93.
result_187 :- result_186.
0.5::route_94.
s2_94 :- route_94, result_187.
s3_94 :- \+route_94, result_187.
0.0001::drop_94.
result_188 :- s3_94, \+drop_94.
result_189 :- s2_94.
result_189 :- result_188.
0.5::route_95.
s2_95 :- route_95, result_189.
s3_95 :- \+route_95, result_189.
0.0001::drop_95.
result_190 :- s3_95, \+drop_95.
result_191 :- s2_95.
result_191 :- result_190.
0.5::route_96.
s2_96 :- route_96, result_191.
s3_96 :- \+route_96, result_191.
0.0001::drop_96.
result_192 :- s3_96, \+drop_96.
result_193 :- s2_96.
result_193 :- result_192.
0.5::route_97.
s2_97 :- route_97, result_193.
s3_97 :- \+route_97, result_193.
0.0001::drop_97.
result_194 :- s3_97, \+drop_97.
result_195 :- s2_97.
result_195 :- result_194.
0.5::route_98.
s2_98 :- route_98, result_195.
s3_98 :- \+route_98, result_195.
0.0001::drop_98.
result_196 :- s3_98, \+drop_98.
result_197 :- s2_98.
result_197 :- result_196.
0.5::route_99.
s2_99 :- route_99, result_197.
s3_99 :- \+route_99, result_197.
0.0001::drop_99.
result_198 :- s3_99, \+drop_99.
result_199 :- s2_99.
result_199 :- result_198.
query(result_199).
