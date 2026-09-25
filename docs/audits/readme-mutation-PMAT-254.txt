# PMAT-254 README gate mutation proof (exit codes: 0 green, non-zero RED; - = gate not run for that plant)
# head before this commit: d7387791ad0e2c8834e37e1e0d0035bf92f103fd; script: docs/audits/readme-mutation-PMAT-254.sh
M0   baseline (no plant)                          lib=0 bin=0 pv=0
M1a  wrong MSRV number, claims NOT re-blessed     lib=101 bin=- pv=-
M1b  wrong MSRV number, claims re-blessed         lib=101 bin=- pv=0
M1c  hand-typed test count                        lib=101 bin=- pv=-
M2a  broken example path in Quick start           lib=0 bin=101 pv=-
M2b  broken example path on an Examples block     lib=101 bin=- pv=-
M3   example output drift (.expected)             lib=- bin=101 pv=-
M4   example no longer runs (README+file)         lib=101 bin=101 pv=-
M5   required section removed                     lib=101 bin=- pv=1
M6   Commands table names a missing subcommand    lib=0 bin=101 pv=0
M7   dead relative link                           lib=101 bin=- pv=0
M8   baseline after restore                       lib=0 bin=0 pv=0
