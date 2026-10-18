import os

def remove_comments(code: str) -> str:
    res = []
    i = 0
    n = len(code)
    in_string = False
    in_line_comment = False
    in_block_comment = 0

    while i < n:
        if in_line_comment:
            if code[i] == '\n':
                in_line_comment = False
                res.append('\n')
            i += 1
            continue

        if in_block_comment > 0:
            if i + 1 < n and code[i:i+2] == '/*':
                in_block_comment += 1
                i += 2
            elif i + 1 < n and code[i:i+2] == '*/':
                in_block_comment -= 1
                i += 2
            else:
                i += 1
            continue

        if in_string:
            res.append(code[i])
            if code[i] == '\\':
                if i + 1 < n:
                    i += 1
                    res.append(code[i])
            elif code[i] == '"':
                in_string = False
            i += 1
            continue

        if code[i] == '"':
            in_string = True
            res.append(code[i])
            i += 1
            continue

        if code[i] == "'" and i + 2 < n and code[i+2] == "'":
            res.append(code[i:i+3])
            i += 3
            continue

        if i + 1 < n and code[i:i+2] == '//':
            in_line_comment = True
            i += 2
            continue

        if i + 1 < n and code[i:i+2] == '/*':
            in_block_comment = 1
            i += 2
            continue

        res.append(code[i])
        i += 1

    lines = ''.join(res).split('\n')
    cleaned = []
    for line in lines:
        if line.strip() == '' and (len(cleaned) == 0 or cleaned[-1].strip() == ''):
            continue
        cleaned.append(line.rstrip())
    return '\n'.join(cleaned).strip() + '\n'

for base_dir in ['src', 'tests']:
    for root, _, files in os.walk(base_dir):
        for f in files:
            if f.endswith('.rs'):
                fpath = os.path.join(root, f)
                with open(fpath, 'r', encoding='utf-8') as handle:
                    content = handle.read()
                cleaned_content = remove_comments(content)
                with open(fpath, 'w', encoding='utf-8') as handle:
                    handle.write(cleaned_content)

print("Processed all .rs files in src/ and tests/")
