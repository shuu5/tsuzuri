//! 判断の記録の面の生成器（便 25・docs/design/delivery-25.md §1 (b)〜(d)）。判断の記録の正本 1 本
//! （`adr/ADR-n.yaml`）から人が読むページ 1 枚を組む。生成物の文字列は 正本の値（α・escape して逐語）・
//! 名札の表（β・この file の表）・正本から数えた数（γ）のどれかで、案内の文は出さない。
//! 面に依らない口（head と site-bar・章の帯・card・toc・承認欄の帯・foot・部品の名札）は `face.rs` の `Frame` を呼ぶ。
//! 章の h2 のうち契約が名指すのは 01・02 と承認欄だけなので、03〜05 は章の名をそのまま h2 に出す（新しい字面を持たない）。

use std::path::Path;

use crate::face::{self, Frame, R, X, anchor, card};
use crate::parts::catalog::Component;

/// 判断の記録の面が使う部品（7 種）。
pub const PARTS: [Component; 7] = [
    Component::FreshnessStamp,
    Component::FontSizeControl,
    Component::DocCoverBand,
    Component::ApprovalBlock,
    Component::ChapterDeckBand,
    Component::SectionLeadCallout,
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

/// 章 01〜05 の帯の class と kicker の絵記号（絵記号は要件書の面の章 01〜05 の写し）。
const BANDS: [(&str, &str); 5] = [
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
];

/// 判断の記録の面の骨格。nav の aria-current は付かない（読める面の nav に判断の記録は無い）。
const FRAME: Frame = Frame {
    name: "判断の記録",
    source: "adr/ADR-n.yaml",
    favicon: "<link rel=\"icon\" href=\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect width='32' height='32' rx='7' fill='%235f45a6'/%3E%3Ctext x='16' y='22' font-size='16' font-weight='700' text-anchor='middle' fill='%23ffffff' font-family='sans-serif'%3E要%3C/text%3E%3C/svg%3E\">",
    current: 3,
    first: 1,
    bands: &BANDS,
    prev: ("srs.html", "要件書"),
    next: ("index.html", "入口"),
    parts: &PARTS,
};

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

fn dc(c: Component) -> String {
    FRAME.dc(c)
}

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
}

/// 根拠の id の行き先を解くための、他の正本の id の一覧。
struct Ctx {
    /// 条 id
    articles: Vec<String>,
    /// 規範文 id → その条の id
    statements: Vec<(String, String)>,
    /// rules 行の id
    rules: Vec<String>,
    /// 要件書の id（要件・非機能要件・受入基準・制約・ゴール）
    reqs: Vec<String>,
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
    let counts = counts(&a)?;

