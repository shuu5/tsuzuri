#!/usr/bin/env python3
"""folio2 design-intent の床（day-1 の暫定 script・M0 で folio の検査に置換する）。使い方: python3 scripts/check_draft.py（repo root で・design-intent/*.yaml を読む）。引数: --dir <dir>（既定 design-intent）/ --names draft（*.draft.yaml を読む・執筆中の下書き用）/ --freeze-anchor / --emit-amends。
写しで回すときも版管理（git init + commit）の中で回す＝anchor の削除を版管理と照合できない写しは「まだ分からない」（rc 2）になる。

検査（すべて決定的・fail-closed: 読めない／parse できないは rc 2）:
  schema  : 憲法 schema 節の top_level / meta / precedence / article / statement / mechanism の欄集合（未知の欄は落とす = N-3）
            enums / tier_mechanism_allowed / live / stage・polarity（reject・build-check に必須）/ 1 文 1 極性（R-11）/ plain 必須（R-10）
            条 id の形（P-n / A-n / N-n・改訂の範囲の節名と衝突しない）/ 規範文 id の重複（全体）/ 写しの欄名は文字列で「.」を含まない（欄の道の衝突を塞ぐ）
  refs    : 参照 id の未解決 0（R-4・母集団 = 正本 4 file・内部 3 空間 = 要件書 id・条 id〔枝番含む・anchor の列に在った過去の id も知っている〕・rules 行 id）
            どの条からも参照されない rules 行 0（R-4 双方向）/ rules 行の article ⇔ 条の relations.rules
  rules   : rules schema（top_level / 行の欄 / enums / 凍結行の裁定 id）/ kind_map（R 行だけ）
  vocab   : 本文（憲法の見出し・規範文・平易文・前文 / rules の what / 要件書の見出し・規範文・平易文・制約）の英字の語のうち
            語彙（terms・field_terms の term / en・identifiers）に無いもの（R-9・R-12 の機械側）/ 語彙の定義の閉包（定義文の英字語も同じ母集団）
  polarity: 極性一覧（P-18.3）を mechanism の stage / polarity と rules 行の stage から生成し、in-loop の本数を出す
            （P-18.4「in-loop 0 なら落とす」は live: delivery-0 ゆえ発効時点では「まだ分からない」として出す）
  adr     : 判断の記録（adr/ADR-n.yaml・欄の決まりは adr/schema.yaml・ADR-1）の欄・値域・非空・退けた案（採用 1）・撤退条件（P-8.1）・承認欄（N-4・承認者・裁定 id の形・対話面 R-8）
            床が読む欄の決まりは床の定数（FLOOR）で、adr/schema.yaml の同名欄はそれと型も値も違わないことを要る（data 側で動かせない・N-3.1）。*_note の欄だけが人の説明（検査の外）
            発効した判断（accepted / retired + 承認欄）だけが条文の改訂を説明できる。amends ⇔ amended_by（A-2）。retired の後継の列は accepted に到達する（輪・未発効の後継は落とす）
            判断の記録の本文の英字語 0（「日本語（原語）」の形）・本文の参照 id の実在・basis が id の形、は欄の決まりの規則として無条件に課す（rules 行 R-4 / R-9 の行と母集団は変えない）
  anchor  : 凍結 anchor の列（anchors/index.yaml + anchors/constitution-<版>.yaml・ADR-2）。現行の写し ≡ 最新 anchor（N-4）・版の一致（A-2）
            anchor 全欄の digest（json 正規化・sha256）・索引との 1:1・previous の列（根 = v1.0・切れは rc 2・索引が在って entries が空は rc 2〔anchor file が残っていれば列の外として rc 1〕・索引だけの削除は rc 1。
            ただし版管理の履歴に anchor が在れば「履歴に在ったが無い」の違反が先に立つ＝rc 2 になるのは履歴にも anchor が無い写しだけ）。digest の方式が違う anchor は digest を検算できない（まだ分からない）が、写しの内容の照合（現行 ≡ anchor）は行う
            meta.approval の写しの一致・承認一覧 ≡ 各判断の記録の承認欄（凍結後の書き換えを落とす）・symlink 拒否（正本 4 file・anchors/ と adr/ とその中の各 file・design-intent 自体）
            列の全区間で、隣り合う anchor の差分を欄単位で全件並べ、その版を名指す発効した判断の amends と 1:1 に消し込む（余りも不足も落とす）＝凍結後に過去の版の記録を書き換えても落ちる
            値は型付き（一覧・表・数・真偽・null は json の 1 値・空文字は（空）の印）。規範文の改番・廃止した番号の再利用・印を値に持つ欄（現行の写し）を落とす（P-7.1）。版の綴りは v<数>.<数>
            版管理との照合（環境変数 GIT_* は継承しない・全 ref〔--all〕を見る）: HEAD にある anchor が作業ツリーに無い・履歴に一度でも在った anchor が無い（削除を commit しても）・履歴に在った同じ形式の anchor と中身が違う（差し替え・書き換え。形式が違う古い anchor は移行の痕跡として見ない）
            ・anchors/ か anchor file が版管理から除外（ignore・追跡済みでも）・最新版以外の未追跡の anchor・design-intent 自体が版管理の根・版管理の根が design-intent の上に無い、は落とす。版管理が無い・commit が無い（HEAD 無し）・浅い写し（shallow）・読めない、は「まだ分からない」（見えないは無いではない）
            発効した判断の記録が名指す版（amends.version）の anchor が列に無い（最新版を消して前の版へ戻す）・列の根の版を名指す amends（根には改訂前が無い＝架空の記録）は落とす
            列の根（v1.0）の anchor の digest は床の定数 ROOT_DIGEST（根は 1 度きり＝別の写しで同じ版を凍結し直して持ち帰っても、版管理の見え方に依らず落ちる。根を作り直すのは移行＝床の外の手順）
            版を上げて未凍結の間（執筆中）は「版が違う」を出し、記録の 1:1 の消し込みは測らない（条の消失・改番・番号の再利用は測る）
            --freeze-anchor: 全検査が 0 違反かつ「まだ分からない」が無いときだけ凍結し索引へ追記（版は最新より新しい・差分と発効した判断が要る）。列の始め直し（履歴に anchor が在った・記録が在る）は認めない
            --emit-amends: 最新 anchor と現行の欄単位の差分を amends にそのまま貼れる 1 行 1 件の形（- {json}）で印字する（判断の記録を書く補助・read-only。違反があれば stderr に出し終了コードは素の床と同じ）
  counts  : meta.counts と実数の一致
rc: 0 = 全部通った / 1 = 違反あり / 2 = 読めない・測れない（parse 不能・型違い・symlink・anchor 0 本・索引が空・列切れ・digest や写しの取り方の方式違い・版管理が無い / commit が無い / 浅い写し）。1 と 2 が同時に立つときは 1
"""
import re, sys, os, json, pathlib, collections, argparse, hashlib, copy
try:
    import yaml
except ImportError:
    print('check_draft: pyyaml が無い', file=sys.stderr); sys.exit(2)
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from strict_yaml import StrictLoader   # 重複キーを拒む loader（render_preview.py と共有）

ap = argparse.ArgumentParser(); ap.add_argument('--dir', default='design-intent'); ap.add_argument('--names', default='placed', choices=['draft', 'placed'])
ap.add_argument('--freeze-anchor', action='store_true'); ap.add_argument('--emit-amends', action='store_true')
args = ap.parse_args()
def _hook(t, v, tb):   # 想定外の例外（型違い・欠落）は「違反あり」でなく「読めない」（rc 2）
    sys.stderr.write(f'check_draft: 読めない・型が違う（想定外の例外）: {t.__name__}: {v}\n'); sys.stderr.flush(); os._exit(2)
sys.excepthook = _hook
ARG_DIR = pathlib.Path(args.dir)
if ARG_DIR.is_symlink(): print(f'check_draft: design-intent 自体が symlink（{ARG_DIR}）＝認めない', file=sys.stderr); sys.exit(2)
HERE = ARG_DIR.resolve()
if not HERE.is_dir(): print(f'check_draft: design-intent が dir でない: {ARG_DIR}', file=sys.stderr); sys.exit(2)
SUF = '.draft.yaml' if args.names == 'draft' else '.yaml'
def load_path(p):
    if p.is_symlink(): print(f'check_draft: symlink は認めない: {p}', file=sys.stderr); sys.exit(2)
    if not p.exists(): print(f'check_draft: 正本が無い: {p}', file=sys.stderr); sys.exit(2)
    if not p.is_file(): print(f'check_draft: file でない: {p}', file=sys.stderr); sys.exit(2)
    try: return yaml.load(p.read_text(encoding='utf-8'), Loader=StrictLoader)
    except Exception as ex: print(f'check_draft: parse できない: {p}: {ex}', file=sys.stderr); sys.exit(2)
