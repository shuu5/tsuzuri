#!/usr/bin/env python3
"""folio2 design-intent の床（day-1 の暫定 script・M0 で folio の検査に置換する）。使い方: python3 scripts/check_draft.py（repo root で・design-intent/*.yaml を読む）。

検査（すべて決定的・fail-closed: 読めない／parse できないは rc 2）:
  schema  : 憲法 schema 節の top_level / meta / precedence / article / statement / mechanism の欄集合（未知の欄は落とす = N-3）
            enums / tier_mechanism_allowed / live / stage・polarity（reject・build-check に必須）/ 1 文 1 極性（R-11）/ plain 必須（R-10）
  refs    : 参照 id の未解決 0（R-4・母集団 = 正本 4 file の全欄・内部 3 空間 = 要件書 id・条 id〔枝番含む〕・rules 行 id）
            どの条からも参照されない rules 行 0（R-4 双方向）/ rules 行の article ⇔ 条の relations.rules
  rules   : rules schema（top_level / 行の欄 / enums / 凍結行の裁定 id）/ kind_map（R 行だけ）
  vocab   : 本文（憲法の見出し・規範文・平易文・前文 / rules の what / 要件書の見出し・規範文・平易文・制約）の英字の語のうち
            語彙（terms・field_terms の term / en・identifiers）に無いもの（R-9・R-12 の機械側）/ 語彙の定義の閉包（定義文の英字語も同じ母集団）
  inject  : 注入の母集団（前文 + 規範文）の本数が inject_check.derive と一致
  polarity: 極性一覧（P-18.3）を mechanism の stage / polarity と rules 行の stage から生成し、in-loop の本数を出す
            （P-18.4「in-loop 0 なら落とす」は live: delivery-0 ゆえ発効時点では「まだ分からない」として出す）
  counts  : meta.counts と実数の一致
rc: 0 = 全部通った / 1 = 違反あり / 2 = 読めない
"""
import re, sys, pathlib, collections, argparse
try:
    import yaml
except ImportError:
    print('check_draft: pyyaml が無い', file=sys.stderr); sys.exit(2)

ap = argparse.ArgumentParser(); ap.add_argument('--dir', default='design-intent'); ap.add_argument('--names', default='placed', choices=['draft', 'placed'])
args = ap.parse_args()
HERE = pathlib.Path(args.dir).resolve()
SUF = '.draft.yaml' if args.names == 'draft' else '.yaml'
def load(name):
    p = HERE / (name + SUF)
    if not p.exists():
        print(f'check_draft: 正本が無い: {p}', file=sys.stderr); sys.exit(2)
    try:
        return yaml.safe_load(p.read_text(encoding='utf-8'))
    except Exception as ex:
        print(f'check_draft: parse できない: {p}: {ex}', file=sys.stderr); sys.exit(2)

c = load('constitution'); r = load('rules'); v = load('vocabulary'); s = load('srs')
errs = []
def err(kind, msg): errs.append((kind, msg))
def keys_ok(kind, where, obj, spec):
    ks = set(obj.keys()); req = set(spec.get('required', [])); opt = set(spec.get('optional', []))
    if not req <= ks: err(kind, f"{where}: 必須欄が無い: {sorted(req - ks)}")
    if ks - req - opt: err(kind, f"{where}: 未知の欄（N-3）: {sorted(ks - req - opt)}")

# ── schema（憲法）──
sc = c['schema']; en = sc['enums']
if set(c.keys()) - set(sc['top_level']): err('schema', f"憲法の未知の節（N-3）: {sorted(set(c.keys()) - set(sc['top_level']))}")
keys_ok('schema', 'meta', c['meta'], sc['meta'])
pr = c.get('precedence') or {}
keys_ok('schema', '前文', pr, sc['precedence'])
def mech_ok(where, m, tier=None):
    keys_ok('schema', f'{where}.mechanism', m, sc['mechanism'])
    if m.get('kind') not in en['mechanism_kind']: err('schema', f"{where}: mechanism.kind が値域外: {m.get('kind')}")
    if m.get('live') not in en['mechanism_live']: err('schema', f"{where}: mechanism.live が値域外: {m.get('live')}")
    if m.get('kind') == 'none' and not m.get('note'): err('schema', f"{where}: mechanism none に理由が無い")
    if m.get('kind') in sc['mechanism']['stage_polarity_required_for']:
        if m.get('stage') not in en['stage']: err('schema', f"{where}: mechanism.stage が無いか値域外（極性一覧の元・P-18.3）: {m.get('stage')}")
        if m.get('polarity') not in en['polarity']: err('schema', f"{where}: mechanism.polarity が無いか値域外: {m.get('polarity')}")
    if tier and m.get('kind') not in sc['tier_mechanism_allowed'][tier]: err('schema', f"{where}: 段 {tier} に機構 {m.get('kind')} は座れない（G3）")
