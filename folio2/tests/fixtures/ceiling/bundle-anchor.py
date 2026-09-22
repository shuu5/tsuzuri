#!/usr/bin/env python3
"""天井の材料の束の凍結 anchor を組む独立の script（便 97・docs/design/delivery-97.md §1 (c)・P-10.1 / P-10.2）。

folio の code を 1 行も呼ばない（python3 の標準 library だけ・PyYAML を使わない）。凍結の土台 bundle/ の正本（source/）と
面（faces/）から 4 観点の束（sources/・faces/・question.yaml・finding.yaml・reads.yaml）を memory の上で組み直し、
観点ごとに file の数・連結の byte 数・要約値を bundle-anchor.txt と同じ字で標準出力へ書く。

- 天井の正本 ceiling.yaml から読むのは documents・viewpoints・weights と、生成区間 schema の finding・verdicts・record
- sources/: documents の file 形は byte のまま 1 本、dir 形（末尾が /）は直下の .yaml を名の byte 順に全部
- faces/: doc の id ごとの面の名の形（下の FACES）に当たる faces/ の直下の file
- 要約値: 束の file を観点の dir からの相対 path の byte 順に並べ、中身を区切りなしに連結した sha256
- 絞り（便 98・docs/design/delivery-98.md §1 (b)(c)(d)）: sources/ の写しは観点の reads が挙げた最上位の節と骨格 6 語と
  file の頭の行だけを byte のまま残す。落とした節と 宣言に在るが正本に無い節 は reads.yaml の末尾に注釈の行で出し、
  その名の総数を 落とした節の数・正本に無い節の数 として書く
"""

import hashlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
SOURCE = os.path.join(HERE, "bundle", "source")
FACES_DIR = os.path.join(HERE, "bundle", "faces")

# 文書の id → 面の名の形（("exact", 名) か ("affix", 頭, 尾)）。無い id は面を持たない
FACES = {
    "index": ("exact", "index.html"),
    "constitution": ("exact", "constitution.html"),
    "srs": ("exact", "srs.html"),
    "adr": ("affix", "adr-", ".html"),
    "design-note": ("affix", "note-", ".html"),
}


def split_top(text):
    """flow の中身を最上位の `,` で割る（[] と {} の入れ子は割らない）。"""
    parts, depth, cur = [], 0, ""
    for ch in text:
        if ch in "[{":
            depth += 1
        elif ch in "]}":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        parts.append(cur.strip())
    return parts


def scalar(text):
    text = text.strip()
    if len(text) >= 2 and text[0] == text[-1] and text[0] in "\"'":
        return text[1:-1]
    return text


def flow(text):
    """`{k: v, ...}` は dict、`[a, ...]` は list、ほかは文。"""
    text = text.strip()
    if text.startswith("{") and text.endswith("}"):
        out = {}
        for part in split_top(text[1:-1]):
            key, _, value = part.partition(":")
            out[key.strip()] = flow(value)
        return out
    if text.startswith("[") and text.endswith("]"):
        return [flow(p) for p in split_top(text[1:-1])]
    return scalar(text)


def section(lines, name, indent=0):
    """字下げ indent の `name:` の行の下で、字下げが深い行を返す（注釈と空行は除く）。"""
    head = " " * indent + name + ":"
    for i, line in enumerate(lines):
        if line == head or line.startswith(head + " "):
            body = []
            for rest in lines[i + 1:]:
                if rest.strip() == "" or rest.lstrip().startswith("#"):
                    continue
                if len(rest) - len(rest.lstrip(" ")) <= indent:
                    break
                body.append(rest)
            return line[len(head):].strip(), body
    raise SystemExit("ceiling.yaml: " + name + ": 読めない")


def value(lines, name, indent):
    return flow(section(lines, name, indent)[0])