def load(name): return load_path(HERE / (name + SUF))

c = load('constitution'); r = load('rules'); v = load('vocabulary'); s = load('srs')
errs = []
def err(kind, msg): errs.append((kind, msg))
def keys_ok(kind, where, obj, spec):
    if not isinstance(obj, dict): err(kind, f"{where}: 型が違う（欄の表でない）: {type(obj).__name__}"); return
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
ART_ID_RE = re.compile(r'(?:P|A|N)-\d+')   # 条 id の形（内部 3 空間の条 id・R-4）。改訂の範囲の節名（schema / precedence / articles …）と衝突しない
ids = [str(a.get('id')) for a in c['articles']]; st_list = []
if len(ids) != len(set(ids)): err('schema', f'条 id が重複: {[i for i, n in collections.Counter(ids).items() if n > 1]}')
for a in c['articles']:
    keys_ok('schema', a.get('id', '?'), a, sc['article'])
    if not ART_ID_RE.fullmatch(str(a.get('id'))): err('schema', f"{a.get('id')}: 条 id の形（P-n / A-n / N-n）でない")
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
        if st.get('pattern') not in en['pattern']: err('schema', f"{st.get('id')}: pattern が値域外: {st.get('pattern')}")
        if st.get('strength') not in en['strength']: err('schema', f"{st.get('id')}: strength が値域外: {st.get('strength')}")
        t = str(st.get('text', '')).strip()
        if not t.endswith('。'): err('schema', f"{st.get('id')}: 「。」で終わらない")
        if t.count('。') != 1: err('schema', f"{st.get('id')}: 1 文でない（「。」が {t.count('。')}）")
        neg = t.rstrip('。').endswith('ない')
        if (st.get('strength') == 'must-not') != neg: err('R-11', f"{st.get('id')}: strength={st.get('strength')} と文末「…{t[-8:]}」が不一致")
        if not str(st.get('id')).startswith(str(a['id']) + '.'): err('schema', f"{st.get('id')}: 条 id {a['id']} の枝番でない")
        st_list.append(str(st.get('id')))
st_ids = set(st_list)
_dup_st = [i for i, n in collections.Counter(st_list).items() if n > 1]
if _dup_st: err('schema', f"規範文 id が重複（同じ id の規範文が 2 本以上＝欄単位の消し込みが id で潰れる・P-7.1）: {_dup_st}")
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

# ── adr（判断の記録の正本・P-8 / A-2 / N-4 の live: adr・ADR-1）──
# 床の定数（FLOOR）: 欄の決まりの閾値・値域・置き場は床が持ち、adr/schema.yaml の schema 節（*_note を除く）はこれと型も値も違わない写しであること（data 側で動かせない・N-3.1）
ADR_DIR = HERE / 'adr'; ANCH_DIR = HERE / 'anchors'
A22_MIN = ['id', 'title', 'tier', 'binds', 'statements']        # A-2.2「条文（id・見出し・規範文・段・縛る相手）」の機械形 = 写しの取り方（条）
ST_FIELDS = ('text', 'pattern', 'strength')                      # 規範文の中身（本文・型・強度）。id は欄の道に使う
SCOPE_MIN = ['schema', 'precedence', 'articles']                 # A-2.2「schema 節・前文」+ 条文。憲法 schema.amendment_scope はこれを含む
NEW_MARK, DEL_MARK, EMPTY_MARK = '（新設）', '（削除）', '（空）'; MARKS = (NEW_MARK, DEL_MARK, EMPTY_MARK)
ROOT_DIGEST = 'acb52acd04b5d3a1feaf9ad5f0138f7614ce31964144b46ead914bde86e866ed'   # 列の根（v1.0）の anchor の digest。根は 1 度きり＝別の写しで同じ版を凍結し直しても（版管理の見え方に依らず）落ちる。変えるのは移行（床の外の手順・判断の記録）
FIRST_VER = 'v1.0'; DIGEST_ALGO = 'sha256-json-1'; VERSION_PATTERN = '^v[0-9]+\\.[0-9]+$'   # 版の綴り（v1.0・v1.10。v1.0.0 のような同じ版の別綴りを列に並べない）
FILE_KEYS = ['kind', 'digest_algo', 'version', 'previous', 'projection', 'meta_approval', 'approvals', 'content', 'digest']
OWNER = '持ち主'; APPROVER = ['持ち主', 'planner 席']; SURFACE = ['R-8']; EFFECTIVE = ['accepted', 'retired']
APPROVAL_FIELDS = ['who', 'date', 'ruling', 'verbatim', 'surface']
FLOOR = {
    'id_pattern': '^ADR-[1-9][0-9]*$',
    'date_format': '^\\d{4}-\\d{2}-\\d{2}$',
    'ruling_pattern': '[a-z]\\d-[0-9a-z]+(\\.\\d+)?',
    'owner': OWNER,
    'required': ['id', 'title', 'status', 'date', 'context', 'decision', 'options', 'basis', 'retreat', 'plain'],
    'optional': ['amends', 'grill', 'approval', 'consequences', 'supersedes', 'superseded_by', 'note'],
    'non_empty': ['title', 'context', 'decision', 'plain'],
    'enums': {'status': ['proposed', 'accepted', 'retired'], 'verdict': ['adopted', 'rejected'], 'retreat_kind': ['spike', 'measure', 'ruling'], 'approver': APPROVER, 'surface': SURFACE},
    'effective_status': EFFECTIVE,
    'option': {'required': ['id', 'name', 'text', 'verdict', 'reason'], 'optional': []},
    'options_rule': {'min': 2, 'adopted': 1},
    'retreat': {'required': ['kind', 'condition'], 'optional': []},
    'amends_entry': {'required': ['target', 'field', 'version', 'previous_text', 'new_text'], 'optional': [], 'new_article_marker': NEW_MARK, 'deleted_marker': DEL_MARK, 'empty_marker': EMPTY_MARK},
    'grill': {'required': ['when', 'who', 'where', 'summary'], 'optional': []},
    'approval': {'required': APPROVAL_FIELDS, 'optional': []},
    'amended_by_entry': {'required': ['adr', 'date', 'approved_by', 'ruling', 'previous_text', 'rationale'], 'optional': []},
    'anchor': {'dir': 'anchors', 'file_name': 'constitution-<version>.yaml', 'index_file': 'index.yaml', 'first_version': FIRST_VER, 'root_digest': ROOT_DIGEST, 'version_pattern': VERSION_PATTERN, 'digest_algo': DIGEST_ALGO,
               'file_keys': FILE_KEYS, 'projection_article_fields': A22_MIN, 'statement_fields': ['id', *ST_FIELDS], 'scope_minimum': SCOPE_MIN},
}
def real_under_here(p, what):   # symlink と design-intent の外の実体を拒む
    ok = True
    if p.is_symlink(): err('adr', f"{what}: symlink は認めない（{p}）"); ok = False
    try: p.resolve().relative_to(HERE)
    except ValueError: err('adr', f"{what}: design-intent の外を指している（{p}）"); ok = False
    return ok
if ADR_DIR.is_symlink() or not ADR_DIR.is_dir(): print(f'check_draft: adr/ が dir でない（symlink・file・不在）: {ADR_DIR}', file=sys.stderr); sys.exit(2)
asd = load_path(ADR_DIR / 'schema.yaml')
if not isinstance(asd, dict) or set(asd.keys()) - {'meta', 'schema', 'plain'} or not isinstance(asd.get('schema'), dict): print(f"check_draft: adr/schema.yaml の形が違う（節は meta / schema / plain）", file=sys.stderr); sys.exit(2)
def strip_notes(x):
    if isinstance(x, dict): return {k: strip_notes(vv) for k, vv in x.items() if not str(k).endswith('_note')}
    return x
def floor_diff(data, floor, path=''):   # 写し（data）と床の定数の食い違いを欄の道で列挙する（型も見る: 2.0 ≠ 2・true ≠ 1）
    out = []
    if isinstance(floor, dict):
        if not isinstance(data, dict): return [f"{path or 'schema'}（欄の表でない）"]
        for k in sorted(set(data) | set(floor), key=str):
            p = f'{path}.{k}' if path else str(k)
            if k not in floor: out.append(f'{p}（未知の欄＝機械が読まない欄は *_note で終える）')
            elif k not in data: out.append(f'{p}（欠落）')
            else: out += floor_diff(data[k], floor[k], p)
        return out
    if isinstance(floor, list):
        if not isinstance(data, list) or len(data) != len(floor): return [path]
        return [p for i, (a_, b_) in enumerate(zip(data, floor)) for p in floor_diff(a_, b_, f'{path}[{i}]')]
    return [] if (type(data) is type(floor) and data == floor) else [path]
