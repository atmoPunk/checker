import sys
import os
import re
import subprocess
import hashlib
import json
from collections import defaultdict

folder = sys.argv[1]
student = sys.argv[2] if 2 < len(sys.argv) else None
excluded_imports_file = sys.argv[3] if 3 < len(sys.argv) else None

excluded_imports = []

if excluded_imports_file:
    with open(excluded_imports_file) as f:
        excluded_imports = list(f.readlines())
        excluded_imports = list(map(lambda s: s.strip(), excluded_imports))

# print(excluded_imports)

start_dir = os.getcwd()
(_, _, files) = next(os.walk(folder))

os.chdir(folder)

files = list(filter(lambda f: f.endswith('.cpp') or f.endswith('.c') or f.endswith('.h') or f.endswith('.hpp'), files))

content = []
marker_counter = 0
already_included = set()
additional_cpps = set()

def add_to_content(filename):
    for excl in excluded_imports:
        if excl in filename:
            return
    if filename in already_included:
        return
    print(os.getcwd(), filename, sep='/')
    with open(filename) as f:
        already_included.add(filename)
        for line in f.readlines(): # TODO: will not understand if include is in block comment
            if line.startswith('#include "'):
                incl = line[10: 10 + line[10:].find('"') ]
                incl_dirs = incl.rsplit('/', 1)
                cur_dir = os.getcwd()
                if len(incl_dirs) > 1:
                    os.chdir(incl_dirs[0])
                    add_to_content(incl_dirs[1])
                    (_, _, next_files) = next(os.walk(os.getcwd()))
                    next_files = list(filter(lambda f: f.endswith('.cpp') or f.endswith('.c') or f.endswith('.h') or f.endswith('.hpp'), next_files))
                    for next_filename in sorted(next_files, key=lambda x: not x.endswith('.h') and not x.endswith('.hpp')):  # TODO: does not understand dirs
                        add_to_content(next_filename)
                else:
                    add_to_content(incl)
                os.chdir(cur_dir)
            elif line.startswith('#include"'):
                incl = line[9: 9 + line[9:].find('"')]
                incl_dirs = incl.rsplit('/', 1)
                cur_dir = os.getcwd()
                if len(incl_dirs) > 1:
                    os.chdir(incl_dirs[0])
                    add_to_content(incl_dirs[1])
                    (_, _, next_files) = next(os.walk(os.getcwd()))
                    next_files = list(filter(lambda f: f.endswith('.cpp') or f.endswith('.c') or f.endswith('.h') or f.endswith('.hpp'), next_files))
                    for next_filename in sorted(next_files, key=lambda x: not x.endswith('.h') and not x.endswith('.hpp')):  # TODO: does not understand dirs
                        add_to_content(next_filename)
                else:
                    add_to_content(incl)
                os.chdir(cur_dir)
            else:
                content.append(line)
    content.append('\n')

for filename in sorted(files, key=lambda x: not x.endswith('.h') and not x.endswith('.hpp')):  # TODO: does not understand dirs
    add_to_content(filename)

os.chdir(start_dir)

with open('tmp-concat.cpp', 'w') as f:
    for line in content:  # TODO: check if multiple '#pragma once' break anything
        if line.startswith('#include <') or line.startswith('#include<'):  # include stdlib
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
impl_level = -1
with open('tmp-ast-stripped') as f:
    token = None
    for line in f.readlines():
        if ' implicit ' in line and impl_level == -1:
            # print("FOUND IMPLICIT")
            # print(line)
            for (index, ch) in enumerate(line):
                if ch.isalpha():
                    impl_level = index
                    break
            continue
        if impl_level != -1:
            for ii in range(impl_level + 1):
                if line[ii].isalpha():
                    impl_level = -1
                    # print('DROP IMPLICIT')
                    # print(line)
                    break
            else:
                continue    
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

with open('tmp-ast-no-impl-and-lines', 'w') as f:
    for token in tokens:
        for line in token:
            f.write(line)
            f.write('\n')

clang_address = re.compile(r'0x[0-9a-f]{12}')
clang_pos = re.compile(r'((<|, )(tmp-concat\.cpp|line|col|scratch space)(:[0-9]+){0,2})+')
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
# TODO: Constructor, Destructor, CXXCtorInitializer, NamespaceDecl, etc
template_decl = re.compile(r'ClassTemplateDecl (0x[0-9a-f]{12}) (.+) (\w+)')  # 3rd

this_expr = re.compile(r'CXXThisExpr (0x[0-9a-f]{12}) (.+) \'(const )?(\w+) \*\'( implicit)? this')  # 4th
membr_expr = re.compile(r'MemberExpr (0x[0-9a-f]{12}) (.+) (->|\.)(\w+)? (.+)')  # 4th
templ_parm_decl = re.compile(r'NonTypeTemplateParmDecl (.+) (\w+)')  # 2nd



use_identifier = re.compile(r'(Var|ParmVar|Function|NonTypeTemplateParm) (0x[0-9a-f]{12}) (\'.+\') (\'.+\')')
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
    for (i, _) in enumerate(token):
        if m := decl_identifier.match(token[i]):
            tmp = token[i][:m.start(4)]
            tmp += token[i][m.end(4) + 1:]
            token[i] = tmp
        if m := decl_record.match(token[i]):
            tmp = token[i][:m.start(4)]
            tmp += token[i][m.end(4) + 1:]
            token[i] = tmp
        if m := decl_method.match(token[i]):
            tmp = token[i][:m.start(3)]
            tmp += token[i][m.end(3) + 1:]
            token[i] = tmp
        if m := decl_field.match(token[i]):
            tmp = token[i][:m.start(3)]
            tmp += token[i][m.end(3) + 1:]
            token[i] = tmp 
        if m := this_expr.match(token[i]):
            tmp = token[i][:m.start(4)]
            tmp += token[i][m.end(4) + 1:]
            token[i] = tmp
        if m := membr_expr.match(token[i]):
            tmp = token[i][:m.start(4)]
            tmp += token[i][m.end(4):]  # do not skip whitespace
            token[i] = tmp
        if m := template_decl.match(token[i]):
            tmp = token[i][:m.start(3)]
            tmp += token[i][m.end(3) + 1:]
            token[i] = tmp
        if u := use_identifier.search(token[i]):
            tmp = token[i][:u.start(3)]
            tmp += token[i][u.end(3) + 1:]
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
        # print(t)
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

    
