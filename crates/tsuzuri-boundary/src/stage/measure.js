// 受入 12 条の測りの式（行 j-runner・要件 NFR1・規則の行 R-23）。
// CDP の口の measure だけが Runtime.evaluate で撃つ決まった 1 つの式で、頁を書き換えない（写しの中の「?」の印を消すだけ）。
// 見える要素から事実を集めて JSON の字で返す。url と errors と switches は空で返し、runner（audit の case）が埋める。
// getEventListeners は DevTools の console の命令（includeCommandLineAPI が真の時だけ在る）。
(() => {
  const vw = document.documentElement.clientWidth;
  const vh = window.innerHeight;
  const shown = (e) => {
    const r = e.getBoundingClientRect();
    const s = getComputedStyle(e);
    return r.width > 0 && r.height > 0 && s.display !== "none" && s.visibility !== "hidden";
  };
  const seen = Array.from(document.body.querySelectorAll("*")).filter(shown);
  const name = (e) => {
    const cls = typeof e.className === "string" ? e.className.trim().split(/\s+/).filter(Boolean) : [];
    return [e.tagName.toLowerCase() + (e.id ? "#" + e.id : "")].concat(cls).join(".");
  };
  const words = (t) => (t || "").replace(/\s+/g, " ").trim();
  const overflow = seen
    .filter((e) => getComputedStyle(e).overflowX === "visible" && e.scrollWidth - e.clientWidth > 1)
    .map(name);
  const parts = seen.filter((e) => e.matches("a, button, .chip, h1, h2, h3, h4, h5, h6, .hd, [data-t]"));
  const overlap = [];
  parts.forEach((a, i) => {
    const p = a.getBoundingClientRect();
    parts.slice(i + 1).forEach((b) => {
      if (a.contains(b) || b.contains(a)) return;
      const q = b.getBoundingClientRect();
      const w = Math.min(p.right, q.right) - Math.max(p.left, q.left);
      const h = Math.min(p.bottom, q.bottom) - Math.max(p.top, q.top);
      if (w > 1 && h > 1) overlap.push(name(a) + " × " + name(b));
    });
  });
  const entered = (e) => {
    for (let n = e; n; n = n.parentElement) {
      if ((getEventListeners(n).pointerenter || []).length > 0) return true;
    }
    return false;
  };
  const nocard = seen
    .filter((e) => (e.matches("a[href]") && e.getAttribute("href").includes("page=node")) || e.matches(".node"))
    .filter((e) => !entered(e))
    .map(name);
  const headings = seen.filter((e) => e.matches("h1, h2, h3, h4, h5, h6, .hd, nav a")).map((e) => {
    const copy = e.cloneNode(true);
    copy.querySelectorAll(".q").forEach((q) => q.remove());
    return { key: e.getAttribute("data-v") || "", text: words(copy.textContent) };
  });
  const skip = ".legend, .lgline, [role=tab], .tabs, .seg, button, .num, .nid, .tid, .mono, [data-t], .q, script, style";
  const first = [];
  const walk = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  for (let t = walk.nextNode(); t; t = walk.nextNode()) {
    const p = t.parentElement;
    const piece = words(t.nodeValue);
    if (!piece || !p || !shown(p) || p.closest(skip)) continue;
    if (p.getBoundingClientRect().top < vh) first.push(piece);
  }
  const titles = seen.filter((e) => e.matches("[data-t]")).map((e) => words(e.textContent));
  const nodes = seen.filter((e) => e.matches(".nid, .tid")).map((e) => ({
    id: words(e.textContent),
    text: words((e.closest("a") || e.parentElement || e).textContent),
  }));
  const switch_at = seen
    .filter((e) => e.matches("button.seg, .seg button, [role=tab], [aria-pressed], [data-tab]"))
    .map((e) => ({ label: words(e.textContent), r: e.getBoundingClientRect() }))
    .map(({ label, r }) => ({ label, x: Math.round(r.left + r.width / 2), y: Math.round(r.top + r.height / 2) }))
    .filter(({ x, y }) => x >= 0 && y >= 0 && x < vw && y < vh);
  const libraries = Array.from(document.querySelectorAll("script[src], link[rel~=stylesheet][href], link[rel~=preload][href], link[rel~=modulepreload][href]"))
    .map((e) => e.getAttribute("src") || e.getAttribute("href"));
  return JSON.stringify({
    url: "",
    hscroll: document.documentElement.scrollWidth - vw,
    overflow,
    overlap,
    nocard,
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
