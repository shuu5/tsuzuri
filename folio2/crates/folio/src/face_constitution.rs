//! 憲法の面の生成器（便 14・docs/design/delivery-14.md §1 (b)）。見本 `preview/constitution.html` の骨格（部品・要素の
//! 入れ子・class の並び）に、正本 4 file（constitution・rules・vocabulary・srs）の値を escape して差し込む。
//! 生成物の文字列は 正本の値（α）・名札の表（β・`face.rs`）・正本から数えた数（γ）のどれかで、部品の名札は
//! 部品目録から組み立て時に導出した `Component` の関数 name からだけ出す（ADR-5 決定 (3)）。
//! 面に依らない口（head と site-bar・章の帯・card・toc・foot・部品の名札）は `face.rs` の `Frame` を呼ぶ（便 15）。
//! 読み手（正本を読んで面の文脈と節の中身を組む側・条と文脈と改訂の段の型・読みと検査・欠番・関係の欄・改訂の段の読み）は
//! 便 116 で `face_constitution_read.rs`（層 4）へ降ろした。面の口と HTML を書く側と名札と数の口はここに残す。

use std::path::Path;
use std::sync::LazyLock;

use crate::catalog::Component;
use crate::constitution_enums as ce;
use crate::cursor::{self, R, X, esc};
use crate::face::{
    self, DOC_STATUS, Frame, MAX_RAIL_NODES, Tier, anchor, binds_label, card, count_word,
    hint, hint_q, mechanism_kind_label, mechanism_live_label, mechanism_live_meaning,
    pattern_label, polarity_label, rationale, retreat_kind_label, rule_kind_label,
    rule_status_class, stage_label, strength_label, strength_meaning, val,
};
use crate::face_constitution_read::{
    Amend, Approved, Art, Ctx, Step, amendments, approved, check_counts, context,
    missing_numbers, relations, step,
};
use crate::rules;

/// 憲法の面が使う部品（15 種・便 40 で ceiling-stamp を足した）。
pub const PARTS: [Component; 15] = [
    Component::FreshnessStamp,
    Component::CeilingStamp,
    Component::FontSizeControl,
    Component::DocCoverBand,
    Component::ChapterDeckBand,
    Component::SectionLeadCallout,
    Component::ItemRow,
    Component::PrincipleAmendmentHistory,
    Component::FigurePanel,
    Component::PipelineRail,
    Component::RailNode,
    Component::Stepper,
    Component::AmendmentExample,
    Component::GlossaryTermTable,
    Component::ApprovalBlock,
];

/// 章 00〜08 の帯の class と kicker の絵記号（見本の各章の字面）。
const BANDS: [(&str, &str); 9] = [
    (
        "band-1",
        "<path d=\"M12 3l2.6 6.2L21 10l-5 4.3L17.5 21 12 17.6 6.5 21 8 14.3 3 10l6.4-.8z\"/>",
    ),
    (
        "band-5",
        "<path d=\"M4 19V5h16v14z\"/><path d=\"M8 9h8M8 13h5\"/>",
    ),
    ("band-2", "<path d=\"M20 6L9 17l-5-5\"/>"),
    (
        "band-3",
        "<circle cx=\"12\" cy=\"12\" r=\"9\"/><path d=\"M12 8v4M12 16h.01\"/>",
    ),
    (
        "band-4",
        "<circle cx=\"12\" cy=\"12\" r=\"9\"/><path d=\"M5.6 5.6l12.8 12.8\"/>",
    ),
    ("band-6", "<path d=\"M4 6h16M4 12h16M4 18h10\"/>"),
    (
        "band-5",
        "<path d=\"M4 20h4l10-10-4-4L4 16z\"/><path d=\"M12 6l4 4\"/>",
    ),
    (
        "band-1",
        "<path d=\"M4 19.5A2.5 2.5 0 0 1 6.5 17H20\"/><path d=\"M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z\"/>",
    ),
    (
        "band-5",
        "<path d=\"M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1\"/><path d=\"M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1\"/>",
    ),
];

/// 憲法の面の骨格。
static FRAME: LazyLock<Frame> = LazyLock::new(|| Frame {
    name: "憲法",
    source: "constitution.yaml",
    favicon: "<link rel=\"icon\" href=\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect width='32' height='32' rx='7' fill='%231e7b65'/%3E%3Ctext x='16' y='22' font-size='16' font-weight='700' text-anchor='middle' fill='%23ffffff' font-family='sans-serif'%3E憲%3C/text%3E%3C/svg%3E\">",
    current: 1,
    first: 0,
    bands: &BANDS,
    prev: face::link("index.html", "入口"),
    next: face::link("srs.html", "要件書"),
    parts: &PARTS,
});

/// 属性 data-component（名札は憲法の面の部品の一覧からだけ出す）。
fn dc(c: Component) -> String {
    FRAME.dc(c)
}

/// 正本 → 憲法の面の HTML（決定的）。天井の名札は印から読む（便 40・便 83）。
pub fn derive(dir: &Path) -> R<String> {
    let c_doc = cursor::load(dir, "constitution.yaml")?;
    let r_doc = cursor::load(dir, "rules.yaml")?;
    let v_doc = cursor::load(dir, "vocabulary.yaml")?;
    let s_doc = cursor::load(dir, "srs.yaml")?;
    let c = X::root(&c_doc, "constitution.yaml");
    let r = X::root(&r_doc, "rules.yaml");
    let v = X::root(&v_doc, "vocabulary.yaml");
    let s = X::root(&s_doc, "srs.yaml");

    let ctx = context(&c, &r, &s)?;
    let m = c.f("meta")?;
    check_counts(&ctx, &m.f("counts")?)?;
    let stamp = face::ceiling_stamp(dir)?;
    // 版を上げた発効した判断の行と今の版の承認（便 144）
    let rows = amendments(dir)?;
    let ap = approved(&m, &rows)?;

    let mut o: Vec<String> = Vec::new();
    head(&mut o, &m, &ap, &stamp)?;
    cover(&mut o, &ctx, &c, &m, &ap)?;
    toc(&mut o, &ctx, &c, &r)?;
    north_star(&mut o, &c)?;
    reading(&mut o, &ctx, &v)?;
    for (i, (key, tier)) in ctx.tiers.iter().enumerate() {
        tier_chapter(&mut o, &ctx, i, key, tier)?;
    }
    rules_chapter(&mut o, &c, &r)?;
    amendment_chapter(&mut o, &ctx, &c, &m)?;
    glossary_chapter(&mut o, &c, &v)?;
    sources_chapter(&mut o, &c)?;
    approval(&mut o, &m, &ap, &rows)?;
    foot(&mut o, &ctx, &m)?;
    // 本文の判断の記録の番号を判断の記録の面へのリンクに（便 135・要件書の面と同じ 1 つの口）
    Ok(face::link_ids(&format!("{}\n", o.join("\n")), false, |id| {
        face::adr_face(dir, id)
    }))
}

