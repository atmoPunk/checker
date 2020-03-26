import sys
import os
import re
import subprocess
import hashlib
import json
from collections import defaultdict

folder = sys.argv[1]
student = sys.argv[2] if 2 < len(sys.argv) else None

(_, _, files) = next(os.walk(folder))

files = list(filter(lambda f: f.endswith('.cpp') or f.endswith('.c') or f.endswith('.h') or f.endswith('.hpp'), files))


content = []
marker_counter = 0
already_included = set()

def add_to_content(filename):
    if filename in already_included:
        return
    with open(os.path.join(folder, filename)) as f:
        already_included.add(filename)
        for line in f.readlines():
            if line.startswith('#include "'):
                incl = line[10:-2]
                add_to_content(incl)
            else:
                content.append(line)
    content.append('\n')

for filename in sorted(files, key=lambda x: not x.endswith('.h') and not x.endswith('.hpp')):  # TODO: does not understand dirs
    add_to_content(filename)

with open('tmp-concat.cpp', 'w') as f:
    for line in content:  # TODO: check if multiple '#pragma once' break anything
        if line.startswith('#include <'):  # global include
            f.write('int __INCLUDE_MARKER_START_' + str(marker_counter) + '__;\n')
            f.write(line)
            f.write('int __INCLUDE_MARKER_END_' + str(marker_counter) + '__;\n')
            marker_counter += 1
        else:
            f.write(line)

sp = subprocess.run(['clang', '-Xclang', '-ast-dump', '-fsyntax-only', '-fno-color-diagnostics', '-w', 'tmp-concat.cpp'], capture_output=True)
with open('tmp-ast', 'wb') as f:
    f.write(sp.stdout)

buffer = []
find_end = False
with open('tmp-ast') as f:
    lines = f.readlines()
    for i in range(len(lines)):
        if find_end:
            if '__INCLUDE_MARKER_END_' not in lines[i]:
                continue
            else:
                find_end = False
        elif '__INCLUDE_MARKER_START_' in lines[i]:
            find_end = True
        else:
            buffer.extend(lines[i])

with open('tmp-ast-stripped', 'w') as f:
    for line in buffer:
        f.write(line)

tokens = []
with open('tmp-ast-stripped') as f:
    token = None
    for line in f.readlines():
        if line[1] == '-' and line[2].isalpha():
            if token:
                tokens.append(token)
            token = []
            token.append(line[2:-1])
        elif token:
            for (index, ch) in enumerate(line):
                if ch.isalpha():
                    token.append(line[index:-1])
                    break
    if token:
        tokens.append(token)


clang_address = re.compile(r'0x[0-9a-f]{12}')
clang_pos = re.compile(r'((<|, )(tmp-concat\.cpp|line|col)(:[0-9]+){0,2})+')
clang_exact_pos = re.compile(r'(line|col)(:[0-9]+){0,2}')
clang_invalid_sloc = re.compile(r'<invalid sloc>')

decl_identifier = re.compile(r'(VarDecl|ParmVarDecl|FunctionDecl) (0x[0-9a-f]{12}) (.+) (.+) (\'.+\')( cinit)?')
# 2nd group - address
# 4th group - id
decl_record = re.compile(r'CXXRecordDecl (0x[0-9a-f]{12}) (.+) (class|struct) ([0-9a-zA-Z]+)( definition)?')
# 4th group - id
decl_method = re.compile(r'CXXMethodDecl (0x[0-9a-f]{12}) (.+) ([0-9a-zA-Z]+) (\'.+\')')
# 3rd group
decl_field = re.compile(r'FieldDecl (0x[0-9a-f]{12}) (.+) ([0-9a-zA-Z]+) (\'.+\')')
# 3rd_group
# TODO: MemberExpr, Constructor, Destructor, CXXCtorInitializer ...
# TODO: Skip implicit from clang? -> implicit construtors, desctructors etc
this_expr = re.compile(r'CXXThisExpr (0x[0-9a-f]{12}) (.+) \'(const )?(\w+) \*\'( implicit)? this')  # 4th
membr_expr = re.compile(r'MemberExpr (0x[0-9a-f]{12}) (.+) (->|\.)(\w+)? (.+)')  # 4th


use_identifier = re.compile(r'(Var|ParmVar|Function) (0x[0-9a-f]{12}) (\'.+\') (\'.+\')')
# 2nd group - address
# 3th group - id

skip = False

hashes = defaultdict(int)
trig1 = None
trig2 = None
trig3 = None

# TODO: remove identifier names or make them kinda same
# (in tokens make transform them to 'a', 'b', 'c' and so on)
# or something like that
for token in tokens:
    for (i, t) in enumerate(token):
        m = decl_identifier.match(t)
        if m:
            tmp = t[:m.start(4)]
            tmp += t[m.end(4) + 1:]
            token[i] = tmp
        m = decl_record.match(t)
        if m:
            tmp = t[:m.start(4)]
            tmp += t[m.end(4) + 1:]
            token[i] = tmp
        m = decl_method.match(t)
        if m:
            tmp = t[:m.start(3)]
            tmp += t[m.end(3) + 1:]
            token[i] = tmp
        m = decl_field.match(t)
        if m:
            tmp = t[:m.start(3)]
            tmp += t[m.end(3) + 1:]
            token[i] = tmp
        m = this_expr.match(t)
        if m:
            tmp = t[:m.start(4)]
            tmp += t[m.end(4) + 1:]
            token[i] = tmp
        m = membr_expr.match(t)
        if m:
            tmp = t[:m.start(4)]
            tmp += t[m.end(4):]  # do not skip whitespace
            token[i] = tmp
        u = use_identifier.search(t)
        if u:
            tmp = t[:u.start(3)]
            tmp += t[u.end(3) + 1:]
            token[i] = tmp


for token in tokens:
    for t in token:
        if clang_invalid_sloc.search(t):  # skip clang internal lines
            skip = True
            break
    if skip:
        skip = False
        continue
    
    for i, l in enumerate(token):
        mtch = clang_pos.search(l)
        if mtch:
            tmpl = l[:mtch.start()]
            tmpl += l[mtch.end() + 1:]
        else:
            tmpl = l
        li = tmpl.split()
        without_add = []
        for tk in li:
            if clang_address.match(tk):
                continue
            if clang_exact_pos.match(tk):
                continue
            without_add.append(tk)
        token[i] = ' '.join(without_add)
    

    for t in token:
        print(t)
        if trig1 is None:
            trig1 = t
        elif trig2 is None:
            trig2 = t
        elif trig3 is None:
            trig3 = t
            hashes[hashlib.md5(bytes((trig1 + trig2 + trig3), 'utf-8')).hexdigest()] += 1
        else:
            trig1 = trig2
            trig2 = trig3
            trig3 = t
            hashes[hashlib.md5(bytes((trig1 + trig2 + trig3), 'utf-8')).hexdigest()] += 1

with open(student if student else 'result', 'w') as f:
    json.dump(hashes, f)

    