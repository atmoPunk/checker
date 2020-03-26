import sys
import json
from collections import defaultdict

inp = sys.argv[1]

index = {}

fileindex = {}

try:
    with open('index') as f:
        index = json.load(f)
except FileNotFoundError:
    pass

try:
    with open('fileindex') as f:
        fileindex = json.load(f)
except FileNotFoundError:
    pass

index = defaultdict(int, index)
fileindex = defaultdict(list, fileindex)

with open(inp) as f:
    hashes = json.load(f)

for h, count in hashes.items():
    index[h] += count
    fileindex[h].append(inp)


with open('index', 'w') as f:
    json.dump(index, f)

with open('fileindex', 'w') as f:
    json.dump(fileindex, f)