/// 段の名を「・」で繋いだもの。
fn tier_names(ctx: &Ctx<'_>) -> String {
    ctx.tiers
        .iter()
        .map(|(_, t)| t.name)
        .collect::<Vec<_>>()
        .join("・")
}

fn tier_count(ctx: &Ctx<'_>, key: &str) -> usize {
    ctx.arts.iter().filter(|a| a.tier_key == key).count()
}

/// 規範文の末尾に出る札（MUST / MUST NOT / SHOULD）の意味を言う凡例 1 行（便 63・読みやすさ F-1）。
/// 札の字も意味の字も `face.rs` の名札の表から組み、この file に写しを持たない（1 か所の正本・P-6.3）。
/// 並びは導出した型の ALL の順（= 憲法の値域の file の順・要件書の面 §3 の凡例と同じ）。
fn tier_legend() -> String {
    let words = ce::Strength::ALL
        .iter()
        .map(|s| format!("{} = {}", strength_label(*s), strength_meaning(*s)))
        .collect::<Vec<_>>()
        .join("／");
    format!("<div class=\"legend-line\"><span>凡例:</span><span>{words}</span></div>")
}

/// 章の名（00〜08）。
fn chapter_name(ctx: &Ctx<'_>, i: usize) -> &'static str {
    match i {
        0 => "北極星",
        1 => "読み方",
        2..=4 => ctx.tiers[i - 2].1.name,
        5 => "数値の表（rules）",
        6 => "改訂",
        7 => "用語集",
        _ => "出所",
    }
}

// ── 骨格 ──

fn head(o: &mut Vec<String>, m: &X<'_>, ap: &Approved, stamp: &str) -> R<()> {
    let version = m.ef("version")?;
    let status = m.f("status")?.lookup(DOC_STATUS, "文書の状態")?;
    // 今の版を名指す発効した判断が無ければ鮮度の札に「まだ分からない」（便 144）
    let (shown, label) = ap.standing.stamp(&version, status);
    // 日付は今の版の承認（draft は生成日・便 145）
    let approved = (ap.standing != face::Standing::Draft).then(|| ap.date.clone());
    let (dated, date) = face::dated(m, approved)?;
    FRAME.head_dated(
        o,
        &format!("folio2 — 憲法（不変原則・{version}）"),
        (dated, &date),
        &shown,
        &label,
        stamp,
    );
    Ok(())
}

fn cover(o: &mut Vec<String>, ctx: &Ctx<'_>, c: &X<'_>, m: &X<'_>, ap: &Approved) -> R<()> {
    let ns = c.f("north_star")?;
    let total = ctx.arts.len();
    o.push(format!("<header {}>", dc(Component::DocCoverBand)));
    o.push("<p class=\"cover-eyebrow\"><span class=\"doc-type\">憲法 (Constitution)</span> <span>folio2 — 不変原則</span></p>".to_string());
    o.push(format!(
        "<h1>folio2 の憲法 — {total} の約束を「{}」の {} 段で</h1>",
        tier_names(ctx),
        ctx.tiers.len()
    ));
    o.push(format!("<p class=\"cover-sub\">{}</p>", ns.ef("for_whom")?));
    o.push(format!(
        "<div class=\"summary-card\"><span class=\"ic\">北</span><div><p class=\"lab\">北極星（1 文）</p><p class=\"txt\">{}</p></div></div>",
        ns.ef("statement")?
    ));
    o.push("<div class=\"cover-meta\">".to_string());
    o.push(format!(
        "<span class=\"m\"><span class=\"k\">原則の総数</span><span class=\"v\">{total} 件</span></span>"
    ));
    let breakdown = ctx
        .tiers
        .iter()
        .enumerate()
        .map(|(i, (key, t))| {
            format!(
                "<a href=\"#s{}\">{}&nbsp;{}</a>",
                i + 2,
                t.name,
                tier_count(ctx, key)
            )
        })
        .collect::<Vec<_>>()
        .join(" · ");
    o.push(format!(
        "<span class=\"m\"><span class=\"k\">内訳</span><span class=\"v\">{breakdown}</span></span>"
    ));
    o.push(format!(
        "<span class=\"m\"><span class=\"k\">版</span><span class=\"v\">{} / {}</span></span>",
        m.ef("version")?,
        ap.date
    ));
    o.push("</div>".to_string());
    // 承認の日付は今の版の承認・今の版を名指す判断の番号を添える（便 144）
    let state = if is_effective(m)? {
        let decisions = if ap.ids.is_empty() {
            String::new()
        } else {
            format!("・判断の記録 {}", ap.ids.join("・"))
        };
        ap.standing.cover(
            &m.ef("version")?,
            format!("発効・拘束力あり（承認 {}{decisions}）", ap.date),
        )
    } else {
        "未承認のため拘束力なし → 持ち主の承認で発効".to_string()
    };
    o.push(format!(
        "<p class=\"cover-status\"><span class=\"k\">状態</span><span>{state}（<a href=\"#approval\">承認欄へ</a>）</span></p>"
    ));
    o.push("</header>".to_string());
    Ok(())
}

fn is_effective(m: &X<'_>) -> R<bool> {
    let status = m.f("status")?;
    status.lookup(DOC_STATUS, "文書の状態")?;
    Ok(status.v.as_str() == Some("effective"))
}

/// 各章の h2 の字面（00〜08）。
fn chapter_h2(ctx: &Ctx<'_>, i: usize, c: &X<'_>, r: &X<'_>) -> R<String> {
    Ok(match i {
        0 => "何のために・誰のために・何をあきらめるか".to_string(),
        1 => format!("{} 段の意味 — {}", ctx.tiers.len(), tier_names(ctx)),
        2..=4 => count_word(tier_count(ctx, &ctx.tiers[i - 2].0), "原則"),
        5 => format!(
            "数値と作法の表 — 閾値行 {}・開発規律行 {}",
            r.f("thresholds")?.seq()?.len(),
            r.f("discipline")?.seq()?.len()
        ),
        6 => format!(
            "変えるときの手続き — {} 段",
            c.f("amendment")?.f("steps")?.seq()?.len()
        ),
        7 => "本文に出てくる専門語のやさしい説明".to_string(),
        _ => "この憲法はどこから来たか".to_string(),
    })
}

fn toc(o: &mut Vec<String>, ctx: &Ctx<'_>, c: &X<'_>, r: &X<'_>) -> R<()> {
    let heads = (0..BANDS.len())
        .map(|i| Ok((chapter_name(ctx, i).to_string(), chapter_h2(ctx, i, c, r)?)))
        .collect::<R<Vec<_>>>()?;
    FRAME.toc(o, &heads, "作成 / 承認");
    Ok(())
}

// ── 章 ──

fn north_star(o: &mut Vec<String>, c: &X<'_>) -> R<()> {
    let ns = c.f("north_star")?;
    let pr = c.f("precedence")?;
    let lead = format!("達成の判定: {}", ns.ef("judged_by")?);
    FRAME.band(
        o,
        0,
        "北極星",
        "何のために・誰のために・何をあきらめるか",
        Some(&lead),
    );
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!(
        "<div {} style=\"--band-n:3\">",
        dc(Component::SectionLeadCallout)
    ));
    o.push(card(
        "card accent brand",
        None,
        "北極星（1 文）",
        &format!("<p class=\"ct\">{}</p>", ns.ef("statement")?),
    ));
    o.push(card(
        "card accent",
        None,
        "誰のため",
        &format!("<p class=\"ct\">{}</p>", ns.ef("for_whom")?),
    ));
    o.push(card(
        "card accent",
        None,
        "迷ったらどちらへ倒すか（前文）",
        &format!(
            "<p class=\"ct\">{}</p><p class=\"cd\">{}</p><p class=\"meta-chips\">{}</p>",
            pr.ef("text")?,
            pr.ef("plain")?,
            hint("根拠", &rationale(&pr.f("rationale")?)?)
        ),
    ));
    o.push("</div>".to_string());
    o.push("</div>".to_string());
    Ok(())
}

