/* tsuzuri mock v3 — 共通の JS（外部の lib 0・fetch は同じ site の vocab.json と data/*.json だけ）
 * 形で語る: 見出しは vocab.json（2 欄 = english / rephrase）から組む・節点と札は全部 hover card（出所の file と行つき）・
 * 状態は 5 値の記号・初心者 / 経験者の切替。帯は出所の正本の名 7 つ（constitution → pipeline）。
 * account board（account/）も同じ file を使う = 状態の記号と判定の関数が 2 面で同じ。 */
(function (root) {
  'use strict';
  var V = null, SUM = {}, G = null, MODE = 'beginner', SEATD = null, BASE = '', PAGE_KIND = 'project';
  // 出所の帯 7（順は憲法の順位 precedence と同じ）
  var BANDS = ['constitution', 'rules', 'ADR', 'SRS', 'design-note', 'beads', 'pipeline'];
  var BAND_PATH = { constitution: 'design-intent/constitution.yaml', rules: 'design-intent/rules.yaml', ADR: 'design-intent/adr/ADR-n.yaml',
    SRS: 'design-intent/srs.yaml', 'design-note': 'design-intent/design-note/*.yaml・docs/design/*.md', beads: '.beads/issues.jsonl', pipeline: '<state dir>/fleet/events.jsonl' };
  function bandCls(b) { return 'band-' + String(b).toLowerCase().replace(/\s+/g, '-'); }
  function bandVar(b) { return 'var(--' + bandCls(b) + ')'; }
  var STAGE_COL = { Intake: 'wait', Queued: 'wait', Blocked: 'wait', Spawned: 'run', Implemented: 'run', Gated: 'run', Reviewed: 'wait',
    Questioned: 'stop', Failed: 'stop', Stopped: 'stop', Landed: 'land' };
  var COL_V = { wait: 'col_wait', run: 'col_run', stop: 'col_stop', land: 'col_land' };
  var SNAP = '2026-09-25T04:25:00Z';          // graph の時点（外部の台帳 scribe2 の pipeline の見本）
  var SILENT_MS = 30 * 60000;                   // run: 最後の合図から 30 分で応答なし

  /* ---------- 小道具 ---------- */
  function esc(s) { return String(s == null ? '' : s).replace(/[&<>"']/g, function (c) { return { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]; }); }
  function cut(s, n) { var a = Array.from(String(s || '')); return a.length > n ? a.slice(0, n - 1).join('') + '…' : a.join(''); }
  function t36(s) { return cut(s, 36); }
  function ms(ts) { if (typeof ts === 'number') return ts; var s = String(ts); if (/T\d\d:\d\dZ$/.test(s)) s = s.replace('Z', ':00Z'); return Date.parse(s); }
  function hm(t) { var d = new Date(ms(t)); return pad(d.getUTCHours()) + ':' + pad(d.getUTCMinutes()) + 'Z'; }
  function hmd(t, ref) { var d = new Date(ms(t)); return (Math.abs(ms(t) - (ref || ms(t))) > 20 * 3600000 ? pad(d.getUTCMonth() + 1) + '-' + pad(d.getUTCDate()) + ' ' : '') + hm(t); }
  function pad(n) { return (n < 10 ? '0' : '') + n; }
  function durMs(d) {
    var m = Math.max(0, Math.floor(d / 60000));
    if (m < 60) return m + 'm';
    var h = Math.floor(m / 60); if (h < 48) return h + 'h' + (m % 60 ? pad(m % 60) : '');
    return Math.floor(h / 24) + 'd';
  }
  function hms(d) { d = Math.max(0, Math.floor(d / 1000)); return Math.floor(d / 3600) + ':' + pad(Math.floor(d / 60) % 60) + ':' + pad(d % 60); }
  function param(n) { return new URLSearchParams(location.search).get(n); }
  function store(k, v) { try { if (v === undefined) return localStorage.getItem(k); localStorage.setItem(k, v); } catch (e) { return null; } }
  function $(s, r) { return (r || document).querySelector(s); }
  function $$(s, r) { return Array.prototype.slice.call((r || document).querySelectorAll(s)); }

  /* ---------- mock の時計 ---------- */
  var T0 = Date.now(), BOARD_BASE = ms(param('now') || SNAP);
  function now() { return BOARD_BASE + (Date.now() - T0); }       // 外部の台帳の pipeline の見本の時計（04:25Z から）
  var SEAT_BASE = null;                                              // 器の記録の時点（build の時刻）から実時間で進む
  var SEAT_VIEW = null, SEAT_VIEW_AT = 0;                            // 再生のつまみ・?at= の時刻（null = 時計どおり）
  function seatNowT() { return SEAT_VIEW == null ? (SEAT_BASE || Date.now()) + (Date.now() - T0) : SEAT_VIEW + (Date.now() - SEAT_VIEW_AT); }
  function setSeatView(t) { SEAT_VIEW = t; SEAT_VIEW_AT = Date.now(); tick(); }

  /* ---------- 語彙（2 欄 = english / rephrase の和集合） ---------- */
  function vt(k) { // key → {label, plain, internal, col}
    if (!V) return { label: k };
    var e = V.english[k]; if (e) return { label: e.label, plain: e.note, internal: e.internal, col: 'english' };
    var r = V.rephrase[k]; if (r) return { label: r.label, plain: r.plain, internal: r.internal + (r.orig ? '\n原語: ' + r.orig : ''), col: 'rephrase', orig: r.orig };
    if (k.indexOf('k:') === 0 || k.indexOf('stage:') === 0 || k.indexOf('e:') === 0) return { label: k.replace(/^[a-z]+:/, ''), missing: true };
    return { label: '〔語彙表に無い: ' + k + '〕', missing: true };
  }
  function L(k) { return vt(k).label; }
  function kindLabel(kind) { return L('k:' + kind); }
  /* 見出し（語彙表の語だけ）: <hN class="hd" data-v=k><span class="hd-t" data-term=k>語</span><button class="q">?</button></hN> */
  function H(tag, k, extra) {
    return '<' + tag + ' class="hd" data-v="' + esc(k) + '"><span class="hd-t" data-term="' + esc(k) + '">' + esc(L(k)) + '</span>' + qmark(k) + (extra || '') + '</' + tag + '>';
  }
  function HS(k) { // 見出しの形をした短い札（列の名・札の名）
    return '<span class="hd" data-v="' + esc(k) + '"><span class="hd-t" data-term="' + esc(k) + '">' + esc(L(k)) + '</span>' + qmark(k) + '</span>';
  }
  function qmark(k) { return '<button type="button" class="q" data-term="' + esc(k) + '" aria-label="' + esc(L(k)) + ' の説明">?</button>'; }
  /* 英語のまま出す語（段・状態）: 語 + 「?」の注釈 */
  function EN(k) { return '<span class="en" data-term="' + esc(k) + '" tabindex="0">' + esc(L(k)) + '</span>'; }

  /* ---------- icon（生の SVG） ---------- */
  var IC = {
    home: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M3 11l9-7 9 7v9a1 1 0 0 1-1 1h-5v-6H9v6H4a1 1 0 0 1-1-1z"/></svg>',
    ask: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M4 5h16v11H9l-5 4z"/><path d="M10 9.5a2 2 0 1 1 2.8 1.8c-.5.3-.8.7-.8 1.2"/><circle cx="12" cy="14.5" r=".6" fill="currentColor"/></svg>',
    map: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><rect x="3" y="4" width="18" height="4" rx="1"/><rect x="3" y="10" width="18" height="4" rx="1"/><rect x="3" y="16" width="18" height="4" rx="1"/></svg>',
    gaps: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="11" cy="11" r="6"/><path d="M20 20l-4.5-4.5"/><path d="M8.5 11h5"/></svg>',
    hourglass: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linejoin="round" aria-hidden="true"><path d="M6 3h12M6 21h12"/><path d="M7 3c0 5 5 6 5 9s-5 4-5 9h10c0-5-5-6-5-9s5-4 5-9" /><path d="M9.5 19h5l-2.5-2.5z" fill="currentColor" stroke="none"/></svg>',
    clock: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" aria-hidden="true"><circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/></svg>',
    redo: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" aria-hidden="true"><path d="M20 12a8 8 0 1 1-2.3-5.7"/><path d="M20 4v5h-5"/></svg>',
    qmark: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" aria-hidden="true"><path d="M4 5h16v11H9l-5 4z"/></svg>',
    cross: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" aria-hidden="true"><path d="M6 6l12 12M18 6L6 18"/></svg>',
    stop: '<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="6" y="6" width="12" height="12" rx="1" fill="currentColor"/></svg>',
    check: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" aria-hidden="true"><path d="M5 12l5 5 9-10"/></svg>',
    warn: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" aria-hidden="true"><path d="M12 3l10 18H2z"/><path d="M12 10v5"/><circle cx="12" cy="18" r=".8" fill="currentColor"/></svg>',
    person: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="12" cy="8" r="4"/><path d="M4 21c1-4 4-6 8-6s7 2 8 6"/></svg>',
    code: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M8 7l-5 5 5 5M16 7l5 5-5 5M14 4l-4 16"/></svg>',
    arrow: '<svg class="arrow" viewBox="0 0 12 12" aria-hidden="true"><path d="M2 6h7M6 3l3 3-3 3" fill="none" stroke="currentColor" stroke-width="1.6"/></svg>',
    folder: '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M3 6h6l2 2h10v11H3z"/></svg>',
    file: '<svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M6 3h8l4 4v14H6z"/><path d="M14 3v4h4"/></svg>',
    up: '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.4" aria-hidden="true"><path d="M12 19V5M6 11l6-6 6 6"/></svg>',
    board: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 9h18M9 9v11"/></svg>',
    link: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1"/><path d="M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1"/></svg>',
    logo: '<svg class="logo" viewBox="0 0 24 24" aria-hidden="true"><rect x="2" y="2" width="20" height="20" rx="6" fill="var(--accent)"/><path d="M7 8h10M7 12h10M7 16h6" stroke="#fff" stroke-width="2" stroke-linecap="round"/></svg>',
    vessel: '<svg class="logo" viewBox="0 0 24 24" aria-hidden="true"><rect x="2" y="2" width="20" height="20" rx="6" fill="var(--ink)"/><path d="M6 9h12M6 13h12M6 17h12" stroke="var(--bg)" stroke-width="2" stroke-linecap="round"/><circle cx="17" cy="6" r="2" fill="var(--st-run)"/></svg>'
  };

  /* ---------- 稼働の記録の幅（?span=24h|6h|3h・右端 = 見ている時点・左端 = その幅だけ前） ---------- */
  var SPANS = { '24h': { ms: 24 * 3600000, tick: 6 * 3600000, tickL: '6 時間' }, '6h': { ms: 6 * 3600000, tick: 3600000, tickL: '1 時間' }, '3h': { ms: 3 * 3600000, tick: 30 * 60000, tickL: '30 分' } };
  function spanKey() { var k = param('span'); return SPANS[k] ? k : '24h'; }
  function spanMs() { return SPANS[spanKey()].ms; }
  function spanTicks(t) { var sp = SPANS[spanKey()], a = t - sp.ms, out = []; for (var x = Math.ceil(a / sp.tick) * sp.tick; x <= t; x += sp.tick) out.push(x); return out; }
  function setSpan(k) { var u = new URL(location.href); if (k === '24h') u.searchParams.delete('span'); else u.searchParams.set('span', k); history.replaceState(null, '', u); }
  function spanSeg() {
    return '<span class="spanbar">' + HS('span') + '<span class="seg" role="group" aria-label="' + esc(L('span')) + '">' + Object.keys(SPANS).map(function (k) { return '<button type="button" data-span="' + k + '" aria-pressed="' + (k === spanKey()) + '">' + k + '</button>'; }).join('') + '</span></span>';
  }
  function spanAxis(t) { // 目盛の文字（位置は幅の %）
    var sp = SPANS[spanKey()], ts = spanTicks(t), step = ts.length > 6 ? 2 : 1;
    return '<div class="saxis" aria-hidden="true">' + ts.map(function (x, i) { var pc = (x - (t - sp.ms)) / sp.ms * 100; return i % step || pc > 92 ? '' : '<span style="left:' + pc.toFixed(2) + '%">' + hm(x).replace('Z', '') + '</span>'; }).join('') + '</div>';
  }

  /* ---------- 状態の記号（5 値・色 + 形 + 動き）・project dashboard と account board で同じ ---------- */
  var ST_V = { run: 'st_run', wait: 'st_wait', limit: 'st_limit', silent: 'st_silent', unknown: 'st_unknown' };
  /* ---------- 稼働の記録（strip）: 両 board が同じこの関数で描く。色・高さ・線の形は CSS の class（.sg-* / .mk-*）だけで決める ----------
   * 凡例の見本（tooltip と常時の 1 行）も同じ class で描く = 実物が変われば凡例も変わる */
  var STRIP_LOW = 0.45, STRIP_CLS = { run: 'sg sg-run', wait: 'sg sg-wait', limit: 'sg sg-limit', silent: 'sg sg-silent', unknown: 'sg sg-unknown' };
  var MARK_CLS = { acct: 'mk mk-acct', spawn: 'mk mk-spawn', stop: 'mk mk-stop', tick: 'mk mk-tick', now: 'mk mk-now' };
  function stripH(v, H) { return v === 'wait' || v === 'unknown' ? H * STRIP_LOW : H; }
  function stripSVG(stateAt, marks, t, H) {
    H = H || 16; var SP = spanMs(), A = t - SP, W = 288, seg = [], prev = null, x0 = 0, step = SP / W;
    for (var i = 0; i <= W; i++) { var v = stateAt(A + i * step); if (v !== prev) { if (prev) seg.push([x0, i, prev]); prev = v; x0 = i; } }
    seg.push([x0, W, prev]);
    var s = '<svg class="stripsvg" viewBox="0 0 ' + W + ' ' + H + '" preserveAspectRatio="none" aria-hidden="true">';
    seg.forEach(function (g) { var h = stripH(g[2], H); s += '<rect class="' + STRIP_CLS[g[2]] + '" x="' + g[0] + '" y="' + (H - h) + '" width="' + Math.max(1, g[1] - g[0]) + '" height="' + h + '"/>'; });
    spanTicks(t).forEach(function (x) { var px = ((x - A) / step).toFixed(1); s += '<line class="' + MARK_CLS.tick + '" x1="' + px + '" x2="' + px + '" y1="0" y2="' + H + '"/>'; });
    (marks || []).forEach(function (m) { if (m.t < A || m.t > t) return; var x = ((m.t - A) / step).toFixed(1); s += '<line class="' + MARK_CLS[m.kind] + '" x1="' + x + '" x2="' + x + '" y1="0" y2="' + H + '"/>'; });
    s += '<line class="' + MARK_CLS.now + '" x1="' + (W - 1) + '" x2="' + (W - 1) + '" y1="0" y2="' + H + '"/>';
    return s + '</svg>';
  }
  /* 見本（幅 28 px・高さは実物と同じ比 = 帯の高さ H に対する低い帯の比 STRIP_LOW） */
  function stripSample(kind, H) {
    H = H || 16; var W = 28, s = '<svg class="ssample" width="' + W + '" height="' + H + '" viewBox="0 0 ' + W + ' ' + H + '" aria-hidden="true" data-sample="' + kind + '">';
    if (STRIP_CLS[kind]) { var h = stripH(kind, H); s += '<rect class="' + STRIP_CLS[kind] + '" x="0" y="' + (H - h) + '" width="' + W + '" height="' + h + '"/>'; }
    else if (kind === 'spawnstop') s += '<line class="' + MARK_CLS.spawn + '" x1="9" x2="9" y1="0" y2="' + H + '"/><line class="' + MARK_CLS.stop + '" x1="19" x2="19" y1="0" y2="' + H + '"/>';
    else if (kind === 'ticks' || kind === 'axis') {
      var sp = SPANS[spanKey()], n = Math.round(sp.ms / sp.tick);
      s += '<rect class="' + STRIP_CLS.wait + '" x="0" y="' + (H - stripH('wait', H)) + '" width="' + W + '" height="' + stripH('wait', H) + '"/>';
      for (var i = 1; i < n; i++) { var x = (W * i / n).toFixed(1); s += '<line class="' + MARK_CLS.tick + '" x1="' + x + '" x2="' + x + '" y1="0" y2="' + H + '"/>'; }
      if (kind === 'axis') s += '<line class="' + MARK_CLS.now + '" x1="' + (W - 1) + '" x2="' + (W - 1) + '" y1="0" y2="' + H + '"/>';
    } else if (MARK_CLS[kind]) s += '<line class="' + MARK_CLS[kind] + '" x1="14" x2="14" y1="0" y2="' + H + '"/>';
    return s + '</svg>';
  }
  /* 常時の凡例（1 行・記号 8 つ + 語・390 幅では 2 行に折る） */
  var LEG8 = [['run', 'lg_run'], ['wait', 'lg_wait'], ['silent', 'lg_silent'], ['limit', 'lg_limit'], ['unknown', 'lg_unknown'], ['acct', 'lg_acct'], ['spawnstop', 'lg_spawnstop'], ['now', 'lg_now']];
  function stripLegend() {
    return '<div class="sleg" aria-label="' + esc(L('history')) + '">' + LEG8.map(function (x) { return '<span class="sleg-i" data-term="history">' + stripSample(x[0], 12) + '<span>' + esc(L(x[1])) + '</span></span>'; }).join('') + '</div>';
  }
  function stIcon(v, lg, extraCls) {
    var inner = v === 'limit' ? IC.hourglass : '<i></i>';
    return '<span class="st st-' + v + (lg ? ' lg' : '') + (extraCls ? ' ' + extraCls : '') + '" role="img" aria-label="' + esc(L(ST_V[v])) + '" data-stv="' + v + '">' + inner + '</span>';
  }
  function stLabel(v) { return '<span class="stlabel st-' + v + '">' + esc(L(ST_V[v])) + '</span>'; }

  /* ---------- 注釈の形（直し 8: 1 行目 = 要点・箇条書き 4 項まで・「詳しく ▸」で畳む）----------
   * 語彙の plain は行の並び: 1 行目 = 要点（≤ 40 字）/ 2 行目から = 項（記号 + ≤ 20 字）/ 行「▸」より後 = 詳しく。
   * 行の中の {…} は実物の見本に置き換える（凡例は実物と同じ class で描く = 実物が変われば凡例も変わる）。 */
  function symHTML(tok) {
    var p = tok.split(':'), k = p[0], a = p[1], b = p.slice(2).join(':');
    if (k === 'st') return stIcon(a);
    if (k === 'strip') return stripSample(a, 14);
    if (k === 'band') return shape(a, { fill: true });
    if (k === 'edge') return '<svg class="esmp" viewBox="0 0 28 10" width="28" height="10" aria-hidden="true">' + edgeSVG(styleOf(a), 1, 5, 27, 5, true) + '</svg>';
    if (k === 'gi') return '<span class="gi ' + a + ' sm" aria-hidden="true">' + ({ ok: '✓', ng: '!' }[a] || '') + '</span>';
    if (k === 'nxm') return '<span class="nxm ' + a + '" aria-hidden="true">' + esc(b || 'c') + '</span>';
    if (k === 'tk') return '<span class="tkhb smpl"><span class="tk tk-' + a + '">' + symHTML('gi:' + (a === 'healthy' ? 'ok' : a === 'absent' ? 'unknown' : 'ng')) + '<b>' + a + '</b></span></span>';
    if (k === 'hb') return '<span class="tkhb smpl"><span class="hb hb-' + a + '"><span class="en">hb</span><b>' + a + '</b></span></span>';
    if (k === 'cand') return '<ul class="cands2 smpl"><li class="' + ({ next: 'ok next', ok: 'ok', resv: 'ok resv', ng: 'ng' }[a] || 'ok') + '">' + (a === 'resv' ? '<span class="resvtag">' + esc(b) + '</span>' : a === 'ng' ? '<span class="why">' + esc(b) + '</span>' : '<span class="mono">' + esc(b) + '</span>') + '</li></ul>';
    if (k === 'badge') return '<span class="nav smpl"><span class="badge">' + esc(a) + '</span></span>';
    if (k === 'thr') return '<span class="thrs" aria-hidden="true"><i></i>' + esc(a) + '</span>';
    if (k === 'net') { var c = { down: ['↓', 'net-down'], up: ['↑', 'net-up'], flat: ['→', 'net-flat'] }[a] || ['→', 'net-flat']; return '<span class="net ' + c[1] + '"><b>' + c[0] + '</b></span>'; }
    if (k === 'ic') return '<span class="icsm">' + (IC[a] || '') + '</span>';
    if (k === 'occ') return '<span class="occ smpl ' + esc(a) + '">' + esc(b) + '</span>';
    if (k === 'fig') return figSVG(a);
    return esc('{' + tok + '}');
  }
  function inl(s) { // 行の中身: {tok} → 見本・`x` → code・他は esc
    return String(s).split(/(\{[^{}]+\}|`[^`]+`)/).map(function (x) {
      if (/^\{[^{}]+\}$/.test(x)) return symHTML(x.slice(1, -1));
      if (/^`[^`]+`$/.test(x)) return '<code>' + esc(x.slice(1, -1)) + '</code>';
      return esc(x);
    }).join('');
  }
  function itemHTML(ln) { // 項: 先頭の記号（1 語 / {tok}）+ 本文
    if (/^\{fig:[^}]+\}$/.test(ln)) return '<li class="figli">' + inl(ln) + '</li>';
    var m = ln.match(/^(\{[^{}]+\}|\S{1,3})\s+(.*)$/);
    if (!m) return '<li><span class="it">' + inl(ln) + '</span></li>';
    return '<li><span class="sy">' + inl(m[1]) + '</span><span class="it">' + inl(m[2]) + '</span></li>';
  }
  function noteParts(txt) {
    var lines = String(txt || '').split('\n'), i = lines.indexOf('▸');
    var main = i < 0 ? lines : lines.slice(0, i), more = i < 0 ? [] : lines.slice(i + 1);
    return { head: main[0] || '', items: main.slice(1).filter(Boolean), more: more.filter(Boolean) };
  }
  function noteHTML(txt, open) {
    var p = noteParts(txt);
    return '<div class="n1">' + inl(p.head) + '</div>' + (p.items.length ? '<ul class="nl">' + p.items.map(itemHTML).join('') + '</ul>' : '') +
      (p.more.length ? '<details class="more"' + (open ? ' open' : '') + '><summary>詳しく ▸</summary><ul class="nl">' + p.more.map(itemHTML).join('') + '</ul></details>' : '');
  }
  function intHTML(txt) { return '<div class="int">' + String(txt || '').split('\n').map(function (l) { return '<div>' + inl(l) + '</div>'; }).join('') + '</div>'; }

  /* ---------- 手順と流れの図（3〜5 箱・幅 ≤ 320・ここ 1 か所で定義）----------
   * 箱: [x, y, w, 文, class]・線: [from, to, 文?, 'dash'?]。class は実物と共有（nxm の色・状態の色）。 */
  var FIGS = {
    back: { h: 104, box: [[4, 38, 70, '戻る を押す', ''], [110, 6, 92, '窓がある → 前面へ', 'ok'], [110, 70, 92, '無い → 新しく開く', ''], [236, 38, 80, 'この窓を閉じる', 'end']],
      ln: [[0, 1, ''], [0, 2, ''], [1, 3, ''], [2, 3, '']] },
    move: { h: 92, box: [[4, 8, 92, '① 移動の承認', 'ok'], [116, 8, 92, '② /exit 待ち', ''], [228, 8, 88, '③ 起こし直し', 'ok'], [116, 60, 92, '戻らない → 保留', 'ng']],
      ln: [[0, 1, ''], [1, 2, ''], [1, 3, '', 'dash']] },
    next: { h: 58, box: [[2, 14, 56, 'a 限度', ''], [66, 14, 56, 'b 応答なし', ''], [130, 14, 56, 'c run', ''], [194, 14, 64, 'd〜f 質問', ''], [266, 14, 52, 'g なし', 'off']],
      ln: [[0, 1, ''], [1, 2, ''], [2, 3, ''], [3, 4, '']], cap: '← 左ほど優先 · 先頭の 1 つが大きく出る' },
    reserve: { h: 100, box: [[2, 8, 68, '門で絞る', ''], [80, 8, 70, '鍵で並べる', ''], [160, 8, 76, 'Tier1 が取る', 'ok'], [244, 8, 76, 'Tier2 は残り', 'ok'], [160, 62, 160, '残りなし → 移り先なし', 'ng']],
      ln: [[0, 1, ''], [1, 2, ''], [2, 3, ''], [3, 4, '', 'dash']] },
    promo: { h: 64, box: [[4, 18, 80, 'memo', ''], [120, 4, 92, 'task', 'ok'], [120, 36, 92, 'ADR・rule 等', 'ok'], [240, 18, 76, '閉じる', 'off']],
      ln: [[0, 1, '昇格'], [0, 2, ''], [1, 3, '', 'dash']] },
    tick: { h: 50, box: [[2, 10, 84, '最後の tick', ''], [100, 10, 106, '≤ 2×周期 healthy', 'ok'], [220, 10, 98, '> 2×周期 stale', 'ng']],
      ln: [[0, 1, ''], [1, 2, '']] },
    hover: { h: 56, box: [[4, 12, 90, '← 根拠 2 段', 'up'], [114, 12, 90, 'この節点', 'self'], [224, 12, 92, '影響 2 段 →', 'down']],
      ln: [[1, 0, ''], [1, 2, '']] }
  };
  function figFs(s, w) { var u = 0; Array.from(s).forEach(function (c) { u += /[\x20-\x7e]/.test(c) ? 0.62 : 1.02; }); return Math.max(8, Math.min(10.5, (w - 8) / u)).toFixed(1); }
  function figSVG(name, cls) { // cls = 箱ごとの class の上書き（例: 次の一手の図で、その project の on の項を実物の色に）
    var f = FIGS[name]; if (!f) return '';
    var s = '<svg class="fig fig-' + name + '" viewBox="0 0 320 ' + f.h + '" width="320" height="' + f.h + '" role="img" aria-label="図: ' + esc(name) + '">' +
      '<defs><marker id="fa-' + name + '" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" orient="auto"><path d="M0 0L8 4L0 8z" fill="var(--ink-3)"/></marker></defs>';
    var B = f.box.map(function (b, i) { return { x: b[0], y: b[1], w: b[2], h: 30, t: b[3], c: cls && cls[i] != null ? cls[i] : b[4] }; });
    f.ln.forEach(function (l) {
      var a = B[l[0]], b = B[l[1]], x1, y1, x2, y2;
      if (Math.abs(a.y - b.y) < 4) { x1 = a.x < b.x ? a.x + a.w : a.x; x2 = a.x < b.x ? b.x : b.x + b.w; y1 = y2 = a.y + 15; }
      else if (a.x + a.w <= b.x) { x1 = a.x + a.w; y1 = a.y + 15; x2 = b.x; y2 = b.y + 15; }
      else { x1 = Math.max(a.x, b.x) + 20; y1 = a.y < b.y ? a.y + 30 : a.y; x2 = x1; y2 = a.y < b.y ? b.y : b.y + 30; }
      s += '<path d="M' + x1 + ' ' + y1 + ' L' + x2 + ' ' + y2 + '" stroke="var(--ink-3)" stroke-width="1.4" fill="none"' + (l[3] ? ' stroke-dasharray="3 3"' : '') + ' marker-end="url(#fa-' + name + ')"/>';
      if (l[2]) s += '<text x="' + ((x1 + x2) / 2) + '" y="' + (Math.min(y1, y2) - 3) + '" font-size="9" text-anchor="middle" fill="var(--ink-3)">' + esc(l[2]) + '</text>';
    });
    B.forEach(function (b) {
      s += '<g class="fb fb-' + (b.c || 'n') + '"><rect x="' + b.x + '" y="' + b.y + '" width="' + b.w + '" height="' + b.h + '" rx="5"/>' +
        '<text x="' + (b.x + b.w / 2) + '" y="' + (b.y + 19) + '" font-size="' + figFs(b.t, b.w) + '" text-anchor="middle">' + esc(b.t) + '</text></g>';
    });
    if (f.cap) s += '<text x="160" y="' + (f.h - 4) + '" font-size="9.5" text-anchor="middle" fill="var(--ink-3)">' + esc(f.cap) + '</text>';
    return s + '</svg>';
  }

  /* ---------- hover card の描画（1 本）: 題 / 種類と状態の記号 / 値 / 出所 の 4 行 + 「詳しく ▸」----------
   * o = { ti, k, v, src, more: [html], fig, ex: [経験者の行], srcFull }。ti と k と v は html（esc 済み）。 */
  function card4(o) {
    var more = (o.more || []).filter(Boolean).map(function (x) { return '<div class="r2">' + x + '</div>'; }).join('') + (o.fig ? '<div class="figw">' + figSVG(o.fig) + '</div>' : '') +
      (o.srcFull && o.srcFull !== o.src ? '<div class="r2 srcf">' + IC.file + '<span>' + esc(o.srcFull) + '</span></div>' : '') +
      (o.ex && o.ex.length ? o.ex.map(function (x) { return '<div class="id">' + esc(x) + '</div>'; }).join('') : '');
    return '<div class="c4"><div class="r ti">' + (o.ti || '') + '</div>' + (o.k ? '<div class="r k">' + o.k + '</div>' : '') + (o.v ? '<div class="r v">' + o.v + '</div>' : '') +
      '<div class="r src">' + IC.file + '<span>' + esc(o.src || '出所なし') + '</span></div></div>' +
      (o.figTop ? '<div class="figw">' + figSVG(o.figTop, o.figCls) + '</div>' : '') +
      (more ? '<details class="more"><summary>詳しく ▸</summary>' + more + '</details>' : '');
  }
  function chunk(s, n) { // 長い文（出所の要約）を句読点で ≤ n 字の行に分ける（詳しくの中だけ）
    var a = Array.from(String(s || '')), out = [], cur = '';
    String(s || '').split(/(?<=[、。・，）])/).forEach(function (p) {
      if (Array.from(cur + p).length <= n) { cur += p; return; }
      if (cur) out.push(cur); cur = '';
      var q = Array.from(p); while (q.length > n) { out.push(q.slice(0, n).join('')); q = q.slice(n); } cur = q.join('');
    });
    if (cur) out.push(cur); return a.length ? out : [];
  }
  function baseName(p) { return String(p || '').split('・')[0].replace(/^.*\/(?=[^/]+\/[^/]+$)/, '…/'); }

  /* ---------- session（orchestrator）の状態: 器の seat state + 口座の残量の event + 器の生存 → 5 値（純関数） ---------- */
  function lastIdx(arr, t) { // ts 昇順の配列で ts <= t の最後の添字（二分探索）
    var lo = 0, hi = arr.length - 1, r = -1;
    while (lo <= hi) { var mid = (lo + hi) >> 1, v = arr[mid].ts != null ? arr[mid].ts : arr[mid]; if (v <= t) { r = mid; lo = mid + 1; } else hi = mid - 1; }
    return r;
  }
  function lastAt(arr, t, pred) {
    if (!arr) return null;
    if (!pred) { var i = lastIdx(arr, t); return i < 0 ? null : arr[i]; }
    var r = null; for (var j = 0; j < arr.length; j++) { var x = arr[j]; if ((x.ts != null ? x.ts : x) <= t && pred(x)) r = x; } return r;
  }
  /* 器の data（account/data/board.json の seat 1 つ・同じ形の seat-t3.json）→ 判定の入力 */
  function fxFromSeat(seat, allowance, moves, ticks) {
    var al = {};
    Object.keys(allowance || {}).forEach(function (a) {
      al[a] = {}; Object.keys(allowance[a]).forEach(function (w) {
        al[a][w] = allowance[a][w].map(function (x) { return { ts: ms(x[0]), used_pct: x[1], resets_at: x[2] ? ms(x[2]) : null, model: x[3] }; });
      });
    });
    var acc = (seat.account_hist || []).map(function (x) { return { ts: ms(x[0]), account: x[1] }; });
    if (!acc.length && seat.account) acc = [{ ts: 0, account: seat.account.label }];
    var tk = (ticks || []).map(ms).concat((seat.tick || []).map(function (x) { return ms(x[0]); })).sort(function (a, b) { return a - b; });
    return { readable: true, target: seat.target, model: seat.model, seat: (seat.state || []).map(function (x) { return { ts: ms(x[0]), state: x[1] }; }),
      account: acc, allowance: al, tick: tk,
      // 管理 tick と heartbeat（器の契約の実物 = seat/<target>/tick-last と heartbeat-off・ts は epoch 秒）: tick-last が無い席は absent
      tickLast: seat.tick_last ? (seat.tick_last.ts ? { ts: ms(seat.tick_last.ts), decision: seat.tick_last.decision, reason: seat.tick_last.reason } : { unreadable: true }) : null,
      hb: seat.heartbeat_off ? 'off' : 'on', hbTs: seat.heartbeat_off ? seat.heartbeat_off.ts : null, stateDir: seat.state_dir || null,
      tickSrc: seat.tick_last_src, hbSrc: seat.hb_src,
      groupMove: (moves || []).filter(function (m) { return m.target === seat.target && m.kind === 'GroupMovePending'; }).map(function (m) { return { ts: ms(m.ts), account: m.account }; }) };
  }
  function seatStatus(fx, t) {
    if (!fx || !fx.readable) return { v: 'unknown', why: '記録が読めない' };
    var s = lastAt(fx.seat, t);
    if (!s) return { v: 'unknown', why: 'state の記録がまだ無い' };
    var acc = lastAt(fx.account || [], t), a = acc ? acc.account : null, lim = null;
    var ws = a && fx.allowance && fx.allowance[a] ? fx.allowance[a] : {};
    Object.keys(ws).forEach(function (w) {
      var arr = ws[w], i = lastIdx(arr, t); if (i < 0) return;
      var m = arr[i];
      if (m.used_pct < 100 || !m.resets_at || m.resets_at <= t) return;
      if (m.model && fx.model && m.model !== fx.model) return;      // model の枠は model が合う席にだけ掛かる
      var since = m.ts; for (var j = i; j >= 0; j--) { if (arr[j].used_pct >= 100) since = arr[j].ts; else break; }
      if (acc && acc.ts > since) since = acc.ts;                   // 口座へ移った時刻より前には遡らない
      if (!lim || m.resets_at > lim.resume) lim = { v: 'limit', since: since, resume: m.resets_at, used: m.used_pct, window: w, account: a, model: m.model };
    });
    if (lim) { lim.move = !!lastAt(fx.groupMove || [], t, function (x) { return x.ts >= lim.since; }); return lim; }
    // 応答なし = 管理 tick が stale（器の側の周が止まっている）。判じるのは tick-last が在り、見ている時点がその記録の後のとき
    var th = tickHealth(fx, t);
    if (th.auth && th.v === 'stale') return { v: 'silent', last: th.last, account: a, tick: th };
    // busy が長いのは「動いている（長い）」= 応答なしではない（境 = rules seat.tick_stale_s）
    if (s.state === 'busy') return { v: 'run', since: s.ts, account: a, long: t - s.ts > TICK_RULES.stale * 1000 };
    return { v: 'wait', since: s.ts, account: a };
  }
  /* 管理 tick の健康: 健全 = 今 − 最後の tick ≤ 2 × seat.tick_interval_s（判じるのは読み手） */
  var TICK_RULES = { interval: 15, stale: 1800, src: null };
  function setTickRules(r) { if (!r) return; if (r.tick_interval_s && r.tick_interval_s.value != null) TICK_RULES.interval = Number(r.tick_interval_s.value); if (r.tick_stale_s && r.tick_stale_s.value != null) TICK_RULES.stale = Number(r.tick_stale_s.value); TICK_RULES.src = 'rules/manifest.toml seat.tick_interval_s / seat.tick_stale_s'; }
  /* 値は器の scribe2 seat tick status と同じ 4 つ: healthy | stale | absent（tick-last が無い）| unreadable（ts= が読めない）。
   * tick-last は最後の 1 行だけなので、記録より前の時点（つまみで遡った時点）は判じない = past（画面は ―）。
   * mock の時計は記録の時点（built_at）から進むが、tick-last は記録の時点の値で判じる（新しい記録が届かないため）。 */
  function tickHealth(fx, t) {
    var lim = 2 * TICK_RULES.interval * 1000;
    if (!fx || !fx.tickLast) return { v: 'absent', src: 'tick-last', auth: false, limit: lim };
    var tl = fx.tickLast;
    if (tl.unreadable) return { v: 'unreadable', src: 'tick-last', auth: false, limit: lim };
    if (t < tl.ts) return { v: 'past', src: 'tick-last', auth: false, limit: lim, last: tl.ts, reason: tl.reason, decision: tl.decision };
    var at = SEAT_BASE ? Math.min(t, Math.max(SEAT_BASE, tl.ts)) : t;
    return { v: at - tl.ts <= lim ? 'healthy' : 'stale', last: tl.ts, age: at - tl.ts, src: 'tick-last', auth: true, limit: lim, reason: tl.reason, decision: tl.decision };
  }
  function ageTxt(d) { return d < 60000 ? Math.max(0, Math.round(d / 1000)) + 's' : durMs(d); }
  /* heartbeat（打刻の合図）の on / off: 器の CLI を撃つ。mock は画面の状態だけ変える（file は書かない） */
  var HB_OVR = {}, FX_BY_TARGET = {};
  function hbState(fx) { var o = HB_OVR[fx.target]; return { v: o || fx.hb || 'on', mock: !!o && o !== (fx.hb || 'on') }; }
  function hbCmd(fx, to) { return 'scribe2 seat heartbeat ' + to + ' --state-dir ' + (fx.stateDir || '<S>') + ' --target ' + fx.target; }
  function doctorTick(fx, t) { // doctor の tick= の語（rules の周期が読めないときは no-rule:<rule id>）
    if (!TICK_RULES.src) return 'no-rule:seat.tick_interval_s';
    var th = tickHealth(fx, t); return th.v === 'past' ? '―' : th.v;
  }
  function doctorSeatLines(fx, s, account, t) { // 器の doctor の席の行と同じ形を 60 字以下の行に折る（経験者の詳しく）
    var line = 'seat: role=' + (s.role || 'orchestrator') + ' target=' + s.target + ' account=' + (account || '?') + ' model=' + (s.model || '?') + ' heartbeat=' + (fx && fx.target ? hbState(fx).v : '?') + ' tick=' + doctorTick(fx, t);
    return line.split(' ').reduce(function (o, w) { var l = o[o.length - 1]; if (l != null && Array.from(l + ' ' + w).length <= 60) o[o.length - 1] = l + ' ' + w; else o.push(w); return o; }, []);
  }
  function tkhbHTML(fx, t) {
    if (!fx || !fx.target) return '';
    FX_BY_TARGET[fx.target] = fx;
    var th = tickHealth(fx, t), hb = hbState(fx);
    var gi = th.v === 'healthy' ? '<span class="gi ok" aria-hidden="true">✓</span>' : th.v === 'stale' || th.v === 'unreadable' ? '<span class="gi ng" aria-hidden="true">!</span>' : '<span class="gi unknown" aria-hidden="true"></span>';
    return '<span class="tkhb" data-tkhb="' + esc(fx.target) + '">' +
      '<span class="tk tk-' + th.v + '" data-term="tick_health" tabindex="0"' + (th.reason ? ' title="reason=' + esc(th.reason) + '"' : '') + '>' + gi + '<span class="en">tick</span><b>' + esc(th.v === 'past' ? '―' : th.v) + '</b>' + (th.age != null ? '<span class="num">' + ageTxt(th.age) + '</span>' : '') + '</span>' +
      '<span class="hb hb-' + hb.v + '" data-term="heartbeat" tabindex="0"><span class="en">heartbeat</span><b>' + hb.v + '</b></span>' +
      '<button type="button" class="btn hbbtn" data-hb="' + esc(fx.target) + '" data-hbto="' + (hb.v === 'on' ? 'off' : 'on') + '">' + esc(L(hb.v === 'on' ? 'hb_to_off' : 'hb_to_on')) + '</button>' +
      (hb.mock ? '<span class="mocktag">' + esc(L('mock_not_sent')) + '</span>' : '') + '</span>';
  }
  var hbDlg = null;
  function openHB(target, to) {
    var fx = FX_BY_TARGET[target]; if (!fx) return;
    if (!hbDlg) { hbDlg = document.createElement('dialog'); hbDlg.className = 'hbdlg'; hbDlg.id = 'hbdlg'; document.body.appendChild(hbDlg); }
    hbDlg.innerHTML = '<form method="dialog"><h3 class="hd"><span class="hd-t">heartbeat を ' + to + ' にする</span></h3>' +
      '<p class="small">' + esc(L('hb_dlg_lead')) + '</p><pre class="cmd" id="hbcmd">' + esc(hbCmd(fx, to)) + '</pre>' +
      '<p class="small muted">' + esc(L('hb_dlg_scope')) + '</p><p class="mocktag">' + esc(L('mock_not_sent')) + '（file は書かない・画面の状態だけ変える）</p>' +
      '<div class="row" style="justify-content:flex-end;gap:8px"><button class="btn" value="cancel">やめる</button><button class="btn primary" value="ok" id="hbok">CLI を撃つ（mock）</button></div></form>';
    hbDlg.onclose = function () {
      if (hbDlg.returnValue !== 'ok') return;
      HB_OVR[target] = to; var t = seatNowT();
      $$('[data-tkhb]').forEach(function (el) { if (el.getAttribute('data-tkhb') === target) el.outerHTML = tkhbHTML(FX_BY_TARGET[target], t); });
      if (root.__tz_onHB) root.__tz_onHB(target, to);
    };
    hbDlg.returnValue = ''; hbDlg.showModal();
  }
  document.addEventListener('click', function (e) { var b = e.target.closest && e.target.closest('[data-hb]'); if (!b) return; e.preventDefault(); e.stopPropagation(); openHB(b.getAttribute('data-hb'), b.getAttribute('data-hbto')); }, true);
  /* 自己検査: 5 値を 1 本ずつ fixture で（§16.2 受入条件 (5)・model の枠の扱いを足した） */
  function seatSelfTest() {
    var mk = function (o) { var b = { readable: true, model: 'Fable', seat: [{ ts: 0, state: 'busy' }], account: [{ ts: 0, account: 'x' }], allowance: { x: {} }, tick: [0, 60000] }; Object.keys(o || {}).forEach(function (k) { b[k] = o[k]; }); return b; };
    var cases = [];
    cases.push(['busy・器が生きている → 動いている', seatStatus(mk(), 120000).v, 'run']);
    cases.push(['idle・器が生きている → 待っている', seatStatus(mk({ seat: [{ ts: 0, state: 'idle' }] }), 120000).v, 'wait']);
    var lim = mk({ allowance: { x: { five_hour: [{ ts: 1000, used_pct: 100, resets_at: 9e6 }] } } });
    cases.push(['AllowanceMeasured used_pct=100 → 限度で止まっている', seatStatus(lim, 120000).v, 'limit']);
    cases.push(['resets_at を過ぎた → 限度から戻る', seatStatus(lim, 9.1e6).v === 'limit' ? 'limit' : 'ok', 'ok']);
    var mdl = mk({ model: 'sonnet', allowance: { x: { seven_day_model: [{ ts: 1000, used_pct: 100, resets_at: 9e6, model: 'Fable' }] } } });
    cases.push(['model の枠が 100 でも別の model の席は止めない', seatStatus(mdl, 120000).v, 'run']);
    var tl = function (last) { return mk({ tickLast: { ts: last } }); };
    cases.push(['tick-last が 2 × 周期の内 → 応答なしではない', seatStatus(tl(100000), 100000 + 2 * TICK_RULES.interval * 1000).v, 'run']);
    cases.push(['tick-last が 2 × 周期より古い → 応答なし', seatStatus(tl(100000), 100000 + 2 * TICK_RULES.interval * 1000 + 1000).v, 'silent']);
    var lg = seatStatus(mk({ tickLast: { ts: 60000 + (TICK_RULES.stale + 60) * 1000 } }), 60000 + (TICK_RULES.stale + 61) * 1000);
    cases.push(['busy が tick_stale_s を越えても tick が健全 → 動いている（長い）', lg.v + (lg.long ? '+long' : ''), 'run+long']);
    cases.push(['tick-last が無い → absent（応答なしではない）', tickHealth(mk(), 1000).v + '/' + seatStatus(mk(), 1000).v, 'absent/run']);
    cases.push(['tick-last が読めない → unreadable', tickHealth(mk({ tickLast: { unreadable: true } }), 1000).v, 'unreadable']);
    cases.push(['記録より前の時点 → 判じない（past）', tickHealth(tl(100000), 50000).v + '/' + seatStatus(tl(100000), 50000).v, 'past/run']);
    cases.push(['記録が読めない → 測れていない', seatStatus({ readable: false }, 0).v, 'unknown']);
    var r = cases.map(function (c) { return { case: c[0], got: c[1], want: c[2], ok: c[1] === c[2] }; });
    root.__tz_seat_selftest = r;
    try { console.table(r); } catch (e) { /* noop */ }
    return r;
  }
  var SEAT_FX = null, BOARD = null, LEDGER = null;
  var FEED_LAST = Date.now(), FEED_ON = param('feed') !== 'off';

  /* ---------- graph の索引 ---------- */
  function Graph(data) {
    var g = { data: data, meta: data.meta, byKey: {}, inc: {}, out: {}, into: {} };
    data.nodes.forEach(function (n) { g.byKey[n.key] = n; g.inc[n.key] = []; g.out[n.key] = []; g.into[n.key] = []; });
    data.edges.forEach(function (e) {
      if (g.inc[e.from]) { g.inc[e.from].push(e); g.out[e.from].push(e); }
      if (g.inc[e.to] && e.to !== e.from) { g.inc[e.to].push(e); g.into[e.to].push(e); }
    });
    g.deg = function (k) { var s = {}; (g.inc[k] || []).forEach(function (e) { s[e.from === k ? e.to : e.from] = 1; }); return Object.keys(s).length; };
    return g;
  }
  /* 辺の「根拠の側」（上り）: 含む系は条の側・それ以外は to の側 */
  function upOf(g, e) {
    if (e.type === 'in-article' || e.type === 'article' || e.type === 'relations.rules') {
      var a = g.byKey[e.from], b = g.byKey[e.to];
      if (a && a.kind === '条' && !(b && b.kind === '条')) return e.from;
      if (b && b.kind === '条') return e.to;
    }
    return e.to;
  }
  function modeQS() { return param('mode') ? '&mode=' + MODE : ''; }
  function href(key) {
    return BASE + 'bead.html?id=' + encodeURIComponent(key) + modeQS();   // 全節点を bead.html が id で描く（question も task も）
  }
  function bandIdx(k) { var n = G.byKey[k]; return n ? BANDS.indexOf(n.band) : 9; }
  /* 近傍: 根拠の側（この節点が拠る先・upOf の向き）と影響の側（この節点に拠るもの）を別々に k 段まで辿る。
   * 兄弟へは広げない（向きを 1 つに決めて辿るので、子 → 親 → 別の子 にはならない）。hub（次数 > hubDeg）は起点以外で止める */
  function around(g, key, o) {
    o = o || {};
    var ku = o.up != null ? o.up : 2, kd = o.down != null ? o.down : 2, cap = o.cap || 40, perCol = o.perCol || 10, hubDeg = o.hubDeg || 30;
    var seen = {}; var order = [key], folded = [];
    seen[key] = { key: key, col: 0, hop: 0, via: null, type: null, style: null, up: null };
    function walk(dir, K) {
      var frontier = [key];
      for (var h = 1; h <= K; h++) {
        var next = [];
        frontier.forEach(function (u) {
          if (u !== key && g.deg(u) > hubDeg) { if (folded.indexOf(u) < 0) folded.push(u); return; }
          (g.inc[u] || []).slice().sort(function (a, b) { return (a.type + a.from + a.to).localeCompare(b.type + b.from + b.to); }).forEach(function (e) {
            var v = e.from === u ? e.to : e.from;
            if (v === u || seen[v]) return;
            var vUp = upOf(g, e) === v;
            if ((dir < 0) !== vUp) return;
            seen[v] = { key: v, col: dir * h, hop: h, via: u, type: e.type, style: e.style, up: vUp };
            order.push(v); next.push(v);
          });
        });
        frontier = next;
      }
    }
    walk(-1, ku); walk(1, kd);
    var hubHidden = 0; folded.forEach(function (u) { hubHidden += Math.max(0, g.deg(u) - 1); });
    var cols = {}; order.slice(1).forEach(function (k) { var c = seen[k].col; (cols[c] = cols[c] || []).push(k); });
    var shown = [key], capCut = 0;
    [-1, 1, -2, 2, -3, 3].forEach(function (c) {
      (cols[c] || []).sort(function (a, b) { return bandIdx(a) - bandIdx(b) || String(a).localeCompare(String(b), 'ja', { numeric: true }); }).forEach(function (k, i) {
        if (i < perCol && shown.length < cap && (seen[k].hop === 1 || shown.indexOf(seen[k].via) >= 0)) shown.push(k); else capCut++;
      });
    });
    var rows = shown.map(function (k) { return seen[k]; });
    rows.sort(function (a, b) { return a.col - b.col || bandIdx(a.key) - bandIdx(b.key) || String(a.key).localeCompare(String(b.key), 'ja', { numeric: true }); });
    return { origin: key, rows: rows, shown: rows.length, total: order.length + hubHidden, up: ku, down: kd, cut: { hub: hubHidden, cap: capCut, folded: folded } };
  }

  /* ---------- 節点の状態と要約（出所: summaries.json・bead は title だけ・run は段と理由） ---------- */
  function isOpenQ(n) { return n.kind === '問い' && /^open/.test(n.attrs.status || ''); }
  function isBead(n) { var a = n.attrs || {}; return a.src === 'beads' || (a.src === 'mock' && a.issue_type) || (n.kind === '問い'); }
  function nodeState(n) {
    var a = n.attrs || {}, s = SUM[n.key];
    if (n.kind === '問い') return isOpenQ(n) ? { cls: 'ng', label: 'open' } : { cls: 'ok', label: 'closed' };
    if (n.kind === '裁定') return { cls: 'ok', label: a.provisional ? '記録済（仮）' : '記録済' };
    if (n.kind === '方針') return { cls: 'ok', label: '効いている' };
    if (n.kind === '走行') { var rs = lastStage(n); return rs ? { cls: STAGE_COL[rs.stage] === 'stop' ? 'ng' : 'ok', label: rs.stage } : { cls: 'unknown', label: '測れていない' }; }
    if (a.status === 'open' || a.status === 'in_progress') return { cls: 'open', label: a.status };
    if (a.status === 'closed') return { cls: 'ok', label: 'closed' };
    if (s && s.status) return { cls: s.status === 'proposed' || s.status === '未定' ? 'unknown' : 'ok', label: s.status };
    return { cls: 'unknown', label: '状態なし' };
  }
  function lastStage(run) { var st = run.attrs.stages || []; if (!st.length) return null; var l = st[st.length - 1]; return { stage: l[1], at: l[0], detail: l[2] }; }
  function summaryOf(n) {
    var s = SUM[n.key], a = n.attrs || {};
    if (s) return { plain: s.plain, eng: s.eng, parent: s.parent, src: s.src };
    if (isBead(n)) return { plain: null, eng: null, src: 'bead は title だけ（description は持ち込まない）', bead: true };
    if (n.kind === '裁定') return { plain: '質問に「おすすめどおり」で答えた決定です（仮）。', eng: 'answers → ' + (a.question || '?') + '・' + (a.at || '') + (a.batch ? '・まとめて承認 ' + a.batch : ''), src: 'graph.json の欄（mock の見本）' };
    if (n.kind === '方針') return { plain: a.gist ? String(a.gist).replace(/^要旨（[^）]*）:\s*/, '') : null, eng: 'scope=' + (a.scope || '?'), src: 'graph.json の要旨（mock の言い換えの見本）' };
    if (n.kind === '走行') {
      var rs = lastStage(n); if (!rs) return { plain: null, eng: null };
      return { plain: 'いまの段は ' + rs.stage + '。' + (reasonShort(rs, n).full || ''), eng: 'RunStage=' + rs.stage + (rs.detail ? ' detail=' + rs.detail : '') + '・段 ' + (a.stages || []).length, src: '器の event（段と理由）' };
    }
    return { plain: null, eng: null, src: '材料なし' };
  }
  function srcText(n) { var s = n.src || {}; return (s.file || '出所なし') + (s.line ? ':' + s.line : '') + (s.note ? '（' + s.note + '）' : (s.file && !s.line ? '（行は測れていない）' : '')); }

  /* ---------- run と pipeline dashboard ---------- */
  function runsOf(g, beadKey) {
    return (g.into[beadKey] || []).filter(function (e) { return e.type === 'run_of'; }).map(function (e) { return g.byKey[e.from]; })
      .filter(Boolean).sort(function (a, b) { return a.attrs.started_at.localeCompare(b.attrs.started_at); });
  }
  function runStateAt(run, T) {
    var st = run.attrs.stages.filter(function (s) { return s[0] <= T; });
    if (!st.length) return null;
    var last = st[st.length - 1];
    var v = run.attrs.verdicts.filter(function (x) { return x[0] <= T; });
    return { stage: last[1], at: last[0], detail: last[2], verdict: v.length ? v[v.length - 1] : null, stages: st };
  }
  function classify(rs) {
    var s = rs.stage, v = rs.verdict;
    if (s === 'Reviewed') return v && v[2] !== 'PASS' ? 'stop' : 'wait';
    if (s === 'Gated' && v && v[1] === 'Gated' && v[2] !== 'PASS' && v[0] === rs.at) return 'stop';
    return STAGE_COL[s] || 'wait';
  }
  /* 止まった理由を「印 + 段の名」に畳む（全文は hover card） */
  function reasonShort(rs, run) {
    var s = rs.stage, v = rs.verdict, d = String(rs.detail || '');
    if (s === 'Questioned') return { ic: 'qmark', w: 'Questioned', full: 'runner の質問で止まっている（' + (d.replace('about:', '') || '理由の欄が空') + '）' };
    if (s === 'Stopped') { var q = (rs.stages || []).filter(function (x) { return x[1] === 'Questioned'; }).pop();
      return { ic: 'stop', w: 'Stopped', full: q ? '質問の後に止めた（' + String(q[2] || '').replace('about:', '') + '）' : '器が止めた（理由の欄が空）' }; }
    if (s === 'Failed') return { ic: 'cross', w: 'Failed', full: 'Failed（' + (d || '理由の欄が空') + '）' };
    if (s === 'Reviewed' && v && v[2] !== 'PASS') return { ic: 'cross', w: 'Reviewed ' + v[2], full: '起動前の審査で ' + v[2] + (v[3] ? '（' + v[3] + '）' : '') + '・次の段が無い' };
    if (s === 'Gated' && v && v[2] !== 'PASS') return { ic: 'cross', w: 'Gated ' + v[2], full: '検査で ' + v[2] + (v[3] ? '（' + v[3] + '）' : '') };
    if (s === 'Landed') return { ic: 'check', w: 'Landed', full: '本線に取り込んだ' };
    return { ic: 'clock', w: s, full: s + (d ? '（' + d + '）' : '') };
  }
  function board(g, T) {
    var day = T.slice(0, 10), cols = { wait: [], run: [], stop: [], land: [] };
    g.data.nodes.filter(function (n) { return n.repo === 's2' && n.attrs && n.attrs.src === 'beads'; }).forEach(function (b) {
      if (b.attrs.created_at && b.attrs.created_at > T) return;
      var openAtT = !(b.attrs.closed_at && b.attrs.closed_at <= T);
      var runs = runsOf(g, b.key).filter(function (r) { return r.attrs.started_at <= T; });
      if (!runs.length) return;
      var run = runs[runs.length - 1], rs = runStateAt(run, T);
      if (!rs) return;
      var col = classify(rs);
      if (col === 'land') { if (rs.at.slice(0, 10) !== day) return; } else if (!openAtT) return;
      var lastEv = rs.stages[rs.stages.length - 1][0];
      var q = (g.out[run.key] || []).filter(function (e) { return e.type === 'raised'; }).map(function (e) { return e.to; })[0];
      var silent = col === 'run' && (ms(T) - ms(lastEv)) > SILENT_MS;
      cols[col].push({ bead: b, run: run, n: runs.length, col: col, rs: rs, why: reasonShort(rs, run), since: rs.at, last: lastEv, account: run.attrs.account,
        question: (rs.stage === 'Questioned' || rs.stage === 'Stopped') ? q : null, st: col === 'run' ? (silent ? 'silent' : 'run') : col === 'land' ? null : 'wait' });
    });
    Object.keys(cols).forEach(function (k) { cols[k].sort(function (a, b) { return String(b.since).localeCompare(String(a.since)); }); });
    return cols;
  }
  /* kanban の札: 状態の記号 + 題 + meta 2 つまで（止まっているは理由が meta の 1 つ目） */
  function kcardHTML(c) {
    var b = c.bead;
    var m1 = c.col === 'stop' ? '<span class="why">' + IC[c.why.ic] + esc(c.why.w) + '</span>' : '<span>' + IC.redo + c.n + '</span>';
    var m2 = c.col === 'land' ? '<span>' + IC.check + '<span class="num">' + esc(hm(c.since)) + '</span></span>'
      : '<span' + (c.st === 'silent' ? ' style="color:var(--st-silent);font-weight:700"' : '') + '>' + IC.clock + '<span class="num" data-since="' + ms(c.st === 'silent' ? c.last : c.since) + '">' + durMs(now() - ms(c.st === 'silent' ? c.last : c.since)) + '</span></span>';
    var sym = c.st ? stIcon(c.st) : '<span class="st" style="color:var(--s-land)">' + IC.check.replace('<svg', '<svg width="14" height="14"') + '</span>';
    return '<a class="kcard nl' + (c.col === 'stop' ? ' why-stop' : '') + '" href="' + href(b.key) + '" data-key="' + esc(b.key) + '" data-run="' + esc(c.run.key) + '">' +
      '<div class="t">' + sym + '<span class="tt" data-t>' + esc(t36(b.title)) + '</span></div><div class="m"><span class="kid">' + esc(b.id) + '</span>' + m1 + m2 + '</div></a>';
  }

  /* ---------- 質問・材料 ---------- */
  /* design-intent に未反映（持ち主の追加 2026-09-26）: (a) open の memo で昇格先（promoted_to）の線が無い (b) 処分の宣言が無いあなたの決定 = G7 の集合（gaps と同じ関数）
   * (c) 要望 = 答えに「要望」の印が付いた question で、touches の先が答えの後に変わっていない。年齢の古い順。heads = memo の 4 節の見出し（本文は読まない） */
  function unreflected(g, now, heads) {
    var out = [], t3 = g.data.nodes.filter(function (n) { return n.repo === 't3'; });
    var age = function (s) { return s ? now - ms(s.length === 17 ? s.replace('Z', ':00Z') : s) : null; };
    t3.filter(function (n) { return n.kind === 'memo' && !/^closed/.test(n.attrs.status || ''); }).forEach(function (n) {
      if ((g.out[n.key] || []).some(function (e) { return e.type === 'promoted_to'; })) return;
      out.push({ key: n.key, id: n.id, title: n.title, kind: 'memo', age: age(n.attrs.created_at), next: 'nx_promote', heads: n.attrs.sections || (heads && heads[n.id] != null ? Ledger0.headsOf(heads[n.id]) : null), sample: isSample(n) });
    });
    var g7 = gaps(g, 't3', { sample: true }).filter(function (r) { return r.id === 'G7'; })[0];
    (g7 ? g7.keys : []).forEach(function (k) { var n = g.byKey[k]; if (n) out.push({ key: k, id: n.id, title: n.title, kind: '裁定', age: age(n.attrs.at), next: 'nx_declare', sample: isSample(n) }); });
    t3.filter(function (n) { return n.kind === '問い' && n.attrs.request && n.attrs.answered_at; }).forEach(function (n) {
      var at = ms(n.attrs.answered_at.length === 17 ? n.attrs.answered_at.replace('Z', ':00Z') : n.attrs.answered_at);
      var tos = (g.out[n.key] || []).filter(function (e) { return e.type === 'touches'; }).map(function (e) { return g.byKey[e.to]; }).filter(Boolean);
      var moved = tos.some(function (x) { var u = x.updated || (x.attrs && x.attrs.updated_at); return u && ms(u.length === 17 ? u.replace('Z', ':00Z') : u) > at; });
      if (!moved) out.push({ key: n.key, id: n.id, title: n.title, kind: '要望', age: now - at, next: 'nx_reflect', touches: tos.map(function (x) { return x.id; }), sample: isSample(n) });
    });
    return out.sort(function (a, b) { return (b.age || 0) - (a.age || 0); });
  }
  var Ledger0 = { headsOf: function (b) { return ['出所', '観測', '候補', '昇格条件'].filter(function (h, i) { return b & (1 << i); }); } };
  function isSample(n) { return !!(n && n.attrs && n.attrs.sample); } // mock の見本データ（台帳・要件書には無い）
  function openQuestions(g) {
    // 決定待ちの列 = 実の question だけ（mock の見本の question は地図・一覧・近傍・台帳の面には出るが、答える列には入れない）
    return g.data.nodes.filter(function (n) { return n.kind === '問い' && n.repo === 't3' && n.attrs.status === 'open' && !isSample(n); })
      .sort(function (a, b) { return a.attrs.posted_at.localeCompare(b.attrs.posted_at) || a.id.localeCompare(b.id, 'ja', { numeric: true }); });
  }
  function material(g, qk) {
    var touches = (g.out[qk] || []).filter(function (e) { return e.type === 'touches'; }).map(function (e) { return e.to; });
    var blocking = (g.into[qk] || []).filter(function (e) { return e.type === 'blocks'; }).map(function (e) { return e.from; });
    return { touches: touches, blocking: blocking };
  }
  function rulingOf(g, qk) { var e = (g.into[qk] || []).filter(function (e) { return e.type === 'answers'; })[0]; return e ? g.byKey[e.from] : null; }

  /* ---------- 節点の印と link（形は 1 つ・色は帯ごとに 1 色） ---------- */
  function shape(band, opts) {
    opts = opts || {};
    return '<span class="shape ' + bandCls(band) + (opts.fill ? ' fill' : '') + (opts.big ? ' big' : '') + '"' + (opts.style ? ' style="' + opts.style + '"' : '') + ' aria-hidden="true"></span>';
  }
  function nodeShape(n, big) {
    var st = nodeState(n);
    var style = n.kind === '問い' && isOpenQ(n) ? 'color:var(--s-stop)' : '';
    return shape(n.band, { fill: st.cls !== 'open' && !(n.kind === '問い' && isOpenQ(n)), big: big, style: style });
  }
  function bandChip(b) { return '<span class="bchip ' + bandCls(b) + '" data-term="b:' + esc(b) + '" tabindex="0"><i></i>' + esc(L('b:' + b)) + '</span>'; }
  /* 一覧の項目: 印 + 番号 + 題（+ 右に小さい数）— 全頁で同じ形 */
  function itemNode(key, aside, opt) {
    opt = opt || {};
    var n = G.byKey[key];
    if (!n) return '<li>' + '<span class="gi ng" aria-hidden="true">×</span><span class="ttl">' + esc(L('not_found')) + '</span></li>';
    return '<li>' + (opt.num ? '<span class="nb">' + opt.num + '</span>' : nodeShape(n)) + '<a class="ttl nl" href="' + href(key) + '" data-key="' + esc(key) + '"><span class="nid">' + esc(n.id) + '</span> <span data-t>' + esc(t36(n.title)) + '</span></a>' +
      (aside ? '<span class="aside">' + aside + '</span>' : '') + (opt.sub || '') + '</li>';
  }

  /* ---------- 線（style は族で決まる・面には型の名だけを出す） ---------- */
  var STYLE_OF = { blocks: 'bold', 'in-article': 'dotted', article: 'dotted', 'part-of': 'dotted', 'parent-child': 'dotted', run_of: 'dotted', ran_by: 'dotted', supersedes: 'arrow', amends: 'arrow' };
  function styleOf(type) { return STYLE_OF[type] || 'solid'; }
  function edgeSVG(style, x1, y1, x2, y2, straight) {
    var st = { solid: 'stroke="var(--ink-3)" stroke-width="1.2"', bold: 'stroke="var(--s-stop)" stroke-width="3"',
      dotted: 'stroke="var(--ink-3)" stroke-width="1.4" stroke-dasharray="2 3"', arrow: 'stroke="var(--st-limit)" stroke-width="2"' }[style] || 'stroke="var(--ink-3)"';
    var d = straight ? 'M' + x1 + ' ' + y1 + ' L ' + x2 + ' ' + y2 : 'M' + x1 + ' ' + y1 + ' C ' + ((x1 + x2) / 2) + ' ' + y1 + ', ' + ((x1 + x2) / 2) + ' ' + y2 + ', ' + x2 + ' ' + y2;
    var p = '<path d="' + d + '" fill="none" ' + st + '/>';
    if (style === 'arrow') p += '<polygon points="' + (x2 - 6) + ',' + (y2 - 3.5) + ' ' + x2 + ',' + y2 + ' ' + (x2 - 6) + ',' + (y2 + 3.5) + '" fill="var(--st-limit)"/>';
    return p;
  }
  /* 凡例: その眺めに実際に出ている帯と辺の型だけ（帯の名の横に正本の path） */
  function legendKey() { // 形・色・縁の 3 行（囲みの角では意味を出さない）
    var sq = '<svg viewBox="0 0 14 14" class="lk" aria-hidden="true"><rect x="2" y="2" width="10" height="10" rx="1" fill="var(--ink-2)"/></svg>';
    var ci = '<svg viewBox="0 0 14 14" class="lk" aria-hidden="true"><circle cx="7" cy="7" r="5" fill="var(--ink-2)"/></svg>';
    var co = BANDS.map(function (b) { return '<i style="background:' + bandVar(b) + '"></i>'; }).join('');
    var bx = function (st, txt) { return '<svg viewBox="0 0 30 16" class="lk wide" aria-hidden="true"><rect x="1.5" y="1.5" width="27" height="13" rx="4" fill="var(--panel)" ' + st + '/>' + (txt ? '<rect x="7" y="7" width="16" height="2" fill="var(--ink-3)"/>' : '') + '</svg>'; };
    return '<div class="legend lkey"><span data-term="lg_shape" tabindex="0">' + sq + ci + esc(L('lg_shape')) + '</span>' +
      '<span data-term="lg_color" tabindex="0"><span class="sw7">' + co + '</span>' + esc(L('lg_color')) + '</span>' +
      '<span data-term="lg_border" tabindex="0">' + bx('stroke="var(--s-stop)" stroke-width="2"') + bx('stroke="var(--ink)" stroke-width="3"') + bx('stroke="var(--line-2)"', true) + esc(L('lg_border')) + '</span></div>';
  }
  function legendHTML(bands, types, opt) {
    bands = bands || BANDS; types = types || [];
    return legendKey() + (opt && opt.hover ? '<div class="legend lkey"><span data-term="lg_hover" tabindex="0"><svg viewBox="0 0 30 16" class="lk wide" aria-hidden="true"><rect x="1.5" y="1.5" width="27" height="13" rx="4" fill="var(--panel)" stroke="var(--accent)" stroke-width="2.5"/></svg>' + esc(L('lg_hover')) + '</span></div>' : '') + '<div class="legend">' + bands.map(function (b) { return '<span data-term="b:' + esc(b) + '" tabindex="0">' + shape(b, { fill: true }) + '<b>' + esc(L('b:' + b)) + '</b><code class="path">' + esc(BAND_PATH[b]) + '</code></span>'; }).join('') + '</div>' +
      (types.length ? '<div class="legend">' + types.map(function (t) { return '<span data-term="e:' + esc(t) + '" tabindex="0"><svg viewBox="0 0 28 10" aria-hidden="true">' + edgeSVG(styleOf(t), 1, 5, 27, 5, true) + '</svg><code>' + esc(t) + '</code></span>'; }).join('') + '</div>' : '');
  }

  /* ---------- つながり（図: 出所の帯 × 距離の列・節点に id と短い題・hover card） ---------- */
  /* beads の帯は種類の行（sub-row）に分ける: 順 = epic → task → memo → question → あなたの決定 → 受け → 全体への指示（持ち主 05:19Z）。
   * 帯の左の名は beads のまま・行の頭に種類の名を小さく・節点 0 の行は畳んで「task 0」の 1 語 */
  var BEADS_LANES = ['epic', '契約', 'memo', '問い', '裁定', '受け', '方針'];
  function laneOf(n) { return n && n.band === 'beads' ? (BEADS_LANES.indexOf(n.kind) >= 0 ? n.kind : '契約') : ''; }
  function lanesOf(band) { return band === 'beads' ? BEADS_LANES : ['']; }
  var LANE0 = 18; // 0 の行の高さ
  function laneLabel(k, n) { return L('k:' + k) + (n === 0 ? ' 0' : ''); }
  function svgAround(r) {
    var COLS = [-3, -2, -1, 0, 1, 2, 3].filter(function (c) { return c === 0 || r.rows.some(function (x) { return x.col === c; }); });
    var CW = Math.max(168, Math.min(300, Math.floor(1100 / COLS.length))), LW = 128, NH = 40, GAP = 8, PAD = 12, HEAD = 28, NW = CW - 20, CHARS = Math.floor((NW - 16) / 12.5);
    var colName = { '-3': 'nb_up3', '-2': 'nb_up2', '-1': 'nb_up', '0': 'nb_self', '1': 'nb_down', '2': 'nb_down2', '3': 'nb_down3' };
    var cells = {}; r.rows.forEach(function (x) { var n = G.byKey[x.key]; var l = n ? n.band : 'beads', k = laneOf(n || { band: 'beads', kind: '契約' }); (cells[x.col + '|' + l + '|' + k] = cells[x.col + '|' + l + '|' + k] || []).push(x); });
    var cnt = function (l, k) { var m = 0; COLS.forEach(function (c) { m = Math.max(m, (cells[c + '|' + l + '|' + k] || []).length); }); return m; };
    var used = BANDS.filter(function (l) { return lanesOf(l).some(function (k) { return cnt(l, k); }); });
    var laneY = {}, laneH = {}, bandY = {}, bandH = {}, W = LW + COLS.length * CW, y = HEAD, pos = {};
    used.forEach(function (l) {
      bandY[l] = y;
      lanesOf(l).forEach(function (k) { var m = cnt(l, k); laneY[l + '|' + k] = y; laneH[l + '|' + k] = l === 'beads' ? (m ? m * (NH + GAP) + PAD : LANE0) : Math.max(44, Math.max(1, m) * (NH + GAP) + PAD); y += laneH[l + '|' + k]; });
      bandH[l] = y - bandY[l];
    });
    var s = ['<svg class="hlsvg" viewBox="0 0 ' + W + ' ' + (y + 4) + '" style="max-width:' + W + 'px" role="group" aria-label="' + esc(L('around')) + '">'];
    COLS.forEach(function (c, i) { s.push('<text x="' + (LW + i * CW + CW / 2) + '" y="18" font-size="12" font-weight="600" text-anchor="middle" fill="var(--ink-3)">' + esc(L(colName[c])) + '</text>'); });
    used.forEach(function (l, li) {
      s.push('<rect x="0" y="' + bandY[l] + '" width="' + W + '" height="' + bandH[l] + '" fill="' + bandVar(l) + '" fill-opacity="' + (li % 2 ? .05 : .09) + '"/>');
      s.push('<text x="8" y="' + (bandY[l] + 14) + '" font-size="12" font-weight="700" fill="' + bandVar(l) + '">' + esc(L('b:' + l)) + '</text>');
      if (l !== 'beads') s.push('<text x="8" y="' + (bandY[l] + 30) + '" font-size="9" fill="var(--ink-3)" font-family="ui-monospace,monospace">' + esc(cut(BAND_PATH[l].replace('design-intent/', 'di/'), 15)) + '</text>');
      lanesOf(l).forEach(function (k, ki) {
        var ly = laneY[l + '|' + k], m = cnt(l, k);
        if (l === 'beads') {
          if (ki) s.push('<line class="lane-sep" x1="0" x2="' + W + '" y1="' + ly + '" y2="' + ly + '"/>');
          s.push('<text class="lane-lab' + (m ? '' : ' zero') + '" x="' + (LW - 6) + '" y="' + (ly + (m ? 16 : 13)) + '" text-anchor="end" data-lane="' + esc(k) + '">' + esc(laneLabel(k, m)) + '</text>');
        }
        COLS.forEach(function (c, i) { (cells[c + '|' + l + '|' + k] || []).forEach(function (x, j) { pos[x.key] = { x: LW + i * CW + 10, y: ly + PAD / 2 + j * (NH + GAP), w: NW, h: NH }; }); });
      });
    });
    r.rows.forEach(function (x) {
      if (!x.via || !pos[x.via] || !pos[x.key]) return;
      var a = pos[x.via], b = pos[x.key];
      var ax = b.x > a.x ? a.x + a.w : b.x < a.x ? a.x : a.x + a.w / 2, bx = b.x > a.x ? b.x : b.x < a.x ? b.x + b.w : b.x + b.w / 2;
      s.push('<g class="e" data-u="' + esc(x.up ? x.key : x.via) + '" data-d="' + esc(x.up ? x.via : x.key) + '">' + edgeSVG(x.style, ax, a.y + a.h / 2, bx, b.y + b.h / 2) + '</g>');
    });
    r.rows.forEach(function (x) { var n = G.byKey[x.key], p = pos[x.key]; if (p && n) s.push(nodeBox(n, p, x.col === 0, CHARS)); });
    s.push('</svg>');
    return s.join('');
  }
  /* 形は 2 種だけ: 四角 = file に書かれた行（constitution・rules・ADR・SRS・design-note）／丸 = 台帳と器にある動くもの（beads・pipeline） */
  function isMoving(band) { return band === 'beads' || band === 'pipeline'; }
  /* 図の節点 1 つ: 囲みは 1 種（rx 4 の四角）で意味を持たない。意味は (1) 頭の記号の形 (2) 色 = 帯 (3) 縁 = 状態 だけ
   * 縁: open の question = 赤・起点 = 太い縁・closed = 灰の文字。1 行目 = id を必ず全部・2 行目 = 題 */
  function nodeBox(n, p, self, chars, extra, idMax) {
    var lc = bandVar(n.band), open = isOpenQ(n), closed = n.attrs && /^closed/.test(n.attrs.status || '') && isMoving(n.band);
    var cx = p.x + 11, cy = p.y + 13, fill = open ? 'none' : lc, stroke = open ? 'var(--s-stop)' : lc;
    var sh = isMoving(n.band) ? '<circle cx="' + cx + '" cy="' + cy + '" r="5" fill="' + fill + '" stroke="' + stroke + '" stroke-width="2"/>'
      : '<rect x="' + (cx - 5) + '" y="' + (cy - 5) + '" width="10" height="10" rx="1" fill="' + fill + '" stroke="' + stroke + '" stroke-width="2"/>';
    var edge = open ? 'var(--s-stop)' : self ? 'var(--ink)' : 'var(--line-2)', ew = self ? 3 : open ? 2 : 1;
    return '<g class="node nl" data-key="' + esc(n.key) + '" data-href="' + esc(href(n.key)) + '" tabindex="0" role="link" aria-label="' + esc(n.id + ' ' + kindLabel(n.kind) + ' ' + n.title) + '">' +
      '<rect class="hit" x="' + p.x + '" y="' + p.y + '" width="' + p.w + '" height="' + p.h + '" rx="4" fill="var(--panel)" stroke="' + edge + '" stroke-width="' + ew + '"/>' + sh +
      '<text class="sid" x="' + (p.x + 21) + '" y="' + (p.y + 16.5) + '" font-size="10.5" font-weight="700" font-family="ui-monospace,SFMono-Regular,Menlo,monospace" fill="' + (closed ? 'var(--ink-3)' : lc) + '" textLength="' + Math.min(idMax || p.w - 26, Array.from(n.id).length * 6.4) + '" lengthAdjust="spacingAndGlyphs">' + esc(n.id) + '</text>' +
      '<text x="' + (p.x + 8) + '" y="' + (p.y + 32) + '" font-size="12" fill="' + (closed ? 'var(--ink-3)' : 'var(--ink)') + '"' + (self ? ' font-weight="700"' : '') + ' data-t>' + esc(cut(n.title, chars)) + '</text>' + (extra || '') + '</g>';
  }
  function chainAround(r) { // 390 幅: 根拠の側と影響の側の一覧（入れ子 2 段まで・2 段目より先は 2 段目に並べる）
    var kids = {}; r.rows.forEach(function (x) { if (x.via && x.col !== 0) (kids[x.via] = kids[x.via] || []).push(x); });
    function desc(k) { var out = []; (kids[k] || []).forEach(function (y) { out.push(y); out = out.concat(desc(y.key)); }); return out; }
    function side(sgn, lab) {
      var top = (kids[r.origin] || []).filter(function (x) { return sgn * x.col > 0; });
      if (!top.length) return '';
      return '<div class="subh">' + esc(L(lab)) + '</div><ul class="items">' + top.map(function (x) {
        var sub = desc(x.key);
        return itemNode(x.key, '<code>' + esc(x.type) + '</code>', { sub: sub.length ? '</li><li class="nest"><ul class="items">' + sub.map(function (y) { return itemNode(y.key, '<code>' + esc(y.type) + '</code>'); }).join('') + '</ul>' : '' });
      }).join('') + '</ul>';
    }
    return side(-1, 'nb_up') + side(1, 'nb_down');
  }
  /* 段数（?k=1|2|3）と畳み（?fold=up|down|both）は URL に残す */
  function nbK() { var k = Number(param('k')); return k >= 1 && k <= 3 ? k : 2; }
  function nbFold() { var f = param('fold'); return f === 'up' || f === 'down' || f === 'both' ? f : ''; }
  function aroundBlock(key) {
    var k = nbK(), fold = nbFold(), fu = fold === 'up' || fold === 'both', fd = fold === 'down' || fold === 'both';
    var r = around(G, key, { up: fu ? 0 : k, down: fd ? 0 : k });
    var full = around(G, key, { up: k, down: k });
    var nUp = full.rows.filter(function (x) { return x.col < 0; }).length, nDown = full.rows.filter(function (x) { return x.col > 0; }).length;
    var c = r.cut.hub + r.cut.cap;
    var bands = BANDS.filter(function (b) { return r.rows.some(function (x) { var n = G.byKey[x.key]; return n && n.band === b; }); });
    var types = []; r.rows.forEach(function (x) { if (x.type && types.indexOf(x.type) < 0) types.push(x.type); });
    var sideBtn = function (sd, folded, n) { return '<button type="button" class="btn sm nbside" data-nbfold="' + sd + '" aria-expanded="' + !folded + '" data-term="nb_' + sd + '">' + (folded ? '▸ ' : '▾ ') + esc(L('nb_' + sd)) + ' <span class="num">' + n + '</span></button>'; };
    var ctl = '<div class="nbctl"><span class="sides">' + sideBtn('up', fu, nUp) + '<span class="arr" aria-hidden="true">←</span><b class="self">' + esc(L('nb_self')) + '</b><span class="arr" aria-hidden="true">→</span>' + sideBtn('down', fd, nDown) + '</span>' +
      '<span class="kseg">' + HS('nb_k') + '<span class="seg" role="group" aria-label="' + esc(L('nb_k')) + '">' + [1, 2, 3].map(function (n) { return '<button type="button" data-nbk="' + n + '" aria-pressed="' + (n === k) + '">' + n + '</button>'; }).join('') + '</span></span></div>';
    return '<div class="nb-wrap" data-origin="' + esc(key) + '">' + ctl + legendHTML(bands, types, { hover: true }) + '<div class="nb-graph">' + svgAround(r) + '</div><div class="pinbar" aria-live="polite"></div><div class="nb-chain">' + chainAround(r) + '</div>' +
      '<div class="cutline num" tabindex="0" data-term="cut" data-tip-expert="shown=' + r.shown + ' total=' + r.total + ' up=' + r.up + ' down=' + r.down + ' cut.hub=' + r.cut.hub + ' cut.cap=' + r.cut.cap + '">' + r.shown + ' / ' + r.total + (c ? ' ・ ✂ ' + c : '') + '</div></div>';
  }
  function nbRerender() { $$('.nb-wrap[data-origin]').forEach(function (w) { var d = document.createElement('div'); d.innerHTML = aroundBlock(w.getAttribute('data-origin')); w.replaceWith(d.firstChild); }); setTimeout(audit, 50); }

  /* ---------- hover 強調（近傍図と地図 (c) で共通）: 節点を中心に根拠の側と影響の側を各 2 段光らせ、他を薄くする
   * hub（次数 > 8）は光らせるが先へは広げない・光る節点が 20 を超えたら段を 1 に落とす・click で固定・再 click / Esc で解除 */
  var HL_HUB = 8, HL_MAX = 20;
  function hlCompute(svg, origin) {
    var E = $$('g.e', svg).map(function (e) { return { el: e, u: e.getAttribute('data-u'), d: e.getAttribute('data-d') }; });
    function run(D) {
      var dir = {}, ord = []; dir[origin] = 'self';
      [['up', 'd', 'u'], ['down', 'u', 'd']].forEach(function (c) {
        var fr = [origin];
        for (var h = 1; h <= D; h++) {
          var nx = [];
          fr.forEach(function (k) {
            if (k !== origin && G && G.deg(k) > HL_HUB) return;
            E.forEach(function (e) { if (e[c[1]] === k) { var v = e[c[2]]; if (!dir[v]) { dir[v] = c[0]; ord.push(v); nx.push(v); } } });
          });
          fr = nx;
        }
      });
      return { dir: dir, ord: ord };
    }
    var D = 2, r = run(2);
    if (r.ord.length > HL_MAX) { D = 1; r = run(1); }
    if (r.ord.length > HL_MAX) { r.ord.slice(HL_MAX).forEach(function (k) { delete r.dir[k]; }); r.ord = r.ord.slice(0, HL_MAX); }
    var litE = E.filter(function (e) { var du = r.dir[e.u], dd = r.dir[e.d]; return (du === 'up' && (dd === 'up' || dd === 'self')) || (dd === 'down' && (du === 'down' || du === 'self')); });
    return { origin: origin, depth: D, dir: r.dir, ord: r.ord, edges: litE, up: r.ord.filter(function (k) { return r.dir[k] === 'up'; }).length, down: r.ord.filter(function (k) { return r.dir[k] === 'down'; }).length };
  }
  function hlApply(svg, origin) {
    var h = hlCompute(svg, origin);
    svg.classList.add('hl-on');
    $$('g.node', svg).forEach(function (n) { var d = h.dir[n.getAttribute('data-key')]; n.classList.toggle('lit', !!d); n.classList.toggle('lit-up', d === 'up'); n.classList.toggle('lit-down', d === 'down'); n.classList.toggle('lit-self', d === 'self'); });
    var le = h.edges.map(function (x) { return x.el; });
    $$('g.e', svg).forEach(function (e) { e.classList.toggle('lit', le.indexOf(e) >= 0); });
    root.__tz_hl = { origin: origin, depth: h.depth, up: h.up, down: h.down, lit: h.ord.length, litNodes: $$('g.node.lit', svg).length - 1, litEdges: $$('g.e.lit', svg).length };
    return h;
  }
  function hlClear(svg) { svg.classList.remove('hl-on'); $$('g.node.lit, g.e.lit', svg).forEach(function (x) { x.classList.remove('lit', 'lit-up', 'lit-down', 'lit-self'); }); }
  function pinbarOf(svg) { var w = svg.closest('.nb-wrap, .gpanel'); return w ? w.querySelector('.pinbar') : null; }
  function hlPin(svg, key) {
    svg.__pin = key; var h = hlApply(svg, key), n = G && G.byKey[key], pb = pinbarOf(svg);
    if (pb) pb.innerHTML = '<span class="pinned">' + esc(L('pinned')) + ' <b class="mono">' + esc(n ? n.id : key) + '</b> · ' + esc(L('nb_up')) + ' ' + h.up + ' · ' + esc(L('nb_down')) + ' ' + h.down + ' · ' + h.depth + '</span>' +
      '<a class="btn sm" href="' + esc(href(key)) + '">' + esc(L('open_node')) + ' ›</a><button type="button" class="btn sm" data-unpin="1">' + esc(L('unpin')) + '</button>';
  }
  function hlUnpin(svg) { svg.__pin = null; hlClear(svg); var pb = pinbarOf(svg); if (pb) pb.innerHTML = ''; }
  function bindHL() {
    function nodeOf(t) { var n = t && t.closest ? t.closest('svg.hlsvg g.node') : null; return n; }
    document.addEventListener('mouseover', function (e) { var n = nodeOf(e.target); if (!n) return; var svg = n.ownerSVGElement || n.closest('svg'); if (!svg.__pin) hlApply(svg, n.getAttribute('data-key')); });
    document.addEventListener('mouseout', function (e) { var n = nodeOf(e.target); if (!n) return; var svg = n.closest('svg'); if (svg.__pin) return; var to = nodeOf(e.relatedTarget); if (!to || to.closest('svg') !== svg) hlClear(svg); });
    document.addEventListener('focusin', function (e) { var n = nodeOf(e.target); if (n) { var svg = n.closest('svg'); if (!svg.__pin) hlApply(svg, n.getAttribute('data-key')); } });
    document.addEventListener('click', function (e) {
      var ub = e.target.closest && e.target.closest('[data-unpin]');
      if (ub) { var w = ub.closest('.nb-wrap, .gpanel'), sv = w && w.querySelector('svg.hlsvg'); if (sv) hlUnpin(sv); return; }
      var kb = e.target.closest && e.target.closest('[data-nbk]');
      if (kb) { var u = new URL(location.href); if (kb.getAttribute('data-nbk') === '2') u.searchParams.delete('k'); else u.searchParams.set('k', kb.getAttribute('data-nbk')); history.replaceState(null, '', u); nbRerender(); return; }
      var fb = e.target.closest && e.target.closest('[data-nbfold]');
      if (fb) {
        var f = nbFold(), sd = fb.getAttribute('data-nbfold'), up = f === 'up' || f === 'both', dn = f === 'down' || f === 'both';
        if (sd === 'up') up = !up; else dn = !dn;
        var nf = up && dn ? 'both' : up ? 'up' : dn ? 'down' : '', u2 = new URL(location.href);
        if (nf) u2.searchParams.set('fold', nf); else u2.searchParams.delete('fold'); history.replaceState(null, '', u2); nbRerender(); return;
      }
      var n = nodeOf(e.target);
      if (n) { var svg = n.closest('svg'); if (svg.__dragged) return; var k = n.getAttribute('data-key'); if (svg.__pin === k) hlUnpin(svg); else hlPin(svg, k); return; }
      var nd = e.target.closest && e.target.closest('g.node[data-href]'); if (nd) location.href = nd.getAttribute('data-href');
    });
    document.addEventListener('dblclick', function (e) { var n = nodeOf(e.target); if (n) location.href = n.getAttribute('data-href'); });
    document.addEventListener('keydown', function (e) { if (e.key === 'Escape') $$('svg.hlsvg').forEach(function (sv) { if (sv.__pin) hlUnpin(sv); }); });
  }

  /* ---------- 抜けの検査（tsuzuri だけ） ---------- */
  var GAP_TEXT = {
    G1: '線の両端が実在する', G2: 'open の task は design-note の行を 1 つ指す', G3: '発効した ADR と rule に、あなたの決定が結ばれている',
    G4: 'question は関わる所を 1 つ以上持つ', G5: '迷子の項目がない（全部が epic から辿れる）', G6: 'blocks の線が輪になっていない',
    G7: '宙に浮いたあなたの決定がない', G8: '台帳の種類の約束を守っている', G9: '題で名指した項目に線が引かれている',
    G10: '番号の形が重ならない', G11: 'run はどれも 1 つの task に属する', G12: '止まった run の question が記録にある'
  };
  function gaps(g, repo, opt) { // opt.sample = mock の見本も数える（未反映の一覧だけが使う・抜けの検査の頁と次の一手 (f) は実の data だけ）
    var withS = !!(opt && opt.sample);
    // 抜けの検査は実の data だけを見る（mock の見本 = 台帳・要件書に無いものは数えない）
    var N = g.data.nodes.filter(function (n) { return n.repo === repo && (withS || !isSample(n)); });
    var E = g.data.edges.filter(function (e) { var a = g.byKey[e.from]; return (a ? a.repo : e.from.split(':')[0]) === repo && (withS || (!isSample(a) && !isSample(g.byKey[e.to]))); });
    var beads = N.filter(function (n) { return n.attrs && (n.attrs.src === 'beads' || (n.attrs.src === 'mock' && n.attrs.issue_type)); });
    var R = [];
    function res(id, st, detail, keys, texts) { R.push({ id: id, name: GAP_TEXT[id], status: st, detail: detail, keys: keys || [], texts: texts || [], n: (keys || []).length + (texts || []).length }); }
    var dangling = E.filter(function (e) { return !g.byKey[e.from] || !g.byKey[e.to]; });
    res('G1', dangling.length ? 'fail' : 'pass', '線 ' + E.length + ' 本を数えた', [], dangling.map(function (e) { return e.from + ' → ' + e.to; }));
    var openC = beads.filter(function (b) { return b.kind === '契約' && b.attrs.status === 'open'; });
    var g2 = openC.filter(function (b) { return (g.out[b.key] || []).filter(function (e) { return e.type === 'design' && g.byKey[e.to]; }).length !== 1; });
    res('G2', g2.length ? 'fail' : 'pass', 'open の task ' + openC.length + ' 件', g2.map(function (b) { return b.key; }));
    var eff = N.filter(function (n) { return n.kind === '判断の記録' || n.kind === '規則行'; });
    var ruled = eff.filter(function (n) { return (g.out[n.key] || []).some(function (e) { return e.type === 'ruled_by'; }); });
    res('G3', eff.length ? 'unknown' : 'pass', '対象 ' + eff.length + ' 件・結ばれている ' + ruled.length + ' 件。ruled_by の欄がまだ形に無く、判定できない', eff.filter(function (n) { return ruled.indexOf(n) < 0; }).map(function (n) { return n.key; }));
    var qs = N.filter(function (n) { return n.kind === '問い'; });
    var noTouch = qs.filter(function (q) { return !(g.out[q.key] || []).some(function (e) { return e.type === 'touches'; }); });
    res('G4', noTouch.length ? 'fail' : 'pass', 'question ' + qs.length + ' 件', noTouch.map(function (q) { return q.key; }));
    var roots = beads.filter(function (b) { return !b.attrs.parent && b.kind === 'epic'; }), rootKey = roots.length ? roots[0].key : null;
    var orphan = beads.filter(function (b) { var k = b.key, guard = 0; while (guard++ < 50) { var p = (g.out[k] || []).filter(function (e) { return e.type === 'parent-child'; })[0]; if (!p) return k !== rootKey; if (!g.byKey[p.to]) return true; k = p.to; } return true; });
    res('G5', orphan.length ? 'fail' : 'pass', '項目 ' + beads.length + ' 件', orphan.map(function (b) { return b.key; }));
    var bl = E.filter(function (e) { return e.type === 'blocks'; }), adj = {}; bl.forEach(function (e) { (adj[e.from] = adj[e.from] || []).push(e.to); });
    var color = {}, cyc = [];
    function dfs(u, path) { color[u] = 1; path.push(u); (adj[u] || []).forEach(function (v) { if (color[v] === 1) cyc.push(path.slice(path.indexOf(v)).concat(v)); else if (!color[v]) dfs(v, path); }); path.pop(); color[u] = 2; }
    Object.keys(adj).forEach(function (u) { if (!color[u]) dfs(u, []); });
    res('G6', cyc.length ? 'fail' : 'pass', 'blocks ' + bl.length + ' 本', [], cyc.map(function (c) { return c.join(' → '); }));
    var rul = N.filter(function (n) { return n.kind === '裁定'; });
    var floating = rul.filter(function (r) { return !(g.out[r.key] || []).some(function (e) { return e.type === 'answers' && g.byKey[e.to]; }); });
    res('G7', floating.length ? 'fail' : 'pass', 'あなたの決定 ' + rul.length + ' 件（見本）', floating.map(function (r) { return r.key; }));
    var viol = beads.filter(function (b) { return !b.attrs.quadrant; });
    res('G8', viol.length ? 'fail' : 'pass', '項目 ' + beads.length + ' 件', viol.map(function (b) { return b.key; }));
    var RX = /\b(?:ADR-\d+|[PNADR]-\d+(?:\.\d+)?|t3-hub(?:\.\d+)*)\b/g, g9 = [];
    beads.forEach(function (b) { (b.title.match(RX) || []).filter(function (x) { return x !== b.id; }).forEach(function (x) {
      var k = repo + ':' + x; if (!(g.inc[b.key] || []).some(function (e) { return e.from === k || e.to === k; })) g9.push(b.key); }); });
    res('G9', g9.length ? 'fail' : 'unknown', '題だけで数えた。本文は持ち込まないので、本文の側は分からない', g9);
    var fam = function (id) { var f = 0; if (/^[A-Z]/.test(id) && !/[@#]/.test(id)) f++; if (/^[a-z][a-z0-9]*-[0-9a-z]+(\.\d+)*$/.test(id)) f++; if (/#/.test(id) && !/@/.test(id)) f++; if (/@/.test(id) || /^[^\x00-\x7f]/.test(id)) f++; return f; };
    var amb = N.filter(function (n) { return fam(n.id) !== 1; }), seenId = {}, dup = [];
    N.forEach(function (n) { if (seenId[n.id]) dup.push(n.key); seenId[n.id] = 1; });
    res('G10', (amb.length || dup.length) ? 'fail' : 'pass', '番号 ' + N.length + ' 個', amb.map(function (n) { return n.key; }).concat(dup));
    var runs = N.filter(function (n) { return n.kind === '走行'; });
    if (!runs.length) { res('G11', 'pass', 'tsuzuri の run は 0 件（state dir の fleet に RunCreated が無い）', []); res('G12', 'pass', 'tsuzuri の run は 0 件', []); }
    else {
      var g11 = runs.filter(function (r) { var ro = (g.out[r.key] || []).filter(function (e) { return e.type === 'run_of'; }); return ro.length !== 1 || !g.byKey[ro[0].to]; });
      res('G11', g11.length ? 'fail' : 'pass', 'run ' + runs.length + ' 件', g11.map(function (r) { return r.key; }));
      res('G12', 'pass', '', []);
    }
    return R;
  }
  function matrix(g, repo) {
    var ks = {}, M = {}, T = {};
    g.data.edges.forEach(function (e) { var a = g.byKey[e.from], b = g.byKey[e.to]; if (!a || !b || a.repo !== repo) return; ks[a.kind] = ks[b.kind] = 1; var k = a.kind + '|' + b.kind; M[k] = (M[k] || 0) + 1; (T[k] = T[k] || {})[e.type] = 1; });
    var order = Object.keys(ks).sort(function (x, y) { var d = g.data.kind_band; return BANDS.indexOf(d[x]) - BANDS.indexOf(d[y]) || x.localeCompare(y, 'ja'); });
    return { kinds: order, M: M, T: T };
  }

  /* account board の 3 tab（直し 7）: URL の ?tab= に残す（既定 home）・?at= ?sort= ?span= ?mode= は tab をまたいで保つ */
  var ACCT_TABS = ['home', 'session', 'projects'], ACCT_TAB_IC = { home: 'home', session: 'clock', projects: 'folder' };
  function acctTabHref(k, other) { // other = 別の頁（relation.html）から index.html へ
    var u = new URL(location.href); if (other) u = new URL('index.html' + u.search, location.href);
    u.searchParams.set('tab', k); return u.pathname.split('/').pop() + u.search;
  }
  /* ---------- 上端の帯 ---------- */
  function topHTML(active) {
    if (PAGE_KIND === 'account') {
      // account board の tab 列（直し 7）: HOME / session / 各 project は同じ頁の ?tab=・関係と飛び方は別の頁。badge は頁が組んだ後に入れる（#tb-<tab>）
      var cur = active === 'acct_board' ? (ACCT_TABS.indexOf(param('tab')) >= 0 ? param('tab') : 'home') : null;
      var A = ACCT_TABS.map(function (k) { return ['tab_' + k, acctTabHref(k, active !== 'acct_board'), IC[ACCT_TAB_IC[k]], k]; }).concat([['relation', 'relation.html' + (param('mode') ? '?mode=' + MODE : ''), IC.link]]);
      return '<header class="top vessel"><a class="brand" href="index.html' + (param('mode') ? '?mode=' + MODE : '') + '" aria-label="account board">' + IC.vessel + '<span class="name">器</span></a>' +
        '<nav class="nav" aria-label="頁">' + A.map(function (p) {
          var on = p[3] ? p[3] === cur : active === p[0];
          return '<a href="' + esc(p[1]) + '"' + (on ? ' class="on" aria-current="page"' : '') + (p[3] ? ' data-tab="' + p[3] + '"' : '') + ' data-v="' + p[0] + '" data-term="' + p[0] + '">' + p[2] + '<span class="lbl hd-t">' + esc(L(p[0])) + '</span>' +
            (p[3] ? '<span class="badge num" id="tb-' + p[3] + '" hidden></span>' : '') + '</a>';
        }).join('') + '</nav><span class="grow"></span>' + modeSeg() + '</header>';
    }
    var nq = openQuestions(G).length;
    var P = [['home', 'index.html', IC.home], ['questions', 'ask.html', IC.ask, nq], ['map', 'map.html', IC.map], ['gaps', 'gaps.html', IC.gaps]];
    // 戻る = この窓を閉じて account board の窓へ（直し 7・案 2 に統一）: ① 開いていれば前面へ ② 無ければ新しく開く ③ 自分の窓を閉じる
    return '<header class="top"><span class="backwrap"><button type="button" class="upto" id="acctback" aria-label="' + esc(L('acct_back')) + '">' + IC.up + '<span class="lbl">' + esc(L('acct_back')) + '</span></button>' + qmark('acct_back') + '</span>' +
      '<a class="brand" href="index.html' + (param('mode') ? '?mode=' + MODE : '') + '" aria-label="tsuzuri">' + IC.logo + '<span class="name" data-t>tsuzuri</span></a>' +
      '<nav class="nav" aria-label="頁">' + P.map(function (p) {
        return '<a href="' + p[1] + (param('mode') ? '?mode=' + MODE : '') + '"' + (active === p[0] ? ' class="on" aria-current="page"' : '') + ' data-v="' + p[0] + '" data-term="' + p[0] + '">' + p[2] +
          '<span class="lbl hd-t">' + esc(L(p[0])) + '</span>' + (p[3] ? '<span class="badge num" aria-label="' + p[3] + '">' + p[3] + '</span>' : '') + '</a>';
      }).join('') + '</nav><span class="grow"></span>' +
      '<span class="seatpill" id="seatpill" tabindex="0" data-seat="1"></span>' + modeSeg() + '</header>';
  }
  /* project board の「account board へ戻る（この窓を閉じる）」: ① 名前つきの窓 tz-account が開いていれば前面へ（window.open('', 'tz-account') の focus）
   * ② 無ければ新しく開く（空の窓が返るので account board へ移す）③ 自分の窓を閉じる（window.close は script が開いた窓でしか効かない → 閉じられない周は 1 行を出す） */
  function backToBoard() {
    var url = new URL('account/index.html' + (param('mode') ? '?mode=' + MODE : ''), location.href).href, w = null, how = 'blocked';
    try { w = window.open('', 'tz-account'); } catch (e) { w = null; }
    if (w) {
      var blank = true; try { blank = w.location.href === 'about:blank'; } catch (e) { blank = false; }
      if (blank) { w.location.href = url; how = 'opened'; } else how = 'front';
      try { w.focus(); } catch (e) { /* noop */ }
    }
    root.__tz_back = { how: how, at: Date.now() };
    try { var o = JSON.parse(localStorage.getItem('tz-wins') || '{}'), me = String(window.name || '').replace(/^tz-/, ''); if (o[me]) { o[me].closed = Date.now(); localStorage.setItem('tz-wins', JSON.stringify(o)); } } catch (e) { /* noop */ }
    window.close();
    setTimeout(function () {
      if (window.closed) return;
      var n = $('#winnote'); if (!n) { n = document.createElement('div'); n.id = 'winnote'; n.className = 'winnote'; n.setAttribute('role', 'status'); var h = $('header.top'); h.parentNode.insertBefore(n, h.nextSibling); }
      n.innerHTML = '<b>' + esc(L('win_close_hand')) + '</b><span class="small muted">' + esc(how === 'front' ? 'account board の窓は前面に出した' : how === 'opened' ? 'account board を新しい窓で開いた' : 'account board の窓を開けなかった（popup の許可が要る）') + '・この窓は script が開いた窓でないので閉じられない</span>';
      root.__tz_back.note = n.textContent;
    }, 300);
  }
  function modeSeg() { return '<div class="seg" role="group" aria-label="' + esc(L('mode')) + '" data-term="mode"><button type="button" data-mode="beginner" aria-pressed="false">' + esc(L('beginner')) + '</button><button type="button" data-mode="expert" aria-pressed="false">' + esc(L('expert')) + '</button></div>'; }
  function seatPillHTML() {
    var t = seatNowT(), s = seatStatus(SEAT_FX, t);
    if (FEED_ON === false && Date.now() - FEED_LAST > 15000) s = { v: 'unknown', why: '更新が 15 秒届いていない' };
    var eta = s.v === 'limit' ? '<span class="eta num">' + hmd(s.resume, t) + '</span>' : s.v === 'silent' ? '<span class="eta num" style="color:var(--st-silent)">' + durMs(t - s.last) + '</span>' : '';
    return stIcon(s.v, false, 'feed') + '<span class="who">' + esc(L('seat')) + '</span>' + eta;
  }

  /* ---------- hover card（全部の節点と札・出所の file と行を必ず）・tooltip（語の説明）: どちらも 4 行 + 「詳しく ▸」 ---------- */
  var hc, tip, hbk, tbk, hcTimer, tipTimer, hcFor = null, tipFor = null, hideT = {};
  var WSH = { five_hour: '5h', seven_day: '7d', seven_day_model: 'model' };
  function cardContent(el) {
    if (el.getAttribute('data-seat')) { var t9 = seatNowT(); return seatCard(SEAT_FX, L('seat') + ' · t3', 'seat t3:orchestrator', null, { ex: SEAT_FX && SEAT_FX.target ? doctorSeatLines(SEAT_FX, { target: SEAT_FX.target, model: SEAT_FX.model }, seatStatus(SEAT_FX, t9).account, t9) : [] }); }
    if (el.getAttribute('data-card') && root.__tz_card) return root.__tz_card(el);
    var key = el.getAttribute('data-key'), runKey = el.getAttribute('data-run');
    var n = G && G.byKey[key];
    if (!n) return null;
    var st = nodeState(n), sm = summaryOf(n), one = sm.plain || sm.eng, v, more = [];
    var gi = '<span class="gi ' + st.cls.replace('open', 'unknown') + ' sm" aria-hidden="true">' + ({ ok: '✓', ng: '!' }[st.cls] || '') + '</span>';
    if (runKey && G.byKey[runKey]) {
      var run = G.byKey[runKey], rs = runStateAt(run, G.meta.surface_now.length === 17 ? G.meta.surface_now.replace('Z', ':00Z') : G.meta.surface_now) || lastStage(run);
      var why = reasonShort(rs, run), runs = runsOf(G, key);
      v = '<span class="icsm">' + IC.redo + '</span>' + runs.length + ' · ' + esc(rs.stage) + ' · ' + esc(cut(why.full, 20));
      if (Array.from(why.full || '').length > 20) chunk(why.full, 34).forEach(function (x) { more.push(esc(x)); });
    } else {
      var room = 34 - Array.from(n.id).length - 1;
      v = '<span class="mono">' + esc(cut(n.id, 34)) + '</span>' + (one ? (room >= 6 ? ' <span class="dq">' + esc(cut(one, room)) + '</span>' : '') : ' <span class="muted">要約なし</span>');
    }
    if (one && Array.from(one).length > 20) chunk(one, 34).forEach(function (x) { more.push('<span class="dq">' + esc(x) + '</span>'); });
    if (n.repo === 's2') more.push('<span class="chip ext">' + esc(L('external_ledger')) + '</span>');
    var srcN = runKey && G.byKey[runKey] ? G.byKey[runKey] : n, sfull = srcText(srcN), s0 = srcN.src || {};
    return card4({ ti: '<span class="dq">' + esc(cut(n.title, 34)) + '</span>', k: nodeShape(n) + '<span>' + esc(kindLabel(n.kind)) + '</span><span class="bname ' + bandCls(n.band) + '">' + esc(n.band) + '</span><span class="sp"></span>' + gi + '<span>' + esc(st.label) + '</span>',
      v: v, src: cut(baseName(s0.file || '出所なし') + (s0.line ? ':' + s0.line : ''), 34), srcFull: sfull, more: more, ex: MODE === 'expert' ? [(vt('k:' + n.kind).internal || n.kind).split('\n')[0]].concat(sm.src ? chunk(sm.src, 58) : []) : null });
  }
  function seatCard(fx, who, internal, srcOv, opt) {
    opt = opt || {};
    var t = seatNowT(), s = seatStatus(fx, t), v, more = [];
    var th = tickHealth(fx, t), hb = fx && fx.target ? hbState(fx) : { v: '?' };
    if (s.v === 'limit') v = esc(s.account) + ' ' + esc(WSH[s.window] || s.window) + ' ' + s.used + '% · ↻ ' + esc(hmd(s.resume, t));
    else if (s.v === 'silent') v = 'tick ◷ ' + esc(ageTxt(t - s.last)) + ' 前 · 健全 ≤ ' + (2 * TICK_RULES.interval) + 's';
    else if (s.v === 'run' || s.v === 'wait') v = '口座 <span class="mono">' + esc(s.account || '?') + '</span>' + (s.long ? ' · 長い ◷ ' + esc(durMs(t - s.since)) : '');
    else v = esc(cut(s.why || L('st_unknown'), 30));
    var tk = '<span class="tkhb smpl"><span class="tk tk-' + th.v + '" title="tick ' + esc(th.v) + '"><span class="en">tick</span>' + (th.v === 'past' ? '<b>―</b>' : symHTML('gi:' + (th.v === 'healthy' ? 'ok' : th.v === 'absent' ? 'unknown' : 'ng')) + '<b>' + esc(th.v) + '</b>') + '</span><span class="hb hb-' + esc(hb.v) + '"><span class="en">hb</span><b>' + esc(hb.v) + '</b></span></span>';
    if (s.v === 'limit') more.unshift(tk); else v += ' ' + tk;
    if (s.v === 'limit') more.push(esc(hm(s.since)) + ' から止まっている');
    if (th.v !== 'past') more.push('tick ' + esc(th.v) + (th.age != null ? ' ' + esc(ageTxt(th.age)) + ' 前' : '') + ' · 健全 ≤ ' + (2 * TICK_RULES.interval) + 's');
    if (th.reason) more.push('tick-last · reason=' + esc(th.reason)); // reason の語（busy / stamp-recent / heartbeat-off …）は hover だけ
    if (th.decision) more.push('tick-last · decision=' + esc(th.decision));
    if (th.v === 'past') more.push('― = 記録より前の時点は tick を判じない');
    if (hb.mock) more.push('heartbeat は mock の切り替え');
    (opt.more || []).forEach(function (x) { more.push(x); });
    var sfull = (srcOv || ('~/.local/state/scribe2-v2-state-tsuzuri/seat/' + String(fx && fx.target || '').replace(':', '_') + '/state.jsonl・fleet/events.jsonl')) + (fx && fx.tickSrc ? '・' + fx.tickSrc + '・' + (fx.hbSrc || 'heartbeat-off') : '');
    var ex = MODE === 'expert' ? [internal, s.v === 'limit' ? 'used_pct=' + s.used + ' window=' + s.window + (s.model ? ' model=' + s.model : '') : 'state.jsonl + 器の生存'].concat(opt.ex || []).concat(s.v === 'limit' ? ['resets_at=' + new Date(s.resume).toISOString() + (s.move ? ' · GroupMovePending' : '')] : []) : null;
    return card4({ ti: esc(who), k: stIcon(s.v) + '<span>' + esc(L(ST_V[s.v])) + '</span>' + (s.since ? '<span class="sp"></span><span class="num">◷ ' + esc(hm(s.since)) + ' から</span>' : ''), v: v,
      src: cut('seat/' + (fx && fx.target || '?') + '/state.jsonl ほか', 34), srcFull: sfull, more: more, ex: ex });
  }
  /* hover card と「?」の置き場（持ち主 06:4xZ）: pointer の右 16 px・上下は pointer を card の 1 行目に揃える（くちばしが pointer を指す）。
   * 右に入らなければ左（pointer の左 16 px）。上下は枠の内に寄せる。要素の下端には出さない。
   * touch（最後の pointerdown が touch）は今のまま = 要素の下（入らなければ上）。keyboard の focus は要素の左上寄りの点を pointer の代わりにする */
  var PX = -1, PY = -1, PTOUCH = false, BEAK = 16, ALIGN = 20;
  function anchorFor(el) {
    if (PTOUCH) return null;
    var r = el.getBoundingClientRect();
    if (PX >= r.left - 2 && PX <= r.right + 2 && PY >= r.top - 2 && PY <= r.bottom + 2) return { x: PX, y: PY, ptr: true };
    return { x: r.left + Math.min(r.width, 40), y: r.top + Math.min(r.height / 2, ALIGN), ptr: false };
  }
  function place(box, el) {
    var bk = box === hc ? hbk : tbk, bw = box.offsetWidth, bh = box.offsetHeight, vw = document.documentElement.clientWidth, vh = window.innerHeight, a = box.__a;
    if (!a) { // touch: 今のまま
      var r = el.getBoundingClientRect(), x0 = Math.min(Math.max(8, r.left), vw - bw - 8), y0 = r.bottom + 8;
      if (y0 + bh > vh - 8 && r.top - bh - 8 > 8) y0 = r.top - bh - 8;
      box.style.left = x0 + 'px'; box.style.top = y0 + 'px'; box.__side = null; bk.className = 'hbeak'; return;
    }
    var side = 'l', x = a.x + BEAK, y;
    if (x + bw > vw - 8) { side = 'r'; x = a.x - BEAK - bw; }
    if (x < 8) { // 左右どちらにも入らない（狭い幅を mouse で見るとき）: pointer の下 16 px（入らなければ上）
      side = a.y + BEAK + bh <= vh - 8 || a.y - BEAK - bh < 8 ? 'b' : 't';
      x = Math.min(Math.max(8, a.x - 24), vw - bw - 8); y = side === 'b' ? a.y + BEAK : a.y - BEAK - bh;
    } else y = a.y - ALIGN;
    y = Math.min(Math.max(8, y), Math.max(8, vh - bh - 8));
    box.style.left = Math.round(x) + 'px'; box.style.top = Math.round(y) + 'px'; box.__side = side;
    // くちばし: card の pointer 側の辺に、pointer の高さ（辺の内に寄せる）で
    var bx = side === 'l' ? x : side === 'r' ? x + bw : Math.min(Math.max(x + 10, a.x), x + bw - 10), by = side === 'l' || side === 'r' ? Math.min(Math.max(y + 10, a.y), y + bh - 10) : side === 'b' ? y : y + bh;
    bk.className = 'hbeak on s-' + side + (box === tip ? ' tipb' : ''); bk.style.left = Math.round(bx - 6) + 'px'; bk.style.top = Math.round(by - 6) + 'px';
  }
  function mark(el, on) { if (el && el.classList) el.classList.toggle('hv-on', on); }
  /* 当たり判定は「card へ向かう動き」のときだけ: 要素を出たら 150 ms の猶予。その間 pointer が
   * 出た点と card の pointer 側の辺の両端を結ぶ三角形（safe triangle）の内に居れば残す・外へ出たら即消す・card に乗れば残る */
  var GRACE_MS = 150, GR = {};
  function inTri(px, py, ax, ay, bx, by, cx, cy) {
    var d1 = (px - bx) * (ay - by) - (ax - bx) * (py - by), d2 = (px - cx) * (by - cy) - (bx - cx) * (py - cy), d3 = (px - ax) * (cy - ay) - (cx - ax) * (py - ay);
    return !((d1 < 0 || d2 < 0 || d3 < 0) && (d1 > 0 || d2 > 0 || d3 > 0));
  }
  function towardBox(w) {
    var g = GR[w], box = w === 'hc' ? hc : tip; if (!g) return false;
    var b = box.getBoundingClientRect(), s = box.__side;
    if (s === 'l') return inTri(PX, PY, g.x, g.y, b.left, b.top, b.left, b.bottom);
    if (s === 'r') return inTri(PX, PY, g.x, g.y, b.right, b.top, b.right, b.bottom);
    if (s === 'b') return inTri(PX, PY, g.x, g.y, b.left, b.top, b.right, b.top);
    if (s === 't') return inTri(PX, PY, g.x, g.y, b.left, b.bottom, b.right, b.bottom);
    return false;
  }
  function grace(w, fn) { // 要素を出た時点で呼ぶ
    var box = w === 'hc' ? hc : tip;
    if (!box.__a || !box.__side) { later(w, fn); return; } // touch / 置き場が旧式のときは今のまま
    clearTimeout(hideT[w]); GR[w] = { x: PX, y: PY, fn: fn };
    hideT[w] = setTimeout(function () { var g = GR[w]; GR[w] = null; if (g && !box.__over) g.fn(true); }, GRACE_MS); // true = 次の要素の card の予約は消さない
  }
  function graceMove() {
    ['hc', 'tip'].forEach(function (w) {
      var g = GR[w], box = w === 'hc' ? hc : tip; if (!g) return;
      if (box.__over) { GR[w] = null; keep(w); return; }
      if (towardBox(w)) return; // card へ向かっている = 残す（150 ms の猶予のうちに乗れば残る）
      GR[w] = null; keep(w); g.fn(true);
    });
  }
  function later(w, fn) { clearTimeout(hideT[w]); hideT[w] = setTimeout(fn, 240); }
  function keep(w) { clearTimeout(hideT[w]); }
  var shownAt = 0; // focus で出した直後の scroll（focus が起こす scroll into view）では消さない
  function showCard(el) { var html = cardContent(el); if (!html) return; mark(hcFor, false); hc.innerHTML = html; hc.classList.add('on'); hcFor = el; mark(el, true); shownAt = Date.now(); hc.__a = anchorFor(el); GR.hc = null; place(hc, el); }
  function hideCard(pend) { if (pend !== true) clearTimeout(hcTimer); keep('hc'); GR.hc = null; hc.classList.remove('on'); mark(hcFor, false); hcFor = null; hc.__over = false; if (hbk) hbk.className = 'hbeak'; }
  function tipContent(el) {
    var k = el.getAttribute('data-term'), ex = el.getAttribute('data-tip-expert'), exp = MODE === 'expert';
    if (!k) return exp && ex ? intHTML(ex) : null;
    var t = vt(k), sp = SPANS[spanKey()];
    var txt = String(t.plain || '').split('{SPAN}').join(spanKey().replace('h', ' 時間')).split('{TICK}').join(sp.tickL);
    if (!txt && !(exp && t.internal)) return null;
    return '<div class="tl"><b>' + esc(t.label) + '</b></div>' + (txt ? noteHTML(txt, exp) : '') + (exp && t.internal ? intHTML(t.internal) : '');
  }
  function showTip(el) { var h = tipContent(el); if (!h) return; mark(tipFor, false); tip.innerHTML = h; tip.classList.add('on'); tipFor = el; mark(el, true); tip.__a = anchorFor(el); GR.tip = null; place(tip, el); }
  function hideTip(pend) { if (pend !== true) clearTimeout(tipTimer); keep('tip'); GR.tip = null; tip.classList.remove('on', 'pin'); mark(tipFor, false); tipFor = null; tip.__over = false; if (tbk) tbk.className = 'hbeak'; }
  function softHideTip(pend) { if (!tip.classList.contains('pin')) hideTip(pend); } // 自動で消す時は留めた説明を残す
  function bindHover() {
    document.body.insertAdjacentHTML('afterbegin', '<svg width="0" height="0" style="position:absolute" aria-hidden="true" focusable="false"><defs><pattern id="tz-hatch" width="6" height="6" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="6" height="6" fill="var(--st-limit)"/><rect width="2" height="6" fill="rgba(255,255,255,.45)"/></pattern></defs></svg>'); // 限度の斜線（strip と見本で共有）
    hc = document.createElement('div'); hc.className = 'hcard'; hc.setAttribute('role', 'tooltip'); hc.id = 'hcard'; document.body.appendChild(hc);
    hbk = document.createElement('div'); hbk.className = 'hbeak'; hbk.setAttribute('aria-hidden', 'true'); document.body.appendChild(hbk); // card のくちばし
    tip = document.createElement('div'); tip.className = 'tip'; tip.setAttribute('role', 'tooltip'); tip.id = 'tip'; document.body.appendChild(tip);
    tbk = document.createElement('div'); tbk.className = 'hbeak'; tbk.setAttribute('aria-hidden', 'true'); document.body.appendChild(tbk);
    document.addEventListener('pointerdown', function (e) { PTOUCH = e.pointerType === 'touch'; }, true);
    document.addEventListener('mousemove', function (e) { if (PTOUCH && e.sourceCapabilities && e.sourceCapabilities.firesTouchEvents) return; PTOUCH = false; PX = e.clientX; PY = e.clientY; if (GR.hc || GR.tip) graceMove(); }, { capture: true, passive: true });
    function cardEl(t) { return t && t.closest && !t.closest('.hcard,.tip') ? t.closest('[data-key],[data-seat],[data-card]') : null; }
    function tipEl(t) { return t && t.closest && !t.closest('.hcard,.tip') ? t.closest('[data-term],[data-tip-expert]') : null; }
    function inBox(t, box) { return !!(t && box && box.contains(t)); }
    // card と tooltip の上に指を移しても消さない（中の「詳しく ▸」を押せる）
    hc.addEventListener('toggle', function () { if (hcFor) place(hc, hcFor); }, true);
    tip.addEventListener('toggle', function () { if (tipFor) place(tip, tipFor); }, true);
    hc.addEventListener('mouseenter', function () { hc.__over = true; GR.hc = null; keep('hc'); });
    hc.addEventListener('mouseleave', function (e) { hc.__over = false; if (!(hcFor && hcFor.contains(e.relatedTarget))) later('hc', hideCard); });
    tip.addEventListener('mouseenter', function () { tip.__over = true; GR.tip = null; keep('tip'); });
    tip.addEventListener('mouseleave', function (e) { tip.__over = false; if (!tip.classList.contains('pin') && !(tipFor && tipFor.contains(e.relatedTarget))) later('tip', softHideTip); });
    document.addEventListener('mouseover', function (e) {
      if (inBox(e.target, hc) || inBox(e.target, tip)) return;
      var c = cardEl(e.target), t0 = tipEl(e.target);
      if (c && t0 && t0 !== c && c.contains(t0)) c = null; // 札の中の語（「?」・見出し）は語の説明が勝つ
      if (c) { if (c === hcFor) { keep('hc'); GR.hc = null; } if (c !== hcFor) { clearTimeout(hcTimer); hcTimer = setTimeout(function () { showCard(c); }, 300); } if (!tip.classList.contains('pin')) hideTip(); return; }
      var t = tipEl(e.target);
      if (t) { if (t === tipFor) { keep('tip'); GR.tip = null; } if (t !== tipFor && !tip.classList.contains('pin')) { clearTimeout(tipTimer); tipTimer = setTimeout(function () { showTip(t); }, 300); } }
    });
    document.addEventListener('mouseout', function (e) {
      var c = cardEl(e.target); if (c && !c.contains(e.relatedTarget)) { clearTimeout(hcTimer); if (c === hcFor && !inBox(e.relatedTarget, hc)) grace('hc', hideCard); }
      var t = tipEl(e.target); if (t && !t.contains(e.relatedTarget)) { clearTimeout(tipTimer); if (t === tipFor && !inBox(e.relatedTarget, tip) && !tip.classList.contains('pin')) grace('tip', softHideTip); }
    });
    // 「?」を押すと説明を留める（もう一度押すか Esc か外を押すと外す）
    document.addEventListener('click', function (e) {
      var q = e.target.closest && e.target.closest('button.q');
      if (q) { e.preventDefault(); if (tip.classList.contains('pin') && tipFor === q) { hideTip(); return; } showTip(q); tip.classList.add('pin'); return; }
      if (tip.classList.contains('pin') && !inBox(e.target, tip)) hideTip();
    });
    document.addEventListener('focusin', function (e) {
      if (inBox(e.target, hc) || inBox(e.target, tip)) return;
      var c = cardEl(e.target); if (c && (c === e.target)) { showCard(c); return; }
      var t = tipEl(e.target); if (t && !tip.classList.contains('pin')) showTip(t);
    });
    document.addEventListener('focusout', function (e) {
      if (inBox(e.relatedTarget, hc) || inBox(e.relatedTarget, tip)) return;
      if (inBox(e.target, hc) || inBox(e.target, tip)) { later('hc', hideCard); later('tip', softHideTip); return; }
      hideCard(); if (!tip.classList.contains('pin')) hideTip();
    });
    document.addEventListener('keydown', function (e) {
      if (e.key === 'Escape') { hideCard(); hideTip(); closeCoach(); }
      if ((e.key === 'Enter' || e.key === ' ') && e.target.classList && e.target.classList.contains('node')) { e.preventDefault(); location.href = e.target.getAttribute('data-href'); }
    });
    window.addEventListener('scroll', function () { if (Date.now() - shownAt < 400) { if (hcFor) { hc.__a = anchorFor(hcFor); place(hc, hcFor); } return; } hideCard(); if (!tip.classList.contains('pin')) hideTip(); }, { passive: true });
  }

  /* ---------- 表示の型（初心者 / 経験者） ---------- */
  function initMode() {
    var p = param('mode');
    if (p === 'beginner' || p === 'expert') { MODE = p; store('tz-mode', p); }
    else MODE = store('tz-mode') === 'expert' ? 'expert' : 'beginner';
    applyMode();
  }
  function applyMode() {
    document.body.classList.toggle('mode-beginner', MODE === 'beginner');
    document.body.classList.toggle('mode-expert', MODE === 'expert');
    $$('.seg button[data-mode]').forEach(function (b) { b.setAttribute('aria-pressed', String(b.getAttribute('data-mode') === MODE)); });
    $$('details.gmore').forEach(function (d) { d.open = MODE === 'expert'; }); // block の「詳しく」は経験者で開く
  }
  function setMode(m) {
    MODE = m; store('tz-mode', m);
    var u = new URL(location.href); u.searchParams.set('mode', m); history.replaceState(null, '', u);
    applyMode(); hideCard(); hideTip();
    if (m === 'expert') closeCoach(); else if (root.__tz_page === 'home') startCoach(true);
  }

  /* ---------- coach mark（初心者 mode・最初の 3 手） ---------- */
  var COACH = [
    { sel: '#next .nxbig', text: 'まずここ。押すと進む。' },
    { sel: '#seatbody .big', text: 'orchestrator の状態は記号で。砂時計は利用枠の限度で停止中。' },
    { sel: '.kcard', text: '札に指を置くと中身、押すと詳しい頁へ。' }
  ];
  var coachI = -1, coachBox = null, coachRing = null;
  function startCoach(force) {
    if (MODE !== 'beginner') return;
    if (!force && store('tz-coach') === 'done' && param('coach') !== '1') return;
    coachI = 0; drawCoach();
  }
  function drawCoach() {
    closeCoach(true);
    var c = COACH[coachI]; if (!c) { store('tz-coach', 'done'); return; }
    var el = $(c.sel); if (!el) { coachI++; return drawCoach(); }
    var r = el.getBoundingClientRect(), sx = window.scrollX, sy = window.scrollY;
    coachRing = document.createElement('div'); coachRing.className = 'coach-ring';
    coachRing.style.cssText = 'left:' + (r.left + sx - 4) + 'px;top:' + (r.top + sy - 4) + 'px;width:' + (r.width + 8) + 'px;height:' + (r.height + 8) + 'px';
    coachBox = document.createElement('div'); coachBox.className = 'coach'; coachBox.setAttribute('role', 'dialog'); coachBox.setAttribute('aria-label', '手引き');
    coachBox.innerHTML = '<div>' + esc(c.text) + '</div><div class="ft"><span class="dots">' + COACH.map(function (_, i) { return '<i' + (i === coachI ? ' class="on"' : '') + '></i>'; }).join('') + '</span>' +
      '<button type="button" data-c="skip">閉じる</button><button type="button" data-c="next">' + (coachI === COACH.length - 1 ? '分かった' : '次へ') + '</button></div>';
    document.body.appendChild(coachRing); document.body.appendChild(coachBox);
    var vw = document.documentElement.clientWidth;
    var x = Math.min(Math.max(8, r.left + sx), sx + vw - coachBox.offsetWidth - 8);
    coachBox.style.left = x + 'px'; coachBox.style.top = (r.bottom + sy + 12) + 'px';
    coachBox.addEventListener('click', function (e) {
      var b = e.target.closest('button'); if (!b) return;
      if (b.getAttribute('data-c') === 'next') { coachI++; drawCoach(); } else { closeCoach(); store('tz-coach', 'done'); }
    });
  }
  function closeCoach(keep) { if (coachBox) coachBox.remove(); if (coachRing) coachRing.remove(); coachBox = coachRing = null; if (!keep) coachI = -1; }

  /* ---------- 実時間の更新（脈・経過・再開までの残り） ---------- */
  function tick() {
    var pill = $('#seatpill'); if (pill && SEAT_FX) { var h = seatPillHTML(); if (pill.__h !== h.replace(/data-until="\d+"/, '')) { pill.innerHTML = h; pill.__h = h.replace(/data-until="\d+"/, ''); } }
    $$('[data-since]').forEach(function (el) { el.textContent = durMs((el.hasAttribute('data-seatclock') ? seatNowT() : now()) - Number(el.getAttribute('data-since'))); });
    $$('[data-remain]').forEach(function (el) { el.textContent = Math.max(0, Math.ceil((Number(el.getAttribute('data-remain')) - seatNowT()) / 1000)) + ' 秒'; }); // 退避までの残り秒
    $$('[data-until]').forEach(function (el) { var d = Number(el.getAttribute('data-until')) - seatNowT(); el.textContent = el.hasAttribute('data-long') ? hms(d) : hm(Number(el.getAttribute('data-until'))); });
    if (root.__tz_onTick) root.__tz_onTick();
  }
  function feed() { // SSE の代わり: 5 秒ごとに更新が届いた体で脈を速める（?feed=off で届かない → 15 秒で測れていないに落ちる）
    if (!FEED_ON) return;
    FEED_LAST = Date.now();
    $$('.st.feed').forEach(function (s) { s.classList.add('beat'); setTimeout(function () { s.classList.remove('beat'); }, 800); });
  }

  /* ---------- 受入条件の自己測定（console と window.__tz_audit） ---------- */
  function allLabels() {
    var s = {}; [V.english, V.rephrase].forEach(function (col) { Object.keys(col).forEach(function (k) { s[col[k].label] = 1; }); });
    return s;
  }
  function checkHeadings() {
    var labels = allLabels(), bad = [], n = 0;
    $$('h1,h2,h3,h4,h5,h6,.hd,.nav a').forEach(function (h) {
      n++;
      var t = h.querySelector('.hd-t') || h; var k = h.getAttribute('data-v');
      var txt = t.textContent.trim();
      if (!k || !labels[txt] || L(k) !== txt) bad.push(txt || '(空)');
    });
    return { headings: n, notInVocab: bad };
  }
  /* hover card を持つべき要素（持ち主 06:5xZ）= (a) 頁を持つ節点の札 (b) 表や帯の圧縮された cell (c) 記号だけの要素。
   * 中身を全部見せている block（群の枠・orchestrator と口座・台帳の 4 数など）は card を持たない（語の「?」と block の「詳しく ▸」だけ） */
  var CARD_SHOULD = {
    a: 'a[href*="bead.html?id="], g.node',
    b: '.jrow:not(.hrow) > .c-p, .prow:not(.hrow) > .c-need, .prow:not(.hrow) > .c-led, .prow:not(.hrow) > .c-run, .prow:not(.hrow) > .c-orch, .prow:not(.hrow) > .c-acc, .c-more .mi > .c-led, .c-more .mi > .c-run, .c-more .mi > .c-orch, .c-more .mi > .c-acc, ' +
      '.acct-grid > .c-name, ul.cands2:not(.smpl) > li, ul.mv > li:has(> .ic.moved), .srow > .c-ses, .lmid, .urow > .u-id, .nxall > .nxrow, .chip.dormant',
    c: '.projs > .pchip, .pchip.grp, .seatpill, .empty.zrun'
  };
  function auditCards() {
    var sel = CARD_SHOULD.a + ', ' + CARD_SHOULD.b + ', ' + CARD_SHOULD.c;
    var vis = function (el) { return !el.closest('.hcard,.tip') && !!el.getClientRects().length; };
    var should = $$(sel).filter(vis), els = $$('[data-key],[data-card]'), missing = [], unfocusable = [], noSrc = [], extra = [];
    should.forEach(function (el) { var h = el.closest('[data-key],[data-card],[data-seat]'); if (!h || !cardContent(h)) missing.push((h || el).getAttribute('data-key') || (h || el).getAttribute('data-card') || el.className); });
    els.forEach(function (el) {
      if (!vis(el)) return;
      var c = cardContent(el);
      if (!c) return;
      if (!/class="[^"]*\bsrc\b/.test(c)) noSrc.push(el.getAttribute('data-key') || el.getAttribute('data-card'));
      if (!(el.tabIndex >= 0 || el.tagName === 'A')) unfocusable.push(el.getAttribute('data-key') || el.getAttribute('data-card'));
      if (!el.matches(sel)) extra.push(el.getAttribute('data-key') || el.getAttribute('data-card'));
    });
    return { withCard: els.length, should: should.length, cards_runs: $$('[data-run]').length, missing: missing, noSrc: noSrc, unfocusable: unfocusable, extra: extra };
  }
  function auditLists() {
    var bad = [];
    $$('ul.items > li').forEach(function (li) {
      var first = li.firstElementChild; if (!first) return;
      if (li.classList.contains('nest')) return;
      if (!first.matches('.shape,.st,.gi,.nb,input,label')) bad.push('印が無い: ' + li.textContent.slice(0, 20));
    });
    var depth3 = $$('ul.items ul.items ul.items').length;
    var nums = {}; $$('ul.items').forEach(function (ul, i) { var ns = $$(':scope > li > .nb, :scope > li > label > .nb', ul).map(function (x) { return Number(x.textContent); }); if (ns.length) nums[i] = ns; });
    Object.keys(nums).forEach(function (k) { nums[k].forEach(function (v, i) { if (v !== i + 1) bad.push('番号が 1 から続かない: ' + nums[k].join(',')); }); });
    $$('.qcard .nb').forEach(function (x, i) { if (Number(x.textContent) !== i + 1) bad.push('質問の番号が 1 から続かない'); });
    return { bad: bad, depth3: depth3 };
  }
  var PROSE_SKIP = 'nav, header.top, button, .btn, .seg, .sleg, .legend5, .hlg, .kid, .cid, .mono, .num, .wl'; // 散文でない: tab の名（上端の帯）・button の語・凡例の語（常時の凡例・記号の見方）・id と数（mono / num・窓の名 wl）
  function countChars(h) { // 最初の画面（高さ h）に見える本文の字数（数字・題〔data-t〕・記号・空白を除く）
    h = h || window.innerHeight; var w = document.documentElement.clientWidth;
    var n = 0, parts = [], np = 0, pparts = [];
    var walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    var node;
    while ((node = walker.nextNode())) {
      var p = node.parentElement; if (!p) continue;
      if (p.closest('[data-t],script,style,.hcard,.tip,.sr,title')) continue;
      var dd = p.closest('details'); if (dd && !dd.open && !p.closest('summary')) continue; // 畳んだ details の中身は見えない（Chrome は rect を返すので明示で外す）
      var cs = getComputedStyle(p); if (cs.display === 'none' || cs.visibility === 'hidden') continue;
      var hidden = false, q = p; while (q && q !== document.body) { var c2 = getComputedStyle(q); if (c2.display === 'none' || Number(c2.opacity) === 0) { hidden = true; break; } q = q.parentElement; }
      if (hidden) continue;
      var rg = document.createRange(); rg.selectNodeContents(node); var rc = rg.getBoundingClientRect();
      if (rc.bottom <= 0 || rc.top >= h || rc.right <= 0 || rc.left >= w || rc.width === 0) continue;
      var s = node.textContent.replace(/\d{1,2}:\d{2}Z/g, '').replace(/[0-9０-９]/g, '').replace(/[\s\p{P}\p{S}]/gu, '');
      if (s) { n += Array.from(s).length; parts.push(s);
        // 散文の字数（持ち主の規則 2026-09-26）: 凡例の語・tab の名・button の語・数字と id は数えない
        if (!p.closest(PROSE_SKIP)) { np += Array.from(s).length; pparts.push(s); } }
    }
    return { chars: n, parts: parts, prose: np, proseParts: pparts };
  }
  function audit() {
    var r = { page: root.__tz_page, mode: MODE, headings: checkHeadings(), cards: auditCards(), lists: auditLists(),
      firstScreen: countChars(800), hscroll: document.documentElement.scrollWidth - document.documentElement.clientWidth,
      coach: !!$('.coach'), qmarksVisible: $$('.q').filter(function (q) { return q.offsetParent; }).length, seatSelfTest: root.__tz_seat_selftest,
      extra: root.__tz_audit_extra ? root.__tz_audit_extra() : null };
    root.__tz_audit = r;
    try { console.log('[tz-audit]', JSON.stringify({ page: r.page, mode: r.mode, headings: r.headings.headings, notInVocab: r.headings.notInVocab, cards: r.cards.withCard, missing: r.cards.missing.length, noSrc: r.cards.noSrc.length, lists: r.lists, chars: r.firstScreen.chars, hscroll: r.hscroll, extra: r.extra })); } catch (e) { /* noop */ }
    return r;
  }

  /* ---------- 起動 ---------- */
  function toast(msg) { var t = $('#toast'); if (!t) { t = document.createElement('div'); t.id = 'toast'; t.className = 'toast'; t.setAttribute('role', 'status'); document.body.appendChild(t); } t.textContent = msg; t.classList.add('on'); setTimeout(function () { t.classList.remove('on'); }, 2400); }
  function getJSON(u) { return fetch(u).then(function (r) { if (!r.ok) throw new Error(u + ' ' + r.status); return r.json(); }); }
  function start(page, render, data) {
    initMode();
    document.body.insertAdjacentHTML('afterbegin', topHTML(page));
    applyMode();
    $$('.seg button[data-mode]').forEach(function (b) { b.addEventListener('click', function () { setMode(b.getAttribute('data-mode')); }); });
    bindHover(); bindHL();
    var bk = $('#acctback'); if (bk) bk.addEventListener('click', backToBoard);
    render(data);
    seatSelfTest();
    tick(); setInterval(tick, 1000); setInterval(feed, 5000);
    if (page === 'home') setTimeout(function () { startCoach(false); }, 200);
    setTimeout(audit, 500);
  }
  function fail(err) { document.body.innerHTML = '<div class="page"><div class="empty">' + stIcon('unknown') + '<span>data を読めない = 測れていない（' + esc(err) + '）</span></div></div>'; }
  function boot(page, render) {
    root.__tz_page = page;
    Promise.all(['vocab.json', 'data/summaries.json', 'data/graph.json', 'data/seat-t3.json'].concat(page === 'home' ? ['account/data/board.json', 'account/data/ledger.json'] : []).map(function (u) { return /ledger/.test(u) ? getJSON(u).catch(function () { return null; }) : getJSON(u); }))
      .then(function (a) {
        V = a[0]; SUM = a[1].summaries; G = Graph(a[2]); SEATD = a[3]; BOARD = a[4] || null; LEDGER = a[5] || null;
        SEAT_BASE = ms(SEATD.built_at);
        if (param('at')) { SEAT_VIEW = ms(isNaN(Number(param('at'))) ? param('at') : Number(param('at'))); SEAT_VIEW_AT = Date.now(); }
        setTickRules(SEATD.rules); SEAT_FX = fxFromSeat(SEATD.seat, SEATD.allowance, SEATD.moves, SEATD.ticks);
        start(page, render, G);
      }).catch(fail);
  }
  /* account board（器の面）: 語彙は ../vocab.json・data は account/data/board.json */
  function bootAccount(page, render) {
    root.__tz_page = page; PAGE_KIND = 'account'; BASE = '../';
    Promise.all([BASE + 'vocab.json', 'data/board.json', getJSON('data/ledger.json').catch(function () { return null; }), getJSON(BASE + 'data/graph.json').catch(function () { return null; })].map(function (x) { return typeof x === 'string' ? getJSON(x) : x; }))
      .then(function (a) {
        V = a[0]; var B = a[1]; LEDGER = a[2] || null; B.ledger = LEDGER; if (a[3]) { G = Graph(a[3]); B.graph = G; } // 未反映の一覧は tsuzuri の graph（G7 と同じ関数）で数える
        setTickRules(B.pressure);
        SEAT_BASE = ms(B.built_at);
        if (param('at')) { SEAT_VIEW = ms(isNaN(Number(param('at'))) ? param('at') : Number(param('at'))); SEAT_VIEW_AT = Date.now(); }
        start(page, render, B);
      }).catch(fail);
  }

  root.TZ = { BEADS_LANES: BEADS_LANES, laneOf: laneOf, laneLabel: laneLabel, LANE0: LANE0, card4: card4, noteHTML: noteHTML, noteParts: noteParts, intHTML: intHTML, figSVG: figSVG, FIGS: FIGS, symHTML: symHTML, inl: inl, baseName: baseName, cardHTML: function (el) { return cardContent(el); }, tipHTML: function (el) { return tipContent(el); }, WSH: WSH, cut: cut, chunk: chunk, tickHealth: tickHealth, tkhbHTML: tkhbHTML, hbState: hbState, hbCmd: hbCmd, TICK_RULES: TICK_RULES, ageTxt: ageTxt, stripSVG: stripSVG, stripSample: stripSample, stripLegend: stripLegend, STRIP_CLS: STRIP_CLS, MARK_CLS: MARK_CLS, spanKey: spanKey, spanMs: spanMs, spanTicks: spanTicks, setSpan: setSpan, spanSeg: spanSeg, spanAxis: spanAxis, backToBoard: backToBoard, ACCT_TABS: ACCT_TABS, acctTabHref: acctTabHref, hlApply: hlApply, hlClear: hlClear, hlPin: hlPin, hlUnpin: hlUnpin, nbK: nbK, BOARD: function () { return BOARD; }, LEDGER: function () { return LEDGER; }, boot: boot, bootAccount: bootAccount, G: function () { return G; }, V: function () { return V; }, SEATD: function () { return SEATD; }, L: L, vt: vt, H: H, HS: HS, EN: EN, qmark: qmark, IC: IC,
    esc: esc, cut: cut, t36: t36, hm: hm, ms: ms, durMs: durMs, hms: hms, now: now, param: param, href: href, around: around, aroundBlock: aroundBlock, legendHTML: legendHTML, legendKey: legendKey, isMoving: isMoving,
    board: board, kcardHTML: kcardHTML, runsOf: runsOf, runStateAt: runStateAt, reasonShort: reasonShort, classify: classify, openQuestions: openQuestions, isSample: isSample, unreflected: unreflected, material: material,
    rulingOf: rulingOf, gaps: gaps, matrix: matrix, shape: shape, nodeShape: nodeShape, nodeBox: nodeBox, bandChip: bandChip, bandCls: bandCls, bandVar: bandVar, edgeSVG: edgeSVG, styleOf: styleOf, upOf: upOf,
    nodeState: nodeState, summaryOf: summaryOf, srcText: srcText, itemNode: itemNode, stIcon: stIcon, stLabel: stLabel, seatStatus: seatStatus, fxFromSeat: fxFromSeat, seatCard: seatCard, doctorSeatLines: doctorSeatLines, doctorTick: doctorTick,
    seatFX: function () { return SEAT_FX; }, seatNowT: seatNowT, setSeatView: setSeatView, kindLabel: kindLabel, BANDS: BANDS, BAND_PATH: BAND_PATH, STAGE_COL: STAGE_COL, COL_V: COL_V,
    ST_V: ST_V, SNAP: SNAP, hmd: hmd, audit: audit, countChars: countChars, toast: toast, mode: function () { return MODE; }, startCoach: startCoach, lastStage: lastStage, isOpenQ: isOpenQ, $: $, $$: $$ };
})(this);