for _p in floor_diff(strip_notes(asd['schema']), FLOOR): err('adr', f"adr/schema.yaml schema.{_p} が床の定数と違う（欄の決まりの閾値・値域・置き場は床の定数の写し＝data 側で動かせない・N-3.1）")
asc = FLOOR; aen = FLOOR['enums']; an = FLOOR['anchor']
SCOPE = [str(x) for x in (sc.get('amendment_scope') or [])]
if not set(SCOPE_MIN) <= set(SCOPE): err('anchor', f"憲法 schema.amendment_scope {SCOPE} が A-2.2 の範囲の下限 {SCOPE_MIN} を含まない")
for _i in ids:
    if _i in SCOPE: err('schema', f"条 id {_i} が改訂の範囲の節名と同じ（欄の道が衝突する）")
if aen['retreat_kind'] != en['retreat_kind']: err('adr', f"床の retreat_kind {aen['retreat_kind']} が憲法の値域 {en['retreat_kind']} と食い違う（憲法が正・床の定数を直す）")
for sid in aen['surface']:
    if sid not in rows: err('adr', f"対話面 {sid} が rules 行に無い（対話面は rules 行の id で指す・R-8）")
ADR_ID = re.compile(asc['id_pattern']); DATE_RE = re.compile(asc['date_format']); RULING_RE = re.compile(asc['ruling_pattern']); adrs = {}
BASIS_RE = re.compile(r'(?:P|A|N)-\d+(?:\.\d+)?|(?:FR|NFR|AC|CON|GOAL)\d+|(?:R|D)-\d+|ADR-[1-9][0-9]*')
def nonempty(x): return bool(str(x if x is not None else '').strip())
def date_ok(kind, where, x):
    if not DATE_RE.fullmatch(str(x)): err(kind, f"{where}「{x}」が年-月-日でない")
def approval_ok(kind, where, ap_):
    keys_ok(kind, where, ap_, asc['approval'])
    if not isinstance(ap_, dict): return
    for k in ('ruling', 'verbatim'):
        if not nonempty(ap_.get(k)): err(kind, f"{where}.{k} が空")
    date_ok(kind, f'{where}.date', ap_.get('date'))
    if ap_.get('who') not in aen['approver']: err(kind, f"{where}.who が値域外: {ap_.get('who')}")
    if not RULING_RE.search(str(ap_.get('ruling') or '')): err(kind, f"{where}.ruling「{ap_.get('ruling')}」に台帳 id（{asc['ruling_pattern']}）が無い")
    if ap_.get('surface') not in aen['surface']: err(kind, f"{where}.surface が値域外（対話面は rules 行の id で指す・R-8）: {ap_.get('surface')}")
# 凍結 anchor の列に一度でも在った条・規範文の id と範囲の節名（廃止は番号を空けたままにする＝P-7.2・記録はその番号を名指してよい・再利用は落とす）
hist_ids = set(); hist_scopes = set()
for _p in (sorted(ANCH_DIR.glob('constitution-*.yaml')) if ANCH_DIR.is_dir() else []):
    try:
        _d = yaml.load(_p.read_text(encoding='utf-8'), Loader=StrictLoader)
        _arts = ((_d.get('content') or {}).get('articles') or []) if isinstance(_d, dict) and isinstance(_d.get('content'), dict) else []
        for _a in _arts:
            if isinstance(_a, dict): hist_ids.add(str(_a.get('id'))); hist_ids |= {str(_st.get('id')) for _st in (_a.get('statements') or []) if isinstance(_st, dict)}
        if isinstance(_d, dict) and isinstance(_d.get('projection'), dict): hist_scopes |= {str(x) for x in (_d['projection'].get('scope') or [])}
    except Exception: pass
TARGETS = set(ids) | set(SCOPE) | {i for i in hist_ids if '.' not in i} | hist_scopes   # amends の対象 = 現行と列に在った条 id・範囲の節名（範囲を狭める改訂・過去の版の記録も名指せる）
for p in sorted(ADR_DIR.glob('*.yaml')):
    if p.name == 'schema.yaml': continue
    if not real_under_here(p, p.name): continue
    d = load_path(p)
    if not isinstance(d, dict): err('adr', f"{p.name}: 判断の記録が欄の表でない"); continue
    aid = str(d.get('id'))
    keys_ok('adr', aid, d, {'required': asc['required'], 'optional': asc['optional']})
    if not ADR_ID.match(aid): err('adr', f"{p.name}: id「{aid}」が形 {asc['id_pattern']} でない（ゼロ詰めしない・4 桁は外部の記録）")
    if p.stem != aid: err('adr', f"{p.name}: file 名が id {aid} と違う（1 判断 = 1 file・file 名 = id）")
    if aid in adrs: err('adr', f"{aid}: id が重複（P-7）"); continue
    adrs[aid] = d
    if d.get('status') not in aen['status']: err('adr', f"{aid}: status が値域外: {d.get('status')}")
    date_ok('adr', f'{aid}.date', d.get('date'))
    for k in asc['non_empty']:
        if not nonempty(d.get(k)): err('adr', f"{aid}: {k} が空")
    opts = d.get('options') if isinstance(d.get('options'), list) else []
    if not isinstance(d.get('options'), list): err('adr', f"{aid}: options が一覧でない")
    if len(opts) < asc['options_rule']['min']: err('adr', f"{aid}: 案が {len(opts)} 件（退けた案を含めて {asc['options_rule']['min']} 件以上）")
    for o in opts:
        keys_ok('adr', f"{aid}.options[{o.get('id') if isinstance(o, dict) else '?'}]", o, asc['option'])
        if not isinstance(o, dict): continue
        if o.get('verdict') not in aen['verdict']: err('adr', f"{aid}.options[{o.get('id')}]: verdict が値域外: {o.get('verdict')}")
        for k in ('name', 'text', 'reason'):
            if not nonempty(o.get(k)): err('adr', f"{aid}.options[{o.get('id')}].{k} が空")
    n_ad = sum(1 for o in opts if isinstance(o, dict) and o.get('verdict') == 'adopted')
    if n_ad != asc['options_rule']['adopted']: err('adr', f"{aid}: 採用の案が {n_ad} 件（{asc['options_rule']['adopted']} 件）")
    rt = d.get('retreat')
    keys_ok('P-8', f'{aid}.retreat', rt, asc['retreat'])
    if isinstance(rt, dict):
        if rt.get('kind') not in aen['retreat_kind']: err('P-8', f"{aid}: retreat.kind が値域外: {rt.get('kind')}")
        if not nonempty(rt.get('condition')): err('P-8', f"{aid}: 撤退条件が空（P-8.1）")
    if not (isinstance(d.get('basis'), list) and d['basis']): err('adr', f"{aid}: basis（根拠の id）が空")
    else:
        for b in d['basis']:
            if not BASIS_RE.fullmatch(str(b)): err('adr', f"{aid}: basis「{b}」が id の形（条・要件・rules 行・判断の記録）でない（P-5.2）")
    amends = d.get('amends') if isinstance(d.get('amends'), list) else []
    if d.get('amends') is not None and not isinstance(d.get('amends'), list): err('A-2', f"{aid}: amends が一覧でない")
    for e in amends:
        keys_ok('A-2', f"{aid}.amends[{e.get('target') if isinstance(e, dict) else '?'}]", e, asc['amends_entry'])
        if not isinstance(e, dict): continue
        if str(e.get('target')) not in TARGETS: err('A-2', f"{aid}: amends の対象 {e.get('target')} が条 id でも改訂の範囲の節名でもない（現行と列に在ったもの: 条 {len(TARGETS - set(SCOPE) - hist_scopes)} 本・節 {sorted(set(SCOPE) | hist_scopes)}）")
        for k in ('field', 'version', 'previous_text', 'new_text'):
            if not nonempty(e.get(k)): err('A-2', f"{aid}: amends[{e.get('target')}].{k} が空（空の値は印 {EMPTY_MARK} で書く）")
    ap_ = d.get('approval')
    if d.get('status') in asc['effective_status'] and not ap_: err('N-4', f"{aid}: {d.get('status')} なのに approval（逐語・日付・裁定 id・対話面）が無い")
    if ap_ is not None: approval_ok('N-4', f'{aid}.approval', ap_)
    if d.get('grill') is not None:
        keys_ok('A-2', f'{aid}.grill', d['grill'], asc['grill'])
        if isinstance(d['grill'], dict):
            date_ok('A-2', f'{aid}.grill.when', d['grill'].get('when'))
            for k in ('who', 'where', 'summary'):
                if not nonempty(d['grill'].get(k)): err('A-2', f"{aid}.grill.{k} が空")