fn reading(o: &mut Vec<String>, ctx: &Ctx<'_>, v: &X<'_>) -> R<()> {
    let mut lead = None;
    if let Some(ft) = v.g("field_terms")? {
        for t in ft.seq()? {
            let def = t.ef("def")?;
            if t.f("id")?.text()? == "tier" && lead.is_none() {
                lead = Some(def);
            }
        }
    }
    let h2 = format!("{} 段の意味 — {}", ctx.tiers.len(), tier_names(ctx));
    FRAME.band(o, 1, "読み方", &h2, lead.as_deref());
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!(
        "<div {} style=\"--band-n:{}\">",
        dc(Component::SectionLeadCallout),
        ctx.tiers.len()
    ));
    for (_, t) in &ctx.tiers {
        o.push(card(
            &format!("card accent {}", t.color),
            None,
            &format!("{}（{}）", t.name, t.en),
            &format!(
                "<p class=\"ct\">{}</p><p class=\"cd-req\"><b>外すのに要るもの:</b> {}</p>",
                t.meaning, t.remove_html
            ),
        ));
    }
    o.push("</div>".to_string());
    // 条の欠番の行（P-7.2・欠番が無ければ置かない・便 80 §1 (a)）
    let gaps = missing_numbers(ctx)?;
    if !gaps.is_empty() {
        o.push(format!(
            "<div class=\"legend-line\"><span>欠番: {}（条の廃止は状態で表し、番号は空けたままにする — <a class=\"xref\" href=\"#{}\">P-7</a>）</span></div>",
            gaps.join("・"),
            anchor("P-7")
        ));
    }
    o.push("</div>".to_string());
    Ok(())
}

fn tier_chapter(o: &mut Vec<String>, ctx: &Ctx<'_>, i: usize, key: &str, tier: &Tier) -> R<()> {
    let h2 = count_word(tier_count(ctx, key), "原則");
    FRAME.band(o, i + 2, tier.name, &h2, Some(tier.meaning));
    o.push("<div class=\"chapbody\">".to_string());
    // 札の凡例は 3 つの段の章の最初の 1 か所だけ（同じ札が続く章で繰り返さない・便 63）
    if i == 0 {
        o.push(tier_legend());
    }
    o.push("<div class=\"stack\">".to_string());
    for art in ctx.arts.iter().filter(|a| a.tier_key == key) {
        item_row(o, ctx, art)?;
    }
    o.push("</div>".to_string());
    o.push("</div>".to_string());
    Ok(())
}

