#!/usr/bin/env python3
"""正本 4 file（constitution / rules / vocabulary / srs）+ 判断の記録（adr/ADR-n.yaml）→ 読み物 HTML 1 面。手で直さない（再生成する）。"""
import yaml,html,sys,os,glob
import argparse
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from strict_yaml import load as _yload   # 重複キーを拒む loader（床 check_draft.py と共有・後勝ちで黙って描かない）
_ap=argparse.ArgumentParser(); _ap.add_argument('--check',action='store_true'); _ap.add_argument('--write',action='store_true'); _ap.add_argument('--dir',default='design-intent'); _ap.add_argument('--out',default='preview/readable.html'); _a=_ap.parse_args(); os.chdir(_a.dir)
E=html.escape
def _rd(p): return _yload(open(p,encoding='utf-8').read())
c=_rd('constitution.yaml'); r=_rd('rules.yaml')
v=_rd('vocabulary.yaml'); s=_rd('srs.yaml')
VER=c['meta']['version']
ADRS=sorted((_rd(p) for p in glob.glob('adr/ADR-*.yaml')),key=lambda d:int(str(d['id']).split('-')[1]))
ANCH=[str(e.get('version')) for e in (_rd('anchors/index.yaml') or {}).get('entries',[])] if os.path.exists('anchors/index.yaml') else []
ADRTERM=next((t for t in v['terms'] if t.get('id')=='adr'),None)
AST={'proposed':('提案中・拘束力なし','#916626'),'accepted':('発効','#1e7b65'),'retired':('廃止','#666666')}
VERD={'adopted':'採用','rejected':'退けた'}
DOCST={'effective':'発効・拘束力あり','draft':'未承認・拘束力なし'}
TIER={'always':('いつも守る','#1e7b65'),'ask-first':('確認してから変える','#916626'),'never':('絶対にやらない','#b22323')}
STR={'must':'必ず守る','must-not':'決してしない','should':'既定（外すなら理由）'}
MECH={'reject':'機械が落とす','build-check':'生成時の検査','human-review':'人が目で確かめる','none':'なし'}
LIVE={'now':'いま動く','M0':'M0 で動く','delivery-0':'便 0 で動く','M1':'M1 で動く','adr':'ADR 正本の schema 後'}
BIND={'tool':'道具','practice':'作法','both':'両方'}; KIND={'v1-incident':'v1 の実害','scribe2-article':'scribe2 の条','folio2-ruling':'持ち主の裁定'}
PAT={'ubiquitous':'つねに','event':'〜のとき','state':'〜のあいだ','unwanted':'〜になったら','optional':'〜ならば'}
css=open(os.path.join(os.path.dirname(os.path.abspath(__file__)),'render-preview.css'),encoding='utf-8').read()
o=['<!doctype html><html lang="ja"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>folio2 — day-1 文書 %s（憲法・rules・語彙・要件書・判断の記録）</title><style>'%E(VER)+css+'</style></head><body><div class="page">']
m=c['meta']; cnt=m['counts']
o.append('<h1>folio2 — day-1 文書 %s</h1><p class="sub">正本 4 file（<code>constitution.yaml</code> / <code>rules.yaml</code> / <code>vocabulary.yaml</code> / <code>srs.yaml</code>）と判断の記録（<code>adr/ADR-n.yaml</code>）を <code>render_preview.py</code> で読み物にしたもの。この HTML は手で直さない。状態: <b>%s</b>（binding: %s）。G1〜G16・P1〜P5 の裁定を反映。敵対レビュー（<code>review/summary.md</code>）の生存指摘を畳んだ版。</p>'%(E(VER),E(DOCST.get(m.get('status'),str(m.get('status')))),E(str(m.get('binding')))))
o.append('<nav class="toc"><a href="#const">憲法（%d 条）</a><a href="#rules">rules（閾値 %d・開発規律 %d）</a><a href="#vocab">語彙（%d 語 + 欄 %d）</a><a href="#srs">要件書 M0（FR %d・NFR %d・AC %d・CON %d）</a><a href="#adr">判断の記録（%d）</a></nav>'%(len(c['articles']),len(r['thresholds']),len(r['discipline']),len(v['terms']),len(v.get('field_terms',[])),len(s['requirements']),len(s['nonfunctional']),len(s['acceptance']),len(s['constraints']),len(ADRS)))
if m.get('changes_from_v0_2'):
    o.append('<details open><summary>v0.2 からの変更（%d 件）</summary><ul>'%len(m['changes_from_v0_2'])+''.join('<li>%s</li>'%E(str(x)) for x in m['changes_from_v0_2'])+'</ul></details>')
