/* account の判定の関数（account board と project dashboard の home が共有する・data = account/data/board.json）
 * 規則: 群の今の口座 = 記録 > 種・群の orchestrator は今の口座で起きる・新しい pipeline は群の今の口座を使えない（account-lifecycle.md §17 / §20 / §23 / §27 / §28） */
(function () {
  TZ.AcctModel = function (B) {
    var esc = TZ.esc, L = TZ.L, ms = TZ.ms;
  var FROM = ms(B.window_from), BUILT = ms(B.built_at), SPAN = BUILT - FROM;
  var PROJ = {}; B.projects.forEach(function (p) { PROJ[p.name] = p; });
  var GROUP = {}; B.groups.forEach(function (g) { GROUP[g.name] = g; });
  var TICKS = B.ticks.map(ms);
  var seats = B.seats.map(function (s) { return { s: s, fx: TZ.fxFromSeat(s, B.allowance, B.moves, B.ticks), key: s.project + '/' + s.seat }; });
  var runs = B.runs.map(function (r) {
    r.st = r.stages.map(function (x) { return { ts: ms(x[0]), stage: x[1], verdict: x[2], account: x[3] }; });
    r.sp = (r.spawned || []).map(ms); r.so = (r.stopped || []).map(ms);
    r.endT = r.end ? ms(r.end) : null; r.createdT = r.created ? ms(r.created) : (r.st[0] || {}).ts;
    return r;
  });
  var FRESH = (B.pressure && B.pressure.usage_fresh_s && B.pressure.usage_fresh_s.value != null ? B.pressure.usage_fresh_s.value : 300) * 1000; // rules fleet.usage_fresh_s
  var MEASURE_STALE = 60 * 60000, RUN_RECENT = 60 * 60000, RUN_SILENT = 30 * 60000, TICK_SILENT = 15 * 60000;

  /* ---------- 時点 t の値（純関数） ---------- */
  function lastIdx(arr, t, f) { var r = -1; for (var i = 0; i < arr.length; i++) { if (f(arr[i]) <= t) r = i; else break; } return r; }
  function lastLE(arr, t) { var r = null; arr.forEach(function (x) { if (x <= t) r = x; }); return r; }
  function allowAt(acct, w, t) { var a = (B.allowance[acct] || {})[w] || []; var i = lastIdx(a, t, function (x) { return ms(x[0]); }); return i < 0 ? null : { ts: ms(a[i][0]), used: a[i][1], resets: a[i][2] ? ms(a[i][2]) : null, model: a[i][3] }; }
  function acctInfo(acct) { return B.accounts.filter(function (a) { return a.label === acct; })[0] || {}; }
  function acctState(acct, t) {
    var info = acctInfo(acct);
    if (info.retired && ms(info.retired) <= t) return { v: 'retired', retired: ms(info.retired) };
    var last = 0; B.windows.forEach(function (w) { var m = allowAt(acct, w, t); if (m) last = Math.max(last, m.ts); });
    var un = (B.unmeasured[acct] || []).filter(function (x) { return ms(x[0]) <= t; }).pop();
    if (!last) return { v: 'unknown', why: '計測の記録が無い' };
    if (un && ms(un[0]) > last) return { v: 'unknown', why: 'AllowanceUnmeasured reason=' + un[1], last: last };
    if (t - last > MEASURE_STALE) return { v: 'unknown', why: '最後の計測から ' + TZ.durMs(t - last), last: last };
    return { v: 'ok', last: last };
  }
  function maxUsed(acct, t) { var m = -1; B.windows.forEach(function (w) { var x = allowAt(acct, w, t); if (x && (!x.resets || x.resets > t)) m = Math.max(m, x.used); }); return m; }
  /* 群の今の口座（時点 t）: 記録 > 種。最初の記録より前は、その記録の previous */
  function curAt(g, t) {
    var recs = (g.records || []).filter(function (r) { return r.ts; }), last = null;
    recs.forEach(function (r) { if (ms(r.ts) <= t) last = r; });
    if (last) return { account: last.account, ts: ms(last.ts), previous: last.previous, by: 'record', src: last.src };
    if (recs.length && recs[0].previous) return { account: recs[0].previous, ts: null, previous: null, by: 'before', src: recs[0].src };
    return { account: g.seed, ts: null, previous: null, by: 'seed', src: B.sources['host.toml'] };
  }
  function occupantsAt(t) { var o = {}; B.groups.forEach(function (g) { o[curAt(g, t).account] = g.name; }); return o; }
  /* 登録の口座（時点 t）: 登録 row の履歴があれば t 以前の最後の行・最初の登録より前は null（測れない）。履歴が無い席だけ state の口座 */
  function seatAcctAt(x, t) { var h = x.fx.account || [], a = null; h.forEach(function (r) { if (r.ts <= t) a = r.account; }); return h.length ? a : (x.s.account && x.s.account.label); }
  /* pipeline の session（run）: 時点 t の段・口座・worker の生存 */
  function runAt(r, t) {
    var st = r.st.filter(function (x) { return x.ts <= t; }); if (!st.length) return null;
    var l = st[st.length - 1], acc = null; st.forEach(function (x) { if (x.account) acc = x.account; });
    var nsp = r.sp.filter(function (x) { return x <= t; }).length, nso = r.so.filter(function (x) { return x <= t; }).length;
    var lastEv = Math.max(l.ts, lastLE(r.sp, t) || 0, lastLE(r.so, t) || 0);
    return { stage: l.stage, verdict: l.verdict, at: l.ts, account: acc, live: nsp > nso, lastEv: lastEv, ended: r.endT != null && r.endT <= t };
  }
  function runAlive(r, t) { var a = runAt(r, t); if (!a || a.ended) return null; return a.live || t - a.lastEv <= RUN_RECENT ? a : null; }
  function runStopped(a) { return a.verdict === 'FAIL' || a.verdict === 'INCONCLUSIVE' || a.stage === 'Questioned' || a.stage === 'Failed' || a.stage === 'Stopped'; }
  function runStatus(r, t) {
    var a = runAt(r, t);
    if (!a || a.ended || t < r.createdT) return { v: 'unknown', why: '記録なし' };
    if (a.account) { var lim = null; ['five_hour', 'seven_day'].forEach(function (w) { var m = allowAt(a.account, w, t); if (m && m.used >= 100 && m.resets > t) lim = { v: 'limit', resume: m.resets, window: w, used: m.used, account: a.account, since: Math.max(m.ts, a.at) }; }); if (lim) return lim; }
    if (runStopped(a)) return { v: 'wait', stopped: true, since: a.at };
    if (a.stage === 'Spawned' || a.stage === 'Implemented' || a.stage === 'Gated') {
      if (a.live) { var tk = lastLE(TICKS, t) || 0; return t - Math.max(tk, a.lastEv) > TICK_SILENT ? { v: 'silent', last: Math.max(tk, a.lastEv) } : { v: 'run', since: a.at }; }
      return t - a.lastEv > RUN_SILENT ? { v: 'silent', last: a.lastEv } : { v: 'run', since: a.at };
    }
    return { v: 'wait', since: a.at };
  }

  /* ---------- 稼働の記録（24 h の帯）: 色 5 種 + 縦線（口座の移動・SeatSpawned / SeatStopped）+ 見ている時刻 ---------- */
  var COL = { run: 'var(--st-run)', wait: 'var(--st-wait)', limit: 'var(--st-limit)', silent: 'var(--st-silent)', unknown: 'none' };
  /* 稼働の記録: 右端 = 見ている時点 t・左端 = t − 幅（?span=24h|6h|3h）・目盛 = 24h は 6 時間・6h は 1 時間・3h は 30 分ごと */
  function strip(stateAt, marks, t) { return TZ.stripSVG(stateAt, marks, t, 16); } // 描き方は ui.js（home と同じ関数・色は CSS の class）
  /* 候補（集合）と次の移り先（持ち主の裁定 2026-09-26T00:43Z・scribe2 の席）:
   * 門 = 今の口座でない ∧ 他の群の今の口座でない ∧ 退役でない ∧ 実測が読める ∧ 鮮度（rules fleet.usage_fresh_s = 300 秒）の内 ∧ 3 窓とも閾値未満（model の窓 = 役割の model の行）。
   * 鍵（辞書順）= (a) 7 日窓と「群の席の役割の model」（rules seat.model.orchestrator）の行の残量（100 − used）の小さい方が大きい順
   *   → (b) 7 日窓の reset が早い順 → (c) 宣言順。役割の model の行を持たない口座は残量を比べず末尾（その中は (b) → (c)）。5 時間窓は門だけ。
   * 予約（記録しない・周ごとに導出）= 群を宣言順（Tier の数字の昇順）に見て、鍵で最上位の「どの群の今の口座でもなく、先の群の予約でもない」口座をその群の next にする。
   * 器の doctor の next=<label|none> と同じ導出。閾値・実測・role の model は build_account.py の出力（rules の行・AllowanceMeasured）。account board と home が同じこの関数を使う */
  var ROLE_MODEL = B.pressure && B.pressure.role_model ? String(B.pressure.role_model.value || '').toLowerCase() : null;
  function tierNo(name) { var m = /^Tier(\d+)$/.exec(name); return m ? Number(m[1]) : null; }
  var GROUP_ORDER = B.groups.map(function (g, i) { return { g: g, i: i, n: tierNo(g.name) }; })
    .sort(function (x, y) { return (x.n == null ? Infinity : x.n) - (y.n == null ? Infinity : y.n) || x.i - y.i; }).map(function (x) { return x.g; });
  function rankAt(g, t) {
    var cur = curAt(g, t).account, occ = occupantsAt(t), P = B.pressure || {};
    var cap = function (w) { return P[w] && P[w].value != null ? P[w].value : 100; };
    var used = function (m) { return m && m.resets && m.resets <= t ? 0 : m ? m.used : null; };
    var rows = g.accounts.map(function (a, order) {
      var rs = [], st = acctState(a, t);
      if (a === cur) rs.push({ k: 'cur', txt: L('current_account') });
      else if (occ[a]) rs.push({ k: 'occ', txt: '他の群が占有（' + occ[a] + '）' });
      if (st.v === 'retired') rs.push({ k: 'ret', txt: '退役' });
      else if (st.v !== 'ok') rs.push({ k: 'unk', txt: L('st_unknown') });
      else if (Math.min(t, BUILT) - st.last > FRESH) rs.push({ k: 'fresh', txt: '測れていない（' + Math.round(FRESH / 1000) + ' 秒超）' }); // doctor の next= と同じ: 鮮度の外は門で落ちる（doctor は測らない）
      var h5 = allowAt(a, 'five_hour', t), d7 = allowAt(a, 'seven_day', t);
      // model の行 = 役割の model（orchestrator = rules seat.model.orchestrator）の最新の 1 行だけ
      var mr = null; ((B.allowance[a] || {}).seven_day_model || []).forEach(function (x) { if (ms(x[0]) <= t && ROLE_MODEL && String(x[3] || '').toLowerCase() === ROLE_MODEL) mr = { ts: ms(x[0]), used: x[1], resets: x[2] ? ms(x[2]) : null, model: x[3] }; });
      if (st.v === 'ok') {
        if (h5 && used(h5) >= cap('five_hour')) rs.push({ k: 'w', txt: '5h ' + used(h5) + '%' + (h5.resets ? ' ↻ ' + TZ.hmd(h5.resets, t) : '') });
        if (d7 && used(d7) >= cap('seven_day')) rs.push({ k: 'w', txt: '7d ' + used(d7) + '%' + (d7.resets ? ' ↻ ' + TZ.hmd(d7.resets, t) : '') });
        if (mr && used(mr) >= cap('seven_day_model')) rs.push({ k: 'w', txt: 'model ' + used(mr) + '%' + (mr.resets ? ' ↻ ' + TZ.hmd(mr.resets, t) : '') });
      }
      var rem = mr ? Math.min(d7 ? 100 - used(d7) : 100, 100 - used(mr)) : null;
      return { acct: a, order: order, ok: !rs.length, reasons: rs, rem: rem, noModel: !mr, reset7: d7 && d7.resets && d7.resets > t ? d7.resets : null };
    });
    var ok = rows.filter(function (r) { return r.ok; }).sort(function (x, y) {
      return (x.noModel - y.noModel) || (x.noModel ? 0 : (y.rem - x.rem)) || ((x.reset7 || Infinity) - (y.reset7 || Infinity)) || (x.order - y.order); });
    var ng = rows.filter(function (r) { return !r.ok; }).sort(function (x, y) { return x.order - y.order; });
    return ok.concat(ng);
  }
  var RESV_MEMO = { t: null, v: null };
  function reservationsAt(t) { // 群ごとの next と、口座 → 予約した群
    if (RESV_MEMO.t === t) return RESV_MEMO.v;
    var by = {}, resv = {};
    GROUP_ORDER.forEach(function (g) {
      var list = rankAt(g, t), nx = list.filter(function (r) { return r.ok && !resv[r.acct]; })[0];
      if (nx) resv[nx.acct] = g.name;
      by[g.name] = { next: nx ? nx.acct : null, list: list };
    });
    RESV_MEMO = { t: t, v: { by: by, resv: resv } };
    return RESV_MEMO.v;
  }
  function candidatesAt(g, t) {
    var R = reservationsAt(t), me = R.by[g.name];
    var list = me.list.map(function (r) { var x = {}; for (var k in r) x[k] = r[k]; x.resvBy = R.resv[r.acct] && R.resv[r.acct] !== g.name ? R.resv[r.acct] : null; return x; });
    return { next: me.next, list: list, reserved: R.resv };
  }
  function candHTML(g, t, big) { // 群の枠と home の枠で同じ見た目
    var c = candidatesAt(g, t);
    var head = '<div class="nextt' + (c.next ? '' : ' none') + '">' + TZ.HS('next_target') + (c.next ? '<b class="mono">' + esc(c.next) + '</b>' : '<b>' + esc(L('no_target')) + '</b><span class="mono small">next=none</span>' + (c.list.some(function (r) { return r.reasons.some(function (x) { return x.k === 'fresh'; }); }) ? '<span class="why" data-term="no_target" tabindex="0">鮮度の外あり</span>' : '')) + '</div>';
    return head + '<ul class="cands2">' + c.list.map(function (r) {
      var cls = (r.ok && !r.resvBy ? 'ok' : r.ok ? 'ok resv' : 'ng') + (r.acct === c.next ? ' next' : '');
      return '<li class="' + cls + '" data-card="cand:' + esc(g.name + '|' + r.acct) + '" tabindex="0"><span class="mono">' + esc(r.acct) + '</span>' +
        (r.ok ? (r.resvBy ? '<span class="resvtag" data-term="reserved" tabindex="0">' + esc(r.resvBy) + ' の予約</span>' : '') + '<span class="kv num">' + (r.noModel ? 'model の行なし · 末尾' : '残量 ' + r.rem + '%') + (r.reset7 ? ' · 7d ↻ ' + TZ.durMs(r.reset7 - t) : '') + '</span>'
          : r.reasons.map(function (x) { return '<span class="why">' + esc(x.txt) + '</span>'; }).join('')) + '</li>';
    }).join('') + '</ul>';
  }
  /* 候補の札の hover card（4 行 + 詳しく: 門の 5 つ・鍵・予約の図） */
  function candCard(gname, acct, t) {
    var g = GROUP[gname]; if (!g) return null;
    var c = candidatesAt(g, t), r = c.list.filter(function (x) { return x.acct === acct; })[0]; if (!r) return null;
    var has = function (k) { return r.reasons.some(function (x) { return x.k === k; }); };
    var gi = function (ok) { return '<span class="gi ' + (ok ? 'ok' : 'ng') + ' sm" aria-hidden="true">' + (ok ? '✓' : '!') + '</span>'; };
    var state = acct === c.next ? L('next_target') : r.ok && r.resvBy ? r.resvBy + ' の予約' : r.ok ? '門を通った' : '門で落ちた';
    var v = r.ok ? (r.noModel ? 'model の行なし → 末尾' : '残量 ' + r.rem + '%') + (r.reset7 ? ' · 7d ↻ ' + TZ.durMs(r.reset7 - t) : '') : r.reasons.map(function (x) { return x.txt; }).join(' / ');
    var gate = [[!has('cur'), '今の口座でない'], [!has('occ'), '他の群の今の口座でない'], [!has('ret'), '退役でない'], [!has('unk'), '実測の記録が読める'], [!has('fresh'), Math.round(FRESH / 1000) + ' 秒以内の実測'], [!has('w'), '3 つの窓とも閾値未満']];
    return TZ.card4({ ti: '<span class="mono">' + esc(acct) + '</span><span>· ' + esc(gname) + ' の候補</span>', k: gi(r.ok) + '<span>' + esc(state) + '</span>',
      v: esc(TZ.cut(v, 34)), src: 'host.toml の accounts と実測',
      srcFull: 'host.toml [[account-group]] name = "' + gname + '" の accounts・*/fleet/events.jsonl の AllowanceMeasured（account=' + acct + '）・判定 = acct.js reservationsAt',
      more: ['<b>門</b>'].concat(gate.map(function (x) { return gi(x[0]) + '<span>' + esc(x[1]) + '</span>'; })).concat(r.ok ? ['<b>鍵</b> 残量の小さい方 → 7d の reset → 宣言順'] : []), fig: 'reserve' });
  }
  function seatMarks(x) { var m = [], prev = null; (x.fx.account || []).forEach(function (r) { if (prev != null && r.account !== prev) m.push({ t: r.ts, kind: 'acct' }); prev = r.account; }); return m; }
  function runMarks(r) { return r.sp.map(function (t) { return { t: t, kind: 'spawn' }; }).concat(r.so.map(function (t) { return { t: t, kind: 'stop' }; })); }
  function limitTimes() {
    var best = null;
    seats.forEach(function (x) { for (var tt = BUILT; tt >= FROM; tt -= 5 * 60000) { if (TZ.seatStatus(x.fx, tt).v === 'limit') { if (!best || tt > best.t) best = { t: tt, who: x.s.target }; break; } } });
    return best;
  }
  /* project の run の数（pipeline dashboard の 4 列と同じ分け方）: bead ごとの最新の run の、時点 t の段で分ける。
   * land = 今日（UTC）の Landed・stop = Questioned / Failed / Stopped / Reviewed(FAIL・INCONCLUSIVE) / Gated(FAIL)・run = Spawned / Implemented / Gated(PASS)・wait = その他（Intake / Queued / Blocked / Reviewed PASS） */
  function runColsAt(pname, t) {
    var day = new Date(t).toISOString().slice(0, 10), last = {}, c = { wait: 0, run: 0, stop: 0, land: 0 };
    runs.forEach(function (r) { if (r.project !== pname || !(r.createdT <= t)) return; var k = r.bead || r.run; if (!last[k] || last[k].createdT <= r.createdT) last[k] = r; });
    Object.keys(last).forEach(function (k) {
      var a = runAt(last[k], t); if (!a) return;
      if (a.stage === 'Landed') { if (new Date(a.at).toISOString().slice(0, 10) === day) c.land++; return; }
      if (runStopped(a) || (a.stage === 'Gated' && a.verdict && a.verdict !== 'PASS') || (a.stage === 'Reviewed' && a.verdict && a.verdict !== 'PASS')) c.stop++;
      else if (a.stage === 'Spawned' || a.stage === 'Implemented' || a.stage === 'Gated') c.run++;
      else c.wait++;
    });
    return c;
  }
  function spark(acct, t, wins) {
    var W = 150, H = 32, X = function (ts) { return ((ts - FROM) / SPAN * W).toFixed(1); }, Y = function (u) { return (H - 2 - u / 100 * (H - 4)).toFixed(1); };
    var line = function (w, stl) {
      var a = ((B.allowance[acct] || {})[w] || []).filter(function (x) { return ms(x[0]) <= t; });
      if (!a.length) return '';
      return '<polyline fill="none" ' + stl + ' points="' + a.map(function (x) { return X(Math.max(FROM, ms(x[0]))) + ',' + Y(x[1]); }).join(' ') + '"/>';
    };
    return '<svg viewBox="0 0 ' + W + ' ' + H + '" role="img" aria-label="' + esc(acct + ' ' + L('spark')) + '"><line x1="0" x2="' + W + '" y1="' + Y(100) + '" y2="' + Y(100) + '" stroke="var(--line-2)" stroke-dasharray="1 2"/>' +
      (wins || ['seven_day_model', 'five_hour']).map(function (w) { return line(w, w === 'seven_day_model' ? 'stroke="var(--st-limit)" stroke-width="1.4" stroke-dasharray="3 2"' : w === 'seven_day' ? 'stroke="var(--ink-2)" stroke-width="1.6"' : 'stroke="var(--band-beads)" stroke-width="1.6"'); }).join('') +
      '<line x1="' + X(t) + '" x2="' + X(t) + '" y1="0" y2="' + H + '" stroke="var(--ink-3)"/></svg>';
  }
  function meter(acct, w, t, st) {
    var m = allowAt(acct, w, t);
    if (!m || st.v !== 'ok') return '<div class="meter unmeasured"><div class="bar"></div><div class="v"><span>' + (m ? '<s>' + m.used + '%</s>' : '―') + '</span><span>' + esc(L('st_unknown')) + '</span></div></div>';
    var reset = m.resets ? (m.resets > t ? TZ.durMs(m.resets - t) : '0m') : '―';
    var cls = m.used >= 100 ? 'w100' : m.used > 80 ? 'w80' : '';
    var cap = B.pressure && B.pressure[w] ? B.pressure[w].value : 80;
    return '<div class="meter"><div class="bar"><i class="' + cls + '" style="width:' + Math.min(100, m.used) + '%"></i><span class="cap" style="left:' + cap + '%"></span></div><div class="v"><b class="num">' + m.used + '%</b><span class="num">↻ ' + reset + '</span></div></div>';
  }
  var WSHORT = { five_hour: '5h', seven_day: '7d', seven_day_model: 'model' };

  /* ---------- session の行（orchestrator = 席・pipeline = run） ---------- */
  /* 二段の退避（器の契約 s2-07l.652・binary c4d6d20・ADR-0071）: 群が移ったら、tick はまず席へ合図を送り（move-signal に群の記録の ts）、
   * /exit は猶予（rules seat.move_grace_s・起点は群の記録の ts）の後に送る。移動中の席 = 合図の ts が群の今の記録の ts と同じ ∧ 残り > 0。
   * 残り = 記録の ts + 猶予 − 時点 t（秒）。種・猶予 0・越えた周は null（器の hook/group.rs の残りの秒と同じ導出） */
  var GRACE = B.pressure && B.pressure.move_grace_s && B.pressure.move_grace_s.value != null ? Number(B.pressure.move_grace_s.value) : null;
  function moveGraceAt(x, t) {
    var sig = x && x.s && x.s.move_signal, p = x && PROJ[x.s.project], g = p && GROUP[p.group];
    if (!sig || !sig.ts || !g || !GRACE) return null;
    var cur = curAt(g, t); if (cur.by !== 'record' || !cur.ts) return null;
    if (seatAcctAt(x, t) === cur.account) return null; // もう移り先で起こし直した席は移動中でない
    var rt = ms(cur.ts); if (ms(sig.ts) !== rt || t < rt) return null;
    var end = rt + GRACE * 1000; if (end <= t) return null;
    return { rem: Math.ceil((end - t) / 1000), end: end, recTs: rt, group: g.name, to: cur.account, from: cur.previous, sample: !!sig.sample, src: sig.src, grace: GRACE };
  }
  function sessionsAt(t) {
    var rows = [];
    B.projects.forEach(function (p) {
      var xs = seats.filter(function (x) { return x.s.project === p.name; });
      if (!xs.length) rows.push({ proj: p.name, group: p.group, role: 'orchestrator', name: null, acct: null, st: { v: 'unknown', why: L('seat_none') }, since: null, key: 'proj:' + p.name, stateAt: function () { return 'unknown'; }, marks: [] });
      xs.forEach(function (x) {
        var st = TZ.seatStatus(x.fx, t), ra = seatAcctAt(x, t), gc = GROUP[p.group] ? curAt(GROUP[p.group], t).account : null;
        rows.push({ proj: p.name, group: p.group, role: 'orchestrator', name: x.s.target, fx: x.fx, acct: st.account || ra, st: st, moveWait: gc && ra && ra !== gc ? { from: ra, to: gc } : null, grace: moveGraceAt(x, t), since: st.v === 'silent' ? st.last : st.since, key: 'seat:' + x.key,
          stateAt: function (tt) { return TZ.seatStatus(x.fx, tt).v; }, marks: seatMarks(x) });
      });
    });
    runs.forEach(function (r) {
      var a = runAlive(r, t); if (!a) return;
      var st = runStatus(r, t), p = PROJ[r.project] || { group: '?' };
      rows.push({ proj: r.project, group: p.group, role: 'pipeline', name: r.run, acct: a.account, st: st, stage: a.stage, verdict: a.verdict, stopped: !!st.stopped, since: st.v === 'silent' ? st.last : a.at, key: 'run:' + r.run,
        stateAt: function (tt) { return runStatus(r, tt).v; }, marks: runMarks(r) });
    });
    return rows;
  }
  /* ---------- 次の一手（閉じた一覧 7 種・直し 7）: project board の home と account board の HOME が同じ関数で判じる ----------
   * 優先順 a 限度 / 移動 → b 応答なし → c 止まっている run → d 束の承認 → e 質問 → f 発効待ち → g なし。
   * d / e / f は質問の台帳（Q = TZ.nextQ(G)）を読める project だけが判じる。読めない project は na（画面は「―」）。 */
  var SETTLE = B.pressure && B.pressure.settle_s ? B.pressure.settle_s.value : null;
  var CAPW = function (w) { return B.pressure && B.pressure[w] ? B.pressure[w].value : 100; };
  function projCtx(pname, t) { // この project の orchestrator の口座の状況（登録 ≠ 群の今の口座・settle・Pending・Refused・逼迫）
    var p = PROJ[pname]; if (!p) return null;
    var g = GROUP[p.group], x = seats.filter(function (y) { return y.s.project === pname; })[0];
    if (!g || !x) return null;
    var cur = curAt(g, t), acc = seatAcctAt(x, t), st = TZ.seatStatus(x.fx, t);
    var mv = (B.move_rows || []).filter(function (m) { return m.kind === 'move' && m.group === g.name && ms(m.ts) <= t; }).pop();
    var mvT = mv ? ms(mv.ts) : null;
    var pend = mv ? (mv.pending || []).filter(function (q) { return q.target === x.s.target && ms(q.ts) <= t; }).pop() : null;
    var ref = (B.move_rows || []).filter(function (m) { return m.kind === 'refused' && m.group === g.name && ms(m.ts) <= t && (!mvT || ms(m.ts) > mvT); }).pop();
    var wait = !!(acc && acc !== cur.account);
    var inSettle = wait && mv && mv.to === cur.account && SETTLE != null && t - mvT <= SETTLE * 1000;
    var press = [], seen = {};
    [acc, cur.account].forEach(function (a) {
      if (!a || seen[a]) return; seen[a] = 1; if (acctState(a, t).v !== 'ok') return;
      B.windows.forEach(function (w) { var m = allowAt(a, w, t); if (m && (!m.resets || m.resets > t) && m.used >= CAPW(w)) press.push({ a: a, w: w, used: m.used, cap: CAPW(w) }); });
    });
    return { p: p, g: g, x: x, cur: cur, acc: acc, st: st, mv: mv, mvT: mvT, pend: wait ? pend : null, ref: ref, wait: wait, inSettle: inSettle, press: press, settle: SETTLE };
  }
  function stoppedRuns(pname, t) { // この project の run（bead ごとの最新）のうち Failed / Reviewed FAIL・INCONCLUSIVE / Gated FAIL / Stopped（Questioned は質問の側）
    var last = {};
    runs.forEach(function (r) { if (r.project === pname && r.createdT <= t) { var k = r.bead || r.run; if (!last[k] || last[k].createdT <= r.createdT) last[k] = r; } });
    return Object.keys(last).map(function (k) { return last[k]; }).filter(function (r) { var a = runAt(r, t); return a && a.stage !== 'Landed' && a.stage !== 'Questioned' && runStopped(a); });
  }
  var NX_KEYS = ['nx_a', 'nx_b', 'nx_c', 'nx_d', 'nx_e', 'nx_f'];
  function nextAt(pname, t, Q) {
    var p = PROJ[pname], c = projCtx(pname, t), it = [];
    var lim = !!(c && c.st.v === 'limit'), a = null;
    if (c && c.ref) a = 'ref'; else if (c && c.wait && !c.inSettle) a = 'wait'; else if (lim) a = 'lim';
    var line = {
      ref: '口座が逼迫し移り先が無い（GroupMoveRefused）',
      wait: '移動中で席が戻らない' + (c && c.pend ? '（GroupMovePending）' : lim ? '（限度の表示のまま）' : ''),
      lim: '利用枠の限度（移動なし）' + (c && c.st.resume ? '・↻ ' + TZ.hmd(c.st.resume, t) : '') };
    it.push({ k: 'nx_a', na: !c, on: !!a, kind: a, line: a ? line[a] : null });
    var sil = !!(c && c.st.v === 'silent');
    it.push({ k: 'nx_b', na: !c, on: sil, line: sil ? '管理 tick が止まっている（tick stale）' : null });
    var stp = p && p.state_dir ? stoppedRuns(pname, t) : null;
    it.push({ k: 'nx_c', na: !stp, on: !!(stp && stp.length), n: stp ? stp.length : null, runs: stp || [], line: stp && stp.length ? 'run ' + stp.length + ' 件が止まっている（Failed / FAIL / Stopped）' : null });
    if (Q) {
      it.push({ k: 'nx_d', on: Q.nonA1 >= 2, n: Q.nonA1, line: Q.nonA1 >= 2 ? '質問 ' + Q.nonA1 + ' 件をまとめて承認できる' : null });
      it.push({ k: 'nx_e', on: !!Q.q0, q0: Q.q0, line: Q.q0 ? 'いちばん長く待つ質問: ' + TZ.t36(Q.q0.attrs.question || Q.q0.title) : null });
      it.push({ k: 'nx_f', on: Q.f > 0, n: Q.f, line: Q.f ? 'あなたの決定 ' + Q.f + ' 件に処分の宣言が無い' : null });
    } else ['nx_d', 'nx_e', 'nx_f'].forEach(function (k) { it.push({ k: k, na: true, on: false, line: null }); });
    var top = it.filter(function (x) { return x.on; })[0] || null;
    var measured = it.some(function (x) { return !x.na; });
    var topK = top ? top.k : measured ? 'nx_g' : 'unknown';
    return { project: pname, c: c, items: it, top: topK, topItem: top, rank: top ? NX_KEYS.indexOf(top.k) : measured ? 6 : 7, noQ: !Q,
      line: top ? top.line : measured ? (Q ? 'orchestrator が動いている / 待っている' : 'd〜f ― = 質問の台帳なし') : 'state dir も席も無い' };
  }
  function nextAll(t, qOf) { // 全 project の次の一手（優先順 → 宣言順）
    return B.projects.map(function (p, i) { var r = nextAt(p.name, t, qOf ? qOf(p.name) : null); r.order = i; return r; })
      .sort(function (x, y) { return x.rank - y.rank || x.order - y.order; });
  }
    return { projCtx: projCtx, stoppedRuns: stoppedRuns, nextAt: nextAt, nextAll: nextAll, NX_KEYS: NX_KEYS, SETTLE: SETTLE, candidatesAt: candidatesAt, reservationsAt: reservationsAt, GROUP_ORDER: GROUP_ORDER, ROLE_MODEL: ROLE_MODEL, candHTML: candHTML, candCard: candCard, FROM: FROM, BUILT: BUILT, SPAN: SPAN, PROJ: PROJ, GROUP: GROUP, TICKS: TICKS, seats: seats, runs: runs, lastIdx: lastIdx, lastLE: lastLE, allowAt: allowAt, acctInfo: acctInfo, acctState: acctState, maxUsed: maxUsed, curAt: curAt, occupantsAt: occupantsAt, seatAcctAt: seatAcctAt, runAt: runAt, runAlive: runAlive, runStopped: runStopped, runStatus: runStatus, COL: COL, strip: strip, seatMarks: seatMarks, runMarks: runMarks, limitTimes: limitTimes, spark: spark, runColsAt: runColsAt, meter: meter, WSHORT: WSHORT, sessionsAt: sessionsAt, moveGraceAt: moveGraceAt, GRACE: GRACE };
  };
})();

/* 次の一手の d / e / f の材料（tsuzuri の graph = 質問の台帳を読める project だけ）: 同じ関数を home と account board が呼ぶ */
TZ.nextQ = function (G) {
  if (!G) return null;
  var QS = TZ.openQuestions(G), isA1 = function (q) { return (q.attrs.labels || []).some(function (l) { return /^A-1/.test(l); }); };
  var G7 = TZ.gaps(G, 't3').filter(function (r) { return r.id === 'G7'; })[0] || { status: 'unknown', keys: [] };
  return { qs: QS, nonA1: QS.filter(function (q) { return !isA1(q); }).length, q0: QS[0] || null, f: G7.status === 'fail' ? G7.keys.length : 0, g7: G7 };
};
