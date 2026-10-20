#!/usr/bin/env python3
"""dice_scale.py SRC K OUT: the Dice program SRC with the count of its
iterate call (the first, the one the program's result reads) set to K."""
import re
import sys

src, k, out = sys.argv[1], int(sys.argv[2]), sys.argv[3]
s = open(src).read()
i = s.find('iterate(')
if i < 0:
    raise SystemExit(f'{src}: no iterate call')
depth = 0
for j in range(i + len('iterate'), len(s)):
    if s[j] == '(':
        depth += 1
    elif s[j] == ')':
        depth -= 1
        if depth == 0:
            break
m = re.search(r',\s*(\d+)\s*$', s[i:j])
if not m:
    raise SystemExit(f'{src}: the iterate call ends in no count')
open(out, 'w').write(s[:i + m.start(1)] + str(k) + s[i + m.end(1):])
