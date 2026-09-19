//! 判断の記録の面の生成器（便 25・docs/design/delivery-25.md §1 (b)〜(d)）。判断の記録の正本 1 本
//! （`adr/ADR-n.yaml`）から人が読むページ 1 枚を組む。生成物の文字列は 正本の値（α・escape して逐語）・
//! 名札の表（β・この file の表）・正本から数えた数（γ）のどれかで、案内の文は出さない。
//! 面に依らない口（head と site-bar・章の帯・card・toc・承認欄の帯・foot・部品の名札）は `face.rs` の `Frame` を呼ぶ。
//! 章の h2 のうち契約が名指すのは 01・02 と承認欄だけなので、03〜05 は章の名をそのまま h2 に出す（新しい字面を持たない）。
//! 見た目は便 27（delivery-27.md §1 (a)〜(c)）で直した: 表紙は h1 が短い名で title は副題・章 01/02 は文の頭の
//! 列挙を ol へ・章 04 は根拠を 4 群の card（id + 行き先の題）にして撤退条件を格子の外へ出す。
//! 図の章（便 33・FR15）: 正本に任意の図の節（figures）が 1 枚以上あれば章 06「図」を「改訂と帰結」の後・承認欄の
//! 前に置く。中身は設計ノートの面の図の章と同じ字面で、図の本体（SVG）は `figure.rs` の `render` が図の道具で描いた
//! ものを逐語で埋める。図が 1 枚でも導出できなければ面全体を導出しない（全部か無しか）。図が無い面は便 32 までと byte 不変。

use std::path::Path;

use crate::face::{self, Frame, R, X, anchor, card, esc};
use crate::face_note::FIGURE_LABELS;
use crate::figure;
use crate::parts::catalog::Component;

/// 判断の記録の面が使う部品（8 種・便 33 で figure-panel を足した）。
pub const PARTS: [Component; 8] = [
    Component::FreshnessStamp,
    Component::FontSizeControl,
    Component::DocCoverBand,
    Component::ApprovalBlock,
    Component::ChapterDeckBand,
    Component::SectionLeadCallout,
    Component::FigurePanel,
    Component::ItemRow,
];

/// 章 01〜05 の名（見出しの帯の kicker・目次の名）。
const CHAPTERS: [&str; 5] = ["問題", "決定", "案", "根拠と撤退条件", "改訂と帰結"];

/// 章 01〜05 の h2（01・02 は契約の字面・03〜05 は章の名）。
const H2: [&str; 5] = [
    "何が問題か",
    "何を決めたか",
    "案",
    "根拠と撤退条件",
    "改訂と帰結",
];

/// 章 06（図）の名（帯の kicker・目次の名）。
const FIGURES_CHAPTER: &str = "図";

/// 章 01〜06 の帯の class と kicker の絵記号（01〜05 は要件書の面の章 01〜05 の写し・06 は要件書の面の band-6 の写し）。
/// 章の数だけ先頭から切り出して `Frame` に渡す（`static` なので切り出しは 'static）。
static BANDS: [(&str, &str); 6] = [
    (
        "band-1",
        "<circle cx=\"12\" cy=\"12\" r=\"9\"/><circle cx=\"12\" cy=\"12\" r=\"3\"/>",
    ),
    (
        "band-2",
        "<path d=\"M3 7l9-4 9 4-9 4-9-4z\"/><path d=\"M3 7v10l9 4 9-4V7\"/>",
    ),
    ("band-3", "<path d=\"M20 6L9 17l-5-5\"/>"),
    ("band-4", "<path d=\"M4 18h16M6 14V8M12 14V4M18 14v-4\"/>"),
    (
        "band-5",
        "<path d=\"M9 11l3 3L22 4\"/><path d=\"M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11\"/>",
    ),
    ("band-6", "<path d=\"M4 18h16M6 14V8M12 14V4M18 14v-4\"/>"),
];

