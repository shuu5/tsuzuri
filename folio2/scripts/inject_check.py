#!/usr/bin/env python3
"""folio2 — 憲法の生成区間を CLAUDE.md へ書く / 検査する（day-1 の暫定 script・G13=A）。

正本 constitution.yaml の「規範語（〜する／〜しない）を含む条」の規範文を決定的に導出し、
CLAUDE.md の <!-- constitution:begin --> 〜 <!-- constitution:end --> の間だけを差し替える（--write）か、
区間と導出本文の byte 一致を検査する（--check・差があれば rc 1）。--write と --check は同じ導出関数を使う。

fail-closed:
  - 正本が読めない / parse できない / 規範文が 0 本 → rc 2（「無い」を「一致」に化けさせない）
  - CLAUDE.md が無い・marker が 1 対でない・逆順・区間が空 → rc 2（--check は「測れない」を合格にしない）
  - 生成区間の外に規範語（〜する。／〜ない。）で終わる行がある → rc 1（N-2 ②）
  - 生成区間の総 byte が rules 行 R-2 の上限を超える → rc 3（deny・P-14.3）
撤退条件: M0 で `folio inject --write / --check` が land したら本 script を削除する（二重の生成器を持たない = P-2 / P-9）。
依存: Python3 + pyyaml（A-3: 外部ソフトの増減は確認してから。pyyaml は host に既存）。
"""
import argparse, re, sys, pathlib
try:
    import yaml
except ImportError:
    print('inject_check: pyyaml が無い（A-3 の確認の上で入れる）', file=sys.stderr); sys.exit(2)

BEGIN = '<!-- constitution:begin -->'
END = '<!-- constitution:end -->'
STRENGTHS = {'must', 'must-not', 'should'}   # 規範文の判定は正本の欄（strength）で行う（G10: 型は欄に持つ）。文末の語で判定しない。

def derive(constitution_path: pathlib.Path) -> tuple[str, int]:
    """正本 → 生成区間の本文（決定的）。返り値 = (本文, 規範文の本数)。
    母集団 = strength 欄を持つ規範文を 1 本以上持つ条（G13「規範語を含む条」の型付きの定義）。順序は正本の文書順。"""
    data = yaml.safe_load(constitution_path.read_text(encoding='utf-8'))
    lines = []
    for art in data.get('articles', []):
        for s in art.get('statements', []):
            if s.get('strength') not in STRENGTHS:
                raise ValueError(f"{s.get('id')}: strength が規範の値でない: {s.get('strength')!r}")
            text = re.sub(r'\s+', ' ', str(s['text']).strip())
            if not text.endswith('。'):
                raise ValueError(f"{s.get('id')}: 規範文が「。」で終わらない")
            lines.append(f"{s['id']}: {text}")
    pre = data.get('precedence', {}).get('text')
    if pre:
        lines.insert(0, f"順位: {re.sub(r'\s+', ' ', str(pre).strip())}")
    return '\n\n'.join(lines), len(lines)

def region_of(claude_md: str) -> tuple[int, int] | None:
    b = [m.start() for m in re.finditer(re.escape(BEGIN), claude_md)]
    e = [m.start() for m in re.finditer(re.escape(END), claude_md)]
    if len(b) != 1 or len(e) != 1 or e[0] < b[0]:
        return None
    return b[0] + len(BEGIN), e[0]

def r2_limit(rules_path: pathlib.Path | None) -> int | None:
    if not rules_path or not rules_path.exists():
        return None
    rules = yaml.safe_load(rules_path.read_text(encoding='utf-8'))
    for row in rules.get('thresholds', []):
        if row.get('id') == 'R-2':
            m = re.search(r'([\d,]+)\s*byte', str(row.get('value', '')))
            return int(m.group(1).replace(',', '')) if m else None
    return None

def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('--constitution', required=True, type=pathlib.Path)
    ap.add_argument('--claude-md', required=True, type=pathlib.Path)
    ap.add_argument('--rules', type=pathlib.Path, default=None, help='rules.yaml（R-2 の byte 上限を読む・無ければ上限検査を skip せず rc 2）')
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument('--write', action='store_true'); g.add_argument('--check', action='store_true'); g.add_argument('--print', action='store_true')
    a = ap.parse_args()
    if not a.constitution.exists():
        print(f'inject_check: 正本が無い: {a.constitution}', file=sys.stderr); return 2
    try:
        body, n = derive(a.constitution)
    except Exception as ex:  # parse 不能は loud
        print(f'inject_check: 正本を parse できない: {ex}', file=sys.stderr); return 2
    if n == 0:
        print('inject_check: 規範文が 0 本（母集団が空 = 正本が壊れている）', file=sys.stderr); return 2
    size = len(body.encode('utf-8'))
    limit = r2_limit(a.rules)
    if a.rules is not None and limit is None:
        print('inject_check: rules の R-2 が読めない（上限検査を skip しない）', file=sys.stderr); return 2
    if limit is not None and size > limit:
        print(f'inject_check: 生成区間 {size} byte が R-2 上限 {limit} byte を超える（deny・P-14.3）', file=sys.stderr); return 3
    if a.print:
        sys.stdout.write(body + '\n'); print(f'# {n} 規範文 / {size} byte / R-2 上限 {limit}', file=sys.stderr); return 0
    if not a.claude_md.exists():
        print(f'inject_check: CLAUDE.md が無い: {a.claude_md}', file=sys.stderr); return 2
    md = a.claude_md.read_text(encoding='utf-8')
    reg = region_of(md)
    if reg is None:
        print('inject_check: marker が 1 対でない（0 本・2 本・逆順）', file=sys.stderr); return 2
    want = '\n' + body + '\n'
    if a.write:
        new = md[:reg[0]] + want + md[reg[1]:]
        if new != md:
            a.claude_md.write_text(new, encoding='utf-8')
        print(f'inject_check: wrote {n} 規範文 / {size} byte'); return 0
    outside = md[:reg[0]] + md[reg[1]:]
    normative_outside = [ln for ln in outside.splitlines() if re.search(r'(する|ない)。\s*$', ln.strip()) and not ln.strip().startswith('<!--')]
    if normative_outside:
        print(f'inject_check: 区間外に規範語で終わる行が {len(normative_outside)} 行（N-2 ②）: {normative_outside[0][:60]!r}', file=sys.stderr); return 1
    cur = md[reg[0]:reg[1]]
    if cur.strip() == '':
        print('inject_check: 区間が空（未注入）', file=sys.stderr); return 2
    if cur != want:
        print(f'inject_check: DRIFT — 区間 {len(cur.encode())} byte ≠ 導出 {len(want.encode())} byte', file=sys.stderr); return 1
    print(f'inject_check: OK — {n} 規範文 / {size} byte 一致'); return 0

if __name__ == '__main__':
    sys.exit(main())