effective = {k: d for k, d in adrs.items() if d.get('status') in asc['effective_status'] and isinstance(d.get('approval'), dict)}   # 発効した判断（承認欄あり・retired も含む）
def amends_list(d): return [e for e in (d.get('amends') if isinstance(d.get('amends'), list) else []) if isinstance(e, dict)]
for aid, d in adrs.items():
    amends = amends_list(d)
    if amends and aid in effective:
        if d['approval'].get('who') != asc['owner']: err('N-4', f"{aid}: 条文を改訂する発効した判断の承認者が {asc['owner']} でない（{d['approval'].get('who')}）")
        if not isinstance(d.get('grill'), dict): err('A-2', f"{aid}: 条文を改訂する発効した判断に grill の記録が無い（A-2.3）")
    for k in ('supersedes', 'superseded_by'):
        if d.get(k) is not None and str(d[k]) not in adrs: err('adr', f"{aid}: {k} {d[k]} の判断の記録が実在しない")
    if d.get('status') == 'retired' and d.get('superseded_by') is None: err('adr', f"{aid}: retired なのに superseded_by（後継）が無い（P-7.2・消すのでなく置き換える）")
    if d.get('superseded_by') is not None:
        if d.get('status') != 'retired': err('adr', f"{aid}: superseded_by を持つのに status が retired でない（P-7.2）")
        nx = adrs.get(str(d['superseded_by']))
        if nx and str(nx.get('supersedes')) != aid: err('adr', f"{aid}: 後継 {d['superseded_by']} の supersedes に {aid} が無い（双方向）")
    if d.get('supersedes') is not None:
        pv_ = adrs.get(str(d['supersedes']))
        if pv_ and str(pv_.get('superseded_by')) != aid: err('adr', f"{aid}: 置き換えた {d['supersedes']} の superseded_by が {aid} でない（双方向・後継が proposed の間は supersedes を書かない）")
for aid, d in adrs.items():   # retired の後継の列は accepted に到達する（輪・未発効の後継は落とす）
    if d.get('status') != 'retired': continue
    seen = [aid]; cur = d
    while True:
        nid = str(cur.get('superseded_by')); nx = adrs.get(nid)
        if nx is None: break   # 実在しない後継は上で落ちている
        if nid in seen: err('adr', f"{aid}: retired の後継の列が輪になっている（{'→'.join(seen + [nid])}）＝発効している後継が無い（P-7.2）"); break
        seen.append(nid)
        if nx.get('status') == 'accepted': break
        if nx.get('status') != 'retired': err('adr', f"{aid}: retired の後継の列の先 {nid} が発効していない（status {nx.get('status')}）"); break
        cur = nx
for x in (asd.get('meta') or {}).get('decided_by') or []:
    if str(x) not in adrs: err('adr', f"adr/schema.yaml meta.decided_by の {x} が実在しない（欄の決まりの出所の判断が消えている）")
if not (asd.get('meta') or {}).get('decided_by'): err('adr', "adr/schema.yaml meta.decided_by が空（欄の決まりの出所の判断が無い）")

# ── refs（R-4・母集団 = 4 file。判断の記録の参照 id は欄の決まりの規則として同じ関数で見る）──
req_ids = set()
for sect in ('goals', 'requirements', 'nonfunctional', 'acceptance', 'constraints', 'actors', 'outputs'):
    for x in s.get(sect) or []: req_ids.add(x['id'])
known_ids = set(ids) | st_ids | req_ids | set(rule_ids) | hist_ids   # R-4 の内部 3 空間（過去の版の id を含む）。判断の記録の id は別に解決する
sections = {'§5', '§6', '§7', '§8'}
ID_RE = re.compile(r'(?<![A-Za-z0-9-])((?:P|A|N)-\d+(?:\.\d+)?|(?:FR|NFR|AC|CON|GOAL)\d+|(?:R|D)-\d+)(?![A-Za-z0-9])')
ADR_RE = re.compile(r'(?<![A-Za-z0-9-])(ADR-[1-9][0-9]*)(?![A-Za-z0-9])')   # 内部の判断の記録の id（4 桁 ADR-0047 は外部の参照で数えない）
def walk(obj, where, kind_id='refs', kind_adr='A-2'):
    if isinstance(obj, dict):
        for k, vv in obj.items(): walk(vv, f'{where}.{k}', kind_id, kind_adr)
    elif isinstance(obj, list):
        for i, vv in enumerate(obj): walk(vv, f'{where}[{i}]', kind_id, kind_adr)
    elif isinstance(obj, str):
        for m in ID_RE.findall(obj):
            if m not in known_ids: err(kind_id, f"{where}: id {m} が実在しない")
        for m in ADR_RE.findall(obj):
            if m not in adrs: err(kind_adr, f"{where}: 判断の記録 {m} が実在しない")
for name, obj in (('憲法', {k: (vv if k != 'meta' else {kk: x for kk, x in vv.items() if not kk.startswith('changes_from')}) for k, vv in c.items() if k != 'schema'}), ('rules', {k: vv for k, vv in r.items() if k != 'schema'}), ('語彙', v), ('要件書', s)):
    walk(obj, name)
for aid, d in adrs.items(): walk(d, aid, 'adr', 'adr')   # 判断の記録の全欄（欄の決まりの規則・R-4 の行の母集団は変えない）
walk({'plain': asd.get('plain')}, 'adr/schema.yaml', 'adr', 'adr')
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

# ── amended_by ⇔ amends（A-2 / N-4 の双方向・ADR-1）──
ab_spec = asc['amended_by_entry']; art_by_id = {a['id']: a for a in c['articles']}
def amends_of(aid, target): return [e for e in amends_list(adrs[aid]) if e.get('target') == target]
for a in c['articles']:
    ams = a.get('amended_by')
    if ams is not None and not isinstance(ams, list): err('A-2', f"{a['id']}: amended_by が一覧でない"); continue
    for am in ams or []:
        keys_ok('A-2', f"{a['id']}.amended_by", am, ab_spec)
        if not isinstance(am, dict): continue
        for k in ('approved_by', 'ruling', 'previous_text', 'rationale'):
            if not nonempty(am.get(k)): err('N-4', f"{a['id']}: amended_by.{k} が空")
        date_ok('N-4', f"{a['id']}.amended_by.date", am.get('date'))
        if not RULING_RE.search(str(am.get('ruling') or '')): err('N-4', f"{a['id']}: amended_by.ruling「{am.get('ruling')}」に台帳 id が無い")
        ref = adrs.get(str(am.get('adr')))
        if not ref: err('N-4', f"{a['id']}: amended_by.adr {am.get('adr')} の判断の記録が実在しない"); continue
        if str(am.get('adr')) not in effective: err('N-4', f"{a['id']}: amended_by.adr {am['adr']} が発効していない（status {ref.get('status')}・承認欄 {'有' if isinstance(ref.get('approval'), dict) else '無'}）"); continue
        if am.get('approved_by') != ref['approval'].get('who'): err('N-4', f"{a['id']}: amended_by.approved_by「{am.get('approved_by')}」が {am['adr']} の承認者「{ref['approval'].get('who')}」と違う")
        if str(am.get('previous_text')) not in {str(e.get('previous_text')) for e in amends_of(str(am['adr']), a['id'])}: err('A-2', f"{a['id']}: amended_by.previous_text が {am['adr']} の amends（対象 {a['id']}）のどれとも一致しない")
for aid in effective:
    for t in {e.get('target') for e in amends_list(adrs[aid])}:
        if t in art_by_id and not any(isinstance(am, dict) and str(am.get('adr')) == aid for am in art_by_id[t].get('amended_by') or []): err('A-2', f"{aid}: {t} の amended_by に {aid} が無い（双方向）")

# ── anchor（凍結 anchor の列・A-2 / N-4 の差分検査の比較元・P-10.2 の限界を自認・ADR-2）──
INDEX = ANCH_DIR / an['index_file']
def project(doc, scope):   # 写し = 範囲（scope）の各節。条は A22_MIN の 5 欄・規範文は id + ST_FIELDS
    out = {k: copy.deepcopy(doc.get(k)) for k in scope if k != 'articles'}
    if 'articles' in scope:
        out['articles'] = [{k: ([{f: st.get(f) for f in ('id', *ST_FIELDS)} for st in (a.get('statements') or [])] if k == 'statements' else copy.deepcopy(a.get(k))) for k in A22_MIN} for a in doc.get('articles') or []]
    return out
def canon(x): return json.dumps(x, ensure_ascii=False, sort_keys=True, separators=(',', ':'), default=str)
def digest_of(anchor_doc): return hashlib.sha256(canon({k: vv for k, vv in anchor_doc.items() if k != 'digest'}).encode('utf-8')).hexdigest()
def render_val(x):   # 欄の値の型付きの表現。文字列はそのまま（json として読める文字列は引用符付き）・空文字は印・一覧・表・数・真偽・null は json の 1 値
    if isinstance(x, str):
        if x == '': return EMPTY_MARK
        try: json.loads(x)
        except Exception: return x
        return json.dumps(x, ensure_ascii=False)
    return canon(x)
