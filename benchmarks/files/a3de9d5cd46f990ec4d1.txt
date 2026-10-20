0.001::result_then_c.
0.5::f.
result_then_1 :- \+result_then_c, f.
result_then_2 :- \+result_then_c, \+f.
0.001::result_then_c_1.
0.5::f_1.
result_then_1_1 :- \+result_then_c_1, f_1.
result_then_2_1 :- \+result_then_c_1, \+f_1.
result_g :- \+result_then_1, result_then_2.
0.5::f_2.
result_g_1 :- \+result_then_1, \+result_then_2.
result_1 :- result_then_1, result_then_1_1.
result_1 :- result_g, f_2.
result_2 :- result_then_1, result_then_2_1.
result_2 :- result_g, \+f_2.
0.001::result_then_c_2.
0.5::f_3.
result_then_1_2 :- \+result_then_c_2, f_3.
result_then_2_2 :- \+result_then_c_2, \+f_3.
result_g_2 :- \+result_1, result_2.
0.5::f_4.
result_g_3 :- \+result_1, \+result_2.
result_1_1 :- result_1, result_then_1_2.
result_1_1 :- result_g_2, f_4.
result_2_1 :- result_1, result_then_2_2.
result_2_1 :- result_g_2, \+f_4.
0.001::result_then_c_3.
0.5::f_5.
result_then_1_3 :- \+result_then_c_3, f_5.
result_then_2_3 :- \+result_then_c_3, \+f_5.
result_g_4 :- \+result_1_1, result_2_1.
0.5::f_6.
result_g_5 :- \+result_1_1, \+result_2_1.
result_1_2 :- result_1_1, result_then_1_3.
result_1_2 :- result_g_4, f_6.
result_2_2 :- result_1_1, result_then_2_3.
result_2_2 :- result_g_4, \+f_6.
0.001::result_then_c_4.
0.5::f_7.
result_then_1_4 :- \+result_then_c_4, f_7.
result_then_2_4 :- \+result_then_c_4, \+f_7.
result_g_6 :- \+result_1_2, result_2_2.
0.5::f_8.
result_g_7 :- \+result_1_2, \+result_2_2.
result_1_3 :- result_1_2, result_then_1_4.
result_1_3 :- result_g_6, f_8.
result_2_3 :- result_1_2, result_then_2_4.
result_2_3 :- result_g_6, \+f_8.
0.001::result_then_c_5.
0.5::f_9.
result_then_1_5 :- \+result_then_c_5, f_9.
result_then_2_5 :- \+result_then_c_5, \+f_9.
result_g_8 :- \+result_1_3, result_2_3.
0.5::f_10.
result_g_9 :- \+result_1_3, \+result_2_3.
result_1_4 :- result_1_3, result_then_1_5.
result_1_4 :- result_g_8, f_10.
result_2_4 :- result_1_3, result_then_2_5.
result_2_4 :- result_g_8, \+f_10.
0.001::result_then_c_6.
0.5::f_11.
result_then_1_6 :- \+result_then_c_6, f_11.
result_then_2_6 :- \+result_then_c_6, \+f_11.
result_g_10 :- \+result_1_4, result_2_4.
0.5::f_12.
result_g_11 :- \+result_1_4, \+result_2_4.
result_1_5 :- result_1_4, result_then_1_6.
result_1_5 :- result_g_10, f_12.
result_2_5 :- result_1_4, result_then_2_6.
result_2_5 :- result_g_10, \+f_12.
0.001::result_then_c_7.
0.5::f_13.
result_then_1_7 :- \+result_then_c_7, f_13.
result_then_2_7 :- \+result_then_c_7, \+f_13.
result_g_12 :- \+result_1_5, result_2_5.
0.5::f_14.
result_g_13 :- \+result_1_5, \+result_2_5.
result_1_6 :- result_1_5, result_then_1_7.
result_1_6 :- result_g_12, f_14.
result_2_6 :- result_1_5, result_then_2_7.
result_2_6 :- result_g_12, \+f_14.
0.001::result_then_c_8.
0.5::f_15.
result_then_1_8 :- \+result_then_c_8, f_15.
result_then_2_8 :- \+result_then_c_8, \+f_15.
result_g_14 :- \+result_1_6, result_2_6.
0.5::f_16.
result_g_15 :- \+result_1_6, \+result_2_6.
result_1_7 :- result_1_6, result_then_1_8.
result_1_7 :- result_g_14, f_16.
result_2_7 :- result_1_6, result_then_2_8.
result_2_7 :- result_g_14, \+f_16.
0.001::result_then_c_9.
0.5::f_17.
result_then_1_9 :- \+result_then_c_9, f_17.
result_then_2_9 :- \+result_then_c_9, \+f_17.
result_g_16 :- \+result_1_7, result_2_7.
0.5::f_18.
result_g_17 :- \+result_1_7, \+result_2_7.
result_1_8 :- result_1_7, result_then_1_9.
result_1_8 :- result_g_16, f_18.
result_2_8 :- result_1_7, result_then_2_9.
result_2_8 :- result_g_16, \+f_18.
0.001::result_then_c_10.
0.5::f_19.
result_then_1_10 :- \+result_then_c_10, f_19.
result_then_2_10 :- \+result_then_c_10, \+f_19.
result_g_18 :- \+result_1_8, result_2_8.
0.5::f_20.
result_g_19 :- \+result_1_8, \+result_2_8.
result_1_9 :- result_1_8, result_then_1_10.
result_1_9 :- result_g_18, f_20.
result_2_9 :- result_1_8, result_then_2_10.
result_2_9 :- result_g_18, \+f_20.
0.001::result_then_c_11.
0.5::f_21.
result_then_1_11 :- \+result_then_c_11, f_21.
result_then_2_11 :- \+result_then_c_11, \+f_21.
result_g_20 :- \+result_1_9, result_2_9.
0.5::f_22.
result_g_21 :- \+result_1_9, \+result_2_9.
result_1_10 :- result_1_9, result_then_1_11.
result_1_10 :- result_g_20, f_22.
result_2_10 :- result_1_9, result_then_2_11.
result_2_10 :- result_g_20, \+f_22.
0.001::result_then_c_12.
0.5::f_23.
result_then_1_12 :- \+result_then_c_12, f_23.
result_then_2_12 :- \+result_then_c_12, \+f_23.
result_g_22 :- \+result_1_10, result_2_10.
0.5::f_24.
result_g_23 :- \+result_1_10, \+result_2_10.
result_1_11 :- result_1_10, result_then_1_12.
result_1_11 :- result_g_22, f_24.
result_2_11 :- result_1_10, result_then_2_12.
result_2_11 :- result_g_22, \+f_24.
0.001::result_then_c_13.
0.5::f_25.
result_then_1_13 :- \+result_then_c_13, f_25.
result_then_2_13 :- \+result_then_c_13, \+f_25.
result_g_24 :- \+result_1_11, result_2_11.
0.5::f_26.
result_g_25 :- \+result_1_11, \+result_2_11.
result_1_12 :- result_1_11, result_then_1_13.
result_1_12 :- result_g_24, f_26.
result_2_12 :- result_1_11, result_then_2_13.
result_2_12 :- result_g_24, \+f_26.
0.001::result_then_c_14.
0.5::f_27.
result_then_1_14 :- \+result_then_c_14, f_27.
result_then_2_14 :- \+result_then_c_14, \+f_27.
result_g_26 :- \+result_1_12, result_2_12.
0.5::f_28.
result_g_27 :- \+result_1_12, \+result_2_12.
result_1_13 :- result_1_12, result_then_1_14.
result_1_13 :- result_g_26, f_28.
result_2_13 :- result_1_12, result_then_2_14.
result_2_13 :- result_g_26, \+f_28.
0.001::result_then_c_15.
0.5::f_29.
result_then_1_15 :- \+result_then_c_15, f_29.
result_then_2_15 :- \+result_then_c_15, \+f_29.
result_g_28 :- \+result_1_13, result_2_13.
0.5::f_30.
result_g_29 :- \+result_1_13, \+result_2_13.
result_1_14 :- result_1_13, result_then_1_15.
result_1_14 :- result_g_28, f_30.
result_2_14 :- result_1_13, result_then_2_15.
result_2_14 :- result_g_28, \+f_30.
0.001::result_then_c_16.
0.5::f_31.
result_then_1_16 :- \+result_then_c_16, f_31.
result_then_2_16 :- \+result_then_c_16, \+f_31.
result_g_30 :- \+result_1_14, result_2_14.
0.5::f_32.
result_g_31 :- \+result_1_14, \+result_2_14.
result_1_15 :- result_1_14, result_then_1_16.
result_1_15 :- result_g_30, f_32.
result_2_15 :- result_1_14, result_then_2_16.
result_2_15 :- result_g_30, \+f_32.
0.001::result_then_c_17.
0.5::f_33.
result_then_1_17 :- \+result_then_c_17, f_33.
result_then_2_17 :- \+result_then_c_17, \+f_33.
result_g_32 :- \+result_1_15, result_2_15.
0.5::f_34.
result_g_33 :- \+result_1_15, \+result_2_15.
result_1_16 :- result_1_15, result_then_1_17.
result_1_16 :- result_g_32, f_34.
result_2_16 :- result_1_15, result_then_2_17.
result_2_16 :- result_g_32, \+f_34.
0.001::result_then_c_18.
0.5::f_35.
result_then_1_18 :- \+result_then_c_18, f_35.
result_then_2_18 :- \+result_then_c_18, \+f_35.
result_g_34 :- \+result_1_16, result_2_16.
0.5::f_36.
result_g_35 :- \+result_1_16, \+result_2_16.
result_1_17 :- result_1_16, result_then_1_18.
result_1_17 :- result_g_34, f_36.
result_2_17 :- result_1_16, result_then_2_18.
result_2_17 :- result_g_34, \+f_36.
0.001::result_then_c_19.
0.5::f_37.
result_then_1_19 :- \+result_then_c_19, f_37.
result_then_2_19 :- \+result_then_c_19, \+f_37.
result_g_36 :- \+result_1_17, result_2_17.
0.5::f_38.
result_g_37 :- \+result_1_17, \+result_2_17.
result_1_18 :- result_1_17, result_then_1_19.
result_1_18 :- result_g_36, f_38.
result_2_18 :- result_1_17, result_then_2_19.
result_2_18 :- result_g_36, \+f_38.
0.001::result_then_c_20.
0.5::f_39.
result_then_1_20 :- \+result_then_c_20, f_39.
result_then_2_20 :- \+result_then_c_20, \+f_39.
result_g_38 :- \+result_1_18, result_2_18.
0.5::f_40.
result_g_39 :- \+result_1_18, \+result_2_18.
result_1_19 :- result_1_18, result_then_1_20.
result_1_19 :- result_g_38, f_40.
result_2_19 :- result_1_18, result_then_2_20.
result_2_19 :- result_g_38, \+f_40.
0.001::result_then_c_21.
0.5::f_41.
result_then_1_21 :- \+result_then_c_21, f_41.
result_then_2_21 :- \+result_then_c_21, \+f_41.
result_g_40 :- \+result_1_19, result_2_19.
0.5::f_42.
result_g_41 :- \+result_1_19, \+result_2_19.
result_1_20 :- result_1_19, result_then_1_21.
result_1_20 :- result_g_40, f_42.
result_2_20 :- result_1_19, result_then_2_21.
result_2_20 :- result_g_40, \+f_42.
0.001::result_then_c_22.
0.5::f_43.
result_then_1_22 :- \+result_then_c_22, f_43.
result_then_2_22 :- \+result_then_c_22, \+f_43.
result_g_42 :- \+result_1_20, result_2_20.
0.5::f_44.
result_g_43 :- \+result_1_20, \+result_2_20.
result_1_21 :- result_1_20, result_then_1_22.
result_1_21 :- result_g_42, f_44.
result_2_21 :- result_1_20, result_then_2_22.
result_2_21 :- result_g_42, \+f_44.
0.001::result_then_c_23.
0.5::f_45.
result_then_1_23 :- \+result_then_c_23, f_45.
result_then_2_23 :- \+result_then_c_23, \+f_45.
result_g_44 :- \+result_1_21, result_2_21.
0.5::f_46.
result_g_45 :- \+result_1_21, \+result_2_21.
result_1_22 :- result_1_21, result_then_1_23.
result_1_22 :- result_g_44, f_46.
result_2_22 :- result_1_21, result_then_2_23.
result_2_22 :- result_g_44, \+f_46.
0.001::result_then_c_24.
0.5::f_47.
result_then_1_24 :- \+result_then_c_24, f_47.
result_then_2_24 :- \+result_then_c_24, \+f_47.
result_g_46 :- \+result_1_22, result_2_22.
0.5::f_48.
result_g_47 :- \+result_1_22, \+result_2_22.
result_1_23 :- result_1_22, result_then_1_24.
result_1_23 :- result_g_46, f_48.
result_2_23 :- result_1_22, result_then_2_24.
result_2_23 :- result_g_46, \+f_48.
0.001::result_then_c_25.
0.5::f_49.
result_then_1_25 :- \+result_then_c_25, f_49.
result_then_2_25 :- \+result_then_c_25, \+f_49.
result_g_48 :- \+result_1_23, result_2_23.
0.5::f_50.
result_g_49 :- \+result_1_23, \+result_2_23.
result_1_24 :- result_1_23, result_then_1_25.
result_1_24 :- result_g_48, f_50.
result_2_24 :- result_1_23, result_then_2_25.
result_2_24 :- result_g_48, \+f_50.
0.001::result_then_c_26.
0.5::f_51.
result_then_1_26 :- \+result_then_c_26, f_51.
result_then_2_26 :- \+result_then_c_26, \+f_51.
result_g_50 :- \+result_1_24, result_2_24.
0.5::f_52.
result_g_51 :- \+result_1_24, \+result_2_24.
result_1_25 :- result_1_24, result_then_1_26.
result_1_25 :- result_g_50, f_52.
result_2_25 :- result_1_24, result_then_2_26.
result_2_25 :- result_g_50, \+f_52.
0.001::result_then_c_27.
0.5::f_53.
result_then_1_27 :- \+result_then_c_27, f_53.
result_then_2_27 :- \+result_then_c_27, \+f_53.
result_g_52 :- \+result_1_25, result_2_25.
0.5::f_54.
result_g_53 :- \+result_1_25, \+result_2_25.
result_1_26 :- result_1_25, result_then_1_27.
result_1_26 :- result_g_52, f_54.
result_2_26 :- result_1_25, result_then_2_27.
result_2_26 :- result_g_52, \+f_54.
0.001::result_then_c_28.
0.5::f_55.
result_then_1_28 :- \+result_then_c_28, f_55.
result_then_2_28 :- \+result_then_c_28, \+f_55.
result_g_54 :- \+result_1_26, result_2_26.
0.5::f_56.
result_g_55 :- \+result_1_26, \+result_2_26.
result_1_27 :- result_1_26, result_then_1_28.
result_1_27 :- result_g_54, f_56.
result_2_27 :- result_1_26, result_then_2_28.
result_2_27 :- result_g_54, \+f_56.
0.001::result_then_c_29.
0.5::f_57.
result_then_1_29 :- \+result_then_c_29, f_57.
result_then_2_29 :- \+result_then_c_29, \+f_57.
result_g_56 :- \+result_1_27, result_2_27.
0.5::f_58.
result_g_57 :- \+result_1_27, \+result_2_27.
result_1_28 :- result_1_27, result_then_1_29.
result_1_28 :- result_g_56, f_58.
result_2_28 :- result_1_27, result_then_2_29.
result_2_28 :- result_g_56, \+f_58.
0.001::result_then_c_30.
0.5::f_59.
result_then_1_30 :- \+result_then_c_30, f_59.
result_then_2_30 :- \+result_then_c_30, \+f_59.
result_g_58 :- \+result_1_28, result_2_28.
0.5::f_60.
result_g_59 :- \+result_1_28, \+result_2_28.
result_1_29 :- result_1_28, result_then_1_30.
result_1_29 :- result_g_58, f_60.
result_2_29 :- result_1_28, result_then_2_30.
result_2_29 :- result_g_58, \+f_60.
0.001::result_then_c_31.
0.5::f_61.
result_then_1_31 :- \+result_then_c_31, f_61.
result_then_2_31 :- \+result_then_c_31, \+f_61.
result_g_60 :- \+result_1_29, result_2_29.
0.5::f_62.
result_g_61 :- \+result_1_29, \+result_2_29.
result_1_30 :- result_1_29, result_then_1_31.
result_1_30 :- result_g_60, f_62.
result_2_30 :- result_1_29, result_then_2_31.
result_2_30 :- result_g_60, \+f_62.
0.001::result_then_c_32.
0.5::f_63.
result_then_1_32 :- \+result_then_c_32, f_63.
result_then_2_32 :- \+result_then_c_32, \+f_63.
result_g_62 :- \+result_1_30, result_2_30.
0.5::f_64.
result_g_63 :- \+result_1_30, \+result_2_30.
result_1_31 :- result_1_30, result_then_1_32.
result_1_31 :- result_g_62, f_64.
result_2_31 :- result_1_30, result_then_2_32.
result_2_31 :- result_g_62, \+f_64.
0.001::result_then_c_33.
0.5::f_65.
result_then_1_33 :- \+result_then_c_33, f_65.
result_then_2_33 :- \+result_then_c_33, \+f_65.
result_g_64 :- \+result_1_31, result_2_31.
0.5::f_66.
result_g_65 :- \+result_1_31, \+result_2_31.
result_1_32 :- result_1_31, result_then_1_33.
result_1_32 :- result_g_64, f_66.
result_2_32 :- result_1_31, result_then_2_33.
result_2_32 :- result_g_64, \+f_66.
0.001::result_then_c_34.
0.5::f_67.
result_then_1_34 :- \+result_then_c_34, f_67.
result_then_2_34 :- \+result_then_c_34, \+f_67.
result_g_66 :- \+result_1_32, result_2_32.
0.5::f_68.
result_g_67 :- \+result_1_32, \+result_2_32.
result_1_33 :- result_1_32, result_then_1_34.
result_1_33 :- result_g_66, f_68.
result_2_33 :- result_1_32, result_then_2_34.
result_2_33 :- result_g_66, \+f_68.
0.001::result_then_c_35.
0.5::f_69.
result_then_1_35 :- \+result_then_c_35, f_69.
result_then_2_35 :- \+result_then_c_35, \+f_69.
result_g_68 :- \+result_1_33, result_2_33.
0.5::f_70.
result_g_69 :- \+result_1_33, \+result_2_33.
result_1_34 :- result_1_33, result_then_1_35.
result_1_34 :- result_g_68, f_70.
result_2_34 :- result_1_33, result_then_2_35.
result_2_34 :- result_g_68, \+f_70.
0.001::result_then_c_36.
0.5::f_71.
result_then_1_36 :- \+result_then_c_36, f_71.
result_then_2_36 :- \+result_then_c_36, \+f_71.
result_g_70 :- \+result_1_34, result_2_34.
0.5::f_72.
result_g_71 :- \+result_1_34, \+result_2_34.
result_1_35 :- result_1_34, result_then_1_36.
result_1_35 :- result_g_70, f_72.
result_2_35 :- result_1_34, result_then_2_36.
result_2_35 :- result_g_70, \+f_72.
0.001::result_then_c_37.
0.5::f_73.
result_then_1_37 :- \+result_then_c_37, f_73.
result_then_2_37 :- \+result_then_c_37, \+f_73.
result_g_72 :- \+result_1_35, result_2_35.
0.5::f_74.
result_g_73 :- \+result_1_35, \+result_2_35.
result_1_36 :- result_1_35, result_then_1_37.
result_1_36 :- result_g_72, f_74.
result_2_36 :- result_1_35, result_then_2_37.
result_2_36 :- result_g_72, \+f_74.
0.001::result_then_c_38.
0.5::f_75.
result_then_1_38 :- \+result_then_c_38, f_75.
result_then_2_38 :- \+result_then_c_38, \+f_75.
result_g_74 :- \+result_1_36, result_2_36.
0.5::f_76.
result_g_75 :- \+result_1_36, \+result_2_36.
result_1_37 :- result_1_36, result_then_1_38.
result_1_37 :- result_g_74, f_76.
result_2_37 :- result_1_36, result_then_2_38.
result_2_37 :- result_g_74, \+f_76.
0.001::result_then_c_39.
0.5::f_77.
result_then_1_39 :- \+result_then_c_39, f_77.
result_then_2_39 :- \+result_then_c_39, \+f_77.
result_g_76 :- \+result_1_37, result_2_37.
0.5::f_78.
result_g_77 :- \+result_1_37, \+result_2_37.
result_1_38 :- result_1_37, result_then_1_39.
result_1_38 :- result_g_76, f_78.
result_2_38 :- result_1_37, result_then_2_39.
result_2_38 :- result_g_76, \+f_78.
0.001::result_then_c_40.
0.5::f_79.
result_then_1_40 :- \+result_then_c_40, f_79.
result_then_2_40 :- \+result_then_c_40, \+f_79.
result_g_78 :- \+result_1_38, result_2_38.
0.5::f_80.
result_g_79 :- \+result_1_38, \+result_2_38.
result_1_39 :- result_1_38, result_then_1_40.
result_1_39 :- result_g_78, f_80.
result_2_39 :- result_1_38, result_then_2_40.
result_2_39 :- result_g_78, \+f_80.
0.001::result_then_c_41.
0.5::f_81.
result_then_1_41 :- \+result_then_c_41, f_81.
result_then_2_41 :- \+result_then_c_41, \+f_81.
result_g_80 :- \+result_1_39, result_2_39.
0.5::f_82.
result_g_81 :- \+result_1_39, \+result_2_39.
result_1_40 :- result_1_39, result_then_1_41.
result_1_40 :- result_g_80, f_82.
result_2_40 :- result_1_39, result_then_2_41.
result_2_40 :- result_g_80, \+f_82.
0.001::result_then_c_42.
0.5::f_83.
result_then_1_42 :- \+result_then_c_42, f_83.
result_then_2_42 :- \+result_then_c_42, \+f_83.
result_g_82 :- \+result_1_40, result_2_40.
0.5::f_84.
result_g_83 :- \+result_1_40, \+result_2_40.
result_1_41 :- result_1_40, result_then_1_42.
result_1_41 :- result_g_82, f_84.
result_2_41 :- result_1_40, result_then_2_42.
result_2_41 :- result_g_82, \+f_84.
0.001::result_then_c_43.
0.5::f_85.
result_then_1_43 :- \+result_then_c_43, f_85.
result_then_2_43 :- \+result_then_c_43, \+f_85.
result_g_84 :- \+result_1_41, result_2_41.
0.5::f_86.
result_g_85 :- \+result_1_41, \+result_2_41.
result_1_42 :- result_1_41, result_then_1_43.
result_1_42 :- result_g_84, f_86.
result_2_42 :- result_1_41, result_then_2_43.
result_2_42 :- result_g_84, \+f_86.
0.001::result_then_c_44.
0.5::f_87.
result_then_1_44 :- \+result_then_c_44, f_87.
result_then_2_44 :- \+result_then_c_44, \+f_87.
result_g_86 :- \+result_1_42, result_2_42.
0.5::f_88.
result_g_87 :- \+result_1_42, \+result_2_42.
result_1_43 :- result_1_42, result_then_1_44.
result_1_43 :- result_g_86, f_88.
result_2_43 :- result_1_42, result_then_2_44.
result_2_43 :- result_g_86, \+f_88.
0.001::result_then_c_45.
0.5::f_89.
result_then_1_45 :- \+result_then_c_45, f_89.
result_then_2_45 :- \+result_then_c_45, \+f_89.
result_g_88 :- \+result_1_43, result_2_43.
0.5::f_90.
result_g_89 :- \+result_1_43, \+result_2_43.
result_1_44 :- result_1_43, result_then_1_45.
result_1_44 :- result_g_88, f_90.
result_2_44 :- result_1_43, result_then_2_45.
result_2_44 :- result_g_88, \+f_90.
0.001::result_then_c_46.
0.5::f_91.
result_then_1_46 :- \+result_then_c_46, f_91.
result_then_2_46 :- \+result_then_c_46, \+f_91.
result_g_90 :- \+result_1_44, result_2_44.
0.5::f_92.
result_g_91 :- \+result_1_44, \+result_2_44.
result_1_45 :- result_1_44, result_then_1_46.
result_1_45 :- result_g_90, f_92.
result_2_45 :- result_1_44, result_then_2_46.
result_2_45 :- result_g_90, \+f_92.
0.001::result_then_c_47.
0.5::f_93.
result_then_1_47 :- \+result_then_c_47, f_93.
result_then_2_47 :- \+result_then_c_47, \+f_93.
result_g_92 :- \+result_1_45, result_2_45.
0.5::f_94.
result_g_93 :- \+result_1_45, \+result_2_45.
result_1_46 :- result_1_45, result_then_1_47.
result_1_46 :- result_g_92, f_94.
result_2_46 :- result_1_45, result_then_2_47.
result_2_46 :- result_g_92, \+f_94.
0.001::result_then_c_48.
0.5::f_95.
result_then_1_48 :- \+result_then_c_48, f_95.
result_then_2_48 :- \+result_then_c_48, \+f_95.
result_g_94 :- \+result_1_46, result_2_46.
0.5::f_96.
result_g_95 :- \+result_1_46, \+result_2_46.
result_1_47 :- result_1_46, result_then_1_48.
result_1_47 :- result_g_94, f_96.
result_2_47 :- result_1_46, result_then_2_48.
result_2_47 :- result_g_94, \+f_96.
0.001::result_then_c_49.
0.5::f_97.
result_then_1_49 :- \+result_then_c_49, f_97.
result_then_2_49 :- \+result_then_c_49, \+f_97.
result_g_96 :- \+result_1_47, result_2_47.
0.5::f_98.
result_g_97 :- \+result_1_47, \+result_2_47.
result_1_48 :- result_1_47, result_then_1_49.
result_1_48 :- result_g_96, f_98.
result_2_48 :- result_1_47, result_then_2_49.
result_2_48 :- result_g_96, \+f_98.
0.001::result_then_c_50.
0.5::f_99.
result_then_1_50 :- \+result_then_c_50, f_99.
result_then_2_50 :- \+result_then_c_50, \+f_99.
result_g_98 :- \+result_1_48, result_2_48.
0.5::f_100.
result_g_99 :- \+result_1_48, \+result_2_48.
result_1_49 :- result_1_48, result_then_1_50.
result_1_49 :- result_g_98, f_100.
result_2_49 :- result_1_48, result_then_2_50.
result_2_49 :- result_g_98, \+f_100.
0.001::result_then_c_51.
0.5::f_101.
result_then_1_51 :- \+result_then_c_51, f_101.
result_then_2_51 :- \+result_then_c_51, \+f_101.
result_g_100 :- \+result_1_49, result_2_49.
0.5::f_102.
result_g_101 :- \+result_1_49, \+result_2_49.
result_1_50 :- result_1_49, result_then_1_51.
result_1_50 :- result_g_100, f_102.
result_2_50 :- result_1_49, result_then_2_51.
result_2_50 :- result_g_100, \+f_102.
0.001::result_then_c_52.
0.5::f_103.
result_then_1_52 :- \+result_then_c_52, f_103.
result_then_2_52 :- \+result_then_c_52, \+f_103.
result_g_102 :- \+result_1_50, result_2_50.
0.5::f_104.
result_g_103 :- \+result_1_50, \+result_2_50.
result_1_51 :- result_1_50, result_then_1_52.
result_1_51 :- result_g_102, f_104.
result_2_51 :- result_1_50, result_then_2_52.
result_2_51 :- result_g_102, \+f_104.
0.001::result_then_c_53.
0.5::f_105.
result_then_1_53 :- \+result_then_c_53, f_105.
result_then_2_53 :- \+result_then_c_53, \+f_105.
result_g_104 :- \+result_1_51, result_2_51.
0.5::f_106.
result_g_105 :- \+result_1_51, \+result_2_51.
result_1_52 :- result_1_51, result_then_1_53.
result_1_52 :- result_g_104, f_106.
result_2_52 :- result_1_51, result_then_2_53.
result_2_52 :- result_g_104, \+f_106.
0.001::result_then_c_54.
0.5::f_107.
result_then_1_54 :- \+result_then_c_54, f_107.
result_then_2_54 :- \+result_then_c_54, \+f_107.
result_g_106 :- \+result_1_52, result_2_52.
0.5::f_108.
result_g_107 :- \+result_1_52, \+result_2_52.
result_1_53 :- result_1_52, result_then_1_54.
result_1_53 :- result_g_106, f_108.
result_2_53 :- result_1_52, result_then_2_54.
result_2_53 :- result_g_106, \+f_108.
0.001::result_then_c_55.
0.5::f_109.
result_then_1_55 :- \+result_then_c_55, f_109.
result_then_2_55 :- \+result_then_c_55, \+f_109.
result_g_108 :- \+result_1_53, result_2_53.
0.5::f_110.
result_g_109 :- \+result_1_53, \+result_2_53.
result_1_54 :- result_1_53, result_then_1_55.
result_1_54 :- result_g_108, f_110.
result_2_54 :- result_1_53, result_then_2_55.
result_2_54 :- result_g_108, \+f_110.
0.001::result_then_c_56.
0.5::f_111.
result_then_1_56 :- \+result_then_c_56, f_111.
result_then_2_56 :- \+result_then_c_56, \+f_111.
result_g_110 :- \+result_1_54, result_2_54.
0.5::f_112.
result_g_111 :- \+result_1_54, \+result_2_54.
result_1_55 :- result_1_54, result_then_1_56.
result_1_55 :- result_g_110, f_112.
result_2_55 :- result_1_54, result_then_2_56.
result_2_55 :- result_g_110, \+f_112.
0.001::result_then_c_57.
0.5::f_113.
result_then_1_57 :- \+result_then_c_57, f_113.
result_then_2_57 :- \+result_then_c_57, \+f_113.
result_g_112 :- \+result_1_55, result_2_55.
0.5::f_114.
result_g_113 :- \+result_1_55, \+result_2_55.
result_1_56 :- result_1_55, result_then_1_57.
result_1_56 :- result_g_112, f_114.
result_2_56 :- result_1_55, result_then_2_57.
result_2_56 :- result_g_112, \+f_114.
0.001::result_then_c_58.
0.5::f_115.
result_then_1_58 :- \+result_then_c_58, f_115.
result_then_2_58 :- \+result_then_c_58, \+f_115.
result_g_114 :- \+result_1_56, result_2_56.
0.5::f_116.
result_g_115 :- \+result_1_56, \+result_2_56.
result_1_57 :- result_1_56, result_then_1_58.
result_1_57 :- result_g_114, f_116.
result_2_57 :- result_1_56, result_then_2_58.
result_2_57 :- result_g_114, \+f_116.
0.001::result_then_c_59.
0.5::f_117.
result_then_1_59 :- \+result_then_c_59, f_117.
result_then_2_59 :- \+result_then_c_59, \+f_117.
result_g_116 :- \+result_1_57, result_2_57.
0.5::f_118.
result_g_117 :- \+result_1_57, \+result_2_57.
result_1_58 :- result_1_57, result_then_1_59.
result_1_58 :- result_g_116, f_118.
result_2_58 :- result_1_57, result_then_2_59.
result_2_58 :- result_g_116, \+f_118.
0.001::result_then_c_60.
0.5::f_119.
result_then_1_60 :- \+result_then_c_60, f_119.
result_then_2_60 :- \+result_then_c_60, \+f_119.
result_g_118 :- \+result_1_58, result_2_58.
0.5::f_120.
result_g_119 :- \+result_1_58, \+result_2_58.
result_1_59 :- result_1_58, result_then_1_60.
result_1_59 :- result_g_118, f_120.
result_2_59 :- result_1_58, result_then_2_60.
result_2_59 :- result_g_118, \+f_120.
0.001::result_then_c_61.
0.5::f_121.
result_then_1_61 :- \+result_then_c_61, f_121.
result_then_2_61 :- \+result_then_c_61, \+f_121.
result_g_120 :- \+result_1_59, result_2_59.
0.5::f_122.
result_g_121 :- \+result_1_59, \+result_2_59.
result_1_60 :- result_1_59, result_then_1_61.
result_1_60 :- result_g_120, f_122.
result_2_60 :- result_1_59, result_then_2_61.
result_2_60 :- result_g_120, \+f_122.
0.001::result_then_c_62.
0.5::f_123.
result_then_1_62 :- \+result_then_c_62, f_123.
result_then_2_62 :- \+result_then_c_62, \+f_123.
result_g_122 :- \+result_1_60, result_2_60.
0.5::f_124.
result_g_123 :- \+result_1_60, \+result_2_60.
result_1_61 :- result_1_60, result_then_1_62.
result_1_61 :- result_g_122, f_124.
result_2_61 :- result_1_60, result_then_2_62.
result_2_61 :- result_g_122, \+f_124.
0.001::result_then_c_63.
0.5::f_125.
result_then_1_63 :- \+result_then_c_63, f_125.
result_then_2_63 :- \+result_then_c_63, \+f_125.
result_g_124 :- \+result_1_61, result_2_61.
0.5::f_126.
result_g_125 :- \+result_1_61, \+result_2_61.
result_1_62 :- result_1_61, result_then_1_63.
result_1_62 :- result_g_124, f_126.
result_2_62 :- result_1_61, result_then_2_63.
result_2_62 :- result_g_124, \+f_126.
0.001::result_then_c_64.
0.5::f_127.
result_then_1_64 :- \+result_then_c_64, f_127.
result_then_2_64 :- \+result_then_c_64, \+f_127.
result_g_126 :- \+result_1_62, result_2_62.
0.5::f_128.
result_g_127 :- \+result_1_62, \+result_2_62.
result_1_63 :- result_1_62, result_then_1_64.
result_1_63 :- result_g_126, f_128.
result_2_63 :- result_1_62, result_then_2_64.
result_2_63 :- result_g_126, \+f_128.
0.001::result_then_c_65.
0.5::f_129.
result_then_1_65 :- \+result_then_c_65, f_129.
result_then_2_65 :- \+result_then_c_65, \+f_129.
result_g_128 :- \+result_1_63, result_2_63.
0.5::f_130.
result_g_129 :- \+result_1_63, \+result_2_63.
result_1_64 :- result_1_63, result_then_1_65.
result_1_64 :- result_g_128, f_130.
result_2_64 :- result_1_63, result_then_2_65.
result_2_64 :- result_g_128, \+f_130.
0.001::result_then_c_66.
0.5::f_131.
result_then_1_66 :- \+result_then_c_66, f_131.
result_then_2_66 :- \+result_then_c_66, \+f_131.
result_g_130 :- \+result_1_64, result_2_64.
0.5::f_132.
result_g_131 :- \+result_1_64, \+result_2_64.
result_1_65 :- result_1_64, result_then_1_66.
result_1_65 :- result_g_130, f_132.
result_2_65 :- result_1_64, result_then_2_66.
result_2_65 :- result_g_130, \+f_132.
0.001::result_then_c_67.
0.5::f_133.
result_then_1_67 :- \+result_then_c_67, f_133.
result_then_2_67 :- \+result_then_c_67, \+f_133.
result_g_132 :- \+result_1_65, result_2_65.
0.5::f_134.
result_g_133 :- \+result_1_65, \+result_2_65.
result_1_66 :- result_1_65, result_then_1_67.
result_1_66 :- result_g_132, f_134.
result_2_66 :- result_1_65, result_then_2_67.
result_2_66 :- result_g_132, \+f_134.
0.001::result_then_c_68.
0.5::f_135.
result_then_1_68 :- \+result_then_c_68, f_135.
result_then_2_68 :- \+result_then_c_68, \+f_135.
result_g_134 :- \+result_1_66, result_2_66.
0.5::f_136.
result_g_135 :- \+result_1_66, \+result_2_66.
result_1_67 :- result_1_66, result_then_1_68.
result_1_67 :- result_g_134, f_136.
result_2_67 :- result_1_66, result_then_2_68.
result_2_67 :- result_g_134, \+f_136.
0.001::result_then_c_69.
0.5::f_137.
result_then_1_69 :- \+result_then_c_69, f_137.
result_then_2_69 :- \+result_then_c_69, \+f_137.
result_g_136 :- \+result_1_67, result_2_67.
0.5::f_138.
result_g_137 :- \+result_1_67, \+result_2_67.
result_1_68 :- result_1_67, result_then_1_69.
result_1_68 :- result_g_136, f_138.
result_2_68 :- result_1_67, result_then_2_69.
result_2_68 :- result_g_136, \+f_138.
0.001::result_then_c_70.
0.5::f_139.
result_then_1_70 :- \+result_then_c_70, f_139.
result_then_2_70 :- \+result_then_c_70, \+f_139.
result_g_138 :- \+result_1_68, result_2_68.
0.5::f_140.
result_g_139 :- \+result_1_68, \+result_2_68.
result_1_69 :- result_1_68, result_then_1_70.
result_1_69 :- result_g_138, f_140.
result_2_69 :- result_1_68, result_then_2_70.
result_2_69 :- result_g_138, \+f_140.
0.001::result_then_c_71.
0.5::f_141.
result_then_1_71 :- \+result_then_c_71, f_141.
result_then_2_71 :- \+result_then_c_71, \+f_141.
result_g_140 :- \+result_1_69, result_2_69.
0.5::f_142.
result_g_141 :- \+result_1_69, \+result_2_69.
result_1_70 :- result_1_69, result_then_1_71.
result_1_70 :- result_g_140, f_142.
result_2_70 :- result_1_69, result_then_2_71.
result_2_70 :- result_g_140, \+f_142.
0.001::result_then_c_72.
0.5::f_143.
result_then_1_72 :- \+result_then_c_72, f_143.
result_then_2_72 :- \+result_then_c_72, \+f_143.
result_g_142 :- \+result_1_70, result_2_70.
0.5::f_144.
result_g_143 :- \+result_1_70, \+result_2_70.
result_1_71 :- result_1_70, result_then_1_72.
result_1_71 :- result_g_142, f_144.
result_2_71 :- result_1_70, result_then_2_72.
result_2_71 :- result_g_142, \+f_144.
0.001::result_then_c_73.
0.5::f_145.
result_then_1_73 :- \+result_then_c_73, f_145.
result_then_2_73 :- \+result_then_c_73, \+f_145.
result_g_144 :- \+result_1_71, result_2_71.
0.5::f_146.
result_g_145 :- \+result_1_71, \+result_2_71.
result_1_72 :- result_1_71, result_then_1_73.
result_1_72 :- result_g_144, f_146.
result_2_72 :- result_1_71, result_then_2_73.
result_2_72 :- result_g_144, \+f_146.
0.001::result_then_c_74.
0.5::f_147.
result_then_1_74 :- \+result_then_c_74, f_147.
result_then_2_74 :- \+result_then_c_74, \+f_147.
result_g_146 :- \+result_1_72, result_2_72.
0.5::f_148.
result_g_147 :- \+result_1_72, \+result_2_72.
result_1_73 :- result_1_72, result_then_1_74.
result_1_73 :- result_g_146, f_148.
result_2_73 :- result_1_72, result_then_2_74.
result_2_73 :- result_g_146, \+f_148.
0.001::result_then_c_75.
0.5::f_149.
result_then_1_75 :- \+result_then_c_75, f_149.
result_then_2_75 :- \+result_then_c_75, \+f_149.
result_g_148 :- \+result_1_73, result_2_73.
0.5::f_150.
result_g_149 :- \+result_1_73, \+result_2_73.
result_1_74 :- result_1_73, result_then_1_75.
result_1_74 :- result_g_148, f_150.
result_2_74 :- result_1_73, result_then_2_75.
result_2_74 :- result_g_148, \+f_150.
0.001::result_then_c_76.
0.5::f_151.
result_then_1_76 :- \+result_then_c_76, f_151.
result_then_2_76 :- \+result_then_c_76, \+f_151.
result_g_150 :- \+result_1_74, result_2_74.
0.5::f_152.
result_g_151 :- \+result_1_74, \+result_2_74.
result_1_75 :- result_1_74, result_then_1_76.
result_1_75 :- result_g_150, f_152.
result_2_75 :- result_1_74, result_then_2_76.
result_2_75 :- result_g_150, \+f_152.
0.001::result_then_c_77.
0.5::f_153.
result_then_1_77 :- \+result_then_c_77, f_153.
result_then_2_77 :- \+result_then_c_77, \+f_153.
result_g_152 :- \+result_1_75, result_2_75.
0.5::f_154.
result_g_153 :- \+result_1_75, \+result_2_75.
result_1_76 :- result_1_75, result_then_1_77.
result_1_76 :- result_g_152, f_154.
result_2_76 :- result_1_75, result_then_2_77.
result_2_76 :- result_g_152, \+f_154.
0.001::result_then_c_78.
0.5::f_155.
result_then_1_78 :- \+result_then_c_78, f_155.
result_then_2_78 :- \+result_then_c_78, \+f_155.
result_g_154 :- \+result_1_76, result_2_76.
0.5::f_156.
result_g_155 :- \+result_1_76, \+result_2_76.
result_1_77 :- result_1_76, result_then_1_78.
result_1_77 :- result_g_154, f_156.
result_2_77 :- result_1_76, result_then_2_78.
result_2_77 :- result_g_154, \+f_156.
0.001::result_then_c_79.
0.5::f_157.
result_then_1_79 :- \+result_then_c_79, f_157.
result_then_2_79 :- \+result_then_c_79, \+f_157.
result_g_156 :- \+result_1_77, result_2_77.
0.5::f_158.
result_g_157 :- \+result_1_77, \+result_2_77.
result_1_78 :- result_1_77, result_then_1_79.
result_1_78 :- result_g_156, f_158.
result_2_78 :- result_1_77, result_then_2_79.
result_2_78 :- result_g_156, \+f_158.
0.001::result_then_c_80.
0.5::f_159.
result_then_1_80 :- \+result_then_c_80, f_159.
result_then_2_80 :- \+result_then_c_80, \+f_159.
result_g_158 :- \+result_1_78, result_2_78.
0.5::f_160.
result_g_159 :- \+result_1_78, \+result_2_78.
result_1_79 :- result_1_78, result_then_1_80.
result_1_79 :- result_g_158, f_160.
result_2_79 :- result_1_78, result_then_2_80.
result_2_79 :- result_g_158, \+f_160.
0.001::result_then_c_81.
0.5::f_161.
result_then_1_81 :- \+result_then_c_81, f_161.
result_then_2_81 :- \+result_then_c_81, \+f_161.
result_g_160 :- \+result_1_79, result_2_79.
0.5::f_162.
result_g_161 :- \+result_1_79, \+result_2_79.
result_1_80 :- result_1_79, result_then_1_81.
result_1_80 :- result_g_160, f_162.
result_2_80 :- result_1_79, result_then_2_81.
result_2_80 :- result_g_160, \+f_162.
0.001::result_then_c_82.
0.5::f_163.
result_then_1_82 :- \+result_then_c_82, f_163.
result_then_2_82 :- \+result_then_c_82, \+f_163.
result_g_162 :- \+result_1_80, result_2_80.
0.5::f_164.
result_g_163 :- \+result_1_80, \+result_2_80.
result_1_81 :- result_1_80, result_then_1_82.
result_1_81 :- result_g_162, f_164.
result_2_81 :- result_1_80, result_then_2_82.
result_2_81 :- result_g_162, \+f_164.
0.001::result_then_c_83.
0.5::f_165.
result_then_1_83 :- \+result_then_c_83, f_165.
result_then_2_83 :- \+result_then_c_83, \+f_165.
result_g_164 :- \+result_1_81, result_2_81.
0.5::f_166.
result_g_165 :- \+result_1_81, \+result_2_81.
result_1_82 :- result_1_81, result_then_1_83.
result_1_82 :- result_g_164, f_166.
result_2_82 :- result_1_81, result_then_2_83.
result_2_82 :- result_g_164, \+f_166.
0.001::result_then_c_84.
0.5::f_167.
result_then_1_84 :- \+result_then_c_84, f_167.
result_then_2_84 :- \+result_then_c_84, \+f_167.
result_g_166 :- \+result_1_82, result_2_82.
0.5::f_168.
result_g_167 :- \+result_1_82, \+result_2_82.
result_1_83 :- result_1_82, result_then_1_84.
result_1_83 :- result_g_166, f_168.
result_2_83 :- result_1_82, result_then_2_84.
result_2_83 :- result_g_166, \+f_168.
0.001::result_then_c_85.
0.5::f_169.
result_then_1_85 :- \+result_then_c_85, f_169.
result_then_2_85 :- \+result_then_c_85, \+f_169.
result_g_168 :- \+result_1_83, result_2_83.
0.5::f_170.
result_g_169 :- \+result_1_83, \+result_2_83.
result_1_84 :- result_1_83, result_then_1_85.
result_1_84 :- result_g_168, f_170.
result_2_84 :- result_1_83, result_then_2_85.
result_2_84 :- result_g_168, \+f_170.
0.001::result_then_c_86.
0.5::f_171.
result_then_1_86 :- \+result_then_c_86, f_171.
result_then_2_86 :- \+result_then_c_86, \+f_171.
result_g_170 :- \+result_1_84, result_2_84.
0.5::f_172.
result_g_171 :- \+result_1_84, \+result_2_84.
result_1_85 :- result_1_84, result_then_1_86.
result_1_85 :- result_g_170, f_172.
result_2_85 :- result_1_84, result_then_2_86.
result_2_85 :- result_g_170, \+f_172.
0.001::result_then_c_87.
0.5::f_173.
result_then_1_87 :- \+result_then_c_87, f_173.
result_then_2_87 :- \+result_then_c_87, \+f_173.
result_g_172 :- \+result_1_85, result_2_85.
0.5::f_174.
result_g_173 :- \+result_1_85, \+result_2_85.
result_1_86 :- result_1_85, result_then_1_87.
result_1_86 :- result_g_172, f_174.
result_2_86 :- result_1_85, result_then_2_87.
result_2_86 :- result_g_172, \+f_174.
0.001::result_then_c_88.
0.5::f_175.
result_then_1_88 :- \+result_then_c_88, f_175.
result_then_2_88 :- \+result_then_c_88, \+f_175.
result_g_174 :- \+result_1_86, result_2_86.
0.5::f_176.
result_g_175 :- \+result_1_86, \+result_2_86.
result_1_87 :- result_1_86, result_then_1_88.
result_1_87 :- result_g_174, f_176.
result_2_87 :- result_1_86, result_then_2_88.
result_2_87 :- result_g_174, \+f_176.
0.001::result_then_c_89.
0.5::f_177.
result_then_1_89 :- \+result_then_c_89, f_177.
result_then_2_89 :- \+result_then_c_89, \+f_177.
result_g_176 :- \+result_1_87, result_2_87.
0.5::f_178.
result_g_177 :- \+result_1_87, \+result_2_87.
result_1_88 :- result_1_87, result_then_1_89.
result_1_88 :- result_g_176, f_178.
result_2_88 :- result_1_87, result_then_2_89.
result_2_88 :- result_g_176, \+f_178.
0.001::result_then_c_90.
0.5::f_179.
result_then_1_90 :- \+result_then_c_90, f_179.
result_then_2_90 :- \+result_then_c_90, \+f_179.
result_g_178 :- \+result_1_88, result_2_88.
0.5::f_180.
result_g_179 :- \+result_1_88, \+result_2_88.
result_1_89 :- result_1_88, result_then_1_90.
result_1_89 :- result_g_178, f_180.
result_2_89 :- result_1_88, result_then_2_90.
result_2_89 :- result_g_178, \+f_180.
0.001::result_then_c_91.
0.5::f_181.
result_then_1_91 :- \+result_then_c_91, f_181.
result_then_2_91 :- \+result_then_c_91, \+f_181.
result_g_180 :- \+result_1_89, result_2_89.
0.5::f_182.
result_g_181 :- \+result_1_89, \+result_2_89.
result_1_90 :- result_1_89, result_then_1_91.
result_1_90 :- result_g_180, f_182.
result_2_90 :- result_1_89, result_then_2_91.
result_2_90 :- result_g_180, \+f_182.
0.001::result_then_c_92.
0.5::f_183.
result_then_1_92 :- \+result_then_c_92, f_183.
result_then_2_92 :- \+result_then_c_92, \+f_183.
result_g_182 :- \+result_1_90, result_2_90.
0.5::f_184.
result_g_183 :- \+result_1_90, \+result_2_90.
result_1_91 :- result_1_90, result_then_1_92.
result_1_91 :- result_g_182, f_184.
result_2_91 :- result_1_90, result_then_2_92.
result_2_91 :- result_g_182, \+f_184.
0.001::result_then_c_93.
0.5::f_185.
result_then_1_93 :- \+result_then_c_93, f_185.
result_then_2_93 :- \+result_then_c_93, \+f_185.
result_g_184 :- \+result_1_91, result_2_91.
0.5::f_186.
result_g_185 :- \+result_1_91, \+result_2_91.
result_1_92 :- result_1_91, result_then_1_93.
result_1_92 :- result_g_184, f_186.
result_2_92 :- result_1_91, result_then_2_93.
result_2_92 :- result_g_184, \+f_186.
0.001::result_then_c_94.
0.5::f_187.
result_then_1_94 :- \+result_then_c_94, f_187.
result_then_2_94 :- \+result_then_c_94, \+f_187.
result_g_186 :- \+result_1_92, result_2_92.
0.5::f_188.
result_g_187 :- \+result_1_92, \+result_2_92.
result_1_93 :- result_1_92, result_then_1_94.
result_1_93 :- result_g_186, f_188.
result_2_93 :- result_1_92, result_then_2_94.
result_2_93 :- result_g_186, \+f_188.
0.001::result_then_c_95.
0.5::f_189.
result_then_1_95 :- \+result_then_c_95, f_189.
result_then_2_95 :- \+result_then_c_95, \+f_189.
result_g_188 :- \+result_1_93, result_2_93.
0.5::f_190.
result_g_189 :- \+result_1_93, \+result_2_93.
result_1_94 :- result_1_93, result_then_1_95.
result_1_94 :- result_g_188, f_190.
result_2_94 :- result_1_93, result_then_2_95.
result_2_94 :- result_g_188, \+f_190.
0.001::result_then_c_96.
0.5::f_191.
result_then_1_96 :- \+result_then_c_96, f_191.
result_then_2_96 :- \+result_then_c_96, \+f_191.
result_g_190 :- \+result_1_94, result_2_94.
0.5::f_192.
result_g_191 :- \+result_1_94, \+result_2_94.
result_1_95 :- result_1_94, result_then_1_96.
result_1_95 :- result_g_190, f_192.
result_2_95 :- result_1_94, result_then_2_96.
result_2_95 :- result_g_190, \+f_192.
0.001::result_then_c_97.
0.5::f_193.
result_then_1_97 :- \+result_then_c_97, f_193.
result_then_2_97 :- \+result_then_c_97, \+f_193.
result_g_192 :- \+result_1_95, result_2_95.
0.5::f_194.
result_g_193 :- \+result_1_95, \+result_2_95.
result_1_96 :- result_1_95, result_then_1_97.
result_1_96 :- result_g_192, f_194.
result_2_96 :- result_1_95, result_then_2_97.
result_2_96 :- result_g_192, \+f_194.
0.001::result_then_c_98.
0.5::f_195.
result_then_1_98 :- \+result_then_c_98, f_195.
result_then_2_98 :- \+result_then_c_98, \+f_195.
result_g_194 :- \+result_1_96, result_2_96.
0.5::f_196.
result_g_195 :- \+result_1_96, \+result_2_96.
result_1_97 :- result_1_96, result_then_1_98.
result_1_97 :- result_g_194, f_196.
result_2_97 :- result_1_96, result_then_2_98.
result_2_97 :- result_g_194, \+f_196.
0.001::result_then_c_99.
0.5::f_197.
result_then_1_99 :- \+result_then_c_99, f_197.
result_then_2_99 :- \+result_then_c_99, \+f_197.
result_g_196 :- \+result_1_97, result_2_97.
0.5::f_198.
result_g_197 :- \+result_1_97, \+result_2_97.
result_1_98 :- result_1_97, result_then_1_99.
result_1_98 :- result_g_196, f_198.
result_2_98 :- result_1_97, result_then_2_99.
result_2_98 :- result_g_196, \+f_198.
query(result_1_98).
query(result_2_98).