mech_ok('前文', pr.get('mechanism') or {})
ids = [a['id'] for a in c['articles']]; st_ids = set()
if len(ids) != len(set(ids)): err('schema', f'条 id が重複: {[i for i, n in collections.Counter(ids).items() if n > 1]}')
for a in c['articles']:
    keys_ok('schema', a.get('id', '?'), a, sc['article'])
    if a.get('tier') not in en['tier']: err('schema', f"{a['id']}: tier が値域外: {a.get('tier')}")
    if a.get('binds') not in en['binds']: err('schema', f"{a['id']}: binds が値域外: {a.get('binds')}")
    mech_ok(a['id'], a.get('mechanism') or {}, a['tier'])
    if not str(a.get('plain') or '').strip(): err('R-10', f"{a['id']}: plain が無い")
    for x in a.get('rationale') or []:
        if set(x.keys()) != {'kind', 'ref'}: err('schema', f"{a['id']}: rationale の欄が壊れている（flow mapping の「,」）: {sorted(x.keys())}")
        elif x['kind'] not in en['rationale_kind']: err('schema', f"{a['id']}: rationale.kind が値域外: {x['kind']}")
    if a.get('retreat'):
        if set(a['retreat'].keys()) != {'kind', 'condition'}: err('schema', f"{a['id']}: retreat の欄が {sorted(a['retreat'].keys())}")
        elif a['retreat'].get('kind') not in en['retreat_kind']: err('schema', f"{a['id']}: retreat.kind が値域外")
    if not a.get('statements'): err('schema', f"{a['id']}: 規範文が 0 本")
    for st in a.get('statements') or []:
        if set(st.keys()) != set(sc['statement']['required']): err('schema', f"{st.get('id')}: 規範文の欄が {sorted(st.keys())}")
        if st.get('pattern') not in en['pattern']: err('schema', f"{st['id']}: pattern が値域外: {st.get('pattern')}")
        if st.get('strength') not in en['strength']: err('schema', f"{st['id']}: strength が値域外: {st.get('strength')}")
        t = str(st.get('text', '')).strip()
        if not t.endswith('。'): err('schema', f"{st['id']}: 「。」で終わらない")
        if t.count('。') != 1: err('schema', f"{st['id']}: 1 文でない（「。」が {t.count('。')}）")
        neg = t.rstrip('。').endswith('ない')
        if (st.get('strength') == 'must-not') != neg: err('R-11', f"{st['id']}: strength={st.get('strength')} と文末「…{t[-8:]}」が不一致")
        if not st['id'].startswith(a['id'] + '.'): err('schema', f"{st['id']}: 条 id {a['id']} の枝番でない")
        st_ids.add(st['id'])
cnt = collections.Counter(a['tier'] for a in c['articles'])
for k, n in c['meta']['counts'].items():
    if cnt.get(k, 0) != n: err('counts', f"meta.counts.{k}={n} だが実数 {cnt.get(k, 0)}")

# ── rules ──
rs = r['schema']; ren = rs['enums']
if set(r.keys()) - set(rs['top_level']): err('rules', f"rules の未知の節（N-3）: {sorted(set(r.keys()) - set(rs['top_level']))}")
rule_ids = []; rows = {}
for sect, spec in (('thresholds', rs['threshold_row']), ('discipline', rs['discipline_row'])):
    for row in r.get(sect) or []:
        rid = row.get('id'); rule_ids.append(rid); rows[rid] = (sect, row)
        keys_ok('rules', rid, row, spec)
        if row.get('kind') not in ren['kind']: err('rules', f"{rid}: kind が値域外: {row.get('kind')}")
        if row.get('status') not in ren['status']: err('rules', f"{rid}: status が値域外: {row.get('status')}")
        if 'stage' in row and row.get('stage') not in ren['stage']: err('rules', f"{rid}: stage が値域外: {row.get('stage')}")
        if row.get('article') not in ids: err('refs', f"{rid}: article {row.get('article')} が条に無い")
        if row.get('status') == '凍結' and not row.get('ruling'): err('rules', f"{rid}: 凍結なのに裁定 id が無い（P-17）")