def flatten(obj, prefix, out):   # 節を「欄の道 → 値」に平らにする。表は道を伸ばし、一覧は丸ごと 1 値。節が無い側は空の表
    if obj is None and not prefix: return out
    if isinstance(obj, dict):
        for k, vv in obj.items(): flatten(vv, f'{prefix}.{k}' if prefix else str(k), out)
    else: out[prefix or 'value'] = render_val(obj)
    return out
def flat_article(a):
    out = {k: render_val(a.get(k)) for k in A22_MIN if k not in ('id', 'statements')}
    for st in a.get('statements') or []:
        for f in ST_FIELDS: out[f"statements.{st.get('id')}.{f}"] = render_val(st.get(f))
    return out
def flat_targets(content, scope):   # 対象 → 欄の道 → 値。対象 = 範囲の各節・各条 id
    out = {t: flatten(content.get(t), '', {}) for t in scope if t != 'articles'}
    for a in (content.get('articles') or []) if 'articles' in scope else []: out[str(a.get('id'))] = flat_article(a)
    return out
def order_diff(prev_ids, cur_ids):   # 並びの差分は両側に在る id だけで見る（足した・消した id は欄の差分の側で数える）
    common = set(prev_ids) & set(cur_ids)
    a = '・'.join(x for x in prev_ids if x in common); b = '・'.join(x for x in cur_ids if x in common)
    return None if a == b else (a, b)
def marks_in(obj, where):   # 印（（新設）（削除）（空））を値として持つ欄を落とす（記録の読みと実体が食い違う）
    if isinstance(obj, dict):
        for k, vv in obj.items(): marks_in(vv, f'{where}.{k}')
    elif isinstance(obj, list):
        for i, vv in enumerate(obj): marks_in(vv, f'{where}[{i}]')
    elif isinstance(obj, str) and obj in MARKS: err('A-2', f"{where}: 印「{obj}」を値として持つ欄（記録の読みと実体が食い違う）")
def keys_in(obj, where):   # 写しの欄名は文字列で「.」を含まない（欄の道 a.b と欄名 'a.b' の衝突を塞ぐ）
    if isinstance(obj, dict):
        for k, vv in obj.items():
            if not isinstance(k, str) or '.' in k: err('A-2', f"{where}: 欄名「{k!r}」が文字列でないか「.」を含む（欄の道が衝突する）")
            keys_in(vv, f'{where}.{k}')
    elif isinstance(obj, list):
        for i, vv in enumerate(obj): keys_in(vv, f'{where}[{i}]')
def diff_targets(prev_content, prev_scope, cur_content, cur_scope):   # 対象 → 欄の道 → (前, 今)。無い側は印
    pf, cf = flat_targets(prev_content, prev_scope), flat_targets(cur_content, cur_scope); out = {}
    for t in set(pf) | set(cf):
        pt_, ct_ = pf.get(t, {}), cf.get(t, {}); ch = {}
        for k in set(pt_) | set(ct_):
            if pt_.get(k) != ct_.get(k): ch[k] = (pt_.get(k, NEW_MARK), ct_.get(k, DEL_MARK))
        if ch: out[t] = ch
    if 'articles' in prev_scope and 'articles' in cur_scope:
        pa_ = [str(a.get('id')) for a in prev_content.get('articles') or []]; ca_ = [str(a.get('id')) for a in cur_content.get('articles') or []]
        od = order_diff(pa_, ca_)
        if od: out.setdefault('articles', {})['order'] = od
        pmap = {str(a.get('id')): a for a in prev_content.get('articles') or []}; cmap = {str(a.get('id')): a for a in cur_content.get('articles') or []}
        for i_ in set(pmap) & set(cmap):
            od = order_diff([str(st.get('id')) for st in pmap[i_].get('statements') or []], [str(st.get('id')) for st in cmap[i_].get('statements') or []])
            if od: out.setdefault(i_, {})['statements.order'] = od
    return out
cur_proj = project(c, SCOPE); marks_in(cur_proj, '憲法の写し'); keys_in(cur_proj, '憲法の写し'); cur_ver = str(c['meta']['version']); meta_approval = c['meta'].get('approval')
if not re.fullmatch(VERSION_PATTERN, cur_ver): err('A-2', f"憲法 meta.version「{cur_ver}」の綴りが v<数>.<数> でない")
def ver_key(vs): return tuple(int(x) for x in re.findall(r'\d+', str(vs)))
pending = []; anchors = {}; index = None
if ANCH_DIR.exists() or ANCH_DIR.is_symlink():
    if ANCH_DIR.is_symlink() or not ANCH_DIR.is_dir(): print(f'check_draft: anchors/ が dir でない（symlink か file）: {ANCH_DIR}', file=sys.stderr); sys.exit(2)
    if INDEX.exists():
        if real_under_here(INDEX, INDEX.name):
            index = load_path(INDEX)
            if not (isinstance(index, dict) and index.get('kind') == 'constitution-anchor-index' and isinstance(index.get('entries'), list)): err('anchor', f"{INDEX.name}: 索引の形が違う"); index = None
    for p in sorted(ANCH_DIR.glob('constitution-*.yaml')):
        if not real_under_here(p, p.name): continue
        d = load_path(p)
        if not isinstance(d, dict) or set(d.keys()) != set(FILE_KEYS) or d.get('kind') != 'constitution-anchor': err('anchor', f"{p.name}: anchor の欄が壊れている（固定の欄 {FILE_KEYS}）: {sorted(d.keys()) if isinstance(d, dict) else type(d).__name__}"); continue
        vs = str(d['version'])
        if p.name != an['file_name'].replace('<version>', vs): err('anchor', f"{p.name}: file 名が版 {vs} と違う")
        if vs in anchors: err('anchor', f"{p.name}: 版 {vs} の anchor が重複"); continue
        if not re.fullmatch(VERSION_PATTERN, vs): err('anchor', f"{p.name}: 版の綴り「{vs}」が v<数>.<数> でない")
        if d.get('digest_algo') != DIGEST_ALGO: pending.append(f"{p.name}: digest の方式 {d.get('digest_algo')} が床の {DIGEST_ALGO} と違う＝digest を照合できない（まだ分からない・写しの内容の照合は行う）")
        elif digest_of(d) != str(d.get('digest')): err('anchor', f"{p.name}: digest が中身と一致しない（anchor のどこかが手で変えられた）")
        pj = d.get('projection') if isinstance(d.get('projection'), dict) else {}
        if list(pj.get('article_fields') or []) != A22_MIN or list(pj.get('statement_fields') or []) != ['id', *ST_FIELDS]:
            pending.append(f"{p.name}: anchor の写しの取り方（条 {pj.get('article_fields')}・規範文 {pj.get('statement_fields')}）が床の取り方（条 {A22_MIN}・規範文 {['id', *ST_FIELDS]}）と違う＝比べられない（まだ分からない・床の取り方を変えたなら移行の手順が要る）"); continue
        a_scope = [str(x) for x in (pj.get('scope') or [])]
        if not set(SCOPE_MIN) <= set(a_scope): err('anchor', f"{p.name}: anchor の範囲 {a_scope} が A-2.2 の下限 {SCOPE_MIN} を含まない")
        if not isinstance(d.get('content'), dict) or set(d['content'].keys()) != set(a_scope): err('anchor', f"{p.name}: 写しの節 {sorted(d['content'].keys()) if isinstance(d.get('content'), dict) else '?'} が範囲 {a_scope} と一致しない")
        if d.get('meta_approval') != meta_approval: err('N-4', f"{p.name}: anchor の meta_approval（発効の承認の写し）が憲法 meta.approval と一致しない（承認の記録が書き換えられた・P-12.2）")
        aps = d.get('approvals')
        if not (isinstance(aps, list) and aps): err('anchor', f"{p.name}: 承認の一覧（approvals）が空")
        else:
            for n, ap_ in enumerate(aps):
                if not isinstance(ap_, dict) or not all(nonempty(ap_.get(k)) for k in ('who', 'ruling', 'verbatim')): err('anchor', f"{p.name}: approvals[{n}] に who / ruling / verbatim が無い")
                elif not RULING_RE.search(str(ap_.get('ruling'))): err('anchor', f"{p.name}: approvals[{n}].ruling に台帳 id が無い")
                elif ap_.get('adr') is not None:
                    ref = adrs.get(str(ap_['adr']))
                    if ref is None: err('N-4', f"{p.name}: approvals[{n}].adr {ap_['adr']} の判断の記録が実在しない（凍結時の承認の相手が消えた）")
                    elif not isinstance(ref.get('approval'), dict) or {f: str(ref['approval'].get(f)) for f in APPROVAL_FIELDS} != {f: str(ap_.get(f)) for f in APPROVAL_FIELDS}: err('N-4', f"{p.name}: approvals[{n}]（{ap_['adr']}）の承認の写しが {ap_['adr']} の承認欄と一致しない（凍結後に承認の逐語・日付・裁定 id・対話面が書き換えられた・P-12.2）")
        anchors[vs] = (d, p)
