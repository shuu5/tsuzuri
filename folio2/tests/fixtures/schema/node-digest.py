#!/usr/bin/env python3
"""節点ごとの要約値と残差の凍結 anchor を組む独立の script（便 99・docs/design/delivery-99.md §1 (b)(e)・P-10.1 / P-10.2）。

folio の code を 1 行も呼ばない（python3 の標準 library だけ・YAML の読み口を使わない）。引数の置き場の正本を行の逐語の
byte だけで切り分け、節点の要約値（id の byte 順）と残差の要約値と byte の内訳を node-digest-anchor.txt と同じ字で
標準出力へ書く。

- 節点の頭の行: 正本の節の中の、字下げ 2（憲法の規範文は条の中の字下げ 6）の `- id: <id>` の行か `- {id: <id>,` で始まる行
- block: 頭の行から、空行でなく字下げが頭の行以下である最初の行の直前まで（流れの形の頭は 1 行だけ・判断の記録は file の全行）
- 落とす: 入れ子の節点の block・辺の欄の行（字下げは頭 + 2・判断の記録は 0。より深い続きの行も）・流れの形の行の辺の欄の対
- 畳む: 末尾の空行を落とし、残った行を改行ごと連結した byte の sha256 の先頭 8 字
- 残差: 母集団（索引の正本と、天井の正本の観点の reads が指す文書の file）の全 byte から本文と辺の欄を除いた残り
"""

import hashlib
import os
import re
import sys

# 正本の file → 節点の節
SECTIONS = {
    "constitution.yaml": ["articles"],
    "rules.yaml": ["thresholds", "discipline"],
    "srs.yaml": ["goals", "actors", "outputs", "requirements", "nonfunctional", "acceptance", "constraints"],
}

# 辺の欄の閉じた一覧（正本の file ごと・判断の記録は adr）
EDGE_FIELDS = {
    "constitution.yaml": ["relations"],
    "rules.yaml": ["article", "refs"],
    "srs.yaml": ["basis", "goals", "rules", "adrs", "verifies", "verify.ac"],
    "adr": ["basis", "produced"],
}

ADR_NAME = re.compile(r"^ADR-[0-9]+\.yaml$")


def lines_of(text):
    """行に割る（行の終わりの改行を含める）。"""
    out, start = [], 0
    while start < len(text):
        end = text.find("\n", start)
        if end < 0:
            out.append(text[start:])
            break
        out.append(text[start:end + 1])
        start = end + 1
    return out


def size(text):
    return len(text.encode("utf-8"))


def blank(line):
    return line.strip() == ""


def indent(line):
    return len(line) - len(line.lstrip(" "))


def field_key(line):
    """字下げの後の `key:`（その後が空白か行の終わり）の key。無ければ None。"""
    s = line.lstrip(" ")
    k = s.find(":")
    if k <= 0 or s[k + 1:k + 2] not in (" ", "\n", ""):
        return None
    key = s[:k]
    return key if all(c.isalnum() or c in "_-" for c in key) else None


def head_id(line, depth):
    """字下げ depth の節点の頭の行なら (id, 流れの形か)。でなければ None。"""
    pad = " " * depth
    if line.startswith(pad + "- id: "):
        return line[len(pad + "- id: "):].strip(), False
    if line.startswith(pad + "- {id: "):
        rest = line[len(pad + "- {id: "):]
        ends = [i for i in (rest.find(","), rest.find("}")) if i >= 0]
        if ends:
            return rest[:min(ends)].strip(), True
    return None


def block_end(lines, i, depth, flow):
    """頭の行 i の block の終わり（含まない）。"""
    if flow:
        return i + 1
    j = i + 1
    while j < len(lines) and (blank(lines[j]) or indent(lines[j]) > depth):
        j += 1
    return j


def skip_quoted(line, i):
    """二重引用符の開き i から、閉じの次の位置。"""
    i += 1
    while i < len(line):
        if line[i] == "\\":
            i += 2
            continue
        if line[i] == '"':
            return i + 1
        i += 1
    return i


def drop_pairs(line, names):
    """流れの形の行から欄 names の対を落とす。戻り値 = (残った行, 落とした byte の数)。"""
    dropped, i = 0, 0
    while i < len(line):
        if line[i] == '"':
            i = skip_quoted(line, i)
            continue
        opened = i > 0 and line[i - 1] == "{"
        comma = i > 1 and line[i - 2:i] == ", "
        name = next((n for n in names if line.startswith(n + ": ", i)), None) if opened or comma else None
        if name is None:
            i += 1
            continue
        j, depth = i + len(name) + 2, 0
        while j < len(line):
            c = line[j]
            if c == '"':
                j = skip_quoted(line, j)
                continue
            if c in "[{":
                depth += 1
            elif c in "]}":
                if depth == 0:
                    break
                depth -= 1
            elif c == "," and depth == 0:
                break
            j += 1
        if comma:
            start, end = i - 2, j
        else:
            start, end = i, j
            if line.startswith(", ", end):
                end += 2
        dropped += size(line[start:end])
        line = line[:start] + line[end:]
        i = start
    return line, dropped


