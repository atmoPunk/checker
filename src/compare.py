import sys
import json
from collections import defaultdict

file1 = sys.argv[1]
file2 = sys.argv[2]

matches = 0

with open(file1) as f: 
    hashes1 = json.load(f)

with open(file2) as f:
    hashes2 = json.load(f)

elements1 = sum(hashes1.values())
elements2 = sum(hashes2.values())

hashes1 = defaultdict(int, hashes1)
hashes2 = defaultdict(int, hashes2)

for k, v in hashes1.items():
    matches += min(hashes2[k], v)
print(matches, 'out of', max(elements1, elements2))