/// 判断の記録の面の FRAME の favicon の字面。
const FAVICON: &str = "<link rel=\"icon\" href=\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect width='32' height='32' rx='7' fill='%235f45a6'/%3E%3Ctext x='16' y='22' font-size='16' font-weight='700' text-anchor='middle' fill='%23ffffff' font-family='sans-serif'%3E要%3C/text%3E%3C/svg%3E\">";

/// 判断の記録の面の骨格（章の数 = 5 + 図の章の有無）。nav の aria-current は付かない（読める面の nav に判断の記録は無い）。
fn frame(chapters: usize) -> Frame {
    Frame {
        name: "判断の記録",
        source: "adr/ADR-n.yaml",
        favicon: FAVICON,
        current: 3,
        first: 1,
        bands: &BANDS[..chapters],
        prev: ("srs.html", "要件書"),
        next: ("index.html", "入口"),
        parts: &PARTS,
    }
}

// ── 名札の表（β・表に無い値は導出できない）──

/// 状態（status）→ 見出しの帯と機械の面に出す名札。
const STATUS: &[(&str, &str)] = &[
    ("proposed", "提案中・拘束力なし"),
    ("accepted", "発効"),
    ("retired", "廃止"),
];

/// 案の判定（verdict）→ 名札。
const VERDICT: &[(&str, &str)] = &[("adopted", "採用"), ("rejected", "退けた")];

/// 撤退条件の種類（retreat.kind）→ 名札。
const RETREAT_KIND: &[(&str, &str)] = &[
    ("spike", "小さな試し"),
    ("measure", "数えた値"),
    ("ruling", "持ち主の裁定"),
];

/// 状態の名札（短い）と状態の行（組み立て済みの HTML）。
struct Status {
    label: &'static str,
    line: String,
}

/// 正本から数えた数（γ）。
struct Counts {
    options: usize,
    adopted: usize,
    rejected: usize,
    basis: usize,
    amends: usize,
    figures: usize,
}

/// 根拠の id の行き先を解くための、他の正本の id と題の一覧。
struct Ctx {
    /// 条の（id・title）
    articles: Vec<(String, String)>,
    /// 規範文 id → その条の id
    statements: Vec<(String, String)>,
    /// rules 行の（id・what）
    rules: Vec<(String, String)>,
    /// 要件書の（id・title）（要件・非機能要件・受入基準・制約・ゴール）
    reqs: Vec<(String, String)>,
}

/// 根拠の 4 群の名（順もこのとおり・便 27 §1 (c)）。
const BASIS_GROUPS: [&str; 4] = ["憲法の条", "数値の表", "要件書", "判断の記録"];

/// 根拠の id 1 つの行き先（群・href・題）。href が None なら行き先が無い。
struct Target {
    group: usize,
    href: Option<String>,
    /// 行き先の欄の題（読めなければ空・題の span を出さない）
    title: String,
}

/// 正本 1 本 → 判断の記録の面の HTML（決定的）。
pub fn derive(dir: &Path, id: &str) -> R<String> {
    check_id_shape(id)?;
    let name = format!("adr/{id}.yaml");
    let a_doc = face::load(dir, &name)?;
    let c_doc = face::load(dir, "constitution.yaml")?;
    let r_doc = face::load(dir, "rules.yaml")?;
    let s_doc = face::load(dir, "srs.yaml")?;
    let a = X::root(&a_doc, &name);
    let ctx = context(
        &X::root(&c_doc, "constitution.yaml"),
        &X::root(&r_doc, "rules.yaml"),
        &X::root(&s_doc, "srs.yaml"),
    )?;

    let file_id = a.f("id")?.id()?;
    if file_id != id {
        return Err(format!("{name}: 欄 id「{file_id}」が --id「{id}」と違う"));
    }
    let st = status(&a, dir, &ctx)?;
    let figs = match a.g("figures")? {
        Some(x) => x.seq()?,
        None => Vec::new(),
    };
    let counts = counts(&a, figs.len())?;
    let f = frame(CHAPTERS.len() + usize::from(!figs.is_empty()));

    let mut o: Vec<String> = Vec::new();
    head(&mut o, &f, &a, id, &st)?;
    cover(&mut o, &f, &a, id, &st, &counts)?;
    toc(&f, &mut o, figs.len());
    prose_chapter(&mut o, &f, 1, &a.ef("context")?);
    prose_chapter(&mut o, &f, 2, &a.ef("decision")?);
    options_chapter(&mut o, &f, &a)?;
    basis_chapter(&mut o, &f, &a, dir, &ctx)?;
    amends_chapter(&mut o, &f, &a, dir, &ctx)?;
    if !figs.is_empty() {
        figures_chapter(&mut o, &f, CHAPTERS.len() + 1, &figs, dir, &ctx)?;
    }
    approval_chapter(&mut o, &f, &a, &st)?;
    foot(&mut o, &f, &a, id, &counts)?;
    Ok(format!("{}\n", o.join("\n")))
}

