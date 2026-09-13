#!/usr/bin/env python3
"""床（scripts/check_draft.py）が RED になる fixture を回す runner（f2-648.2 の受入）。fixture の正本は tests/floor_cases.yaml（凍結・P-10.1 の day-1 代替）。
使い方: python3 tests/run_floor_cases.py（repo root で）。各 case: design-intent/ を一時 dir へ写し、**既定でその一時 dir を git の 1 commit にし**（床は版管理の無い写しを「まだ分からない」に落とすため・case に no_git: true で外す）、
mutate を順に当て、床を回して終了コード・出力の語（expect_msg・rc が非 0 なら必須）・出て**いけない**語（expect_not_msg）・stderr の語・file の有無・違反件数（expect_n）を照合する。正本は触らない。
mutate の 1 段: {file, path, value[, add: true]} = 欄を置き換える（欄が無ければ落とす・add で新設）/ {file, path, raw_key, value} = path の表に鍵 raw_key（「.」入り等の生の鍵）を置く /
{file, path, pop: true} = path の一覧の末尾を 1 つ消す / {file, delete: true} = file を消す / {dir, delete_dir: true} = dir を消す / {file, create: <文書>} = file を新しく置く / {file, write_text: <生の文字列>} = 生で書く /
{symlink_dir: <dir>} = dir を design-intent の外へ動かして symlink に置き換える / {symlink_file: <file>} = file を外へ動かして symlink に置き換える /
{git_snapshot: true} / {git_commit: true} = 写しの現状を git の 1 commit にする / {git_ignore_anchors: true} = anchors/ を版管理から除外（追跡も外す）して commit / {git_ignore_anchors_keep_tracked: true} = 追跡は外さず .gitignore だけ足して commit /
{git_ignore_pattern: <pattern>} = .gitignore に file の pattern を足して commit / {git_nested: true} = design-intent 自体を版管理の根にする / {git_reinit_no_commit: true} = .git を捨てて git init だけ（commit 無し）/
{git_orphan_drop_anchors: true} = 新しい根の無い branch（orphan）へ切り anchors/ を追跡から外して消し commit / {git_shallow_clone: true} = 写しを浅い clone（depth 1）に置き換える / {env_git_dir_empty: true} = 以後の床の起動に GIT_DIR / GIT_WORK_TREE を空の repo へ向けて渡す /
{refreeze_in_fresh_repo: true} = 写しを anchors/ 無しの新しい repo に写して凍結し、できた anchor を持ち帰る（別の写しで列を始め直す手口）/ {anchor_forge: {file, path, value}} = anchor の欄を書き換えて digest と索引を計算し直す（揃えて書き換える改竄）/
{freeze_anchor: true} = 写しの現行を anchor に凍結する（--freeze-anchor）。途中の凍結は rc 0 を要る（freeze_rc で上書き）・途中の凍結の後は写しを commit する（commit: false で外す）/
{emit_amends_into: <adr file>} = --emit-amends の出力（# 行を除く）をそのまま YAML として読み、その判断の記録の amends に置く（貼れる形の検査）。
凍結が最後の段なら、その実行が case の結果。凍結が rc 0 なら続けて素の床も回し rc 0 を要る。途中の凍結が rc 0 なら写しを commit する（凍結した anchor は commit するのが正規の手順）。expected_cases（件数）を pin する。
rc: 0 = 全 case が期待どおり / 1 = 期待と違う case あり / 2 = 読めない"""
import re, sys, shutil, subprocess, tempfile, pathlib, os, json, hashlib
try:
    import yaml
except ImportError:
    print('run_floor_cases: pyyaml が無い', file=sys.stderr); sys.exit(2)
ROOT = pathlib.Path(__file__).resolve().parents[1]
SRC = ROOT / 'design-intent'; CHECK = ROOT / 'scripts' / 'check_draft.py'
try:
    _fx = yaml.safe_load((ROOT / 'tests' / 'floor_cases.yaml').read_text(encoding='utf-8'))
    cases = [c for k, v in _fx.items() if k.startswith('cases') for c in v]