def load():
    with open(os.path.join(SOURCE, "ceiling.yaml"), encoding="utf-8") as f:
        lines = f.read().split("\n")
    documents = {}
    for line in section(lines, "documents")[1]:
        row = flow(line.strip()[2:])
        documents[row["id"]] = row["file"]
    viewpoints, cur = [], None
    for line in section(lines, "viewpoints")[1]:
        if line.startswith("  - id: "):
            cur = {"id": scalar(line[len("  - id: "):]), "reads": []}
            viewpoints.append(cur)
        elif line.startswith("      - "):
            row = flow(line[len("      - "):])
            cur["reads"].append((row["doc"], row["fields"]))
        elif line.startswith("    ") and not line.startswith("    reads:"):
            key, _, rest = line.strip().partition(":")
            cur[key] = scalar(rest)
    weights = section(lines, "weights")[1]
    schema = section(lines, "schema")[1]
    finding = section(schema, "finding", 2)[1]
    join = ", ".join
    finding_text = (
        "# 所見の欄の決まり（天井の正本 ceiling.yaml の finding・weights・verdicts・record の写し・folio ceiling が組んだ）\n"
        "finding:\n"
        "  required: [" + join(value(finding, "required", 4)) + "]\n"
        "  optional: [" + join(value(finding, "optional", 4)) + "]\n"
        "  place: {required: [" + join(value(finding, "place", 4)["required"]) + "]}\n"
        "  refute: {values: [" + join(value(finding, "refute", 4)["values"]) + "]}\n"
        "weights:\n"
        "  values: [" + join(value(weights, "values", 2)) + "]\n"
        "  refute: [" + join(value(weights, "refute", 2)) + "]\n"
        "verdicts:\n"
        "  values: [" + join(value(schema, "verdicts", 2)["values"]) + "]\n"
        "record:\n"
        "  required: [" + join(value(schema, "record", 2)["required"]) + "]\n"
    )
    return documents, viewpoints, finding_text


def read(path):
    with open(path, "rb") as f:
        return f.read()


def face_hits(form, names):
    if form[0] == "exact":
        return [n for n in names if n == form[1]]
    head, tail = form[1], form[2]
    return [n for n in names if len(n) > len(head) + len(tail) and n.startswith(head) and n.endswith(tail)]


def is_trivia(line):
    stripped = line.strip()
    return stripped == "" or stripped.startswith("#")


def head_name(line):
    """列 0 の `名:`（: の後が空か空白）なら名、ほかは None。列 0 の `- ` は節の続き。"""
    if line == "" or line[0] in " \t#-":
        return None
    body = line.rstrip("\r\n")
    i = body.find(":")
    while i != -1:
        if i > 0 and (i + 1 == len(body) or body[i + 1] in " \t"):
            return body[:i]
        i = body.find(":", i + 1)
    return None


def cut(raw, keep):
    """最上位の節で切る（便 98 §1 (b) 規則 1〜4）。戻り値 = (残した byte, 節の名の列, 落とした節の名の列)。"""
    lines = raw.decode("utf-8").splitlines(keepends=True)
    names = [head_name(line) for line in lines]
    owner = [None] * len(lines)  # None = file の頭、ほかは節の番号
    sections = []
    cur = None
    i = 0
    while i < len(lines):
        if names[i] is not None:
            sections.append(names[i])
            cur = len(sections) - 1
            owner[i] = cur
            i += 1
            continue
        if cur is not None and is_trivia(lines[i]):
            j = i
            while j < len(lines) and is_trivia(lines[j]):
                j += 1
            if j < len(lines) and names[j] is not None:
                target = len(sections)  # 直後の節へ寄せる
            else:
                target = cur  # 直後が節の続きか file の末尾なら直前の節へ
            for k in range(i, j):
                owner[k] = target
            i = j
            continue
        owner[i] = cur
        i += 1
    # 生成区間は 1 つの塊（規則 4）
    begin = [k for k, line in enumerate(lines) if line.startswith("# folio:schema:begin")]
    end = [k for k, line in enumerate(lines) if line.startswith("# folio:schema:end")]
    groups = {}
    if begin and end and begin[0] < end[0]:
        inside = sorted({owner[k] for k in range(begin[0], end[0] + 1) if names[k] is not None})
        for k in range(begin[0], end[0] + 1):
            groups[k] = inside
    kept_section = [name in keep for name in sections]
    out = []
    for k, line in enumerate(lines):
        if k in groups:
            live = any(kept_section[s] for s in groups[k])
        elif owner[k] is None:
            live = True
        else:
            live = owner[k] < len(sections) and kept_section[owner[k]]
        if live:
            out.append(line)
    dropped = [name for name, kept in zip(sections, kept_section) if not kept]
    return "".join(out).encode("utf-8"), sections, dropped