// ── 読みと解き ──

/// `--id` は `ADR-` + ASCII の数字列 1 つ以上。
fn check_id_shape(id: &str) -> R<()> {
    if id
        .strip_prefix("ADR-")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
    {
        Ok(())
    } else {
        Err(format!("--id「{id}」は id の形でない（ADR- と数字）"))
    }
}

fn digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// 行き先の欄の題（欄が無い・空・読めないときは空）。題は行き先の欄が読めるときだけ出す。
fn field_title(x: &X<'_>, key: &str) -> String {
    match x.g(key) {
        Ok(Some(v)) => v.e().unwrap_or_default(),
        _ => String::new(),
    }
}

fn context(c: &X<'_>, r: &X<'_>, s: &X<'_>) -> R<Ctx> {
    let mut articles = Vec::new();
    let mut statements = Vec::new();
    for a in c.f("articles")?.seq()? {
        let aid = a.f("id")?.id()?.to_string();
        for st in a.f("statements")?.seq()? {
            statements.push((st.f("id")?.id()?.to_string(), aid.clone()));
        }
        articles.push((aid, field_title(&a, "title")));
    }
    let mut rules = Vec::new();
    for section in ["thresholds", "discipline"] {
        for x in r.f(section)?.seq()? {
            rules.push((x.f("id")?.id()?.to_string(), field_title(&x, "what")));
        }
    }
    let mut reqs = Vec::new();
    for section in [
        "requirements",
        "nonfunctional",
        "acceptance",
        "constraints",
        "goals",
    ] {
        for x in s.f(section)?.seq()? {
            reqs.push((x.f("id")?.id()?.to_string(), field_title(&x, "title")));
        }
    }
    Ok(Ctx {
        articles,
        statements,
        rules,
        reqs,
    })
}

/// 判断の記録の題（file が無い・読めない・title の欄が無い・空なら空。面は 2 にしない）。
fn record_title(dir: &Path, id: &str) -> String {
    match face::load(dir, &format!("adr/{id}.yaml")) {
        Ok(doc) => field_title(&X::root(&doc, id), "title"),
        Err(_) => String::new(),
    }
}

