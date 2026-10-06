#!/usr/bin/env python3
"""器の fleet の記録（events.jsonl・読むだけ）から tsuzuri の便の実測を出す。
鍵の形（2026-10-06 の実物）: RunCreated{run,bead,stage:Intake} / RunStage{run,bead,stage,detail} / RunDone{run,bead,stage,detail} /
RunStopped{run,bead,stage:Stopped} / DispatchMark{bead,mark,detail}。RunStage の detail は "verdict:FAIL kind:xxx" などの字。
出す物: 日ごとの便の数・終わりの段の分布・審査と門の verdict と kind の分布・bead ごとの便の数（撃ち直し）・便の長さ・同時に走った便の最大・hold の理由の頭。"""
import json, collections, datetime as dt, re, sys

STATE = "/home/shuu5/.local/state/scribe2-v2-state-tsuzuri"
SINCE = sys.argv[1] if len(sys.argv) > 1 else "2026-09-22"
KINDS = ("RunCreated", "RunStage", "RunDone", "RunStopped", "DispatchMark")

def ts(ev):
    try:
        return dt.datetime.strptime(ev.get("ts", ""), "%Y-%m-%dT%H:%M:%SZ")
    except Exception:
        return None

events = []
with open(f"{STATE}/fleet/events.jsonl") as f:
    for line in f:
        try:
            ev = json.loads(line)
        except Exception:
            continue
        if not isinstance(ev, dict) or ev.get("kind") not in KINDS:
            continue
        t = ts(ev)
        if not t or t.strftime("%Y-%m-%d") < SINCE:
            continue
        events.append((t, ev))
events.sort(key=lambda x: x[0])

created, last_stage, first_done = {}, {}, {}
per_day_created = collections.Counter()
per_day_landed = collections.Counter()
per_day_stopped = collections.Counter()
verdicts = collections.Counter()
stage_seq = collections.Counter()
hold_reasons = collections.Counter()
marks = collections.Counter()
for t, ev in events:
    k = ev["kind"]
    run = ev.get("run", "?")
    day = t.strftime("%Y-%m-%d")
    if k == "RunCreated":
        created[run] = (t, ev)
        per_day_created[day] += 1
    elif k == "RunStage":
        stage = ev.get("stage", "-")
        detail = ev.get("detail", "") or ""
        stage_seq[stage] += 1
        m = re.search(r"verdict:([A-Z]+)(?:[ ,]kind:([a-z-]+))?", detail)
        if m:
            verdicts[(stage, m.group(1), m.group(2) or "-")] += 1
        last_stage[run] = stage
    elif k in ("RunDone", "RunStopped"):
        stage = ev.get("stage", k)
        if run not in first_done:
            first_done[run] = (t, stage)
            if stage == "Landed":
                per_day_landed[day] += 1
            else:
                per_day_stopped[day] += 1
    elif k == "DispatchMark":
        marks[ev.get("mark", "-")] += 1
        if ev.get("mark") == "hold":
            d = (ev.get("detail") or "")
            hold_reasons[d[:70]] += 1

print(f"# since {SINCE}: runs created {len(created)}, runs ended {len(first_done)}")
print("\n# per day: created / landed / stopped-or-other")
for d in sorted(set(per_day_created) | set(per_day_landed) | set(per_day_stopped)):
    print(f"{d}  created {per_day_created[d]:3d}  landed {per_day_landed[d]:3d}  other {per_day_stopped[d]:3d}")

ends = collections.Counter(stage for _, stage in first_done.values())
print("\n# how runs ended (first RunDone/RunStopped stage)")
for stage, n in ends.most_common():
    print(f"{n:5d}  {stage}")

print("\n# RunStage counts")
for stage, n in stage_seq.most_common():
    print(f"{n:5d}  {stage}")

print("\n# verdicts by stage (stage, verdict, kind)")
for (stage, v, kind), n in verdicts.most_common(30):
    print(f"{n:5d}  {stage:<12} {v:<13} {kind}")

print("\n# dispatch marks")
for m, n in marks.most_common():
    print(f"{n:5d}  {m}")
print("# hold reasons (head 70 chars, top 12)")
for r, n in hold_reasons.most_common(12):
    print(f"{n:5d}  {r}")

bead_runs = collections.Counter(ev.get("bead", "-") for _, ev in created.values())
dist = collections.Counter(bead_runs.values())
print(f"\n# beads with runs: {len(bead_runs)}; runs-per-bead distribution")
for n_runs in sorted(dist):
    print(f"  {n_runs} run(s): {dist[n_runs]} beads")
print("# beads with the most runs")
for bead, n in bead_runs.most_common(10):
    print(f"  {n:3d}  {bead}")

# 便の長さと同時実行
intervals = []
for run, (t0, _) in created.items():
    if run in first_done:
        intervals.append((t0, first_done[run][0], first_done[run][1]))
durs = sorted((t1 - t0).total_seconds() / 60 for t0, t1, _ in intervals)
if durs:
    print(f"\n# run duration minutes: n={len(durs)} median={durs[len(durs)//2]:.1f} p90={durs[int(len(durs)*0.9)]:.1f} max={durs[-1]:.0f}")
    landed = sorted((t1 - t0).total_seconds() / 60 for t0, t1, s in intervals if s == "Landed")
    if landed:
        print(f"# landed runs minutes: n={len(landed)} median={landed[len(landed)//2]:.1f} p90={landed[int(len(landed)*0.9)]:.1f}")
edges = []
for t0, t1, _ in intervals:
    edges.append((t0, 1)); edges.append((t1, -1))
edges.sort()
cur, peak = 0, collections.Counter()
busy_minutes = collections.Counter()
prev_t = None
for t, d in edges:
    if prev_t is not None and cur > 0:
        busy_minutes[prev_t.strftime("%Y-%m-%d")] += (t - prev_t).total_seconds() / 60
    cur += d
    day = t.strftime("%Y-%m-%d")
    peak[day] = max(peak[day], cur)
    prev_t = t
print("\n# per day: peak concurrent runs / minutes with >=1 run live")
for d in sorted(peak):
    print(f"{d}  peak {peak[d]}  busy {busy_minutes[d]:6.0f} min")