# 版管理との照合（列の真偽は索引と digest が受け持ち、版管理は「消された・差し替えられた anchor」を早く止める補助）。見えない（git が無い・commit が無い・浅い写し・読めない）は「無い」ではなく「まだ分からない」（P-4.1）
present_anchor_files = {p.name for p in ANCH_DIR.glob('*.yaml')} if ANCH_DIR.is_dir() else set()
tracked_anchor_files = None; ever_anchor_files = None; hist_blobs = {}; git_pending = False   # hist_blobs: anchor 名 → [(commit, 本文)]（全 ref の履歴で追加・変更された anchor）
def anchor_format_same(doc):   # 履歴の anchor が床の今の形式（固定の欄・digest の方式・写しの取り方）と同じか。同じ形式で中身が違えば差し替え、形式が違えば移行の痕跡（床の外の手順）
    if not isinstance(doc, dict) or set(doc.keys()) != set(FILE_KEYS) or doc.get('digest_algo') != DIGEST_ALGO: return False
    pj = doc.get('projection') if isinstance(doc.get('projection'), dict) else {}
    return list(pj.get('article_fields') or []) == A22_MIN and list(pj.get('statement_fields') or []) == ['id', *ST_FIELDS]
try:
    import subprocess
    GIT_ENV = {k: vv for k, vv in os.environ.items() if not k.startswith('GIT_')}   # GIT_DIR / GIT_WORK_TREE 等で照合先をすげ替えられない
    def _git(*a_, cwd=None): return subprocess.run(['git', '-C', str(cwd or HERE), *a_], capture_output=True, text=True, timeout=20, env=GIT_ENV)
    _top = _git('rev-parse', '--show-toplevel')
    if _top.returncode == 0:
        _gt = pathlib.Path(_top.stdout.strip()).resolve()
        if _gt == HERE: err('anchor', f"design-intent 自体が版管理の根（{_gt}）＝床の script と別の版管理では照合にならない")
        elif _gt not in HERE.parents: err('anchor', f"版管理の根 {_gt} が design-intent の上に無い（照合先が違う）")
        elif _git('rev-parse', '--verify', '-q', 'HEAD', cwd=_gt).returncode != 0: git_pending = True; pending.append('版管理に commit が 1 つも無い（HEAD 無し）＝anchor を版管理と照合できない（まだ分からない）。git init だけでなく commit してから回す')
        else:
            _shallow = _git('rev-parse', '--is-shallow-repository', cwd=_gt).stdout.strip() == 'true'
            if _shallow and not present_anchor_files: git_pending = True; pending.append('版管理が浅い写し（shallow）で anchor が 1 本も無い＝履歴を照合できない（まだ分からない）。完全な写しで回す')
            _rel = ANCH_DIR.relative_to(_gt)
            for _p in [_rel, *(_rel / n for n in sorted(present_anchor_files))]:   # 追跡済みでも ignore の規則が当たれば落とす（以後の anchor が版管理に入らなくなる）
                if _git('check-ignore', '-q', '--no-index', str(_p), cwd=_gt).returncode == 0: err('anchor', f"{_p} が版管理から除外（ignore）されている（追跡済みでも規則が当たれば落とす）")
            _ls = _git('ls-tree', '-r', '--name-only', 'HEAD', '--', str(_rel), cwd=_gt)
            _lg = _git('log', '--all', '--format=%H', '--name-status', '--', str(_rel), cwd=_gt)
            if _ls.returncode != 0 or _lg.returncode != 0: git_pending = True; pending.append('版管理を読めない（ls-tree / log が失敗）＝anchor を版管理と照合できない（まだ分からない）')
            else:
                tracked_anchor_files = {pathlib.Path(x).name for x in _ls.stdout.split()}
                ever_anchor_files = set(); _cm = None
                for ln in _lg.stdout.splitlines():
                    ln = ln.strip()
                    if not ln: continue
                    if re.fullmatch(r'[0-9a-f]{40}', ln): _cm = ln; continue
                    m = re.fullmatch(r'([A-Z])\d*\t(.+?)(?:\t(.+))?', ln)
                    if not m or _cm is None: continue
                    st_, path_ = m.group(1), (m.group(3) or m.group(2)); nm = pathlib.Path(path_).name
                    if st_ != 'D': ever_anchor_files.add(nm)
                    if st_ in 'AMRC' and nm.startswith('constitution-'):
                        _sh = _git('show', f'{_cm}:{path_}', cwd=_gt)
                        if _sh.returncode == 0: hist_blobs.setdefault(nm, []).append((_cm[:7], _sh.stdout))
except Exception: tracked_anchor_files = None; ever_anchor_files = None; hist_blobs = {}
if tracked_anchor_files is None and not git_pending: pending.append('版管理（git）が無いか読めない＝anchor の削除を版管理と照合できない（まだ分からない）。写しで回すときも git init + commit の中で回す')
for _missing in sorted((tracked_anchor_files or set()) - present_anchor_files): err('anchor', f"anchors/{_missing} は版管理（HEAD）にあるが作業ツリーに無い（anchor と索引は消さない）")
for _missing in sorted((ever_anchor_files or set()) - (tracked_anchor_files or set()) - present_anchor_files): err('anchor', f"anchors/{_missing} は版管理の履歴に在ったが作業ツリーに無い（削除を commit しても列の始め直しは認めない・移行は床の外の手順で行う）")
for _nm in sorted(present_anchor_files & set(hist_blobs)):   # 履歴に在った同じ形式の anchor と中身が違えば差し替え（凍結物は書き換えない・同じ版は凍結し直さない）
    _cur_text = (ANCH_DIR / _nm).read_text(encoding='utf-8')
    for _cm, _txt in hist_blobs[_nm]:
        if _txt == _cur_text: continue
        try: _hd = yaml.load(_txt, Loader=StrictLoader)
        except Exception: continue
        if anchor_format_same(_hd): err('anchor', f"anchors/{_nm} が版管理の履歴（{_cm}）に在った同じ形式の anchor と中身が違う（凍結物の差し替え・書き換え。同じ版は凍結し直さない）"); break
records_exist = any(isinstance(a.get('amended_by'), list) and a['amended_by'] for a in c['articles']) or any(amends_list(d) for d in effective.values())
prev_anchor = None; newest = None; freeze_plan = None; listed = set()
if index is not None:
    ents = index['entries']
    if not ents: pending.append(f"{INDEX.name}: 索引はあるが entries が空＝比較元が立たない（まだ分からない・P-10.3）。索引と anchor は消さない・空にしない")
    for n, e in enumerate(ents):
        if not isinstance(e, dict) or not {'version', 'previous', 'digest'} <= set(e.keys()): err('anchor', f"{INDEX.name}: entries[{n}] の欄が壊れている"); continue
        vs = str(e['version']); expect_prev = None if n == 0 else str(ents[n - 1].get('version'))
        if n == 0 and vs != FIRST_VER: err('anchor', f"{INDEX.name}: 列の根 {vs} が最初の版 {FIRST_VER}（床の定数）でない")
        if n == 0 and str(e.get('digest')) != ROOT_DIGEST: err('anchor', f"{INDEX.name}: 列の根 {vs} の digest が床の定数と違う（根は 1 度きり・別の中身で凍結し直せない。変えるのは移行＝床の外の手順）")
        if (None if e.get('previous') is None else str(e['previous'])) != expect_prev: err('anchor', f"{INDEX.name}: entries[{n}]（{vs}）の previous {e.get('previous')} が直前の版 {expect_prev} でない（列の付け替え）")
        if vs not in anchors: pending.append(f"anchor の列が切れている: 索引にある版 {vs} の anchor file が無いか読めない＝差分検査は「まだ分からない」（P-10.3）。anchors/ は消さない"); continue
        d = anchors[vs][0]
        if str(d.get('digest')) != str(e['digest']): err('anchor', f"{vs}: anchor の digest が索引の記載と違う（差し替えられた）")
        if (None if d.get('previous') is None else str(d['previous'])) != expect_prev: err('anchor', f"{vs}: anchor の previous {d.get('previous')} が索引の列 {expect_prev} と違う")
    listed = {str(e.get('version')) for e in ents if isinstance(e, dict)}
    for extra in sorted(set(anchors) - listed): err('anchor', f"anchor {extra} が索引に無い（列の外の anchor）")
    if ents and isinstance(ents[-1], dict): newest = str(ents[-1].get('version'))