def cut(lines, idx, field_depth, fields, nodes, ident, state):
    """節点 1 つ: 行の番号の列 idx（入れ子を除いた block）から本文を組み、要約値を nodes へ。"""
    whole = [f for f in fields if "." not in f]
    pairs = {}
    for f in fields:
        if "." in f:
            key, sub = f.split(".", 1)
            pairs.setdefault(key, []).append(sub)
    idx = list(idx)
    while idx and blank(lines[idx[-1]]):
        idx.pop()
    body, k = [], 0
    while k < len(idx):
        n = idx[k]
        line = lines[n]
        state["owned"][n] = True
        if not blank(line) and indent(line) == field_depth and field_key(line) in whole:
            state["edge"] += size(line)
            k += 1
            while k < len(idx) and (blank(lines[idx[k]]) or indent(lines[idx[k]]) > field_depth):
                state["owned"][idx[k]] = True
                state["edge"] += size(lines[idx[k]])
                k += 1
            continue
        s = line.lstrip(" ")
        key = field_key(line)
        if s.startswith("- {"):
            line, d = drop_pairs(line, whole)
            state["edge"] += d
        elif key is not None and s[len(key) + 2:].startswith("{"):
            line, d = drop_pairs(line, whole + pairs.get(key, []))
            state["edge"] += d
        body.append((n, line))
        k += 1
    while body and blank(body[-1][1]):
        state["owned"][body[-1][0]] = False
        body.pop()
    text = "".join(line for _, line in body)
    state["body"] += size(text)
    if ident in nodes:
        raise SystemExit("まだ分からない（節点 " + ident + " が 2 度ある）")
    nodes[ident] = hashlib.sha256(text.encode("utf-8")).hexdigest()[:8]


def scan(name, text, nodes, state):
    """正本 1 file を切り分けて節点を nodes へ足す。戻り値 = 残差の文。"""
    lines = lines_of(text)
    state["owned"] = [False] * len(lines)
    if name.startswith("adr/"):
        ident = next((l[len("id: "):].strip() for l in lines if l.startswith("id: ")), None)
        if not ident:
            raise SystemExit("まだ分からない（" + name + " に id が無い）")
        cut(lines, list(range(len(lines))), 0, EDGE_FIELDS["adr"], nodes, ident, state)
    else:
        fields = EDGE_FIELDS[name]
        section, i = None, 0
        while i < len(lines):
            line = lines[i]
            if not blank(line) and indent(line) == 0:
                key = field_key(line)
                section = key if key in SECTIONS[name] else None
            hit = head_id(line, 2) if section else None
            if hit is None:
                i += 1
                continue
            ident, flow = hit
            end = block_end(lines, i, 2, flow)
            kept, k = [], i
            while k < end:
                inner = head_id(lines[k], 6) if name == "constitution.yaml" and k > i else None
                if inner is None:
                    kept.append(k)
                    k += 1
                    continue
                sub_end = block_end(lines, k, 6, inner[1])
                cut(lines, list(range(k, sub_end)), 8, fields, nodes, inner[0], state)
                k = sub_end
            cut(lines, kept, 4, fields, nodes, ident, state)
            i = end
    return "".join(line for n, line in enumerate(lines) if not state["owned"][n])


def read(root, rel):
    with open(os.path.join(root, rel), encoding="utf-8", newline="") as f:
        return f.read()


def by_bytes(names):
    return sorted(names, key=lambda n: n.encode("utf-8"))


def population(root):
    """母集団: 索引の正本と、天井の正本の観点の reads が指す文書の file（dir 形は直下の .yaml）。"""
    files = {"constitution.yaml", "rules.yaml", "srs.yaml"}
    files.update("adr/" + n for n in os.listdir(os.path.join(root, "adr")) if ADR_NAME.match(n))
    lines = read(root, "ceiling.yaml").split("\n")
    documents, docs, part = {}, set(), None
    for line in lines:
        if line and not line.startswith(" ") and not line.startswith("#"):
            part = line.split(":", 1)[0]
        if part == "documents" and line.startswith("  - {id: "):
            row = dict(p.split(": ", 1) for p in line.strip()[3:].split(", ") if ": " in p)
            documents[row["id"]] = row["file"]
        if part == "viewpoints" and line.startswith("      - {doc: "):
            docs.add(line[len("      - {doc: "):].split(",", 1)[0].strip())
    for doc in docs:
        file = documents[doc]
        if file.endswith("/"):
            d = os.path.join(root, file)
            files.update(file + n for n in os.listdir(d) if n.endswith(".yaml") and os.path.isfile(os.path.join(d, n)))
        else:
            files.add(file)
    return by_bytes(files)


def main():
    if len(sys.argv) != 2:
        raise SystemExit("使い方: node-digest.py <設計文書の置き場>")
    root = sys.argv[1]
    nodes, state = {}, {"body": 0, "edge": 0}
    rest, total = [], 0
    for rel in population(root):
        text = read(root, rel)
        total += size(text)
        is_node = rel in SECTIONS or (rel.startswith("adr/") and ADR_NAME.match(rel[len("adr/"):]))
        if is_node:
            rest.append(scan(rel, text, nodes, state))
        else:
            rest.append(text)
    rest = "".join(rest).encode("utf-8")
    out = ["# 節点の要約値（1 行 = id / 要約値 8 字・タブ区切り・id の byte 順）"]
    out += [ident + "\t" + nodes[ident] for ident in by_bytes(nodes)]
    out.append("# 残差 sha256 " + hashlib.sha256(rest).hexdigest())
    out.append(
        "# 節点 {}・本文の byte {}・辺の欄の byte {}・残差の byte {}・合計 {}".format(
            len(nodes), state["body"], state["edge"], len(rest), total
        )
    )
    sys.stdout.buffer.write(("\n".join(out) + "\n").encode("utf-8"))


if __name__ == "__main__":
    main()
