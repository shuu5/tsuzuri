//! 行 g-accept の歯（接頭辞 bvacc_）: 受入 12 条の測りを新しい面（頁は home と節点・窓は query の win・札と一覧の行は
//! click の吹き出し）に合わせる。幅は規則の行 R-23 の 4 幅、画面は home と 7 つの窓と account board の 3 つの tab、
//! 測りの式は吹き出しの口と札の id と開いた窓と面の中の scroll を読み、fixture は新しい面の URL と口の名を持つ。
//! 測りの式は browser でしか撃てないので字で見る（本物の面での撃ちは席が runner で撃つ）。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

use tsuzuri_boundary::audit::{SCREENS, WIDTHS, facts};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    let path = root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// 字の中の `pub const <name>: &str = "…";` の値。
fn constant(text: &str, name: &str) -> String {
    let head = format!("pub const {name}: &str = \"");
    let at = text.find(&head).unwrap_or_else(|| panic!("{name} が無い")) + head.len();
    text[at..].split('"').next().unwrap_or_default().to_string()
}

/// (2) 測る幅は規則の行 R-23 の 4 幅（1280・960・700・390 の順）で、行の値の字も同じ並び。
#[test]
fn bvacc_widths_are_rule_r23() {
    assert_eq!(WIDTHS, [1280, 960, 700, 390]);
    let rules = read("design-intent/rules.yaml");
    let row = rules
        .lines()
        .find(|l| l.contains("id: R-23,"))
        .expect("規則の行 R-23 が在る");
    let words: Vec<String> = WIDTHS.iter().map(u32::to_string).collect();
    let want = format!("{} の幅で", words.join("・"));
    assert!(row.contains(&want), "R-23 に {want} が無い: {row}");
}

/// (3) 画面は home・帯の印が開く 8 つの窓（窓の宣言の順・相談の窓は行 cs-bar・query の名は topbar の WIN_PARAM）・account board の
/// 3 つの tab の順で、home は最初（節点の頁に開く最初の節点を採る画面）。
#[test]
fn bvacc_screens_open_windows() {
    let topbar = read("crates/tsuzuri-surface/src/topbar.rs");
    assert_eq!(constant(&topbar, "WIN_PARAM"), "win");
    let body = &topbar[topbar
        .find("pub fn key(self) -> &'static str {")
        .expect("Win::key")..];
    let body = &body[..body.find("\n    }\n").expect("key の閉じ")];
    let keys: Vec<&str> = body
        .lines()
        .filter_map(|l| l.split_once("=> \"").map(|(_, r)| r))
        .filter_map(|r| r.split('"').next())
        .collect();
    assert_eq!(
        keys,
        [
            "ask", "stalled", "notices", "seat", "gaps", "legend", "dest", "consult"
        ]
    );
    let mut want = vec!["?".to_string()];
    want.extend(keys.iter().map(|k| format!("?win={k}&")));
    want.extend(
        ["home", "session", "projects"]
            .iter()
            .map(|t| format!("?board=account&tab={t}&")),
    );
    assert_eq!(SCREENS.to_vec(), want);
    assert!(SCREENS.iter().all(|s| !s.contains("page=")), "{SCREENS:?}");
}

/// measure.js の中の、頭の字 head から閉じの字 end の前までの字。
fn cut<'a>(probe: &'a str, head: &str, end: &str) -> &'a str {
    let at = probe
        .find(head)
        .unwrap_or_else(|| panic!("measure.js に {head} が無い"));
    let rest = &probe[at..];
    &rest[..rest
        .find(end)
        .unwrap_or_else(|| panic!("measure.js の {head} の閉じ"))]
}