elif anchors: err('anchor', f"anchor file はあるが索引（{INDEX.name}）が無い（消された）")
elif records_exist: err('N-4', "改訂の記録（amended_by か発効した判断の amends）があるのに anchor が 1 本も無い＝anchor が消された（列の始め直しは認めない）")
chain_versions = [str(e.get('version')) for e in (index['entries'] if index is not None else []) if isinstance(e, dict)]
root_ver = chain_versions[0] if chain_versions else FIRST_VER
verifiable_versions = set(chain_versions[1:]) | ({cur_ver} - {root_ver})   # 発効した判断の amends が名指せる版 = 列の根より後の版（隣接区間で消し込める）か執筆中の版（根でない）だけ
for aid, d in effective.items():   # 列の根を名指す amends は架空の記録（根には改訂前が無い）・列に無い版を名指す amends は最新版の巻き戻し（列の頭は記録が名指す版まで固定）
    for _v in sorted({str(e.get('version')) for e in amends_list(d)} - verifiable_versions):
        if _v == root_ver: err('A-2', f"{aid}: amends が列の根の版 {root_ver} を名指している（根の版には改訂前が無い＝突き合わせる差分が存在しない架空の記録）")
        else: err('anchor', f"{aid}: 発効した判断の記録が名指す版 {_v} の anchor が列に無い（最新版の anchor と索引の項を消して前の版へ戻した・列の頭は記録が名指す版まで固定）")
if tracked_anchor_files is not None:   # 凍結した anchor は commit する。最新版以外の未追跡の anchor は列の差し替え（追跡済みの anchors/ を後から ignore した痕跡も含む）
    for _nm in sorted(present_anchor_files - tracked_anchor_files - {an['index_file']}):
        if newest is None or _nm != an['file_name'].replace('<version>', newest): err('anchor', f"anchors/{_nm} が版管理に追跡されていない（凍結した anchor は commit する。最新の版 {newest} 以外の未追跡の anchor は列の差し替え）")
if newest is not None and newest in anchors:   # 廃止した番号の再利用（過去の版の anchor に在って最新 anchor に無い id の再登場）を落とす（P-7.1）
    _nc = anchors[newest][0]['content']; _na = _nc.get('articles') or []
    newest_ids = {str(a.get('id')) for a in _na} | {str(st.get('id')) for a in _na for st in (a.get('statements') or []) if isinstance(st, dict)}
    for i_ in sorted((set(ids) | st_ids) & (hist_ids - newest_ids)): err('P-7', f"{i_}: 過去の版の anchor に在って最新 anchor {newest} に無い番号の再利用（廃止した番号は空けたまま・P-7.1）")
def structural_diff(prev_doc, cur_content, cur_scope, label):   # 直前 anchor との差分（欄単位）と、記録に依らない検査（条の消失 P-7・改番 P-7.1）
    prev = prev_doc['content']; prev_scope = [str(x) for x in prev_doc['projection']['scope']]; pv = str(prev_doc['version'])
    changed = diff_targets(prev, prev_scope, cur_content, cur_scope)
    prev_ids = {str(a.get('id')) for a in prev.get('articles') or []}; cur_ids = {str(a.get('id')) for a in cur_content.get('articles') or []}
    for i_ in sorted(prev_ids - cur_ids): err('P-7', f"条 {i_} が anchor {pv} に在って現行に無い（{label}・番号は消さない。条の廃止の機構は day-1 に無い＝M0 で決める）")
    del_texts = {}; add_texts = {}
    for t, ch in changed.items():
        for k, (pv_, cv_) in ch.items():
            m = re.fullmatch(r'statements\.(.+)\.text', k)
            if not m: continue
            if cv_ == DEL_MARK: del_texts[pv_] = m.group(1)
            elif pv_ == NEW_MARK: add_texts[cv_] = m.group(1)
    for txt in sorted(set(del_texts) & set(add_texts)): err('P-7', f"規範文の改番: {del_texts[txt]} を消して同じ本文を {add_texts[txt]} として足している（番号は付け替えない・P-7.1）")
    return changed, prev_ids - cur_ids, pv
def verify_pair(prev_doc, cur_content, cur_scope, ver, label):   # その版を名指す発効した判断の amends と差分を 1:1 に消し込む（余りも不足も落とす）。列の全区間と凍結時に回す
    changed, lost, pv = structural_diff(prev_doc, cur_content, cur_scope, label)
    ver_adrs = {k: d for k, d in effective.items() if any(str(e.get('version')) == ver for e in amends_list(d))}
    recorded = {}   # (対象, 欄の道) → (adr, entry)
    for aid, d in ver_adrs.items():
        for e in amends_list(d):
            if str(e.get('version')) != ver: continue
            key = (str(e.get('target')), str(e.get('field')))
            if key in recorded: err('A-2', f"{aid}: {key[0]}.{key[1]}（版 {ver}）の改訂が {recorded[key][0]} と重複して記録されている")
            recorded[key] = (aid, e)
    for t, ch in changed.items():
        if t in lost: continue
        for k, (pv_, cv_) in ch.items():
            rec = recorded.pop((t, k), None)
            if rec is None: err('N-4', f"{t}.{k} が anchor {pv} から変わったが（{label}）、それを amends（version {ver}・field）に持つ発効した判断の記録が無い（前「{pv_[:40]}」→ 今「{cv_[:40]}」）"); continue
            aid, e = rec
            if str(e.get('previous_text')) != pv_ or str(e.get('new_text')) != cv_: err('A-2', f"{aid}: {t}.{k} の previous_text / new_text が anchor {pv} と {label} の値に完全一致しない（前「{pv_[:40]}」今「{cv_[:40]}」）")
    for (t, k), (aid, e) in recorded.items():
        if t in lost: err('A-2', f"{aid}: amends が {t}.{k} の改訂を記録しているが、条 {t} は消されている（条は消さない・P-7）")
        else: err('A-2', f"{aid}: amends が {t}.{k}（版 {ver}）の改訂を記録しているが、anchor {pv} と {label} に差分が無い（記録と差分が食い違う）")
    for t in changed:
        if t in art_by_id and not any(isinstance(am, dict) and str(am.get('adr')) in ver_adrs for am in art_by_id[t].get('amended_by') or []): err('N-4', f"{t} が anchor {pv} から変わったが（{label}）、その版（{ver}）の判断の記録を指す amended_by が無い")
    return changed, ver_adrs
if index is not None:   # 列の全区間: 隣り合う anchor の差分は、その版を名指す発効した判断の記録と 1:1（凍結後に過去の版の記録を書き換えても落ちる）
    _chain = [str(e.get('version')) for e in index['entries'] if isinstance(e, dict)]
    for _pv, _cv in zip(_chain, _chain[1:]):
        if _pv in anchors and _cv in anchors: verify_pair(anchors[_pv][0], anchors[_cv][0]['content'], [str(x) for x in anchors[_cv][0]['projection']['scope']], _cv, f'anchor {_cv}')
emit_lines = None
if args.emit_amends:   # 判断の記録を書く補助（read-only）。常に最新 anchor と比べ、amends にそのまま貼れる 1 行 1 件の形で印字する
    if newest is None or newest not in anchors: emit_lines = ['# 比較元の anchor が無い（最初の版か、列が切れている）']
    else:
        _base = anchors[newest][0]; _changed = diff_targets(_base['content'], [str(x) for x in _base['projection']['scope']], cur_proj, SCOPE)
        emit_lines = [f'# 比較元 anchor {newest} → 現行（版 {cur_ver}）の欄単位の差分。amends にそのまま貼る（version は現行 meta.version の値＝上げる版）']
        for t, ch in sorted(_changed.items()):
            for k, (pv_, cv_) in sorted(ch.items()): emit_lines.append('- ' + json.dumps({'target': t, 'field': k, 'version': cur_ver, 'previous_text': pv_, 'new_text': cv_}, ensure_ascii=False))
this_ver_adrs = {}
if args.freeze_anchor:
    if newest is not None and newest in anchors and ver_key(newest) >= ver_key(cur_ver): print(f'check_draft: 版 {cur_ver} は最新 anchor {newest} より新しくない（同じ版は上書きしない・版を上げてから）', file=sys.stderr); sys.exit(1)
    if newest is None and cur_ver != FIRST_VER: err('anchor', f"最初の anchor は版 {FIRST_VER}（床の定数）でなければならない＝{cur_ver} で列を始め直すことはできない")
    if newest is None and (tracked_anchor_files or ever_anchor_files or records_exist): err('anchor', "anchor が版管理（HEAD か履歴）か記録の上では存在した（消された）ので、列を始め直す凍結は認めない")
    if newest is not None and newest in anchors:
        prev_anchor = anchors[newest][0]
        changed, this_ver_adrs = verify_pair(prev_anchor, cur_proj, SCOPE, cur_ver, '現行')
        if not changed: err('A-2', f"版を {newest} → {cur_ver} に上げたが条文（{'・'.join(SCOPE)}）に差分が無い（意味の無い版上げは凍結しない）")
        if not this_ver_adrs: err('N-4', f"版 {cur_ver} を凍結するには、その版を amends に持つ発効した判断の記録（持ち主の承認）が要る")