if m.get('changes_from_v0_1'):
    o.append('<details><summary>v0.1 からの変更（%d 件）</summary><ul>'%len(m['changes_from_v0_1'])+''.join('<li>%s</li>'%E(str(x)) for x in m['changes_from_v0_1'])+'</ul></details>')
# ── 憲法
o.append('<h2 id="const">憲法（%d 条 = いつも守る %d ／ 確認してから変える %d ／ 絶対にやらない %d）</h2>'%(len(c['articles']),cnt['always'],cnt['ask-first'],cnt['never']))
ns=c['north_star']; o.append('<h3>00 北極星</h3><p><b>%s</b></p><p>誰のため: %s<br>達成の判定: %s</p>'%(E(ns['statement']),E(ns['for_whom']),E(ns['judged_by'])))
pr=c['precedence']; o.append('<h3>前文 — 順位（条ではない・全条に先立つ）</h3><div class="st">%s</div><p class="plain"><b>やさしく言うと</b><br>%s</p><p class="sub">縛る相手: %s ／ 機構: %s — %s</p>'%(E(pr['text']),E(pr['plain']),BIND[pr.get('binds','both')],MECH[pr['mechanism']['kind']],E(pr['mechanism'].get('note',''))))
o.append('<nav class="toc">'+''.join('<a href="#%s">%s %s</a>'%(a['id'],a['id'],E(a['title'])) for a in c['articles'])+'</nav>')
def rel(x):
    if isinstance(x,dict):
        names={'reqs':'要件','rules':'rules 行','articles':'条','sections':'節'}
        return ' ／ '.join('%s: %s'%(names.get(k,k),'・'.join(map(str,vv))) for k,vv in x.items() if vv)
    return '・'.join(map(str,x))
cur=None
for a in c['articles']:
    if a['tier']!=cur:
        cur=a['tier']; o.append('<h3>%s（%s）</h3>'%(TIER[cur][0],{'always':'道具も AI も守る','ask-first':'実行前に持ち主へ聞く','never':'やろうとしたら機械が拒む'}[cur]))
    tn,tc=TIER[a['tier']]
    o.append('<section class="art" id="%s"><h4><span class="gid">%s</span>%s<span class="tier" style="background:%s">%s</span><span class="bind">縛る相手: %s</span></h4>'%(a['id'],a['id'],E(a['title']),tc,tn,BIND[a['binds']]))
    for st in a['statements']: o.append('<div class="st"><span class="sid">%s</span>%s<span class="str">%s・%s</span></div>'%(st['id'],E(st['text']),PAT[st['pattern']],STR[st['strength']]))
    o.append('<p class="plain"><b>やさしく言うと</b><br>%s</p><dl>'%E(a['plain']))
    o.append('<dt>出所</dt><dd>'+'<br>'.join('<b>%s</b>: %s'%(KIND[x['kind']],E(str(x['ref']))) for x in a['rationale'])+'</dd>')
    mch=a['mechanism']; o.append('<dt>どう守らせるか</dt><dd><b>%s</b>（%s）— %s</dd>'%(MECH[mch['kind']],LIVE.get(mch.get('live'),E(str(mch.get('live')))),E(mch.get('note',''))))
    if a.get('retreat'): o.append('<dt>撤退条件</dt><dd>[%s] %s</dd>'%(E(a['retreat']['kind']),E(a['retreat']['condition'])))
    if a.get('relations'): o.append('<dt>関係</dt><dd>%s</dd>'%E(rel(a['relations'])))
    if a.get('supersedes_v1'):
        sv=a['supersedes_v1']; o.append('<dt>v1 から置き換えた条</dt><dd>%s「%s」（%s）<br>理由: %s</dd>'%(E(sv['doc']),E(sv['article']),E(sv['ruling']),E(sv['rationale'])))
    for am in a.get('amended_by',[]) or []: o.append('<dt>改訂来歴</dt><dd>%s（%s・%s・%s）<br>消す文: %s<br>理由: %s</dd>'%(E(str(am['adr'])),E(str(am['date'])),E(am['approved_by']),E(am['ruling']),E(am['previous_text']),E(am['rationale'])))
    if a.get('note'): o.append('<dt>planner 注</dt><dd>%s</dd>'%E(a['note']))
    o.append('</dl></section>')