if len(rule_ids) != len(set(rule_ids)): err('rules', '行 id が重複')
art_mech = {a['id']: a['mechanism']['kind'] for a in c['articles']}
for rid, (sect, row) in rows.items():
    if sect != 'thresholds': continue
    allowed = rs['kind_map_to_constitution'].get(row['kind'], [])
    if art_mech.get(row['article']) not in allowed: err('rules', f"{rid}: kind {row['kind']} と条 {row['article']} の機構 {art_mech.get(row['article'])} が写像表に合わない")

# ── refs（R-4・母集団 = 4 file の全欄）──
req_ids = set()
for sect in ('goals', 'requirements', 'nonfunctional', 'acceptance', 'constraints', 'actors', 'outputs'):
    for x in s.get(sect) or []: req_ids.add(x['id'])
known_ids = set(ids) | st_ids | req_ids | set(rule_ids)
sections = {'§5', '§6', '§7', '§8'}
ID_RE = re.compile(r'(?<![A-Za-z0-9-])((?:P|A|N)-\d+(?:\.\d+)?|(?:FR|NFR|AC|CON|GOAL)\d+|(?:R|D)-\d+)(?![A-Za-z0-9])')
def walk(obj, where):
    if isinstance(obj, dict):
        for k, vv in obj.items(): walk(vv, f'{where}.{k}')
    elif isinstance(obj, list):
        for i, vv in enumerate(obj): walk(vv, f'{where}[{i}]')
    elif isinstance(obj, str):
        for m in ID_RE.findall(obj):
            if m not in known_ids: err('refs', f"{where}: id {m} が実在しない")
for name, obj in (('憲法', {k: (vv if k != 'meta' else {kk: x for kk, x in vv.items() if not kk.startswith('changes_from')}) for k, vv in c.items() if k != 'schema'}), ('rules', {k: vv for k, vv in r.items() if k != 'schema'}), ('語彙', v), ('要件書', s)):
    walk(obj, name)
referenced_rules = set()
for a in c['articles']:
    rel = a.get('relations') or {}
    if not isinstance(rel, dict): err('refs', f"{a['id']}: relations が型付き map でない"); continue
    for k in rel:
        if k not in ('reqs', 'rules', 'articles', 'sections'): err('refs', f"{a['id']}: relations の未知の名前空間 {k}")
    for x in rel.get('sections', []):
        if x not in sections: err('refs', f"{a['id']}: 節 {x} が無い")
    referenced_rules |= set(rel.get('rules', []))
    for st in a['statements']:
        referenced_rules |= set(re.findall(r'\b([RD]-\d+)\b', st['text']))
for rid in rule_ids:
    if rid not in referenced_rules: err('R-4', f"rules 行 {rid} はどの条からも参照されていない（双方向）")
art_rules = {a['id']: set((a.get('relations') or {}).get('rules', [])) for a in c['articles']}
for rid, (sect, row) in rows.items():
    if rid not in art_rules.get(row['article'], set()): err('R-4', f"{rid}: article={row['article']} だが {row['article']} の relations.rules に無い")

# ── vocab（R-9 / R-12 の機械側）──
known = set()
for t in (v.get('terms') or []) + (v.get('field_terms') or []):
    for w in re.findall(r'[A-Za-z][A-Za-z0-9\-\.]*[A-Za-z0-9]|[A-Za-z]', (t.get('term') or '') + ' ' + (t.get('en') or '')): known.add(w.lower())
for g in v.get('identifiers') or []:
    for w in g.get('words') or []: known.add(str(w).lower())