except Exception as ex:
    print(f'run_floor_cases: fixture が読めない: {ex}', file=sys.stderr); sys.exit(2)
if _fx.get('expected_cases') != len(cases): print(f"run_floor_cases: case 件数 {len(cases)} が expected_cases {_fx.get('expected_cases')} と違う（fixture が欠けたか増えた）", file=sys.stderr); sys.exit(2)
for cs in cases:
    if cs['expect_rc'] != 0 and not (cs.get('expect_msg') or cs.get('expect_stderr')): print(f"run_floor_cases: {cs['id']}: rc 非 0 の case には expect_msg か expect_stderr が要る", file=sys.stderr); sys.exit(2)
SEG = re.compile(r'([^\[\].]+)|\[([^\]]+)\]')
def steps(path):
    out = []
    for m in SEG.finditer(path):
        if m.group(1): out.append(m.group(1))
        else:
            t = m.group(2)
            out.append(('find',) + tuple(t.split('=', 1)) if '=' in t else int(t))
    return out
def get(obj, st):
    if isinstance(st, tuple): return next(x for x in obj if str(x.get(st[1])) == st[2])
    return obj[st]
def mutate(doc, path, value, add):
    st = steps(path); cur = doc
    for x in st[:-1]: cur = get(cur, x)
    last = st[-1]
    if isinstance(last, tuple): raise ValueError(f'末尾の段は欄名か添字: {path}')
    if isinstance(cur, dict) and last not in cur and not add: raise KeyError(f'欄 {path} が無い（新設なら add: true）')
    if isinstance(cur, list) and isinstance(last, int) and last == len(cur):
        if not add: raise IndexError(f'{path} は一覧の末尾の次（新設なら add: true）')
        cur.append(value); return
    cur[last] = value
def dump(doc): return yaml.safe_dump(doc, allow_unicode=True, sort_keys=False, width=10**6)
RUN_ENV = {}
def run(work, *flags): return subprocess.run([sys.executable, str(CHECK), '--dir', str(work), *flags], capture_output=True, text=True, env={**os.environ, **RUN_ENV})
def canon(x): return json.dumps(x, ensure_ascii=False, sort_keys=True, separators=(',', ':'), default=str)   # 床と同じ正規化（揃えて書き換える改竄の再現用）
def digest_of(doc): return hashlib.sha256(canon({k: v for k, v in doc.items() if k != 'digest'}).encode('utf-8')).hexdigest()
def gitc(cwd, *cmd): subprocess.run(['git', '-c', 'user.email=fx@example', '-c', 'user.name=fx', *cmd], cwd=cwd, capture_output=True, check=True)
def git_commit(cwd): gitc(cwd, 'add', '-A'); gitc(cwd, 'commit', '-q', '--allow-empty', '-m', 'fixture')
def judge(cs, pr):
    ok = pr.returncode == cs['expect_rc']
    if cs.get('expect_msg') and cs['expect_msg'] not in pr.stdout + pr.stderr: ok = False
    if cs.get('expect_stderr') and cs['expect_stderr'] not in pr.stderr: ok = False
    if cs.get('expect_not_msg') and cs['expect_not_msg'] in pr.stdout + pr.stderr: ok = False
    if 'expect_n' in cs and len([l for l in pr.stdout.splitlines() if l.startswith('[')]) != cs['expect_n']: ok = False
    return ok