am=c['amendment']; o.append('<h3>06 改訂</h3><p class="note">%s</p><ol>'%E(am['declaration'])+''.join('<li><b>%s</b>（%s）— %s　<span class="sub">条: %s</span></li>'%(E(str(st['n'])),E(st['who']),E(st['what']),E(rel(st['article']) if isinstance(st['article'],list) else str(st['article']))) for st in am['steps'])+'</ol><p>発効: %s</p>'%E(am['effective_step']['what']))
o.append('<h3>08 出所</h3><ol>'+''.join('<li><b>%s</b> — %s</li>'%(E(x['name']),E(str(x['ref']))) for x in c['sources'])+'</ol>')
# ── rules
o.append('<h2 id="rules">rules — 閾値の行（数値は条文に書かない）</h2><div class="wrap"><table><tr><th>id</th><th>条</th><th>何の数値か</th><th>値</th><th>種別</th><th>状態</th><th>裁定</th><th>時刻</th><th>注</th></tr>')
for x in r['thresholds']:
    extra='<br>'.join('<b>%s</b>: %s'%(k,E(str(x[k]))) for k in ('projection','basis','same_failure','population','note') if x.get(k))
    o.append('<tr>'+''.join('<td>%s</td>'%E(str(x.get(k) if x.get(k) is not None else '—')) for k in ('id','article','what','value','kind','status','ruling','ruled_at'))+'<td>%s</td></tr>'%extra)
o.append('</table></div><p class="sub">置かない数値: %s（%s）</p>'%(E(' ／ '.join(r['schema']['excluded']['what'])),E(r['schema']['excluded']['why'])))
o.append('<h3>開発規律の行（人が守る作法・条から id で参照）</h3><div class="wrap"><table><tr><th>id</th><th>条</th><th>内容</th><th>状態</th><th>裁定</th><th>時刻</th><th>注</th></tr>')
for x in r['discipline']: o.append('<tr>'+''.join('<td>%s</td>'%E(str(x.get(k) if x.get(k) is not None else '—')) for k in ('id','article','what','status','ruling','ruled_at','note'))+'</tr>')
o.append('</table></div>')
# ── 語彙
o.append('<h2 id="vocab">語彙（%d 語・正本 vocabulary.yaml・読者が引く場所は憲法 §7）</h2><div class="wrap"><table><tr><th>語</th><th>原語</th><th>ひとことで</th><th>定義</th></tr>'%len(v['terms']))
for t in v['terms']: o.append('<tr><td id="v-%s"><b>%s</b></td><td>%s</td><td>%s</td><td>%s</td></tr>'%(E(t['id']),E(t['term']),E(str(t.get('en') or '')),E(t['short']),E(t['def'])))
o.append('</table></div>')
if v.get('field_terms'):
    o.append('<h3>欄の名前（機械層・読者は引かない）</h3><div class="wrap"><table><tr><th>欄</th><th>原語</th><th>定義</th></tr>'+''.join('<tr><td><b>%s</b></td><td>%s</td><td>%s</td></tr>'%(E(t['term']),E(str(t.get('en') or '')),E(t['def'])) for t in v['field_terms'])+'</table></div>')