fn item_row(o: &mut Vec<String>, ctx: &Ctx<'_>, art: &Art<'_>) -> R<()> {
    let a = &art.x;
    let t = art.tier;
    let binds = a.f("binds")?;
    binds_label(binds.parse(ce::Binds::from_name, "縛る相手")?);
    o.push(format!(
        "<article {} class=\"{}\" id=\"{}\">",
        dc(Component::ItemRow),
        t.class,
        anchor(art.id)
    ));
    let badge = format!(
        "<span class=\"hint tier\"><label><input type=\"checkbox\" class=\"vh\" aria-label=\"段「{n}」の意味を開く\"><span class=\"tier-badge {class}\">{n}</span></label><span class=\"hint-body\"><span class=\"hk\">この段 · {n}</span>{meaning}。外すのに要るもの: {remove} <a class=\"xref\" href=\"#s1\">§1 で {count} 段を見る →</a></span></span>",
        n = t.name,
        class = t.class,
        meaning = t.meaning,
        remove = t.remove,
        count = ctx.tiers.len()
    );
    o.push(format!(
        "<div class=\"ir-head\"><span class=\"rid\">{}</span><h3 class=\"rt\">{}</h3><span class=\"badges\">{badge}</span></div>",
        art.id, art.title
    ));

    let mut patterns = Vec::new();
    for st in a.f("statements")?.seq()? {
        let sid = st.ef("id")?;
        let pattern = st.f("pattern")?;
        pattern_label(pattern.parse(ce::Pattern::from_name, "型")?);
        let strength = st.f("strength")?;
        let kw = strength_label(strength.parse(ce::Strength::from_name, "強度")?);
        o.push(format!(
            "<p class=\"norm\"><span class=\"ew\">{sid}</span> {} <span class=\"kw\">{kw}</span></p>",
            st.ef("text")?
        ));
        patterns.push(format!("{sid}: {} / {}", pattern.e()?, strength.e()?));
    }
    o.push(format!(
        "<div class=\"plain\"><span class=\"pk\">やさしく言うと</span>{}</div>",
        a.ef("plain")?
    ));

    // 小窓（根拠・関係・撤退条件・機構）
    let mut chips = vec![hint("根拠", &rationale(&a.f("rationale")?)?)];
    if let Some(rel) = a.g("relations")? {
        let links = relations(ctx, &rel)?;
        if !links.is_empty() {
            chips.push(hint("関係", &links.join("・")));
        }
    }
    if let Some(rt) = a.g("retreat")? {
        chips.push(hint(
            "撤退条件",
            &format!(
                "{}: {}",
                retreat_kind_label(
                    rt.f("kind")?
                        .parse(ce::RetreatKind::from_name, "撤退条件の種別")?
                ),
                rt.ef("condition")?
            ),
        ));
    }
    let mech = a.f("mechanism")?;
    let kind = mech.f("kind")?;
    let live = mech.f("live")?;
    let live_v = live.parse(ce::MechanismLive::from_name, "機構の live")?;
    let mut human = format!(
        "{}・{}（{}）",
        mechanism_kind_label(kind.parse(ce::MechanismKind::from_name, "機構")?),
        mechanism_live_label(live_v),
        mechanism_live_meaning(live_v)
    );
    let mut machine = format!("{} · live: {}", kind.e()?, live.e()?);
    if let Some(stage) = mech.g("stage")? {
        human.push_str(&format!(
            "・{}",
            stage_label(stage.parse(ce::Stage::from_name, "機構の stage")?)
        ));
        machine.push_str(&format!(" · stage: {}", stage.e()?));
    }
    if let Some(polarity) = mech.g("polarity")? {
        human.push_str(&format!(
            "・{}",
            polarity_label(polarity.parse(ce::Polarity::from_name, "機構の polarity")?)
        ));
        machine.push_str(&format!(" · polarity: {}", polarity.e()?));
    }
    if let Some(note) = mech.g("note")? {
        human.push_str(&format!(" — {}", note.e()?));
    }
    chips.push(hint("機構", &human));
    o.push(format!("<p class=\"meta-chips\">{}</p>", chips.concat()));

    // 改訂来歴
    let supersedes = a.g("supersedes_v1")?;
    let amended = match a.g("amended_by")? {
        Some(x) => x.seq()?,
        None => Vec::new(),
    };
    if supersedes.is_some() || !amended.is_empty() {
        let mut h = String::from("<span class=\"am-kick\">改訂来歴</span>");
        if let Some(sv) = &supersedes {
            h.push_str(&format!(
                "<span class=\"am-row\">{} <span class=\"am-meta\">{} の {} を置換（{}）</span></span>",
                sv.ef("ruling")?,
                sv.ef("doc")?,
                sv.ef("article")?,
                sv.ef("rationale")?
            ));
        }
        for am in &amended {
            let previous = am.ef("previous_text")?;
            let why = am.ef("rationale")?;
            h.push_str(&format!(
                "<span class=\"am-row\">{} <span class=\"am-meta\">{} · {} · 承認 {}</span>{}{}</span>",
                am.ef("adr")?,
                am.ef("date")?,
                am.ef("ruling")?,
                am.ef("approved_by")?,
                hint("前の文", &previous),
                hint("理由", &why)
            ));
        }
        o.push(format!(
            "<div {}>{h}</div>",
            dc(Component::PrincipleAmendmentHistory)
        ));
    }

    // 機械のための面（正本の値のまま）
    let mut dl = format!(
        "<dt>binds</dt><dd>{}</dd><dt>mechanism</dt><dd>{machine}</dd><dt>patterns</dt><dd>{}</dd>",
        binds.e()?,
        patterns.join(" · ")
    );
    if let Some(note) = a.g("note")? {
        dl.push_str(&format!("<dt>note</dt><dd>{}</dd>", note.e()?));
    }
    o.push(format!(
        "<details class=\"machine\" data-audience=\"machine\"><summary>機械のための面</summary><dl>{dl}</dl></details>"
    ));
    o.push("</article>".to_string());
    Ok(())
}