/// 根拠の id の群と行き先と題。4 形のどれでもない id は Err。
fn resolve(dir: &Path, ctx: &Ctx, id: &str) -> R<Target> {
    let bad = || format!("根拠の id「{id}」は id の形でない（条・rules 行・要件・判断の記録）");
    let found = |pairs: &[(String, String)], group: usize, href: String| match pairs
        .iter()
        .find(|(k, _)| k == id)
    {
        Some((_, title)) => Target {
            group,
            href: Some(href),
            title: title.clone(),
        },
        None => Target {
            group,
            href: None,
            title: String::new(),
        },
    };
    if let Some(rest) = ["P-", "A-", "N-"].iter().find_map(|p| id.strip_prefix(p)) {
        return match rest.split_once('.') {
            // 枝番付きの規範文 id は条の anchor へ（字は枝番付きのまま・題は条の title）
            Some((n, sub)) if digits(n) && digits(sub) => Ok(ctx
                .statements
                .iter()
                .find(|(sid, _)| sid == id)
                .map_or_else(
                    || Target {
                        group: 0,
                        href: None,
                        title: String::new(),
                    },
                    |(_, aid)| Target {
                        group: 0,
                        href: Some(format!("constitution.html#{}", anchor(aid))),
                        title: ctx
                            .articles
                            .iter()
                            .find(|(k, _)| k == aid)
                            .map_or_else(String::new, |(_, t)| t.clone()),
                    },
                )),
            None if digits(rest) => Ok(found(
                &ctx.articles,
                0,
                format!("constitution.html#{}", anchor(id)),
            )),
            _ => Err(bad()),
        };
    }
    if let Some(rest) = ["R-", "D-"].iter().find_map(|p| id.strip_prefix(p)) {
        return if digits(rest) {
            Ok(found(
                &ctx.rules,
                1,
                format!("constitution.html#{}", anchor(id)),
            ))
        } else {
            Err(bad())
        };
    }
    if let Some(rest) = ["FR", "NFR", "AC", "CON", "GOAL"]
        .iter()
        .find_map(|p| id.strip_prefix(p))
    {
        return if digits(rest) {
            Ok(found(&ctx.reqs, 2, format!("srs.html#{}", anchor(id))))
        } else {
            Err(bad())
        };
    }
    if let Some(rest) = id.strip_prefix("ADR-") {
        return if digits(rest) {
            let file = dir.join("adr").join(format!("{id}.yaml"));
            Ok(if file.is_file() {
                Target {
                    group: 3,
                    href: Some(format!("{}.html", anchor(id))),
                    title: record_title(dir, id),
                }
            } else {
                Target {
                    group: 3,
                    href: None,
                    title: String::new(),
                }
            })
        } else {
            Err(bad())
        };
    }
    Err(bad())
}

/// 行き先の字面（在ればリンク・無ければ「（まだ分からない）」）。
fn link_text(t: &Target, id: &str) -> String {
    match &t.href {
        Some(href) => format!("<a class=\"xref\" href=\"{href}\">{id}</a>"),
        None => format!("{id}（まだ分からない）"),
    }
}

/// 根拠の id 1 つ（在ればリンク・無ければ「（まだ分からない）」）。
fn id_link(dir: &Path, ctx: &Ctx, x: &X<'_>) -> R<String> {
    let id = x.id()?;
    Ok(link_text(&resolve(dir, ctx, id)?, id))
}

/// 状態の名札と状態の行。accepted に承認欄が無い・retired に後継が無い、は導出できない。
fn status(a: &X<'_>, dir: &Path, ctx: &Ctx) -> R<Status> {
    let sx = a.f("status")?;
    let label = sx.lookup(STATUS, "状態")?;
    let line = match sx.v.as_str() {
        Some("accepted") => {
            let ap = a
                .g("approval")?
                .ok_or_else(|| format!("{}: accepted に承認欄が無い", a.at))?;
            format!("発効・拘束力あり（承認 {}）", ap.ef("date")?)
        }
        Some("retired") => {
            let sb = a
                .g("superseded_by")?
                .ok_or_else(|| format!("{}: retired に superseded_by が無い", a.at))?;
            format!("廃止 → 後継 {}", id_link(dir, ctx, &sb)?)
        }
        _ => "提案中・拘束力なし → 持ち主の承認で発効".to_string(),
    };
    Ok(Status { label, line })
}

fn counts(a: &X<'_>, figures: usize) -> R<Counts> {
    let options = a.f("options")?.seq()?;
    if options.is_empty() {
        return Err(format!("{}: 案が 1 つも無い", a.f("options")?.at));
    }
    let mut adopted = 0;
    for o in &options {
        if o.f("verdict")?.lookup(VERDICT, "判定")? == "採用" {
            adopted += 1;
        }
    }
    let amends = match a.g("amends")? {
        Some(x) => x.seq()?.len(),
        None => 0,
    };
    Ok(Counts {
        options: options.len(),
        adopted,
        rejected: options.len() - adopted,
        basis: a.f("basis")?.seq()?.len(),
        amends,
        figures,
    })
}