# ── 要件書
sm=s['meta']; o.append('<h2 id="srs">要件書 M0 — %s</h2><p class="note">%s</p>'%(E(sm['title']),E(sm['promise'])))
o.append('<h3>01 ゴール</h3><ul>'+''.join('<li><b>%s %s</b> — %s</li>'%(g['id'],E(g['title']),E(g['text'])) for g in s['goals'])+'</ul>')
o.append('<h3>02 範囲</h3><p><b>M0 で作る</b>: %s</p><p><b>M0 では作らない</b>: %s</p>'%(E(' ／ '.join(s['scope']['build'])),E(' ／ '.join(s['scope']['not_build']))))
o.append('<h3>図 2 folio が 1 回で通す 7 段</h3><ol>'+''.join('<li><b>%s</b>（%s）— 要件: %s%s</li>'%(E(st['what']),E(st['who']),E('・'.join(st['reqs']) or '（この段を定める要件は無い）'),('　<span class="sub">%s</span>'%E(st['note'])) if st.get('note') else '') for st in s['rail'])+'</ol>')
def req(x):
    o.append('<section class="art" id="%s"><h4><span class="gid">%s</span>%s<span class="tier" style="background:#2a4d6e">%s</span></h4>'%(x['id'],x['id'],E(x['title']),STR[x['strength']]))
    o.append('<div class="st"><span class="sid">%s</span>%s → %s</div>'%(PAT[x['pattern']],E(x['when']),E(x['shall'])))
    o.append('<p class="plain"><b>やさしく言うと</b><br>%s</p><dl>'%E(x['plain']))
    o.append('<dt>根拠（条）</dt><dd>%s</dd><dt>確かめ方</dt><dd>%s: %s（受入: %s）</dd><dt>ゴール</dt><dd>%s</dd>'%(E('・'.join(x['basis']) or '—'),E(x['verify']['method']),E(x['verify']['how']),E('・'.join(x['verify'].get('ac',[])) or '—'),E('・'.join(x['goals']))))
    if x.get('rules'): o.append('<dt>rules 行</dt><dd>%s</dd>'%E('・'.join(x['rules'])))
    if x.get('note'): o.append('<dt>planner 注</dt><dd>%s</dd>'%E(x['note']))
    o.append('</dl></section>')
o.append('<h3>03 機能要件（%d）</h3>'%len(s['requirements'])); [req(x) for x in s['requirements']]
o.append('<h3>04 非機能要件（%d）</h3>'%len(s['nonfunctional'])); [req(x) for x in s['nonfunctional']]
o.append('<h3>05 受入基準（%d）</h3>'%len(s['acceptance']))
for a in s['acceptance']:
    o.append('<section class="art" id="%s"><h4><span class="gid">%s</span>%s</h4><p class="plain"><b>やさしく言うと</b><br>%s</p><dl><dt>確かめる要件</dt><dd>%s</dd><dt>RED になる test</dt><dd>%s<br><code>%s</code></dd>%s</dl></section>'%(a['id'],a['id'],E(a['title']),E(a['plain']),E('・'.join(a['verifies'])),E(a['red_test']['sentence']),E(a['red_test']['fixture']),('<dt>注</dt><dd>%s</dd>'%E(a['note'])) if a.get('note') else ''))
o.append('<p class="sub">%s</p>'%E(s['not_frozen']))
o.append('<h3>06 制約（%d）</h3><div class="wrap"><table><tr><th>id</th><th>制約</th><th>中身</th><th>根拠（条）</th><th>出所</th></tr>'%len(s['constraints']))
for cn in s['constraints']: o.append('<tr><td>%s</td><td>%s</td><td>%s</td><td>%s</td><td>%s</td></tr>'%(cn['id'],E(cn['title']),E(cn['text']),E('・'.join(cn.get('basis',[])) or '—'),E(cn['source'])))
o.append('</table></div>')
o.append('<h3>07 対応表（要件 × ゴール × 受入 × 図・正本から導出）</h3><div class="wrap"><table><tr><th>要件</th>'+''.join('<th>%s</th>'%g['id'] for g in s['goals'])+'<th>受入</th><th>図</th></tr>')
acmap={}
for a in s['acceptance']:
    for q in a['verifies']: acmap.setdefault(q,[]).append(a['id'])
for x in s['requirements']+s['nonfunctional']:
    o.append('<tr><td>%s %s</td>'%(x['id'],E(x['title']))+''.join('<td>%s</td>'%('●' if g['id'] in x['goals'] else '') for g in s['goals'])+'<td>%s</td><td>%s</td></tr>'%(E('・'.join(acmap.get(x['id'],[])) or '—'),E('・'.join(x['figures']))))