    let mut o: Vec<String> = Vec::new();
    head(&mut o, &a, id, &st)?;
    cover(&mut o, &a, id, &st, &counts)?;
    toc(&mut o);
    prose_chapter(&mut o, 1, &a.ef("context")?);
    prose_chapter(&mut o, 2, &a.ef("decision")?);
    options_chapter(&mut o, &a)?;
    basis_chapter(&mut o, &a, dir, &ctx)?;
    amends_chapter(&mut o, &a, dir, &ctx)?;
    approval_chapter(&mut o, &a, &st)?;
    foot(&mut o, &a, id, &counts)?;
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

fn context(c: &X<'_>, r: &X<'_>, s: &X<'_>) -> R<Ctx> {
    let mut articles = Vec::new();
    let mut statements = Vec::new();
    for a in c.f("articles")?.seq()? {
        let aid = a.f("id")?.id()?.to_string();
        for st in a.f("statements")?.seq()? {
            statements.push((st.f("id")?.id()?.to_string(), aid.clone()));
        }
        articles.push(aid);
    }
    let mut rules = Vec::new();
    for section in ["thresholds", "discipline"] {
        for x in r.f(section)?.seq()? {
            rules.push(x.f("id")?.id()?.to_string());
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
            reqs.push(x.f("id")?.id()?.to_string());
        }
    }
    Ok(Ctx {
        articles,
        statements,
        rules,
        reqs,
    })
}

/// 根拠の id の行き先（在れば href・無ければ None）。4 形のどれでもない id は Err。
fn resolve(dir: &Path, ctx: &Ctx, id: &str) -> R<Option<String>> {
    let bad = || format!("根拠の id「{id}」は id の形でない（条・rules 行・要件・判断の記録）");
    let found = |ids: &[String], href: String| ids.iter().any(|k| k == id).then_some(href);
    if let Some(rest) = ["P-", "A-", "N-"].iter().find_map(|p| id.strip_prefix(p)) {
        return match rest.split_once('.') {
            // 枝番付きの規範文 id は条の anchor へ（字は枝番付きのまま）
            Some((n, sub)) if digits(n) && digits(sub) => Ok(ctx
                .statements
                .iter()
                .find(|(sid, _)| sid == id)
                .map(|(_, aid)| format!("constitution.html#{}", anchor(aid)))),
            None if digits(rest) => Ok(found(
                &ctx.articles,
                format!("constitution.html#{}", anchor(id)),
            )),
            _ => Err(bad()),
        };
    }
    if let Some(rest) = ["R-", "D-"].iter().find_map(|p| id.strip_prefix(p)) {
        return if digits(rest) {
            Ok(found(
                &ctx.rules,
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
            Ok(found(&ctx.reqs, format!("srs.html#{}", anchor(id))))
        } else {
            Err(bad())
        };
    }
    if let Some(rest) = id.strip_prefix("ADR-") {
        return if digits(rest) {
            let file = dir.join("adr").join(format!("{id}.yaml"));
            Ok(file.is_file().then(|| format!("{}.html", anchor(id))))
        } else {
            Err(bad())
        };
    }
    Err(bad())
}

/// 根拠の id 1 つ（在ればリンク・無ければ「（まだ分からない）」）。
fn id_link(dir: &Path, ctx: &Ctx, x: &X<'_>) -> R<String> {
    let id = x.id()?;
    Ok(match resolve(dir, ctx, id)? {
        Some(href) => format!("<a class=\"xref\" href=\"{href}\">{id}</a>"),
        None => format!("{id}（まだ分からない）"),
    })
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

fn counts(a: &X<'_>) -> R<Counts> {
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
    })
}

// ── 骨格 ──

fn head(o: &mut Vec<String>, a: &X<'_>, id: &str, st: &Status) -> R<()> {
    FRAME.head(
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

fn cover(o: &mut Vec<String>, a: &X<'_>, id: &str, st: &Status, n: &Counts) -> R<()> {
    o.push(format!("<header {}>", dc(Component::DocCoverBand)));
    o.push(format!(
        "<p class=\"cover-eyebrow\"><span class=\"doc-type\">判断の記録 (ADR)</span> <span>folio2 — {id}</span></p>"
    ));
    o.push(format!("<h1>{}</h1>", a.ef("title")?));
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
    o.push("</div>".to_string());
    o.push(format!(
        "<p class=\"cover-status\"><span class=\"k\">状態</span><span>{}（<a href=\"#approval\">承認欄へ</a>）</span></p>",
        st.line
    ));
    o.push("</header>".to_string());
    Ok(())
}

fn toc(o: &mut Vec<String>) {
    let heads = CHAPTERS
        .iter()
        .zip(H2)
        .map(|(k, t)| ((*k).to_string(), t.to_string()))
        .collect::<Vec<_>>();
    FRAME.toc(o, &heads, "承認");
}

fn band(o: &mut Vec<String>, n: usize) {
    FRAME.band(o, n, CHAPTERS[n - 1], H2[n - 1], None);
}

// ── 章 ──

/// 章 01・02（帯 + chapbody に p 1 つ）。`body` は escape 済み。
fn prose_chapter(o: &mut Vec<String>, n: usize, body: &str) {
    band(o, n);
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!("<p>{body}</p>"));
    o.push("</div>".to_string());
}

/// 章 03（案・採用と退けた案を同じ形で）。
fn options_chapter(o: &mut Vec<String>, a: &X<'_>) -> R<()> {
    band(o, 3);
    o.push("<div class=\"chapbody\">".to_string());
    for opt in a.f("options")?.seq()? {
        let oid = opt.f("id")?.id()?;
        let verdict = opt.f("verdict")?;
        o.push(format!(
            "<article {} id=\"opt-{oid}\">",
            dc(Component::ItemRow)
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

/// 章 04（根拠の id の並びと撤退条件）。
fn basis_chapter(o: &mut Vec<String>, a: &X<'_>, dir: &Path, ctx: &Ctx) -> R<()> {
    band(o, 4);
    o.push("<div class=\"chapbody\">".to_string());
    o.push("<ul>".to_string());
    for b in a.f("basis")?.seq()? {
        o.push(format!("<li>{}</li>", id_link(dir, ctx, &b)?));
    }
    o.push("</ul>".to_string());
    let rt = a.f("retreat")?;
    o.push(format!(
        "<div {} style=\"--band-n:4\">",
        dc(Component::SectionLeadCallout)
    ));
    o.push(card(
        "card",
        None,
        &format!(
            "撤退条件（{}）",
            rt.f("kind")?.lookup(RETREAT_KIND, "撤退条件の種類")?
        ),
        &format!("<p>{}</p>", rt.ef("condition")?),
    ));
    o.push("</div>".to_string());
    o.push("</div>".to_string());
    Ok(())
}

/// 章 05（改訂・帰結・反対側からの確認・置き換え・注）。
fn amends_chapter(o: &mut Vec<String>, a: &X<'_>, dir: &Path, ctx: &Ctx) -> R<()> {
    band(o, 5);
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

fn approval_chapter(o: &mut Vec<String>, a: &X<'_>, st: &Status) -> R<()> {
    FRAME.approval_band(o, "承認", st.label);
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!("<div {}>", dc(Component::ApprovalBlock)));
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

fn foot(o: &mut Vec<String>, a: &X<'_>, id: &str, n: &Counts) -> R<()> {
    let date = a.ef("date")?;
    let basis = a
        .f("basis")?
        .seq()?
        .iter()
        .map(X::e)
        .collect::<R<Vec<_>>>()?
        .join("・");
    let dl = format!(
        "<dt>id</dt><dd>{id}</dd><dt>status</dt><dd>{}</dd><dt>date</dt><dd>{date}</dd><dt>basis</dt><dd>{basis}</dd><dt>amends</dt><dd>{}</dd>",
        a.ef("status")?,
        n.amends
    );
    FRAME.foot(o, id, &date, &dl);
    Ok(())
}
