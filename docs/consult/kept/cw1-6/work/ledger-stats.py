#!/usr/bin/env python3
"""台帳の写し（bundle/ledger.json・束を組んだ時の bd の字）から tsuzuri の設計の動きを出す（読むだけ）。
bead の種類（題名の頭: memo・問い・便・契約・ほか）と状態・日ごとの起票と閉じ・閉じの理由の頭・
持ち主への問いの数・「並列」「失敗」「ぶれ」に触れる memo の題名。"""
import json, collections, re, sys, pathlib

WS = pathlib.Path(__file__).resolve().parent.parent
items = json.load(open(WS / "bundle/ledger.json"))
print(f"# beads in the ledger copy: {len(items)}")

def kind_of(title):
    for head in ("memo", "問い", "便", "契約"):
        if title.startswith(head):
            return head
    return "other"

def day_of(s):
    return (s or "")[:10]

kinds = collections.Counter()
status = collections.Counter()
created_day = collections.Counter()
closed_day = collections.Counter()
questions_day = collections.Counter()
close_reasons = collections.Counter()
keys = collections.Counter()
for it in items:
    for k in it.keys():
        keys[k] += 1
    title = it.get("title", "")
    kind = kind_of(title)
    kinds[kind] += 1
    status[(kind, it.get("status", "-"))] += 1
    created_day[day_of(it.get("created_at") or it.get("created"))] += 1
    if it.get("status") == "closed":
        closed_day[day_of(it.get("closed_at") or it.get("updated_at") or it.get("updated"))] += 1
        reason = (it.get("close_reason") or it.get("closeReason") or "")[:40]
        close_reasons[(kind, reason)] += 1
    if kind == "問い":
        questions_day[day_of(it.get("created_at") or it.get("created"))] += 1

print("\n# item keys seen:", ", ".join(k for k, _ in keys.most_common(20)))
print("\n# kinds by title head")
for k, n in kinds.most_common():
    print(f"{n:5d}  {k}")
print("\n# status by kind")
for (k, s), n in sorted(status.items()):
    print(f"{n:5d}  {k:<5} {s}")
print("\n# created per day (all) / 問い per day")
for d in sorted(x for x in set(created_day) | set(questions_day) if x):
    print(f"{d}  created {created_day[d]:3d}  問い {questions_day[d]:3d}  closed {closed_day[d]:3d}")
print("\n# close reasons by kind (top 15)")
for (k, r), n in close_reasons.most_common(15):
    print(f"{n:5d}  {k:<5} {r}")

words = ["並列", "失敗", "落ち", "FAIL", "INCONCLUSIVE", "ぶれ", "食い違", "撃ち直", "再試行", "遅い", "詰ま"]
print("\n# memos and questions whose title mentions the pain words (latest 40)")
hits = []
for it in items:
    t = it.get("title", "")
    if kind_of(t) in ("memo", "問い") and any(w in t for w in words):
        hits.append((it.get("created_at") or it.get("created") or "", it.get("id"), it.get("status"), t[:120]))
for c, i, s, t in sorted(hits)[-40:]:
    print(f"{c[:10]} {i:<16} {s:<7} {t}")