o.append('</table></div>')
o.append('<h3>承認欄</h3><ul>'+''.join('<li>%s: %s — %s%s</li>'%(E(ap['role']),E(str(ap['who'])),E(str(ap.get('when') or '未')),('（%s）'%E(str(ap['stamp'])) if ap.get('stamp') else '')) for ap in sm['approval'])+'</ul><p class="sub">%s</p>'%E(sm['effective']))
# ── 判断の記録（ADR・正本 adr/ADR-n.yaml・schema は adr/schema.yaml・M0 の生成器は無いので day-1 の暫定）
o.append('<h2 id="adr">判断の記録（ADR・%d 件・正本 <code>adr/ADR-n.yaml</code>・欄の決まりは <code>adr/schema.yaml</code>）</h2><p class="note">%s 凍結 anchor（差分検査の比較元・索引 <code>anchors/index.yaml</code> の列・古い順）: %s</p>'%(len(ADRS),E((ADRTERM or {}).get('def','')),E(' → '.join(ANCH)) if ANCH else '<b>なし（または索引が空）＝差分検査は「まだ分からない」</b>'))
for d in ADRS:
    sn,scol=AST[d['status']]
    o.append('<section class="art" id="%s"><h4><span class="gid">%s</span>%s<span class="tier" style="background:%s">%s</span><span class="bind">%s</span></h4>'%(E(d['id']),E(d['id']),E(d['title']),scol,sn,E(str(d['date']))))
    o.append('<p class="plain"><b>やさしく言うと</b><br>%s</p><dl>'%E(d['plain']))
    o.append('<dt>何が問題か</dt><dd>%s</dd><dt>何を決めたか</dt><dd>%s</dd>'%(E(d['context']),E(d['decision'])))
    o.append('<dt>案（採用は 1 つ）</dt><dd>'+'<br>'.join('<b>%s</b>〔%s〕 %s — %s'%(E(x['name']),VERD[x['verdict']],E(x['text']),E(x['reason'])) for x in d['options'])+'</dd>')
    o.append('<dt>根拠</dt><dd>%s</dd><dt>撤退条件</dt><dd>[%s] %s</dd>'%(E('・'.join(map(str,d['basis']))),E(d['retreat']['kind']),E(d['retreat']['condition'])))
    if d.get('amends'): o.append('<dt>改訂する条文（版・欄）</dt><dd>'+'<br>'.join('<b>%s</b> %s.%s: 「%s」→「%s」'%(E(str(e.get('version'))),E(str(e['target'])),E(str(e.get('field'))),E(str(e['previous_text'])),E(str(e['new_text']))) for e in d['amends'])+'</dd>')
    if d.get('grill'): g=d['grill']; o.append('<dt>反対側からの確認（grill）</dt><dd>%s・%s・%s<br>%s</dd>'%(E(str(g['when'])),E(str(g['who'])),E(str(g['where'])),E(str(g['summary']))))
    if d.get('consequences'): o.append('<dt>この判断で変わること</dt><dd>%s</dd>'%'<br>'.join(E(str(x)) for x in d['consequences']))
    ap=d.get('approval'); o.append('<dt>承認</dt><dd>%s</dd>'%(E('%s・%s・裁定 %s・「%s」（対話面 = rules 行 %s）'%(ap['who'],ap['date'],ap['ruling'],ap['verbatim'],ap['surface'])) if ap else '未（提案中・持ち主の逐語と日付が入ると発効）'))
    for k,lab in (('supersedes','置き換えた判断'),('superseded_by','後継の判断')):
        if d.get(k): o.append('<dt>%s</dt><dd>%s</dd>'%(lab,E(str(d[k]))))
    if d.get('note'): o.append('<dt>planner 注</dt><dd>%s</dd>'%E(str(d['note'])))
    o.append('</dl></section>')
o.append('<p class="sub">この文書の所属: folio2 / day-1 の相談（f2-648.1）/ %s の読み物。<a href="index.html">入口へ戻る</a></p></div></body></html>'%E(VER))
out='\n'.join(o); mode='--check' if _a.check else '--write'; OUT=_a.out
if mode=='--check':
    if not os.path.exists(OUT): print('render: 読み物が無い（未生成）',file=sys.stderr); sys.exit(2)
    cur=open(OUT,encoding='utf-8').read()
    if cur!=out: print('render: DRIFT — 読み物 %d byte ≠ 導出 %d byte（手で直したか正本が変わった）'%(len(cur.encode()),len(out.encode())),file=sys.stderr); sys.exit(1)
    print('render: OK — 読み物は正本と一致（%d byte）'%len(out.encode())); sys.exit(0)
open(OUT,'w',encoding='utf-8').write(out); print('rendered bytes',len(out.encode()))