fn rules_chapter(o: &mut Vec<String>, c: &X<'_>, r: &X<'_>) -> R<()> {
    let thresholds = r.f("thresholds")?.seq()?;
    let discipline = r.f("discipline")?.seq()?;
    let h2 = format!(
        "数値と作法の表 — 閾値行 {}・開発規律行 {}",
        thresholds.len(),
        discipline.len()
    );
    FRAME.band(
        o,
        5,
        "数値の表（rules）",
        &h2,
        Some(&c.ef("rules_pointer")?),
    );
    o.push("<div class=\"chapbody\">".to_string());
    o.push("<div class=\"legend-line\"><span>状態: </span><span><span class=\"state ok\">凍結</span> = 値は動かさない（変えるなら裁定が要る）／<span class=\"state warn\">仮</span> = 値は入っているが確定前／<span class=\"state\">未定</span> = まだ値が無い</span></div>".to_string());
    // 凡例の種別の字は表の種別の欄と同じ名札（`face.rs` の rule_kind_label）から出し、正本の鍵は括弧に添える
    // （便 70・天井の 12 周目の読みやすさ F-5）。鍵が種別の値域に無ければ Err（fail-closed）。
    let kinds = r
        .f("schema")?
        .f("kind_meaning")?
        .pairs()?
        .iter()
        .map(|(k, x)| {
            let label = rules::RuleKind::from_name(k)
                .map(rule_kind_label)
                .ok_or_else(|| format!("{}: rules 行の種別の表に無い値「{k}」", x.at))?;
            Ok(format!("{label}（{}）: {}", esc(k), x.e()?))
        })
        .collect::<R<Vec<_>>>()?
        .join("／");
    o.push(format!(
        "<div class=\"legend-line\"><span>種別: </span><span>{kinds}</span></div>"
    ));

    o.push("<div class=\"tbl-wrap\"><table class=\"tbl\">".to_string());
    o.push("<thead><tr><th>id</th><th>何の数値か</th><th>値</th><th>種別</th><th>裁定</th><th>状態</th></tr></thead>".to_string());
    o.push("<tbody>".to_string());
    for x in &thresholds {
        let mut hints = String::new();
        for (key, label) in [
            ("note", "注"),
            ("population", "母集団"),
            ("projection", "写す範囲"),
            ("basis", "根拠"),
            ("same_failure", "同じ種類"),
        ] {
            if let Some(h) = x.g(key)? {
                hints.push(' ');
                hints.push_str(&hint(label, &h.e()?));
            }
        }
        o.push(format!(
            "<tr id=\"{}\"><td class=\"id\" data-k=\"id\">{}</td><td data-k=\"何の数値か\">{}{hints}</td><td class=\"val\" data-k=\"値\">{}</td><td data-k=\"種別\">{}・{}</td><td data-k=\"裁定\">{}</td><td data-k=\"状態\">{}</td></tr>",
            anchor(x.f("id")?.id()?),
            x.f("id")?.id()?,
            what_with_xref(x)?,
            value_cell(&x.f("value")?, 0)?,
            rule_kind_label(x.f("kind")?.parse(rules::RuleKind::from_name, "rules 行の種別")?),
            stage_label(x.f("stage")?.parse(ce::Stage::from_name, "rules 行の stage")?),
            ruling(x)?,
            state_chip(x)?
        ));
    }
    o.push("</tbody>".to_string());
    o.push("</table></div>".to_string());

    o.push("<div class=\"tbl-wrap\"><table class=\"tbl\">".to_string());
    o.push(
        "<thead><tr><th>id</th><th>作法</th><th>種別</th><th>裁定</th><th>状態</th></tr></thead>"
            .to_string(),
    );
    o.push("<tbody>".to_string());
    for x in &discipline {
        let note = match x.g("note")? {
            Some(n) => format!(" {}", hint("注", &n.e()?)),
            None => String::new(),
        };
        o.push(format!(
            "<tr id=\"{}\"><td class=\"id\" data-k=\"id\">{}</td><td data-k=\"作法\">{}{note}</td><td data-k=\"種別\">{}</td><td data-k=\"裁定\">{}</td><td data-k=\"状態\">{}</td></tr>",
            anchor(x.f("id")?.id()?),
            x.f("id")?.id()?,
            what_with_xref(x)?,
            rule_kind_label(x.f("kind")?.parse(rules::RuleKind::from_name, "rules 行の種別")?),
            ruling(x)?,
            state_chip(x)?
        ));
    }
    o.push("</tbody>".to_string());
    o.push("</table></div>".to_string());
    o.push("</div>".to_string());
    Ok(())
}

/// rules 行の what + 条への xref。
fn what_with_xref(x: &X<'_>) -> R<String> {
    let article = x.f("article")?.id()?;
    Ok(format!(
        "{} <a class=\"xref\" href=\"#{}\">→ {article}</a>",
        x.ef("what")?,
        anchor(article)
    ))
}

/// 閾値行の値の表に現れてよい鍵と、その日本語の小見出し（閉じた表・便 79・§1 (a)）。順は表引きにだけ使う。
const VALUE_KEY_LABELS: &[(&str, &str)] = &[
    ("marks", "規範の印"),
    ("prohibition", "「禁止」の扱い"),
    ("word", "語"),
    ("clause_ends", "直後の字"),
    ("units", "単位"),
];

/// 閾値行の値の枡（便 79・§1 (a)）。表は鍵ごとに「<小見出し>（<鍵>）」を組み、表でない値は `face::val` の字面のまま。
/// 鍵が VALUE_KEY_LABELS に無ければ Err（面は導出できない・P-4.1）。
fn value_cell(x: &X<'_>, d: usize) -> R<String> {
    if x.v.as_map().is_none() {
        return val(x, d);
    }
    let mut parts = Vec::new();
    for (k, vx) in x.pairs()? {
        let label = VALUE_KEY_LABELS
            .iter()
            .find(|(key, _)| *key == k)
            .map(|(_, label)| *label)
            .ok_or_else(|| format!("{}: rules 行の値の表に無い鍵「{k}」", vx.at))?;
        let body = if vx.v.as_map().is_some() {
            format!("<br>{}", value_cell(&vx, d + 1)?)
        } else {
            val(&vx, d + 1)?
        };
        parts.push(format!(
            "{}<b>{}（{}）</b>: {body}",
            "　".repeat(d),
            esc(label),
            esc(k)
        ));
    }
    Ok(parts.join("<br>"))
}