// ── 骨格 ──

fn head(o: &mut Vec<String>, f: &Frame, a: &X<'_>, id: &str, st: &Status) -> R<()> {
    f.head(
        o,
        &format!("folio2 — 判断の記録 {id}（{}）", st.label),
        &a.ef("date")?,
        id,
        st.label,
    );
    Ok(())
}

fn meta_span(k: &str, v: &str) -> String {
    format!("<span class=\"m\"><span class=\"k\">{k}</span><span class=\"v\">{v}</span></span>")
}

fn cover(o: &mut Vec<String>, f: &Frame, a: &X<'_>, id: &str, st: &Status, n: &Counts) -> R<()> {
    o.push(format!("<header {}>", f.dc(Component::DocCoverBand)));
    o.push(format!(
        "<p class=\"cover-eyebrow\"><span class=\"doc-type\">判断の記録 (ADR)</span> <span>folio2 — {id}</span></p>"
    ));
    // h1 は短い名（title は文の長さなので副題へ・便 27 §1 (a)）
    o.push(format!("<h1>判断の記録 {id}</h1>"));
    o.push(format!("<p class=\"sub-title\">{}</p>", a.ef("title")?));
    o.push(format!(
        "<div class=\"summary-card\"><span class=\"ic\">要</span><div><p class=\"lab\">やさしく言うと</p><p class=\"txt\">{}</p></div></div>",
        a.ef("plain")?
    ));
    o.push("<div class=\"cover-meta\">".to_string());
    o.push(meta_span("状態", st.label));
    o.push(meta_span("日付", &a.ef("date")?));
    o.push(meta_span(
        "案",
        &format!(
            "{} 件（採用 {}・退けた {}）",
            n.options, n.adopted, n.rejected
        ),
    ));
    o.push(meta_span("根拠", &format!("{} 件", n.basis)));
    o.push(meta_span("改訂", &format!("{} 件", n.amends)));
    o.push(meta_span(
        "撤退条件",
        a.f("retreat")?
            .f("kind")?
            .lookup(RETREAT_KIND, "撤退条件の種類")?,
    ));
    // 図は 1 枚以上のときだけ（図なしの面は便 32 までと byte 不変）
    if n.figures > 0 {
        o.push(meta_span(FIGURES_CHAPTER, &format!("{} 枚", n.figures)));
    }
    o.push("</div>".to_string());
    o.push(format!(
        "<p class=\"cover-status\"><span class=\"k\">状態</span><span>{}（<a href=\"#approval\">承認欄へ</a>）</span></p>",
        st.line
    ));
    o.push("</header>".to_string());
    Ok(())
}

fn toc(f: &Frame, o: &mut Vec<String>, figures: usize) {
    let mut heads = CHAPTERS
        .iter()
        .zip(H2)
        .map(|(k, t)| ((*k).to_string(), t.to_string()))
        .collect::<Vec<_>>();
    if figures > 0 {
        heads.push((FIGURES_CHAPTER.to_string(), format!("{figures} 枚")));
    }
    f.toc(o, &heads, "承認");
}

fn band(o: &mut Vec<String>, f: &Frame, n: usize) {
    f.band(o, n, CHAPTERS[n - 1], H2[n - 1], None);
}

// ── 章 ──