SKELETON = ["meta", "id", "title", "status", "date", "schema"]


def build(vp, documents, finding_text):
    files = {}
    face_names = sorted(
        (n for n in os.listdir(FACES_DIR) if os.path.isfile(os.path.join(FACES_DIR, n))),
        key=lambda n: n.encode("utf-8"),
    )
    declared = {}
    for doc, fields in vp["reads"]:
        for field in fields:
            top = field.split(".")[0]
            declared.setdefault(doc, [])
            if top not in declared[doc]:
                declared[doc].append(top)
    dropped, present, done = {}, {}, []
    for doc, _ in vp["reads"]:
        if doc in done:
            continue
        done.append(doc)
        file = documents[doc]
        path = os.path.join(SOURCE, file)
        keep = set(declared[doc]) | set(SKELETON)
        present[doc] = set()
        paths = []
        if file.endswith("/"):
            for name in sorted(os.listdir(path), key=lambda n: n.encode("utf-8")):
                if name.endswith(".yaml") and os.path.isfile(os.path.join(path, name)):
                    paths.append((file + name, os.path.join(path, name)))
        else:
            paths.append((file, path))
        for rel, full in paths:
            body, sections, gone = cut(read(full), keep)
            files["sources/" + rel] = body
            present[doc].update(sections)
            if gone:
                dropped[rel] = gone
        if doc in FACES:
            for name in face_hits(FACES[doc], face_names):
                files["faces/" + name] = read(os.path.join(FACES_DIR, name))
    block = "".join("  " + line + "\n" for line in vp["reader"].split("\n"))
    qblock = "".join("  " + line + "\n" for line in vp["question"].split("\n"))
    question = "id: " + vp["id"] + "\nname: " + vp["name"] + "\nreader: |\n" + block + "question: |\n" + qblock
    files["question.yaml"] = question.encode("utf-8")
    files["finding.yaml"] = finding_text.encode("utf-8")
    reads = "".join("- {doc: " + d + ", fields: [" + ", ".join(fs) + "]}\n" for d, fs in vp["reads"])
    for rel in sorted(dropped, key=lambda r: r.encode("utf-8")):
        reads += "# 落とした節 " + rel + ": " + ", ".join(dropped[rel]) + "\n"
    absent = 0
    for doc in done:
        missing = [name for name in declared[doc] if name not in present[doc]]
        if missing:
            reads += "# 宣言に在るが正本に無い節 " + documents[doc] + ": " + ", ".join(missing) + "\n"
            absent += len(missing)
    reads += "# 常に残す節: " + ", ".join(SKELETON) + "\n"
    files["reads.yaml"] = reads.encode("utf-8")
    return files, sum(len(g) for g in dropped.values()), absent


def main():
    documents, viewpoints, finding_text = load()
    out = ["# 凍結 anchor: 天井の材料の束（tests/fixtures/ceiling/bundle-anchor.py が組んだ・folio の code を 1 行も呼ばない）"]
    for vp in viewpoints:
        files, dropped, absent = build(vp, documents, finding_text)
        joined = b"".join(files[k] for k in sorted(files, key=lambda k: k.encode("utf-8")))
        out.append("観点\t" + vp["id"])
        out.append("file数\t" + str(len(files)))
        out.append("byte\t" + str(len(joined)))
        out.append("要約値\t" + hashlib.sha256(joined).hexdigest())
        out.append("落とした節の数\t" + str(dropped))
        out.append("正本に無い節の数\t" + str(absent))
    sys.stdout.buffer.write(("\n".join(out) + "\n").encode("utf-8"))


if __name__ == "__main__":
    main()