/// 裁定の欄で過去の裁定を連ねる区切りの字面（正本 rules.yaml で 1 つに揃っている・便 79・§1 (b)）。
const PREVIOUS_RULING: &str = "）・前の裁定 = ";

/// 裁定の枡（便 79・§1 (b)）。最新の 1 件だけを出し、過去の裁定は小窓「前の裁定 <N> 件」へ畳む。
fn ruling(x: &X<'_>) -> R<String> {
    let text = x.ef("ruling")?;
    let at = x.ef("ruled_at")?;
    let mut pieces = text.split(PREVIOUS_RULING);
    let latest = pieces.next().unwrap_or_default();
    let previous: Vec<&str> = pieces.collect();
    if previous.is_empty() {
        return Ok(format!("{text}（{at}）"));
    }
    let last = previous.len() - 1;
    let body = previous
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let close = if i < last { "）" } else { "" };
            format!("<p>前の裁定 = {p}{close}</p>")
        })
        .collect::<String>();
    Ok(format!(
        "{latest}）（{at}） {}",
        hint(&format!("前の裁定 {} 件", previous.len()), &body)
    ))
}

fn state_chip(x: &X<'_>) -> R<String> {
    let status = x.f("status")?;
    Ok(format!(
        "<span class=\"{}\">{}</span>",
        rule_status_class(status.parse(rules::RuleStatus::from_name, "rules 行の状態")?),
        status.e()?
    ))
}

fn stepper_li(no: &str, s: &Step<'_>) -> String {
    let refs = if s.arts.is_empty() {
        String::new()
    } else {
        format!(
            "（{}）",
            s.arts
                .iter()
                .map(|a| format!("<a class=\"xref\" href=\"#{}\">{}</a>", anchor(a.id), a.id))
                .collect::<Vec<_>>()
                .join("・")
        )
    };
    format!(
        "<li><span class=\"no\">{no}</span><div class=\"body\"><b>{}<span class=\"{}\">{}</span></b>{}{refs}</div></li>",
        s.head,
        if s.owner { "who owner" } else { "who" },
        s.who,
        s.tail.as_deref().unwrap_or("")
    )
}