IDENT = re.compile(r'^(P|A|N|FR|NFR|AC|CON|GOAL|R|D)-?\d|^[RD]-n$|^[a-z]\d-[0-9a-z]+(\.\d+)?$')   # 条・要件・rules 行の id / R-n・D-n / 台帳 id（f2-648・s2-07l.149）
GLOSS = re.compile(r'[\u3040-\u30ff\u4e00-\u9fff][^（）]*?（([^（）]*)）')   # 「日本語（原語）」の形＝グロス済み（R-12 の免除）
def words(text): return re.findall(r'[A-Za-z][A-Za-z0-9\-\.]*[A-Za-z0-9]|[A-Za-z]', text)
body = []
for a in c['articles']:
    body.append((a['id'] + ' title', a['title'])); body.append((a['id'] + ' plain', a['plain']))
    for st in a['statements']: body.append((st['id'], st['text']))
body.append(('前文', pr.get('text', ''))); body.append(('前文 plain', pr.get('plain', '')))
for sect in ('thresholds', 'discipline'):
    for row in r.get(sect) or []: body.append((row['id'] + ' what', str(row['what'])))
for sect in ('requirements', 'nonfunctional'):
    for x in s.get(sect) or []:
        for k in ('title', 'when', 'shall', 'plain'): body.append((f"要件書 {x['id']} {k}", str(x.get(k, ''))))
for x in s.get('acceptance') or []:
    for k in ('title', 'plain'): body.append((f"要件書 {x['id']} {k}", str(x.get(k, ''))))
for x in s.get('constraints') or []:
    for k in ('title', 'text'): body.append((f"要件書 {x['id']} {k}", str(x.get(k, ''))))
for x in s.get('goals') or []: body.append((f"要件書 {x['id']}", x.get('title', '') + ' ' + x.get('text', '')))
for t in v.get('terms') or []: body.append((f"語彙 {t['id']} def", str(t.get('short', '')) + ' ' + str(t.get('def', ''))))
unknown = collections.Counter()
for where, text in body:
    glossed = {x.lower() for g in GLOSS.findall(text) for x in words(g)}
    for w in words(text):
        lw = w.lower()
        if IDENT.match(w) or lw in known or lw in glossed or re.fullmatch(r'[a-z]', lw) or ('--' + w) in text: continue
        unknown[(lw, where)] += 1
for (lw, where), n in sorted(unknown.items()):
    err('R-9', f"{where}: 語彙に無い英字の語「{lw}」")

# ── inject（母集団の本数が導出器と一致）──
try:
    sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
    import inject_check
    _, n_inj = inject_check.derive(HERE / ('constitution' + SUF))
    n_expect = sum(len(a['statements']) for a in c['articles']) + (1 if pr.get('text') else 0)
    if n_inj != n_expect: err('inject', f"注入の本数 {n_inj} ≠ 前文 + 規範文 {n_expect}")
except Exception as ex:
    err('inject', f"inject_check.derive を呼べない: {ex}")

# ── polarity（極性一覧・P-18.3）──
pol = []
for a in c['articles']:
    m = a['mechanism']
    if m.get('kind') in sc['mechanism']['stage_polarity_required_for']: pol.append((a['id'], m['kind'], m.get('stage'), m.get('polarity'), m.get('live')))
for rid, (sect, row) in rows.items():
    if sect == 'thresholds' and row.get('kind') in ('deny', 'build-check'): pol.append((rid, row['kind'], row.get('stage'), 'fail-closed', 'row'))
inloop = [x for x in pol if x[2] == 'in-loop']

# ── 出力 ──
by = collections.Counter(k for k, _ in errs)
for k, m in errs: print(f'[{k}] {m}')
n_st = sum(len(a['statements']) for a in c['articles'])
print(f"# 極性一覧: {len(pol)} 件（in-loop {len(inloop)} / post {len(pol) - len(inloop)}）。P-18.4（in-loop 0 なら落とす）= " + ('合格' if inloop else 'まだ分からない（live delivery-0・便 0 で編集時 guard が入るまで）'), file=sys.stderr)
print(f"# 条 {len(ids)}（{dict(cnt)}）/ 規範文 {n_st} / rules 行 {len(rule_ids)} / 語彙 {len(v.get('terms') or [])} 語 + 欄 {len(v.get('field_terms') or [])} + 識別子 {sum(len(g.get('words') or []) for g in (v.get('identifiers') or []))} / 違反 {len(errs)} {dict(by)}", file=sys.stderr)
sys.exit(1 if errs else 0)