bad = 0
for cs in cases:
    with tempfile.TemporaryDirectory() as td:
        work = pathlib.Path(td) / 'design-intent'; shutil.copytree(SRC, work)
        last = None; note = ''; RUN_ENV.clear()
        muts = [x for m in (cs.get('mutate') or []) for x in (m if isinstance(m, list) else [m])]   # 共通の段（alias の一覧）を 1 段ずつに開く
        try:
            if not cs.get('no_git'): gitc(td, 'init', '-q'); git_commit(td)
            for mu in muts:
                if mu.get('freeze_anchor'):
                    last = run(work, '--freeze-anchor')
                    if mu is not muts[-1] and last.returncode != mu.get('freeze_rc', 0): note += f' ／ 途中の凍結が rc {last.returncode}'
                    if mu is not muts[-1] and last.returncode == 0 and mu.get('commit', True) and (pathlib.Path(td) / '.git').exists(): git_commit(td)   # 凍結した anchor は commit する（正規の手順・commit: false で外す＝未追跡の anchor の経路を測る case 用）
                    continue
                last = None
                if mu.get('emit_amends_into'):
                    em = run(work, '--emit-amends')
                    if em.returncode == 2: note += f' ／ emit が rc {em.returncode}（読めない）'; continue   # 執筆中は違反があって当然（rc 1）＝差分は stdout に出る
                    lst = yaml.safe_load('\n'.join(l for l in em.stdout.splitlines() if not l.startswith('#'))) or []
                    f = work / mu['emit_amends_into']; doc = yaml.safe_load(f.read_text(encoding='utf-8')); doc['amends'] = lst; f.write_text(dump(doc), encoding='utf-8'); continue
                if mu.get('git_snapshot') or mu.get('git_commit'):
                    if not (pathlib.Path(td) / '.git').exists(): gitc(td, 'init', '-q')
                    git_commit(td); continue
                if mu.get('git_ignore_anchors'):
                    (pathlib.Path(td) / '.gitignore').write_text('design-intent/anchors/\n', encoding='utf-8'); gitc(td, 'rm', '-r', '-q', '--cached', 'design-intent/anchors'); git_commit(td); continue
                if mu.get('git_nested'): gitc(work, 'init', '-q'); git_commit(work); continue
                if mu.get('git_ignore_anchors_keep_tracked'):
                    (pathlib.Path(td) / '.gitignore').write_text('design-intent/anchors/\n', encoding='utf-8'); git_commit(td); continue
                if mu.get('git_ignore_pattern'):
                    (pathlib.Path(td) / '.gitignore').write_text(str(mu['git_ignore_pattern']) + '\n', encoding='utf-8'); git_commit(td); continue
                if mu.get('git_reinit_no_commit'): shutil.rmtree(pathlib.Path(td) / '.git'); gitc(td, 'init', '-q'); continue
                if mu.get('git_orphan_drop_anchors'):
                    gitc(td, 'checkout', '-q', '--orphan', 'clean'); gitc(td, 'rm', '-r', '-q', '--cached', 'design-intent/anchors'); shutil.rmtree(work / 'anchors'); git_commit(td); continue
                if mu.get('git_shallow_clone'):
                    sh = pathlib.Path(td) / 'shallow'; subprocess.run(['git', 'clone', '-q', '--depth', '1', 'file://' + str(pathlib.Path(td)), str(sh)], capture_output=True, check=True); work = sh / 'design-intent'; continue
                if mu.get('env_git_dir_empty'):
                    em = pathlib.Path(td) / 'empty-repo'; em.mkdir(); gitc(em, 'init', '-q'); RUN_ENV.update({'GIT_DIR': str(em / '.git'), 'GIT_WORK_TREE': str(td)}); continue
                if mu.get('refreeze_in_fresh_repo'):   # 別の写しで列を始め直して持ち帰る手口
                    fr = pathlib.Path(td) / 'fresh'; shutil.copytree(work, fr / 'design-intent'); shutil.rmtree(fr / 'design-intent' / 'anchors', ignore_errors=True); gitc(fr, 'init', '-q'); git_commit(fr)
                    fz = run(fr / 'design-intent', '--freeze-anchor'); note += '' if fz.returncode != 0 else ' ／ 別の写しでの凍結が rc 0（列の始め直しが通った）'
                    if (fr / 'design-intent' / 'anchors').is_dir():
                        for f_ in (fr / 'design-intent' / 'anchors').glob('*.yaml'): shutil.copy(f_, work / 'anchors' / f_.name)
                    continue
                if 'anchor_forge' in mu:   # anchor の欄を書き換え digest と索引を計算し直す（揃えて書き換える改竄）
                    af = mu['anchor_forge']; f = work / af['file']; doc = yaml.safe_load(f.read_text(encoding='utf-8')); mutate(doc, af['path'], af['value'], af.get('add', False)); doc['digest'] = digest_of(doc); f.write_text(dump(doc), encoding='utf-8')
                    ix = work / 'anchors' / 'index.yaml'; idx = yaml.safe_load(ix.read_text(encoding='utf-8'))
                    for e in idx.get('entries') or []:
                        if str(e.get('version')) == str(doc.get('version')): e['digest'] = doc['digest']
                    ix.write_text(dump(idx), encoding='utf-8'); continue
                if mu.get('symlink_dir') or mu.get('symlink_file'):
                    src_ = work / (mu.get('symlink_dir') or mu.get('symlink_file')); outside = pathlib.Path(td) / 'outside'; outside.mkdir(exist_ok=True); dst = outside / src_.name
                    shutil.move(str(src_), str(dst)); src_.symlink_to(dst); continue
                if mu.get('delete_dir'): shutil.rmtree(work / mu['dir']); continue
                f = work / mu['file']
                if mu.get('delete'): f.unlink(); continue
                if 'create' in mu: f.parent.mkdir(parents=True, exist_ok=True); f.write_text(dump(mu['create']), encoding='utf-8'); continue
                if 'write_text' in mu: f.parent.mkdir(parents=True, exist_ok=True); f.write_text(mu['write_text'], encoding='utf-8'); continue
                doc = yaml.safe_load(f.read_text(encoding='utf-8'))
                if 'swap' in mu:   # 一覧の 2 要素を入れ替える（並びの差分の再現）
                    cur = doc
                    for x in steps(mu['path']): cur = get(cur, x)
                    i_, j_ = mu['swap']; cur[i_], cur[j_] = cur[j_], cur[i_]
                elif 'raw_key' in mu:   # 生の鍵（「.」入り等）を表に置く
                    cur = doc
                    for x in steps(mu['path']): cur = get(cur, x)
                    cur[mu['raw_key']] = mu['value']
                elif mu.get('pop'):   # 一覧の末尾を 1 つ消す
                    cur = doc
                    for x in steps(mu['path']): cur = get(cur, x)
                    cur.pop()
                else: mutate(doc, mu['path'], mu['value'], mu.get('add', False))
                f.write_text(dump(doc), encoding='utf-8')
        except Exception as ex:
            print(f"FAIL {cs['id']}: fixture の適用で例外 {type(ex).__name__}: {ex}"); bad += 1; continue
        if last is not None:
            pr = last
            if pr.returncode == 0:
                pr2 = run(work)
                if pr2.returncode != 0: note += f' ／ 凍結後の素の床が rc {pr2.returncode}'; pr = pr2
        else: pr = run(work)
        ok = judge(cs, pr) and not note
        for rel in cs.get('expect_no_file') or []:
            if (work / rel).exists(): ok = False; note += f' ／ {rel} が書かれている'
        for rel in cs.get('expect_file') or []:
            if not (work / rel).exists(): ok = False; note += f' ／ {rel} が無い'
    print(f"{'PASS' if ok else 'FAIL'} {cs['id']}: rc={pr.returncode}（期待 {cs['expect_rc']}） — {cs['why']}{note}")
    if not ok:
        bad += 1
        for ln in (pr.stdout + pr.stderr).strip().splitlines()[:10]: print('    ' + ln)
print(f"# {len(cases) - bad} / {len(cases)} case が期待どおり", file=sys.stderr)
sys.exit(1 if bad else 0)
