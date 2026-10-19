import os
import ast
import tokenize
import io

def strip_python_code(source):
    tree = ast.parse(source)
    docstring_ranges = []

    def check_body(body):
        for stmt in body:
            if isinstance(stmt, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                if len(stmt.body) == 1 and isinstance(stmt.body[0], ast.Expr) and isinstance(stmt.body[0].value, ast.Constant) and isinstance(stmt.body[0].value.value, str):
                    docstring_ranges.append((stmt.body[0].lineno, stmt.body[0].end_lineno, stmt.body[0].col_offset, True))
                else:
                    check_body(stmt.body)
            elif isinstance(stmt, ast.Expr) and isinstance(stmt.value, ast.Constant) and isinstance(stmt.value.value, str):
                docstring_ranges.append((stmt.lineno, stmt.end_lineno, stmt.col_offset, False))

    check_body(tree.body)

    lines = source.splitlines(keepends=True)
    for start, end, col, is_only in docstring_ranges:
        indent = " " * col
        for l in range(start - 1, end):
            if is_only and l == start - 1:
                lines[l] = indent + "pass\n"
            else:
                lines[l] = "\n"

    source_no_docstrings = "".join(lines)
    tokens = list(tokenize.generate_tokens(io.StringIO(source_no_docstrings).readline))
    lines2 = source_no_docstrings.splitlines(keepends=True)
    for tok in tokens:
        if tok.type == tokenize.COMMENT:
            srow, scol = tok.start
            line_idx = srow - 1
            line_content = lines2[line_idx]
            prefix = line_content[:scol]
            if prefix.strip() == "":
                lines2[line_idx] = "\n"
            else:
                lines2[line_idx] = prefix.rstrip() + "\n"

    final_lines = []
    blank_count = 0
    for l in lines2:
        if l.strip() == "":
            blank_count += 1
            if blank_count <= 1:
                final_lines.append("\n")
        else:
            blank_count = 0
            final_lines.append(l)

    result = "".join(final_lines).lstrip()
    ast.parse(result)
    return result

def process_file(filepath):
    ext = os.path.splitext(filepath)[1]
    if ext != ".py":
        return
    with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
        content = f.read()
    try:
        cleaned = strip_python_code(content)
        with open(filepath, "w", encoding="utf-8") as f:
            f.write(cleaned)
        print(f"Cleaned {filepath}")
    except Exception as e:
        print(f"Failed to clean {filepath}: {e}")

if __name__ == "__main__":
    targets = ["scripts", "agent-harness"]
    for t in targets:
        if os.path.isdir(t):
            for root, _, files in os.walk(t):
                for f in files:
                    if f.endswith(".py"):
                        process_file(os.path.join(root, f))