fn amendment_chapter(o: &mut Vec<String>, ctx: &Ctx<'_>, c: &X<'_>, m: &X<'_>) -> R<()> {
    let am = c.f("amendment")?;
    let step_xs = am.f("steps")?.seq()?;
    if step_xs.len() > MAX_RAIL_NODES {
        return Err(format!(
            "{}: 改訂の段が {} で上限 {MAX_RAIL_NODES}（部品目録の pipeline-rail の max_nodes）を超える",
            am.at,
            step_xs.len()
        ));
    }
    let steps = step_xs
        .iter()
        .map(|x| step(ctx, x))
        .collect::<R<Vec<_>>>()?;
    let effective = step(ctx, &am.f("effective_step")?)?;
    let count = steps.len();

    let h2 = format!("変えるときの手続き — {count} 段");
    FRAME.band(o, 6, "改訂", &h2, Some(&am.ef("declaration")?));
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!(
        "<figure {} data-role=\"diagram\" id=\"fig-amend-flow\">",
        dc(Component::FigurePanel)
    ));
    let legend = "<div class=\"fig-legend\"><span class=\"lg\"><span class=\"sw warn\"></span>持ち主の手番</span><span class=\"lg\"><span class=\"sw neutral\"></span>AI・機械がやる</span><span class=\"lg\"><span class=\"sw line\"></span>番号の順に進む</span></div>";
    o.push(format!(
        "<div class=\"fig-title\"><span class=\"fn\">図 1</span>憲法が変わるときに通る道 — {count} 段・誰がやるか <span class=\"fig-tools\">{}<button class=\"zoom-btn\" type=\"button\">拡大</button></span><button class=\"zoom-close\" type=\"button\">✕ 閉じる</button></div>",
        hint("凡例", legend)
    ));
    o.push(format!(
        "<ol {} style=\"--rail-n:{count}\">",
        dc(Component::PipelineRail)
    ));
    for s in &steps {
        let nt = match &s.tail {
            Some(t) => format!("{} {}", s.head, hint_q(t)),
            None => s.head.clone(),
        };
        let reqs = s
            .arts
            .iter()
            .map(|a| {
                format!(
                    "<a class=\"fig-req\" href=\"#{}\"><span class=\"no\">{}</span>{}</a>",
                    anchor(a.id),
                    a.id,
                    a.title
                )
            })
            .collect::<String>();
        let node = format!(
            "<article {}{} id=\"a-step-{}\"><span class=\"actor\">担当: {}</span><p class=\"nt\">{nt}</p><p class=\"fig-reqs\">{reqs}</p></article>",
            dc(Component::RailNode),
            if s.owner { " class=\"tone-warn\"" } else { "" },
            s.n,
            s.who
        );
        o.push(format!(
            "<li class=\"rail-col\"><span class=\"rail-step\"><span class=\"no\">{}</span></span>",
            s.n
        ));
        if s.owner {
            o.push(format!("<div class=\"rail-slot filled\">{node}</div>"));
            o.push(
                "<div class=\"rail-slot\"><span class=\"lane-hint\">機械は待つ</span></div></li>"
                    .to_string(),
            );
        } else {
            o.push(
                "<div class=\"rail-slot\"><span class=\"lane-hint\">あなたの出番なし</span></div>"
                    .to_string(),
            );
            o.push(format!("<div class=\"rail-slot filled\">{node}</div></li>"));
        }
    }
    o.push("</ol>".to_string());
    o.push(format!(
        "<figcaption><span class=\"ver\">図 1 · {} {} · constitution.yaml</span></figcaption>",
        m.ef("version")?,
        m.ef("generated")?
    ));
    o.push("</figure>".to_string());

    o.push(format!("<ol {}>", dc(Component::Stepper)));
    o.push(stepper_li("0", &effective));
    for s in &steps {
        o.push(stepper_li(&s.n.to_string(), s));
    }
    o.push("</ol>".to_string());

    // 改訂の例（supersedes_v1 を持つ条・amended_by の項ごと・正本の順）
    for art in &ctx.arts {
        let a = &art.x;
        if let Some(sv) = a.g("supersedes_v1")? {
            let mut dl = format!(
                "<dt>出所</dt><dd>{}</dd><dt>裁定</dt><dd>{}</dd><dt>理由</dt><dd>{}</dd>",
                sv.ef("doc")?,
                sv.ef("ruling")?,
                sv.ef("rationale")?
            );
            if let Some(rt) = a.g("retreat")? {
                dl.push_str(&format!(
                    "<dt>撤退条件</dt><dd>{}</dd>",
                    rt.ef("condition")?
                ));
            }
            o.push(format!("<div {}>", dc(Component::AmendmentExample)));
            o.push(format!(
                "<div class=\"ax-k\">改訂の例 — {}（{}）</div>",
                art.id,
                sv.ef("ruling")?
            ));
            o.push(format!(
                "<p class=\"ax-text\"><span class=\"lbl\">消す文</span><del>{}</del><br><span class=\"lbl\">足す文</span><ins>{}: {}</ins></p>",
                sv.ef("article")?,
                art.id,
                art.title
            ));
            o.push(format!("<dl>{dl}</dl>"));
            o.push("</div>".to_string());
        }
        if let Some(amended) = a.g("amended_by")? {
            for ab in amended.seq()? {
                o.push(format!("<div {}>", dc(Component::AmendmentExample)));
                o.push(format!(
                    "<div class=\"ax-k\">改訂 — {}（{}）</div>",
                    art.id,
                    ab.ef("adr")?
                ));
                o.push(format!(
                    "<p class=\"ax-text\"><span class=\"lbl\">消す文</span><del>{}</del><br><span class=\"lbl\">足す文</span><ins><a class=\"xref\" href=\"#{}\">現行の {}</a></ins></p>",
                    ab.ef("previous_text")?,
                    anchor(art.id),
                    art.id
                ));
                o.push(format!(
                    "<dl><dt>判断の記録</dt><dd>{}</dd><dt>承認</dt><dd>{}（{}）</dd><dt>裁定</dt><dd>{}</dd><dt>理由</dt><dd>{}</dd></dl>",
                    ab.ef("adr")?,
                    ab.ef("approved_by")?,
                    ab.ef("date")?,
                    ab.ef("ruling")?,
                    ab.ef("rationale")?
                ));
                o.push("</div>".to_string());
            }
        }
    }

    // 版ごとの変更点（meta のキーのうち changes_from_ で始まるものを文字列の昇順に）
    let mut changes: Vec<(&str, X<'_>)> = m
        .pairs()?
        .into_iter()
        .filter(|(k, _)| k.starts_with("changes_from_"))
        .collect();
    changes.sort_by(|a, b| a.0.cmp(b.0));
    for (key, x) in &changes {
        let items = x.seq()?;
        let lis = items
            .iter()
            .map(|i| Ok(format!("<li>{}</li>", i.e()?)))
            .collect::<R<Vec<_>>>()?
            .concat();
        o.push(format!(
            "<details class=\"note\"><summary>{} からの変更（{} 件）</summary><div><ul>{lis}</ul></div></details>",
            esc(&key["changes_from_".len()..].replace('_', ".")),
            items.len()
        ));
    }
    o.push("</div>".to_string());
    Ok(())
}

fn glossary_chapter(o: &mut Vec<String>, c: &X<'_>, v: &X<'_>) -> R<()> {
    FRAME.band(
        o,
        7,
        "用語集",
        "本文に出てくる専門語のやさしい説明",
        Some(&c.ef("glossary_pointer")?),
    );
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!("<div {}>", dc(Component::GlossaryTermTable)));
    // 語の行は要件書の面の章 08 と共有（便 36・`face.rs`）
    face::glossary_rows(o, v, "terms")?;
    o.push("</div>".to_string());
    // 欄の名前の節も要件書の面の章 08 と同じ字面（便 84）
    face::glossary_field_terms(o, v, &dc(Component::GlossaryTermTable))?;
    o.push("</div>".to_string());
    Ok(())
}

fn sources_chapter(o: &mut Vec<String>, c: &X<'_>) -> R<()> {
    let sources = c.f("sources")?.seq()?;
    FRAME.band(o, 8, "出所", "この憲法はどこから来たか", None);
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!(
        "<div {} style=\"--band-n:{}\">",
        dc(Component::SectionLeadCallout),
        sources.len()
    ));
    for x in &sources {
        o.push(card(
            "card accent",
            None,
            &format!("出所 {}", x.ef("n")?),
            &format!(
                "<p class=\"ct\">{}</p><p class=\"cd\">{}</p>",
                x.ef("name")?,
                x.ef("ref")?
            ),
        ));
    }
    o.push("</div>".to_string());
    o.push("</div>".to_string());
    Ok(())
}