/// 欠けと散文と節点の id の使い道: 節点の link の鎖は見える要素の列 seen から作って口の中を外し、欄 nocard の鎖は
/// 口の欠け nopop を含めて名に写し、散文の walk は skip の中の字を飛ばして first に積み、節点の id は札の id の
/// 字と包む link の字の組に写し、返す object は測った列をそのままの名で返す。見える要素の列はどれも seen（開いた
/// 窓が在ればその中）から作り、文書の全体から要素を引くのは窓の箱と読み先の 2 つだけ。
fn probe_uses(probe: &str) {
    let link = " = seen\n    .filter((e) => (e.matches(\"a[href]\") && e.getAttribute(\"href\").includes(\"page=node\")) || e.matches(\".node\"))\n    .filter((e) => !e.closest(mouth))\n";
    assert_eq!(probe.matches(link).count(), 1, "節点の link の鎖の頭");
    let nocard = cut(probe, "const nocard = ", ";\n");
    assert!(
        nocard.contains("nopop") && nocard.ends_with(".map(name)"),
        "欄 nocard の鎖: {nocard}"
    );
    let walk = cut(
        probe,
        "document.createTreeWalker(root, NodeFilter.SHOW_TEXT);",
        "\n  }\n",
    );
    let skips: Vec<&str> = walk
        .lines()
        .filter(|l| l.ends_with(") continue;"))
        .collect();
    assert!(
        matches!(skips.as_slice(), [l] if l.contains(" || p.closest(skip)")),
        "散文の walk の飛ばし: {skips:?}"
    );
    assert!(
        walk.contains(" first.push(piece);"),
        "散文の片を first に積む"
    );
    let nodes = "const nodes = seen.filter((e) => e.matches(\".nid, .tid, .kid\")).map((e) => ({\n    id: words(e.textContent),\n    text: words((e.closest(\"a\") || e.parentElement || e).textContent),\n  }));";
    assert_eq!(probe.matches(nodes).count(), 1, "節点の id の鎖");
    for head in [
        "const overflow = seen\n",
        "const parts = seen.filter(",
        "const headings = seen.filter(",
        "const titles = seen.filter(",
        "const switch_at = seen\n",
    ] {
        assert_eq!(probe.matches(head).count(), 1, "seen から作る列 {head}");
    }
    assert_eq!(
        probe.matches("document.querySelectorAll(").count(),
        2,
        "文書の全体から引くのは窓の箱と読み先だけ"
    );
    let back = cut(probe, "return JSON.stringify({", "\n  });");
    for key in [
        "overflow", "overlap", "nocard", "headings", "first", "titles", "nodes",
    ] {
        let line = format!("{key},");
        assert!(back.lines().any(|l| l.trim() == line), "返す欄 {key}");
    }
}

/// 窓が開いていても頁の全体のまま読む欄: 返す欄 text は body の見える字（受入 11 条の逐語の印の入力）、横 scroll の
/// 幅 hscroll は文書の scroll の幅から窓の幅を引いた px、読み先の列 libraries は文書の全体の script と stylesheet と
/// preload（窓の中だけで集めるのは見える要素の列と散文の片）。
fn probe_page_wide(probe: &str) {
    let back = cut(probe, "return JSON.stringify({", "\n  });");
    for want in [
        "text: document.body.innerText,",
        "hscroll: document.documentElement.scrollWidth - vw,",
        "libraries,",
    ] {
        assert!(back.lines().any(|l| l.trim() == want), "返す欄 {want}");
    }
    assert_eq!(
        probe
            .matches("const libraries = Array.from(document.querySelectorAll(")
            .count(),
        1,
        "読み先は文書の全体から"
    );
    assert_eq!(
        probe
            .matches("const vw = document.documentElement.clientWidth;")
            .count(),
        1,
        "窓の幅"
    );
}

/// 見える箱の切りと使い道: box は overflow が visible でない祖先ごとに矩形をその箱の矩形で切り、見えるかの判じ
/// （切った矩形 b を作る関数）は b の幅と高さが正かを見て、重なりは 2 つの要素の切った矩形の交わりで見る。
fn probe_boxes(probe: &str) {
    let boxf = cut(probe, "const box = (e) => {", "\n  };\n");
    for want in [
        "for (let n = e.parentElement; n; n = n.parentElement) {",
        "const s = getComputedStyle(n);",
        "if (s.overflowX === \"visible\" && s.overflowY === \"visible\") continue;",
        "const q = n.getBoundingClientRect();",
        "[left, top, right, bottom] = [Math.max(left, q.left), Math.max(top, q.top), Math.min(right, q.right), Math.min(bottom, q.bottom)];",
        "return { left, top, right, bottom };",
    ] {
        assert_eq!(boxf.matches(want).count(), 1, "box の {want}");
    }
    let at = probe.find("    const b = box(e);").expect("見えるかの判じ");
    let head = probe[..at].rfind("\n  const ").expect("判じの関数の頭");
    let judge = cut(&probe[head..], "\n  const ", "\n  };\n");
    let back: Vec<&str> = judge
        .lines()
        .filter(|l| l.trim_start().starts_with("return "))
        .collect();
    assert!(
        matches!(back.as_slice(), [l] if l.contains("b.right > b.left && b.bottom > b.top")),
        "見えるかは切った矩形で判じる: {back:?}"
    );
    let over = cut(probe, "parts.forEach((a, i) => {", "\n  });\n");
    for want in [
        "const p = box(a);",
        "const q = box(b);",
        "const w = Math.min(p.right, q.right) - Math.max(p.left, q.left);",
        "const h = Math.min(p.bottom, q.bottom) - Math.max(p.top, q.top);",
        "if (w > 1 && h > 1) overlap.push(",
    ] {
        assert_eq!(over.matches(want).count(), 1, "重なりの {want}");
    }
}