/// 文の頭の列挙の印「(k) 」の位置（開きの括弧の byte・印の直後の byte）。印と数えるのは、印の直前
/// （末尾の空白を除く）が本文の先頭か句点で、かつ番号が 1 から 1 ずつ増えて続くときだけ（便 27 §1 (b)）。
fn item_marks(body: &str) -> Vec<(usize, usize)> {
    let b = body.as_bytes();
    let mut marks = Vec::new();
    let mut want = 1u32;
    let mut i = 0;
    while i < b.len() {
        // 半角の開き括弧 + ASCII の数字 1 つ以上 + 半角の閉じ括弧 + 半角空白 1 つ
        if b[i] == b'(' {
            let mut j = i + 1;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            if j > i + 1 && b.get(j) == Some(&b')') && b.get(j + 1) == Some(&b' ') {
                let head = body[..i].trim_end();
                let at_head = head.is_empty() || head.ends_with('。');
                if at_head && body[i + 1..j].parse::<u32>() == Ok(want) {
                    marks.push((i, j + 2));
                    want += 1;
                    i = j + 2;
                    continue;
                }
            }
        }
        i += 1;
    }
    marks
}

/// 章 01・02（帯 + chapbody）。`body` は escape 済み。文の頭の印が 2 つ以上なら前置きの p と ol へ分け、
/// 1 つ以下なら p 1 つ（便 27 §1 (b)）。
fn prose_chapter(o: &mut Vec<String>, f: &Frame, n: usize, body: &str) {
    band(o, f, n);
    o.push("<div class=\"chapbody\">".to_string());
    let marks = item_marks(body);
    if marks.len() < 2 {
        o.push(format!("<p>{body}</p>"));
    } else {
        let intro = body[..marks[0].0].trim();
        if !intro.is_empty() {
            o.push(format!("<p class=\"intro\">{intro}</p>"));
        }
        o.push("<ol class=\"items\">".to_string());
        for (k, (_, end)) in marks.iter().enumerate() {
            let stop = marks.get(k + 1).map_or(body.len(), |(start, _)| *start);
            o.push(format!("<li>{}</li>", body[*end..stop].trim()));
        }
        o.push("</ol>".to_string());
    }
    o.push("</div>".to_string());
}

/// 章 03（案・採用と退けた案を同じ形で）。
fn options_chapter(o: &mut Vec<String>, f: &Frame, a: &X<'_>) -> R<()> {
    band(o, f, 3);
    o.push("<div class=\"chapbody\">".to_string());
    for opt in a.f("options")?.seq()? {
        let oid = opt.f("id")?.id()?;
        let verdict = opt.f("verdict")?;
        o.push(format!(
            "<article {} id=\"opt-{oid}\">",
            f.dc(Component::ItemRow)
        ));
        o.push(format!(
            "<div class=\"ir-head\"><span class=\"rid\">案 {oid}</span><h3 class=\"rt\">{}</h3><span class=\"badges\"><span class=\"pill\">{}</span></span></div>",
            opt.ef("name")?,
            verdict.lookup(VERDICT, "判定")?
        ));
        o.push(format!("<p class=\"norm\">{}</p>", opt.ef("text")?));
        o.push(format!(
            "<div class=\"plain\"><span class=\"pk\">理由</span>{}</div>",
            opt.ef("reason")?
        ));
        o.push(format!(
            "<details class=\"machine\" data-audience=\"machine\"><summary>機械のための面</summary><dl><dt>verdict</dt><dd>{}</dd></dl></details>",
            verdict.e()?
        ));
        o.push("</article>".to_string());
    }
    o.push("</div>".to_string());
    Ok(())
}