fn approval(o: &mut Vec<String>, m: &X<'_>, now: &Approved, rows: &[Amend]) -> R<()> {
    let ap = m.f("approval")?;
    let status = m.f("status")?.lookup(DOC_STATUS, "文書の状態")?;
    let stamp = if is_effective(m)? {
        "<span class=\"stamp\">発効</span>"
    } else {
        "<span class=\"stamp todo\">未承認</span>"
    };
    FRAME.approval_band(
        o,
        "作成 / 承認",
        &now.standing.lead(&m.ef("version")?, status),
    );
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!("<div {}>", dc(Component::ApprovalBlock)));
    o.push(format!(
        "<div class=\"sign\"><span class=\"role\">作成</span><span class=\"who\">{}</span><span class=\"when\">{}</span><span class=\"stamp self\">起草</span></div>",
        m.ef("author")?,
        m.ef("generated")?
    ));
    o.push(format!(
        "<div class=\"sign\"><span class=\"role\">承認</span><span class=\"who\">{}</span><span class=\"when\">{} · 逐語「{}」</span><span class=\"when\">裁定: {}／対話面: {}</span>{stamp}</div>",
        ap.ef("who")?,
        ap.ef("date")?,
        ap.ef("verbatim")?,
        ap.ef("ruling")?,
        ap.ef("surface")?
    ));
    // 版ごとの承認の行（版を上げた発効した判断の承認欄から逐語で・判断の番号の順・便 144）
    for a in rows {
        o.push(format!(
            "<div class=\"sign\"><span class=\"role\">承認</span><span class=\"who\">{}</span><span class=\"when\">{} · 版 {} · 逐語「{}」</span><span class=\"when\">判断の記録: {}／裁定: {}／対話面: {}</span><span class=\"stamp\">発効</span></div>",
            a.who, a.date, a.version, a.verbatim, a.id, a.ruling, a.surface
        ));
    }
    o.push("</div>".to_string());
    o.push("</div>".to_string());
    Ok(())
}

fn foot(o: &mut Vec<String>, ctx: &Ctx<'_>, m: &X<'_>) -> R<()> {
    let version = m.ef("version")?;
    let generated = m.ef("generated")?;
    let mut dl = format!(
        "<dt>id</dt><dd>{}</dd><dt>version</dt><dd>{version}</dd><dt>status</dt><dd>{}</dd>",
        m.ef("id")?,
        m.ef("status")?
    );
    for key in ["binding", "baseline"] {
        if let Some(x) = m.g(key)? {
            dl.push_str(&format!("<dt>{key}</dt><dd>{}</dd>", x.raw()?));
        }
    }
    let items = ctx
        .arts
        .iter()
        .map(|a| a.id)
        .collect::<Vec<_>>()
        .join(" / ");
    dl.push_str(&format!("<dt>items</dt><dd>{items}</dd>"));
    FRAME.foot(o, &version, &generated, &dl);
    Ok(())
}

#[cfg(test)]
mod face_constitution_tests {
    use super::*;

    #[test]
    fn face_parts_are_all_allowed_on_the_constitution_face() {
        for part in PARTS {
            assert!(
                part.faces().contains(&"constitution"),
                "{} は憲法の面に置けない",
                part.name()
            );
        }
        assert!(!PARTS.contains(&Component::LaneChip));
        let mut names: Vec<&str> = PARTS.iter().map(|p| p.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 15);
    }

    /// 版を上げた判断の行（who・逐語・裁定・対話面は見ない字）。
    fn row(id: &str, version: &str, date: &str) -> Amend {
        let x = "x".to_string();
        Amend {
            id: id.to_string(),
            version: version.to_string(),
            who: x.clone(),
            date: date.to_string(),
            verbatim: x.clone(),
            ruling: x.clone(),
            surface: x,
        }
    }

    fn approved_of(meta: &str, rows: &[Amend]) -> R<Approved> {
        let v = crate::yaml::parse_typed(meta).unwrap();
        approved(&X::root(&v, "meta"), rows)
    }

    #[test]
    fn f144_approved_follows_the_decisions_naming_the_version() {
        use crate::face::Standing;
        let meta = |status: &str, version: &str, first: &str| {
            format!(
                "{{version: \"{version}\", status: {status}, generated: 2026-09-01, approval: {{date: {first}}}}}"
            )
        };
        let ok = |m: &str, rows: &[Amend]| approved_of(m, rows).unwrap();
        let want = |standing: Standing, date: &str, ids: &[&str]| Approved {
            standing,
            date: date.to_string(),
            ids: ids.iter().map(|s| s.to_string()).collect(),
        };
        let effective = meta("effective", "v1.1", "2026-09-02");
        // 行の無い発効の正本 = 初回の承認
        assert_eq!(
            ok(&effective, &[]),
            want(Standing::Effective, "2026-09-02", &[])
        );
        // 今の版を名指す行が 2 本 = 最も新しい日付（最初の行ではない）と id 2 つ（行の順）
        let rows = [
            row("ADR-2", "v1.0", "2026-09-09"),
            row("ADR-3", "v1.1", "2026-09-05"),
            row("ADR-4", "v1.1", "2026-09-07"),
        ];
        assert_eq!(
            ok(&effective, &rows),
            want(Standing::Effective, "2026-09-07", &["ADR-3", "ADR-4"])
        );
        // 名指す行が無い = Unknown と在る承認の最も新しい日付（初回の承認を含む）
        assert_eq!(
            ok(&effective, &[row("ADR-2", "v1.0", "2026-09-03")]),
            want(Standing::Unknown, "2026-09-03", &[])
        );
        assert_eq!(
            ok(&effective, &[row("ADR-2", "v1.0", "2026-09-01")]),
            want(Standing::Unknown, "2026-09-02", &[])
        );
        // draft は行を読まず生成日
        assert_eq!(
            ok(&meta("draft", "v1.1", "2026-09-02"), &rows),
            want(Standing::Draft, "2026-09-01", &[])
        );
        // 版の字は escape した字どうしで比べる
        assert_eq!(
            ok(
                &meta("effective", "<v1.1>", "2026-09-02"),
                &[row("ADR-5", "&lt;v1.1&gt;", "2026-09-08")]
            ),
            want(Standing::Effective, "2026-09-08", &["ADR-5"])
        );
        assert_eq!(
            ok(
                &meta("effective", "<v1.1>", "2026-09-02"),
                &[row("ADR-5", "<v1.1>", "2026-09-08")]
            )
            .standing,
            Standing::Unknown
        );
        // 表の外の状態は Err
        let e = approved_of(&meta("retired", "v1.1", "2026-09-02"), &rows).unwrap_err();
        assert_eq!(e, "meta.status: 文書の状態 の表に無い値「retired」");
    }
}