/// (4) 測りの式は、札と一覧の行の吹き出しの口（面の pop の 2 つの属性）に自分の click の受け手が無ければ欠けに数え、
/// 口の中の節点の link には hover を問わず、口の中の字を散文に数えず、札の id（class kid）を節点の id に読み、
/// 開いた窓（aria-modal）の中だけを測り、面の中を scroll させる箱の外へ出た部分を切った矩形で重なりを見る。
/// 窓の中だけで集めるのは見える要素の列と散文の片で、返す欄 text と横 scroll の幅と読み先は頁の全体のまま。
/// 字の在る無しに加え、鎖の頭から尾まで・使う所・返す欄までを照らす（probe_uses・probe_boxes・probe_page_wide）。
#[test]
fn bvacc_probe_reads_pops_and_windows() {
    let probe = read("crates/tsuzuri-boundary/src/stage/measure.js");
    let pop = read("crates/tsuzuri-surface/src/widgets/pop.rs");
    let mouth = format!(
        "const mouth = \"[{}], [{}]\";",
        constant(&pop, "CARD_ATTR"),
        constant(&pop, "ROW_ATTR")
    );
    assert_eq!(mouth, "const mouth = \"[data-pop-card], [data-pop-row]\";");
    for want in [
        mouth.as_str(),
        "const nopop = seen.filter((e) => e.matches(mouth)).filter((e) => (getEventListeners(e).click || []).length === 0);",
        "    .filter((e) => !e.closest(mouth))\n    .map(kinds);\n",
        ".mono, [data-t], .q, script, style, \" + mouth;",
        "const nodes = seen.filter((e) => e.matches(\".nid, .tid, .kid\"))",
        "const modal = Array.from(document.querySelectorAll(\"[aria-modal=true]\")).find(shown);",
        "  const root = modal || document.body;",
        "const seen = Array.from(root.querySelectorAll(\"*\")).filter(shown);",
        "document.createTreeWalker(root, NodeFilter.SHOW_TEXT);",
        "if (s.overflowX === \"visible\" && s.overflowY === \"visible\") continue;",
        "    const b = box(e);",
        "    const p = box(a);",
        "      const q = box(b);",
    ] {
        assert_eq!(probe.matches(want).count(), 1, "measure.js の {want}");
    }
    assert!(
        probe.contains(", .kid, .mono,"),
        "散文に数えない id の class"
    );
    probe_uses(&probe);
    probe_boxes(&probe);
    probe_page_wide(&probe);
    let modal = read("crates/tsuzuri-surface/src/widgets/modal.rs");
    assert!(
        modal.contains("role=\"dialog\" aria-modal=\"true\""),
        "窓の箱の印"
    );
    let pipeline = read("crates/tsuzuri-surface/src/project/pipeline.rs");
    assert!(
        pipeline.contains("<span class=\"kid\">"),
        "札の id の class"
    );
}

/// (5) 受入の fixture は新しい面の home の URL（頁の query を持たず mode だけ）と 5 列の板の字を持ち、04 の欠けは
/// 札の口の名（button.kcard）で、題は 36 字。
#[test]
fn bvacc_fixtures_new_surface() {
    let dir = root().join("tests/fixtures/surface/accept-12");
    let mut names: Vec<String> = fs::read_dir(&dir)
        .expect("accept-12 の dir")
        .map(|e| {
            e.expect("dir の項")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|n| n.ends_with(".json") && n != "vocab.json")
        .collect();
    names.sort();
    assert_eq!(names.len(), 13, "{names:?}");
    for name in &names {
        let text = fs::read_to_string(dir.join(name)).expect("fixture を読む");
        assert!(!text.contains("page="), "{name} が頁の query を持つ");
        assert!(!text.contains("4 列"), "{name} が 4 列の字を持つ");
        let got = facts(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(got.url, "http://127.0.0.1:4801/?mode=beginner", "{name}");
        assert!(got.first.iter().any(|p| p.contains("5 列")), "{name}");
        assert_eq!(got.titles[0].chars().count(), 36, "{name}");
        for s in &got.switches {
            assert!(s.before.ends_with("/?mode=beginner"), "{name}: {s:?}");
        }
        let want: &[&str] = if name == "04-hover.json" {
            &["button.kcard"]
        } else {
            &[]
        };
        assert_eq!(got.nocard, want, "{name}");
    }
    let pipeline = read("crates/tsuzuri-surface/src/project/pipeline.rs");
    assert!(pipeline.contains("\"kcard\""), "札の class kcard");
}
