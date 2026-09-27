/* 台帳の処理状況（task / memo の数・速度・純減）の集計 — 純関数 1 本（account board と home が共有・見本と実データも同じ関数）
 * 入力 = account/data/ledger.json の rows: [id, kind(task|memo|question|epic), status, created, updated, closed, parent, [blockers], title, sample(0|1)]
 * 規則（「?」の 1 行と同じ）:
 *   open = status が closed でない・blocked = status が blocked か「open だが blocks の相手が閉じていない」・ready = open で blocks が全部閉じている
 *   速度 = 直近 7 日に閉じた task の数 / 7・lead time = 直近 30 日に閉じた task の created → closed の中央値と p90（日）
 *   純減 = 時点 t の open の task の数 − （t − 24 h / t − 7 日）の数（created ≤ t < closed で数える）・↓ 純減・↑ 純増・→ 横ばい
 *   stale = 更新が 7 日以上無い open の task・epic の進み = 直下の task の closed / 全体（着手中の数と最も古い task の経過は持ち主の裁定 07:3xZ で指標から外した） */
(function () {
  var DAY = 86400000;
  function ms(s) { return s ? Date.parse(s.length === 17 ? s.replace('Z', ':00Z') : s) : null; }
  function pct(a, p) { if (!a.length) return null; var s = a.slice().sort(function (x, y) { return x - y; }), i = (s.length - 1) * p, lo = Math.floor(i), hi = Math.ceil(i); return s[lo] + (s[hi] - s[lo]) * (i - lo); }
  function stats(rows, now, opt) {
    opt = opt || {};
    if (!rows) return null;
    var R = rows.filter(function (r) { return !(opt.noSample && r[9]); }).map(function (r) {
      return { id: r[0], kind: r[1], status: r[2] || 'open', c: ms(r[3]), u: ms(r[4]) || ms(r[3]), x: ms(r[5]), parent: r[6], blk: r[7] || [], title: r[8], sample: !!r[9], heads: r[10] || 0, promo: r[11] || [] };
    }).filter(function (r) { return r.c == null || r.c <= now; });
    var byId = {}; R.forEach(function (r) { byId[r.id] = r; });
    var isClosedAt = function (r, t) { return r.x != null && r.x <= t; };
    var isOpen = function (r) { return !isClosedAt(r, now) && r.status !== 'closed' && r.status !== 'tombstone'; };
    var T = R.filter(function (r) { return r.kind === 'task'; }), open = T.filter(isOpen);
    var unmet = function (r) { return r.blk.filter(function (b) { var x = byId[b]; return x && isOpen(x); }); };
    var blocked = open.filter(function (r) { return r.status === 'blocked' || (r.status !== 'in_progress' && unmet(r).length); });
    var ready = open.filter(function (r) { return r.status === 'open' && !unmet(r).length; });
    var openAt = function (t) { return T.filter(function (r) { return r.c != null && r.c <= t && !(r.x != null && r.x <= t); }).length; };
    var nowN = open.length, d24 = nowN - openAt(now - DAY), d7 = nowN - openAt(now - 7 * DAY);
    var closed7 = T.filter(function (r) { return r.x != null && r.x > now - 7 * DAY && r.x <= now; }).length;
    var lead = T.filter(function (r) { return r.x != null && r.c != null && r.x > now - 30 * DAY && r.x <= now; }).map(function (r) { return (r.x - r.c) / DAY; });
    var days = [], burn = [];
    for (var i = 13; i >= 0; i--) {
      var a = now - (i + 1) * DAY, b = now - i * DAY;
      days.push({ from: a, to: b, created: T.filter(function (r) { return r.c > a && r.c <= b; }).length, closed: T.filter(function (r) { return r.x != null && r.x > a && r.x <= b; }).length });
      burn.push({ t: b, open: openAt(b) });
    }
    var stale = open.filter(function (r) { return r.u != null && now - r.u >= 7 * DAY; });
    var cnt = function (k) { return R.filter(function (r) { return r.kind === k && isOpen(r); }).length; };
    var epics = R.filter(function (r) { return r.kind === 'epic'; }).map(function (e) {
      var kids = T.filter(function (r) { return r.parent === e.id; }), ko = kids.filter(isOpen);
      return { id: e.id, title: e.title, sample: e.sample, open: isOpen(e), total: kids.length, closed: kids.length - ko.length, openKids: ko.length };
    }).filter(function (e) { return e.total > 0; });
    // memo の契約化（昇格）: 昇格 = memo が task か design-intent の行へ移って閉じたこと（見本は promoted_to の辺を持つ・実データは辺が無いので閉じたことで代える）
    var M = R.filter(function (r) { return r.kind === 'memo'; }), mo = M.filter(isOpen);
    var mAges = mo.filter(function (r) { return r.c != null; }).map(function (r) { return now - r.c; });
    var memo = { open: mo.length, wait: mo.filter(function (r) { return (r.heads & 8) && !r.promo.length; }).length,
      promo7: M.filter(function (r) { return r.x != null && r.x > now - 7 * DAY && r.x <= now; }).length,
      ageP50: pct(mAges, .5),
      unpromoted: mo.filter(function (r) { return !r.promo.length; }).map(function (r) { return { id: r.id, title: r.title, age: r.c != null ? now - r.c : null, heads: r.heads, sample: r.sample }; }) };
    return {
      now: now, memo: memo, n: R.length, nSample: R.filter(function (r) { return r.sample; }).length,
      open: { task: nowN, memo: cnt('memo'), question: cnt('question'), epic: cnt('epic') },
      blocked: blocked.length, ready: ready.length,
      perDay: closed7 / 7, closed7: closed7, leadP50: pct(lead, .5), leadP90: pct(lead, .9), leadN: lead.length,
      net24: d24, net7: d7, days: days, burn: burn,
      stale: stale.length, staleIds: stale.map(function (r) { return r.id; }), blockedIds: blocked.map(function (r) { return r.id; }), readyIds: ready.map(function (r) { return r.id; }),
      epics: epics, allTasks: T.length
    };
  }
  function arrow(d) { return d < 0 ? { c: 'down', s: '↓', t: '純減' } : d > 0 ? { c: 'up', s: '↑', t: '純増' } : { c: 'flat', s: '→', t: '横ばい' }; }
  function netHTML(d, lbl) { var a = arrow(d); return '<span class="net net-' + a.c + '" aria-label="' + a.t + ' ' + d + '"><b>' + a.s + '</b><span class="num">' + (d > 0 ? '+' : d < 0 ? '−' : '') + Math.abs(d) + '</span>' + (lbl ? '<span class="sub">' + lbl + '</span>' : '') + '</span>'; }
  /* sparkline 14 日: created（点線）と closed（実線）の 2 本 */
  function spark14(st, W, H) {
    W = W || 120; H = H || 24; var mx = 1; st.days.forEach(function (d) { mx = Math.max(mx, d.created, d.closed); });
    var X = function (i) { return (i / 13 * (W - 2) + 1).toFixed(1); }, Y = function (v) { return (H - 2 - v / mx * (H - 4)).toFixed(1); };
    var pl = function (k, cls) { return '<polyline class="' + cls + '" fill="none" points="' + st.days.map(function (d, i) { return X(i) + ',' + Y(d[k]); }).join(' ') + '"/>'; };
    return '<svg class="lspark" viewBox="0 0 ' + W + ' ' + H + '" width="' + W + '" height="' + H + '" role="img" aria-label="14 日の created と closed">' + pl('created', 'ls-created') + pl('closed', 'ls-closed') + '</svg>';
  }
  /* burndown 14 日: open の task の推移（線）と closed の日次（棒） */
  function burndown(st, W, H) {
    W = W || 320; H = H || 72; var mo = 1, mc = 1; st.burn.forEach(function (b) { mo = Math.max(mo, b.open); }); st.days.forEach(function (d) { mc = Math.max(mc, d.closed); });
    var bw = (W - 8) / 14, s = '<svg class="lburn" viewBox="0 0 ' + W + ' ' + H + '" preserveAspectRatio="none" role="img" aria-label="burndown 14 日">';
    st.days.forEach(function (d, i) { var h = d.closed / mc * (H * .45); s += '<rect class="lb-closed" x="' + (4 + i * bw + 1).toFixed(1) + '" y="' + (H - h).toFixed(1) + '" width="' + Math.max(1, bw - 2).toFixed(1) + '" height="' + h.toFixed(1) + '"/>'; });
    s += '<polyline class="lb-open" fill="none" points="' + st.burn.map(function (b, i) { return (4 + i * bw + bw / 2).toFixed(1) + ',' + (4 + (1 - b.open / mo) * (H * .5)).toFixed(1); }).join(' ') + '"/>';
    return s + '</svg>';
  }
  function days(msv) { return msv == null ? '―' : (msv / DAY < 1 ? Math.round(msv / 3600000) + 'h' : (msv / DAY).toFixed(msv / DAY < 10 ? 1 : 0) + 'd'); }
  var HEADS = ['出所', '観測', '候補', '昇格条件'];
  function headsOf(b) { return HEADS.filter(function (h, i) { return b & (1 << i); }); }
  /* 未反映の一覧（id・題・種類・年齢・次の 1 手の 5 列・年齢の古い順）: memo の行の hover は 4 節の見出しだけ */
  var UF = {};
  function unrefHTML(items, max, src) {
    var esc = TZ.esc, L = TZ.L;
    if (!items || !items.length) return '<div class="small muted">未反映 0</div>';
    var rows = items.slice(0, max || items.length).map(function (u) {
      UF[u.key || u.id] = { u: u, src: src };
      var idc = u.kind === 'memo' ? '<span class="u-id mono" data-card="uf:' + esc(u.key || u.id) + '" tabindex="0">' + esc(u.id) + '</span>' : (u.key ? '<a class="u-id mono nl" href="' + TZ.href(u.key) + '" data-key="' + esc(u.key) + '">' + esc(u.id) + '</a>' : '<span class="u-id mono">' + esc(u.id) + '</span>');
      return '<div class="urow' + (u.sample ? ' smp' : '') + '">' + idc + '<span class="u-t" data-t>' + esc(TZ.cut(u.title || '', 44)) + '</span><span class="u-k">' + esc(u.kind) + '</span><span class="u-a num">' + days(u.age) + '</span><span class="u-n">' + esc(L(u.next)) + '</span></div>';
    }).join('');
    var head = '<div class="urow hrow"><span>id</span><span>題</span><span>' + TZ.HS('u_kind') + '</span><span>' + TZ.HS('u_age') + '</span><span>' + TZ.HS('u_next') + '</span></div>';
    return '<div class="ulist">' + head + rows + '</div>' + (max && items.length > max ? '<div class="small muted">ほか ' + (items.length - max) + ' 件（古い順に ' + max + ' 件まで）</div>' : '');
  }
  function unrefCard(k) {
    var x = UF[k]; if (!x) return null; var u = x.u;
    var heads = u.heads && u.heads.length ? u.heads : [];
    return TZ.card4({ ti: TZ.esc(TZ.cut(u.title || '', 34)), k: TZ.shape('beads') + '<span>memo</span><span class="sp"></span><span class="mono">' + TZ.esc(u.id) + '</span>',
      v: '◷ ' + days(u.age) + ' · 次 → ' + TZ.esc(TZ.L(u.next)) + (u.sample ? ' · 見本' : ''), src: '.beads/issues.jsonl', srcFull: (x.src || '.beads/issues.jsonl') + '（description は見出しの有無だけを見る）',
      more: ['<b>節の見出し</b>'].concat(HEADS.map(function (h) { var on = heads.indexOf(h) >= 0; return '<span class="gi ' + (on ? 'ok' : 'unknown') + ' sm" aria-hidden="true">' + (on ? '✓' : '') + '</span><span>' + TZ.esc(h) + '</span>'; }))
        .concat(['✗ 本文は読まない', u.sample ? '! mock の見本 = 台帳には無い' : '']) });
  }
  /* 台帳の判定（1 語・5 値）: 悪い順 = ! 滞り → ↑ 積み増し → → 停滞 → ✓ 順調 → ― 台帳なし。
   * 閾値（切れる / 変えられる値）: STALE_HALF = stale が open の task の半分を超える・STALL_PER_DAY = closed/日 がこれ未満 = ほぼ 0（持ち主の裁定 07:3xZ で 0.1）。
   * rules 行の候補: ledger.judge_stale_ratio = 0.5・ledger.judge_gain_7d = 0（純減 7d がこれを超えたら積み増し）・ledger.judge_idle_per_day = 0.1。
   * 台帳の処理状況の表と各 project の表が同じこの関数を使う */
  var JUDGE = { STALE_HALF: 0.5, GAIN_7D: 0, STALL_PER_DAY: 0.1 };
  var JV = { bad: { s: '!', k: 'j_bad', r: 0 }, up: { s: '↑', k: 'j_up', r: 1 }, stall: { s: '→', k: 'j_stall', r: 2 }, ok: { s: '✓', k: 'j_ok', r: 3 }, none: { s: '―', k: 'j_none', r: 4 } };
  function judge(x) {
    var v = !x ? 'none' : x.open.task > 0 && x.stale > x.open.task * JUDGE.STALE_HALF ? 'bad' : x.net7 > JUDGE.GAIN_7D ? 'up' : x.perDay < JUDGE.STALL_PER_DAY ? 'stall' : 'ok';
    return { v: v, s: JV[v].s, k: JV[v].k, rank: JV[v].r };
  }
  function judgeHTML(j, big) { return '<span class="jdg j-' + j.v + (big ? ' big' : '') + '" data-term="' + j.k + '" tabindex="0"><b aria-hidden="true">' + j.s + '</b><span>' + TZ.esc(TZ.L(j.k)) + '</span></span>'; }
  /* 自己検査: 5 値を 1 本ずつ（実データに出ない値も判定の枝を踏む） */
  (function () {
    var f = function (task, stale, net7, perDay) { return { open: { task: task }, stale: stale, net7: net7, perDay: perDay }; };
    var C = [['stale > open の半分 → 滞り', f(10, 6, -1, 1), 'bad'], ['純減 7d > 0 → 積み増し', f(10, 2, 3, 1), 'up'], ['closed/日 < 0.1 → 停滞', f(10, 2, 0, 0.05), 'stall'], ['closed/日 = 0.1 → 停滞にしない', f(10, 2, 0, 0.1), 'ok'],
      ['純減 ≤ 0・stale 少 → 順調', f(10, 2, -2, 1), 'ok'], ['台帳が無い → 台帳なし', null, 'none'], ['open の task 0 → 滞りにしない', f(0, 0, 0, 1), 'ok']];
    window.__tz_judge_selftest = C.map(function (c) { var g = judge(c[1]).v; return { case: c[0], got: g, want: c[2], ok: g === c[2] }; });
  })();
  TZ.Ledger = { judge: judge, judgeHTML: judgeHTML, JUDGE: JUDGE, JV: JV, unrefHTML: unrefHTML, unrefCard: unrefCard, headsOf: headsOf, HEADS: HEADS, stats: stats, arrow: arrow, netHTML: netHTML, spark14: spark14, burndown: burndown, days: days, DAY: DAY, ms: ms };
})();