else:
    if index is None and not anchors and not records_exist: pending.append(f"凍結 anchor が 0 本（{ANCH_DIR}）＝A-2 / N-4 の差分検査は「まだ分からない」（P-10.3）。発効版で --freeze-anchor を実行する")
    elif newest is not None and newest in anchors:
        nd, np_ = anchors[newest]
        if newest != cur_ver:
            err('A-2', f"憲法の版 {cur_ver} と最新 anchor の版 {newest} が違う＝版を上げたのに凍結していない（--freeze-anchor）か、版を上げずに直した。凍結するまで記録の消し込みは測らない")
            structural_diff(nd, cur_proj, SCOPE, '現行')   # 執筆中でも条の消失・改番は測る
        elif nd['content'] != cur_proj: err('N-4', f"現行の条文（{'・'.join(SCOPE)} の写し）が凍結 anchor {np_.name} と一致しない＝判断の記録と承認を伴わない改憲（N-4.1）")
if args.freeze_anchor:
    aps = [{'adr': k, **{f: d['approval'].get(f) for f in APPROVAL_FIELDS}} for k, d in this_ver_adrs.items()] if prev_anchor is not None else [{'adr': None, **{f: (meta_approval or {}).get(f) for f in APPROVAL_FIELDS}}]
    fz = {'kind': 'constitution-anchor', 'digest_algo': DIGEST_ALGO, 'version': cur_ver, 'previous': newest if prev_anchor is not None else None,
          'projection': {'scope': list(SCOPE), 'article_fields': list(A22_MIN), 'statement_fields': ['id', *ST_FIELDS]},
          'meta_approval': copy.deepcopy(meta_approval), 'approvals': copy.deepcopy(aps), 'content': copy.deepcopy(cur_proj)}
    fz['digest'] = digest_of(fz); freeze_plan = (ANCH_DIR / an['file_name'].replace('<version>', cur_ver), fz)
    if prev_anchor is None and newest is None and fz['digest'] != ROOT_DIGEST: err('anchor', f"列の根の凍結だが digest {fz['digest'][:12]} が床の定数 {ROOT_DIGEST[:12]} と違う（根は 1 度きり・作り直せない。別の中身で列を始めるのは移行＝床の外の手順）")

# ── vocab（R-9 / R-12 の機械側。判断の記録の本文は欄の決まりの規則として同じ関数で見る）──
known = set()
for t in (v.get('terms') or []) + (v.get('field_terms') or []):
    for w in re.findall(r'[A-Za-z][A-Za-z0-9\-\.]*[A-Za-z0-9]|[A-Za-z]', (t.get('term') or '') + ' ' + (t.get('en') or '')): known.add(w.lower())
for g in v.get('identifiers') or []:
    for w in g.get('words') or []: known.add(str(w).lower())
IDENT = re.compile(r'^(P|A|N|FR|NFR|AC|CON|GOAL|R|D)-?\d|^ADR-[1-9][0-9]*$|^ADR-n$|^[RD]-n$|^[a-z]\d-[0-9a-z]+(\.\d+)?$')   # 条・要件・rules 行の id / R-n・D-n / 台帳 id（f2-648・s2-07l.149）。4 桁の ADR-0047 は免除しない（外部の参照は「日本語（原語）」の形で書く）
GLOSS = re.compile(r'[぀-ヿ一-鿿][^（）]*?（([^（）]*)）')   # 「日本語（原語）」の形＝グロス済み（R-12 の免除）
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
adr_body = []   # 判断の記録の本文（見出し・問題・決定・案・撤退条件・平易文・帰結）+ 欄の決まりの平易文。欄の決まりの規則として無条件に数える（R-9 の行の母集団は変えない）
for aid, d in adrs.items():
    for k in ('title', 'context', 'decision', 'plain'): adr_body.append((f'{aid} {k}', str(d.get(k) or '')))
    adr_body.append((f'{aid} retreat', str((d.get('retreat') if isinstance(d.get('retreat'), dict) else {}).get('condition') or '')))
    for o in (d.get('options') if isinstance(d.get('options'), list) else []): adr_body.append((f"{aid} option {o.get('id') if isinstance(o, dict) else '?'}", ' '.join(str(o.get(k) or '') for k in ('name', 'text', 'reason')) if isinstance(o, dict) else ''))
    for x in d.get('consequences') or []: adr_body.append((f'{aid} consequences', str(x)))
adr_body.append(('adr/schema.yaml plain', str(asd.get('plain') or '')))
def unknown_words(pairs):
    unknown = collections.Counter()
    for where, text in pairs:
        glossed = {x.lower() for g in GLOSS.findall(text) for x in words(g)}
        for w in words(text):
            lw = w.lower()
            if IDENT.match(w) or lw in known or lw in glossed or re.fullmatch(r'[a-z]', lw) or ('--' + w) in text: continue
            unknown[(lw, where)] += 1
    return unknown
for (lw, where), n in sorted(unknown_words(body).items()): err('R-9', f"{where}: 語彙に無い英字の語「{lw}」")
for (lw, where), n in sorted(unknown_words(adr_body).items()): err('adr', f"{where}: 語彙に無い英字の語「{lw}」（判断の記録の本文は「日本語（原語）」の形で書く）")

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
_out = sys.stderr if emit_lines is not None else sys.stdout   # --emit-amends では stdout を貼れる差分だけにし、違反は stderr へ
for k, m in errs: print(f'[{k}] {m}', file=_out)
n_st = sum(len(a['statements']) for a in c['articles'])
print(f"# 極性一覧: {len(pol)} 件（in-loop {len(inloop)} / post {len(pol) - len(inloop)}）。P-18.4（in-loop 0 なら落とす）= " + ('合格' if inloop else 'まだ分からない（live delivery-0・便 0 で編集時 guard が入るまで）'), file=sys.stderr)
print(f"# 条 {len(ids)}（{dict(cnt)}）/ 規範文 {n_st} / rules 行 {len(rule_ids)} / 語彙 {len(v.get('terms') or [])} 語 + 欄 {len(v.get('field_terms') or [])} + 識別子 {sum(len(g.get('words') or []) for g in (v.get('identifiers') or []))} / 判断の記録 {len(adrs)} / anchor {len(anchors)} / 違反 {len(errs)} {dict(by)}", file=sys.stderr)
if args.freeze_anchor:   # 凍結は全検査の後・0 違反かつ「まだ分からない」が無いときだけ（床が落ちる憲法・列の切れた木を anchor にしない）
    if errs or pending: print(f'check_draft: 凍結しない — 違反 {len(errs)} 件・まだ分からない {len(pending)} 件を直してから --freeze-anchor', file=sys.stderr)
    elif freeze_plan is None: print('check_draft: 凍結しない — 凍結の計画が立たなかった', file=sys.stderr); errs.append(('anchor', '凍結できない'))
    else:
        fp, fz = freeze_plan; ANCH_DIR.mkdir(exist_ok=True)
        ents = (index or {'entries': []})['entries'] + [{'version': fz['version'], 'previous': fz['previous'], 'digest': fz['digest']}]
        fp.write_text(f"# folio2 憲法 {cur_ver} の凍結 anchor（ADR-2）。写し（範囲 {'・'.join(SCOPE)}・条は {'・'.join(A22_MIN)}・規範文は id・{'・'.join(ST_FIELDS)}）+ 発効の承認の写し + この版の承認一覧 + digest。手で直さない・消さない・同じ版は上書きしない（check_draft.py --freeze-anchor が全検査 0 違反のときだけ作る）。\n" + yaml.safe_dump(fz, allow_unicode=True, sort_keys=False, width=10**6), encoding='utf-8')
        INDEX.write_text("# folio2 凍結 anchor の索引（追記のみ・ADR-2）。列 = entries の順。手で直さない・消さない・空にしない。\n" + yaml.safe_dump({'kind': 'constitution-anchor-index', 'entries': ents}, allow_unicode=True, sort_keys=False, width=10**6), encoding='utf-8')
        print(f'check_draft: 凍結した: {fp}（条 {len(cur_proj["articles"])}・previous {fz["previous"]}・承認 {len(fz["approvals"])} 件）・索引 {INDEX.name} に追記', file=sys.stderr)
for u in pending: print(f'# まだ分からない: {u}', file=sys.stderr)
if emit_lines is not None:
    for ln in emit_lines: print(ln)
sys.exit(1 if errs else (2 if pending else 0))
