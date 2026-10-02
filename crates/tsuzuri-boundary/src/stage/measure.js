// 受入 12 条の測りの式（行 j-runner・要件 NFR1・規則の行 R-23）。
// CDP の口の measure だけが Runtime.evaluate で撃つ決まった 1 つの式で、頁を書き換えない（写しの中の「?」の印を消すだけ）。
// 見える要素から事実を集めて JSON の字で返す。url と errors と switches は空で返し、runner（audit の case）が埋める。
// getEventListeners は DevTools の console の命令（includeCommandLineAPI が真の時だけ在る）。
(() => {
  const vw = document.documentElement.clientWidth;
  const vh = window.innerHeight;
  // 見える箱: 中だけを scroll させる祖先（overflow が visible でない箱）の外へ出た部分を切った矩形
  // （縦積みの段の pipeline の面とスマホの段の tile・行 g-accept）。
  const box = (e) => {
    const r = e.getBoundingClientRect();
    let [left, top, right, bottom] = [r.left, r.top, r.right, r.bottom];
    for (let n = e.parentElement; n; n = n.parentElement) {
      const s = getComputedStyle(n);
      if (s.overflowX === "visible" && s.overflowY === "visible") continue;
      const q = n.getBoundingClientRect();
      [left, top, right, bottom] = [Math.max(left, q.left), Math.max(top, q.top), Math.min(right, q.right), Math.min(bottom, q.bottom)];
    }
    return { left, top, right, bottom };
  };
  const boxed = (e) => {
    const r = e.getBoundingClientRect();
    const s = getComputedStyle(e);
    const b = box(e);
    return r.width > 0 && r.height > 0 && s.display !== "none" && s.visibility !== "hidden" && b.right > b.left && b.bottom > b.top;
  };
  // 描かれるか: 箱を持っても、閉じた details の中・祖先の display none・visibility hidden の要素は利用者に見えないので、
  // どの条にも数えない（外した数は skipped の hidden に返す・行 g-accept-runner）。
  const drawn = (e) => e.checkVisibility({ visibilityProperty: true });
  const shown = (e) => boxed(e) && drawn(e);
  // 開いた窓（aria-modal の箱）が在れば、測るのは窓の中だけ（窓の外は幕の下で押せない・行 g-accept）。
  const modal = Array.from(document.querySelectorAll("[aria-modal=true]")).find(shown);
  const root = modal || document.body;
  const seen = Array.from(root.querySelectorAll("*")).filter(shown);
  const unseen = Array.from(root.querySelectorAll("*")).filter((e) => boxed(e) && !drawn(e));
  const name = (e) => {
    const cls = typeof e.className === "string" ? e.className.trim().split(/\s+/).filter(Boolean) : [];
    return [e.tagName.toLowerCase() + (e.id ? "#" + e.id : "")].concat(cls).join(".");
  };
  const words = (t) => (t || "").replace(/\s+/g, " ").trim();
  // project board の札と一覧の行の口（click で吹き出しを開く・中の字は札と行の値で散文でない・行 g-accept）。
  const mouth = "[data-pop-card], [data-pop-row]";
  const overflow = seen
    .filter((e) => getComputedStyle(e).overflowX === "visible" && e.scrollWidth - e.clientWidth > 1)
    .map(name);
  const parts = seen.filter((e) => e.matches("a, button, .chip, h1, h2, h3, h4, h5, h6, .hd, [data-t]"));
  const overlap = [];
  parts.forEach((a, i) => {
    const p = box(a);
    parts.slice(i + 1).forEach((b) => {
      if (a.contains(b) || b.contains(a)) return;
      const q = box(b);
      const w = Math.min(p.right, q.right) - Math.max(p.left, q.left);
      const h = Math.min(p.bottom, q.bottom) - Math.max(p.top, q.top);
      if (w > 1 && h > 1) overlap.push(name(a) + " × " + name(b));
    });
  });
  // 節点の札の指の受け手: 自分から body の手前の祖先まで、段ごとに持つ受け手の種類の字（空白で区切る）。
  // 委ねの口（近傍の図の SVG の節点・祖先の mouseover と mouseout）も読み、card を出せるかの判じは audit の側（行 g-accept-fix）。
  const kinds = (e) => {
    const up = [];
    for (let n = e; n && n !== document.body; n = n.parentElement) {
      const l = getEventListeners(n);
      up.push(["pointerenter", "mouseover", "mouseout"].filter((k) => (l[k] || []).length > 0).join(" "));
    }
    return { name: name(e), up };
  };
  // project board の札と一覧の行は hover の card でなく click の吹き出しを出す（要件 FR14・行 g-accept）:
  // 口は自分に click の受け手が無ければ欠けで、口の中の節点の link は hover を問わない。
  const nopop = seen.filter((e) => e.matches(mouth)).filter((e) => (getEventListeners(e).click || []).length === 0);
  const nocard = nopop.map(name);
  const reach = seen
    .filter((e) => (e.matches("a[href]") && e.getAttribute("href").includes("page=node")) || e.matches(".node"))
    .filter((e) => !e.closest(mouth))
    .map(kinds);
  const headings = seen.filter((e) => e.matches("h1, h2, h3, h4, h5, h6, .hd, nav a")).map((e) => {
    const copy = e.cloneNode(true);
    copy.querySelectorAll(".q").forEach((q) => q.remove());
    return { key: e.getAttribute("data-v") || "", text: words(copy.textContent) };
  });
  const skip = ".legend, .lgline, [role=tab], .tabs, .seg, button, .num, .nid, .tid, .kid, .mono, [data-t], .q, script, style, " + mouth;
  // 台帳が書いた字の置き場（面の印 data-ledger-text）は散文と文の予算に数えず、表の行を開いた中（class c-more）の
  // 語と値の対は散文に数えない。外した置き場は skipped に名指して返す（黙って飛ばさない・行 g-accept-runner）。
  const ledger = "[data-ledger-text]";
  const pairs = ".c-more";
  const skipped = seen
    .filter((e) => e.matches(ledger))
    .map((e) => "ledger " + name(e))
    .concat(seen.filter((e) => e.matches(pairs)).map((e) => "pair " + name(e)))
    .concat(unseen.map((e) => "hidden " + name(e)));
  const first = [];
  const walk = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  for (let t = walk.nextNode(); t; t = walk.nextNode()) {
    const p = t.parentElement;
    const piece = words(t.nodeValue);
    if (!piece || !p || !shown(p) || p.closest(skip) || p.closest(ledger) || p.closest(pairs)) continue;
    if (p.getBoundingClientRect().top < vh) first.push(piece);
  }
  const titles = seen.filter((e) => e.matches("[data-t]") && !e.closest(ledger)).map((e) => words(e.textContent));
  const nodes = seen.filter((e) => e.matches(".nid, .tid, .kid")).map((e) => ({
    id: words(e.textContent),
    text: words((e.closest("a") || e.parentElement || e).textContent),
  }));
  // 今の選びの印（aria-selected と aria-pressed の真・aria-current・今の tab の class on）。今の選びは押しても何も
  // 替わらないのが正しいので、runner（audit の case）は now の真の切り替えを押さない（行 g-accept-runner）。
  const current = "[aria-selected=true], [aria-pressed=true], [aria-current]:not([aria-current=false]), .on";
  const switch_at = seen
    .filter((e) => e.matches("button.seg, .seg button, [role=tab], [aria-pressed], [data-tab]"))
    .map((e) => ({ label: words(e.textContent), now: e.matches(current), r: e.getBoundingClientRect() }))
    .map(({ label, now, r }) => ({ label, now, x: Math.round(r.left + r.width / 2), y: Math.round(r.top + r.height / 2) }))
    .filter(({ x, y }) => x >= 0 && y >= 0 && x < vw && y < vh);
  const libraries = Array.from(document.querySelectorAll("script[src], link[rel~=stylesheet][href], link[rel~=preload][href], link[rel~=modulepreload][href]"))
    .map((e) => e.getAttribute("src") || e.getAttribute("href"));
  return JSON.stringify({
    url: "",
    hscroll: document.documentElement.scrollWidth - vw,
    overflow,
    overlap,
    nocard,
    reach,
    skipped,
    headings,
    first,
    errors: [],
    titles,
    nodes,
    switches: [],
    text: document.body.innerText,
    libraries,
    switch_at,
  });
})()