/// 章 04（根拠の 4 群と撤退条件・便 27 §1 (c)）。
fn basis_chapter(o: &mut Vec<String>, f: &Frame, a: &X<'_>, dir: &Path, ctx: &Ctx) -> R<()> {
    band(o, f, 4);
    o.push("<div class=\"chapbody\">".to_string());
    // 群の順は BASIS_GROUPS の順・群の中は basis の順
    let mut groups: [Vec<String>; 4] = std::array::from_fn(|_| Vec::new());
    for b in a.f("basis")?.seq()? {
        let id = b.id()?;
        let t = resolve(dir, ctx, id)?;
        // 題は行き先の欄が読めるときだけ出す
        let title = if t.href.is_some() && !t.title.is_empty() {
            format!("<span>{}</span>", t.title)
        } else {
            String::new()
        };
        groups[t.group].push(format!("<li>{}{title}</li>", link_text(&t, id)));
    }
    o.push(format!(
        "<div {} style=\"--band-n:{}\">",
        f.dc(Component::SectionLeadCallout),
        groups.iter().filter(|g| !g.is_empty()).count()
    ));
    for (k, g) in groups.iter().enumerate() {
        if g.is_empty() {
            continue;
        }
        o.push(card(
            "card",
            None,
            &format!("{}（{}）", BASIS_GROUPS[k], g.len()),
            &format!("<ul class=\"basis\">\n{}\n</ul>", g.join("\n")),
        ));
    }
    o.push("</div>".to_string());
    // 撤退条件は格子の外（1 列に潰さない）
    let rt = a.f("retreat")?;
    o.push(card(
        "card retreat",
        None,
        &format!(
            "撤退条件（{}）",
            rt.f("kind")?.lookup(RETREAT_KIND, "撤退条件の種類")?
        ),
        &format!("<p>{}</p>", rt.ef("condition")?),
    ));
    o.push("</div>".to_string());
    Ok(())
}

/// 章 05（改訂・帰結・反対側からの確認・置き換え・注）。
fn amends_chapter(o: &mut Vec<String>, f: &Frame, a: &X<'_>, dir: &Path, ctx: &Ctx) -> R<()> {
    band(o, f, 5);
    o.push("<div class=\"chapbody\">".to_string());
    let amends = match a.g("amends")? {
        Some(x) => x.seq()?,
        None => Vec::new(),
    };
    if amends.is_empty() {
        o.push("<p>条文の改訂なし</p>".to_string());
    } else {
        o.push("<ul>".to_string());
        for e in &amends {
            o.push(format!(
                "<li>{} {}.{}: 「{}」→「{}」</li>",
                e.ef("version")?,
                e.ef("target")?,
                e.ef("field")?,
                e.ef("previous_text")?,
                e.ef("new_text")?
            ));
        }
        o.push("</ul>".to_string());
    }
    if let Some(cs) = a.g("consequences")? {
        o.push("<h3>この判断で変わること</h3>".to_string());
        o.push("<ul>".to_string());
        for c in cs.seq()? {
            o.push(format!("<li>{}</li>", c.e()?));
        }
        o.push("</ul>".to_string());
    }
    if let Some(g) = a.g("grill")? {
        o.push("<h3>反対側からの確認</h3>".to_string());
        o.push(format!(
            "<p>{}・{}・{}</p>",
            g.ef("when")?,
            g.ef("who")?,
            g.ef("where")?
        ));
        o.push(format!("<p>{}</p>", g.ef("summary")?));
    }
    for (key, label) in [
        ("supersedes", "置き換えた判断"),
        ("superseded_by", "後継の判断"),
    ] {
        if let Some(x) = a.g(key)? {
            o.push(format!("<p>{label}: {}</p>", id_link(dir, ctx, &x)?));
        }
    }
    if let Some(note) = a.g("note")? {
        o.push("<h3>注</h3>".to_string());
        o.push(format!("<p>{}</p>", note.e()?));
    }
    o.push("</div>".to_string());
    Ok(())
}

/// 章 06（図・便 33）。設計ノートの面の図の章と同じ字面: 図ごとに図の枠（figure-panel）を置き、図の本体は
/// `figure::render` の戻り値をそのまま 1 つの行として埋める（escape しない・道具の出力は変えない）。
/// figcaption の根拠は refs の各 id を `id_link` で（無いか空なら「根拠:」以降を出さない）。
/// 図が 1 枚でも導出できなければ Err（面全体が「まだ分からない」・前の面は残る）。
fn figures_chapter(
    o: &mut Vec<String>,
    f: &Frame,
    n: usize,
    figs: &[X<'_>],
    dir: &Path,
    ctx: &Ctx,
) -> R<()> {
    f.band(
        o,
        n,
        FIGURES_CHAPTER,
        &format!("{FIGURES_CHAPTER} {} 枚", figs.len()),
        None,
    );
    o.push("<div class=\"chapbody\">".to_string());
    for (i, fig) in figs.iter().enumerate() {
        let fid = fig.f("id")?.id()?;
        let tx = fig.f("type")?;
        let kind =
            tx.v.as_str()
                .ok_or_else(|| format!("{}: 図の型が文字列でない", tx.at))?;
        let body = figure::render(dir, fid, kind, &fig.f("spec")?)?;
        let label = FIGURE_LABELS
            .iter()
            .find(|(k, _)| *k == kind)
            .map(|(_, l)| *l)
            .ok_or_else(|| format!("図の型「{kind}」は図の道具の型でない"))?;
        let fn_ = format!("図 {}", i + 1);
        o.push(format!(
            "<figure {} data-role=\"diagram\" id=\"{}\">",
            f.dc(Component::FigurePanel),
            esc(fid)
        ));
        o.push(format!(
            "<div class=\"fig-title\"><span class=\"fn\">{fn_}</span>{} <span class=\"fig-tools\"><button class=\"zoom-btn\" type=\"button\">拡大</button></span><button class=\"zoom-close\" type=\"button\">✕ 閉じる</button></div>",
            fig.ef("caption")?
        ));
        o.push(body);
        let mut ver = format!("{fn_} · {label} · {}", esc(fid));
        if let Some(refs) = fig.g("refs")? {
            let ids = refs
                .seq()?
                .iter()
                .map(|q| id_link(dir, ctx, q))
                .collect::<R<Vec<_>>>()?;
            if !ids.is_empty() {
                ver.push_str(&format!(" · 根拠: {}", ids.join("・")));
            }
        }
        o.push(format!(
            "<figcaption><span class=\"ver\">{ver}</span></figcaption>"
        ));
        o.push("</figure>".to_string());
    }
    o.push("</div>".to_string());
    Ok(())
}

fn approval_chapter(o: &mut Vec<String>, f: &Frame, a: &X<'_>, st: &Status) -> R<()> {
    f.approval_band(o, "承認", st.label);
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!("<div {}>", f.dc(Component::ApprovalBlock)));
    match a.g("approval")? {
        Some(ap) => o.push(format!(
            "<div class=\"sign\"><span class=\"role\">承認</span><span class=\"who\">{}</span><span class=\"when\">{}</span><span class=\"when\">逐語「{}」</span><span class=\"stamp\">{}</span></div>",
            ap.ef("who")?,
            ap.ef("date")?,
            ap.ef("verbatim")?,
            ap.ef("ruling")?
        )),
        None => o.push("<p>未（提案中・持ち主の逐語と日付が入ると発効）</p>".to_string()),
    }
    o.push("</div>".to_string());
    o.push("</div>".to_string());
    Ok(())
}

fn foot(o: &mut Vec<String>, f: &Frame, a: &X<'_>, id: &str, n: &Counts) -> R<()> {
    let date = a.ef("date")?;
    let basis = a
        .f("basis")?
        .seq()?
        .iter()
        .map(X::e)
        .collect::<R<Vec<_>>>()?
        .join("・");
    let mut dl = format!(
        "<dt>id</dt><dd>{id}</dd><dt>status</dt><dd>{}</dd><dt>date</dt><dd>{date}</dd><dt>basis</dt><dd>{basis}</dd><dt>amends</dt><dd>{}</dd>",
        a.ef("status")?,
        n.amends
    );
    // 図の数は 1 枚以上のときだけ（図なしの面は便 32 までと byte 不変）
    if n.figures > 0 {
        dl.push_str(&format!("<dt>figures</dt><dd>{}</dd>", n.figures));
    }
    f.foot(o, id, &date, &dl);
    Ok(())
}
