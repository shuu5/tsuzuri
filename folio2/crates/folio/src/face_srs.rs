//! 要件書の面の生成器（便 15・docs/design/delivery-15.md §1 (c)）。見本 `preview/srs.html` の骨格（部品・要素の入れ子・
//! class の並び）に、正本 4 file（srs・constitution・rules・vocabulary）の値を escape して差し込む。
//! 生成物の文字列は 正本の値（α）・名札の表（β・`face.rs` とこの file の章の表）・正本から数えた数（γ）のどれかで、
//! 見本にしか無い案内の文（副題・読者の札・lead と補足・lane-chip・図の説明の小窓）は出さない。
//! 面に依らない口（head と site-bar・章の帯・card・toc・foot・部品の名札）は `face.rs` の `Frame` を呼ぶ。
//! 図の章（便 34・FR15）: 正本に任意の図の節（figures・最上位）が 1 枚以上あれば章 09「図」を用語集の後・承認欄の
//! 前に置く。図の枠は `face.rs` の共有の口（`figure_body` / `figure_panel`）で、判断の記録・設計ノートの面と同じ字面。
//! 図が 1 枚でも導出できなければ面全体を導出しない（全部か無しか）。図が無い面は便 33 までと byte 不変。

use std::path::Path;

use crate::constitution_enums as ce;
use crate::face::{
    self, DOC_STATUS, Frame, MAX_PER_BAND, MAX_RAIL_NODES, MAX_STATE_NODES, METHOD, R, TONE, X,
    anchor, card, hint, hint_q, method_label, pattern_label, strength_label, strength_meaning,
    strength_prio,
};
use crate::parts::catalog::Component;

/// 要件書の面が使う部品（18 種・便 40 で ceiling-stamp を足した）。
pub const PARTS: [Component; 18] = [
    Component::FreshnessStamp,
    Component::CeilingStamp,
    Component::FontSizeControl,
    Component::DocCoverBand,
    Component::ChapterDeckBand,
    Component::SectionLeadCallout,
    Component::FigurePanel,
    Component::ContextBand,
    Component::BandNode,
    Component::PipelineRail,
    Component::RailNode,
    Component::StateStrip,
    Component::StateNode,
    Component::ItemRow,
    Component::AcStateChip,
    Component::RtmGrid,
    Component::GlossaryTermTable,
    Component::ApprovalBlock,
];

/// 章 01〜08 の名。
const CHAPTERS: [&str; 8] = [
    "ゴール",
    "範囲",
    "機能要件",
    "非機能要件",
    "受入基準",
    "制約",
    "対応表",
    "用語集",
];

/// 章 09（図）の名（帯の kicker・目次の名）。
const FIGURES_CHAPTER: &str = "図";

/// 章 01〜09 の帯の class と kicker の絵記号（01〜08 は見本の各章の字面・09 は band-3 と 3 番目の絵記号）。
/// 図の章の有無で 8 本か 9 本を先頭から切り出して `Frame` に渡す（`static` なので切り出しは 'static）。
static BANDS: [(&str, &str); 9] = [
    (
        "band-1",
        "<circle cx=\"12\" cy=\"12\" r=\"9\"/><circle cx=\"12\" cy=\"12\" r=\"3\"/>",
    ),
    (
        "band-5",
        "<path d=\"M3 7l9-4 9 4-9 4-9-4z\"/><path d=\"M3 7v10l9 4 9-4V7\"/>",
    ),
    ("band-2", "<path d=\"M20 6L9 17l-5-5\"/>"),
    ("band-6", "<path d=\"M4 18h16M6 14V8M12 14V4M18 14v-4\"/>"),
    (
        "band-2",
        "<path d=\"M9 11l3 3L22 4\"/><path d=\"M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11\"/>",
    ),
    (
        "band-3",
        "<path d=\"M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z\"/>",
    ),
    (
        "band-5",
        "<path d=\"M4 4h16v16H4z\"/><path d=\"M4 10h16M10 4v16\"/>",
    ),
    (
        "band-1",
        "<path d=\"M4 19.5A2.5 2.5 0 0 1 6.5 17H20\"/><path d=\"M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z\"/>",
    ),
    ("band-3", "<path d=\"M20 6L9 17l-5-5\"/>"),
];

/// 要件書の面の骨格（章の数 = 8 + 図の章の有無）。
fn frame(figures: bool) -> Frame {
    Frame {
        name: "要件書",
        source: "srs.yaml",
        favicon: "<link rel=\"icon\" href=\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect width='32' height='32' rx='7' fill='%235f45a6'/%3E%3Ctext x='16' y='22' font-size='16' font-weight='700' text-anchor='middle' fill='%23ffffff' font-family='sans-serif'%3E要%3C/text%3E%3C/svg%3E\">",
        current: 2,
        first: 1,
        bands: &BANDS[..CHAPTERS.len() + usize::from(figures)],
        prev: ("constitution.html", "憲法"),
        next: ("index.html", "入口"),
        parts: &PARTS,
    }
}

/// 段の担当のうち持ち主の手番の値。
const OWNER: &str = "持ち主";

/// 図 1 の真ん中の帯に置く actor の role。
const TOOL_ROLE: &str = "道具";

/// 見出しを持つ行（id・escape した title）。
pub(crate) struct Item<'a> {
    pub(crate) x: X<'a>,
    pub(crate) id: &'a str,
    pub(crate) title: String,
}

/// rail の段 1 つ。
struct Step<'a> {
    x: X<'a>,
    n: u64,
    who: String,
    owner: bool,
    what: String,
}

/// 導出の文脈（行・参照の先・骨格）。章 07・08 の生成（`face_srs_rtm.rs`）も読む。
pub(crate) struct Ctx<'a> {
    pub(crate) goals: Vec<Item<'a>>,
    pub(crate) fr: Vec<Item<'a>>,
    pub(crate) nfr: Vec<Item<'a>>,
    acs: Vec<Item<'a>>,
    cons: Vec<Item<'a>>,
    /// 条 id → escape した title
    arts: Vec<(&'a str, String)>,
    rule_ids: Vec<&'a str>,
    rail: Vec<Step<'a>>,
    /// 段の n → escape した what
    rail_what: Vec<(u64, String)>,
    verdicts: Option<Vec<X<'a>>>,
    /// 任意の図の節の行（無ければ空）
    figures: Vec<X<'a>>,
    /// 面の骨格（章の数は図の章の有無で変わる）
    pub(crate) frame: Frame,
}

impl<'a> Ctx<'a> {
    /// 要件書の id（GOAL・FR・NFR・AC・CON + 数字）がどれかの節に在るか。
    fn has_req(&self, id: &str) -> bool {
        self.goals
            .iter()
            .chain(&self.fr)
            .chain(&self.nfr)
            .chain(&self.acs)
            .chain(&self.cons)
            .any(|i| i.id == id)
    }

    /// 図の根拠の id 1 つ（在ればリンク・無ければ id の直後に「（まだ分からない）」）。判断の記録の面の resolve と同じ
    /// 4 形 + 判断の記録: 要件書の id は同じ面の anchor・条（枝番付きの規範文 id は条の anchor・字は枝番付きのまま）と
    /// rules 行は憲法の面・判断の記録は `adr-<数>.html`。どの形でもない id は Err。
    fn ref_link(&self, dir: &Path, x: &X<'_>) -> R<String> {
        let id = x.id()?;
        let bad = || {
            format!(
                "{}: 根拠の id「{id}」は id の形でない（条・rules 行・要件・判断の記録）",
                x.at
            )
        };
        let need = |ok: bool| ok.then_some(()).ok_or_else(bad);
        let prefix = |ps: &[&str]| ps.iter().find_map(|p| id.strip_prefix(p));
        let href = if let Some(rest) = prefix(&["P-", "A-", "N-"]) {
            let aid = match rest.split_once('.') {
                Some((n, sub)) if digits(n) && digits(sub) => &id[..id.len() - sub.len() - 1],
                None if digits(rest) => id,
                _ => return Err(bad()),
            };
            self.arts
                .iter()
                .any(|(k, _)| *k == aid)
                .then(|| article_link(aid, id))
        } else if let Some(rest) = prefix(&["R-", "D-"]) {
            need(digits(rest))?;
            self.rule_ids.contains(&id).then(|| article_link(id, id))
        } else if let Some(rest) = prefix(&["FR", "NFR", "AC", "CON", "GOAL"]) {
            need(digits(rest))?;
            self.has_req(id).then(|| xref(id, id))
        } else if let Some(rest) = id.strip_prefix("ADR-") {
            need(digits(rest))?;
            let file = dir.join("adr").join(format!("{id}.yaml"));
            file.is_file()
                .then(|| format!("<a class=\"xref\" href=\"{}.html\">{id}</a>", anchor(id)))
        } else {
            return Err(bad());
        };
        Ok(href.unwrap_or_else(|| format!("{id}（まだ分からない）")))
    }

    fn req(&self, x: &X<'_>) -> R<&Item<'a>> {
        let id = x.id()?;
        self.fr
            .iter()
            .chain(&self.nfr)
            .find(|i| i.id == id)
            .ok_or_else(|| format!("{}: 要件 id「{id}」が無い", x.at))
    }

    pub(crate) fn goal(&self, x: &X<'_>) -> R<&Item<'a>> {
        let id = x.id()?;
        self.goals
            .iter()
            .find(|i| i.id == id)
            .ok_or_else(|| format!("{}: ゴール id「{id}」が無い", x.at))
    }

    pub(crate) fn ac(&self, x: &X<'_>) -> R<&Item<'a>> {
        let id = x.id()?;
        self.acs
            .iter()
            .find(|i| i.id == id)
            .ok_or_else(|| format!("{}: 受入基準 id「{id}」が無い", x.at))
    }

    fn article(&self, x: &X<'_>) -> R<&(&'a str, String)> {
        let id = x.id()?;
        self.arts
            .iter()
            .find(|(k, _)| *k == id)
            .ok_or_else(|| format!("{}: 条 id「{id}」が無い", x.at))
    }

    fn rule(&self, x: &X<'_>) -> R<&'a str> {
        let id = x.id()?;
        self.rule_ids
            .iter()
            .find(|k| **k == id)
            .copied()
            .ok_or_else(|| format!("{}: rules 行 id「{id}」が無い", x.at))
    }

    pub(crate) fn fig(&self, x: &X<'_>) -> R<Where> {
        resolve(&x.text()?, &self.rail_what, self.verdicts.is_some())
            .map_err(|e| format!("{}: {e}", x.at))
    }
}

/// 正本 → 要件書の面の HTML（決定的）。`ceiling` = 天井の束の置き場（解決済み・None = `--ceiling` なし・便 40）。
pub fn derive(dir: &Path, ceiling: Option<&Path>) -> R<String> {
    let s_doc = face::load(dir, "srs.yaml")?;
    let c_doc = face::load(dir, "constitution.yaml")?;
    let r_doc = face::load(dir, "rules.yaml")?;
    let v_doc = face::load(dir, "vocabulary.yaml")?;
    let s = X::root(&s_doc, "srs.yaml");
    let c = X::root(&c_doc, "constitution.yaml");
    let r = X::root(&r_doc, "rules.yaml");
    let v = X::root(&v_doc, "vocabulary.yaml");

    let ctx = context(&s, &c, &r)?;
    let m = s.f("meta")?;
    check_counts(&ctx, &m.f("counts")?)?;
    let stamp = face::ceiling_stamp(dir, ceiling)?;

    let mut o: Vec<String> = Vec::new();
    head(&mut o, &ctx, &m, &stamp)?;
    cover(&mut o, &ctx, &m)?;
    toc(&mut o, &ctx);
    goals_chapter(&mut o, &ctx)?;
    scope_chapter(&mut o, &ctx, &s, &m)?;
    fr_chapter(&mut o, &ctx, &m)?;
    nfr_chapter(&mut o, &ctx)?;
    ac_chapter(&mut o, &ctx, &s)?;
    con_chapter(&mut o, &ctx)?;
    crate::face_srs_rtm::rtm_chapter(&mut o, &ctx)?;
    crate::face_srs_rtm::glossary_chapter(&mut o, &ctx, &s, &v)?;
    if !ctx.figures.is_empty() {
        figures_chapter(&mut o, &ctx, dir)?;
    }
    approval(&mut o, &ctx, &m)?;
    foot(&mut o, &ctx, &m)?;
    Ok(format!("{}\n", o.join("\n")))
}

// ── 読みと検査 ──

fn digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

fn items<'a>(s: &X<'a>, section: &str) -> R<Vec<Item<'a>>> {
    s.f(section)?
        .seq()?
        .into_iter()
        .map(|x| {
            Ok(Item {
                id: x.f("id")?.id()?,
                title: x.ef("title")?,
                x,
            })
        })
        .collect()
}

fn context<'a>(s: &X<'a>, c: &X<'a>, r: &X<'a>) -> R<Ctx<'a>> {
    let arts = c
        .f("articles")?
        .seq()?
        .iter()
        .map(|a| Ok((a.f("id")?.id()?, a.ef("title")?)))
        .collect::<R<Vec<_>>>()?;
    let mut rule_ids = Vec::new();
    for section in ["thresholds", "discipline"] {
        for x in r.f(section)?.seq()? {
            rule_ids.push(x.f("id")?.id()?);
        }
    }

    let rail_x = s.f("rail")?;
    let steps = rail_x.seq()?;
    if steps.len() > MAX_RAIL_NODES {
        return Err(format!(
            "{}: 段が {} で上限 {MAX_RAIL_NODES}（部品目録の pipeline-rail の max_nodes）を超える",
            rail_x.at,
            steps.len()
        ));
    }
    let mut rail = Vec::new();
    for x in steps {
        let who_x = x.f("who")?;
        rail.push(Step {
            n: x.f("n")?.count()?,
            who: who_x.e()?,
            owner: who_x.v.as_str() == Some(OWNER),
            what: x.ef("what")?,
            x,
        });
    }
    let rail_what = rail.iter().map(|st| (st.n, st.what.clone())).collect();

    let verdicts = match s.g("verdicts")? {
        Some(vx) => {
            let rows = vx.seq()?;
            if rows.len() > MAX_STATE_NODES {
                return Err(format!(
                    "{}: 答えが {} で上限 {MAX_STATE_NODES}（部品目録の state-strip の max_nodes）を超える",
                    vx.at,
                    rows.len()
                ));
            }
            Some(rows)
        }
        None => None,
    };
    // 任意の図の節（便 34）。無い・null は 0 枚
    let figures = s
        .g("figures")?
        .map_or_else(|| Ok(Vec::new()), |x| x.seq())?;

    Ok(Ctx {
        goals: items(s, "goals")?,
        fr: items(s, "requirements")?,
        nfr: items(s, "nonfunctional")?,
        acs: items(s, "acceptance")?,
        cons: items(s, "constraints")?,
        arts,
        rule_ids,
        rail,
        rail_what,
        verdicts,
        frame: frame(!figures.is_empty()),
        figures,
    })
}

/// meta の counts（fr・nfr・ac・con）= 数えた数。
fn check_counts(ctx: &Ctx<'_>, counts: &X<'_>) -> R<()> {
    for (key, counted) in [
        ("fr", ctx.fr.len()),
        ("nfr", ctx.nfr.len()),
        ("ac", ctx.acs.len()),
        ("con", ctx.cons.len()),
    ] {
        let x = counts.f(key)?;
        let written = x.count()?;
        if written != counted as u64 {
            return Err(format!("{}: {written} だが数えた行は {counted}", x.at));
        }
    }
    Ok(())
}

fn is_effective(m: &X<'_>) -> R<bool> {
    let status = m.f("status")?;
    status.lookup(DOC_STATUS, "文書の状態")?;
    Ok(status.v.as_str() == Some("effective"))
}

/// 図の参照（要件の figures の値）。
#[derive(Debug, PartialEq)]
enum Fig {
    Rail(u64),
    Context,
    Verdicts,
    All,
}

fn parse_fig(s: &str) -> R<Fig> {
    match s {
        "図1" => Ok(Fig::Context),
        "図3" => Ok(Fig::Verdicts),
        "全段" => Ok(Fig::All),
        _ => s
            .strip_prefix("図2-")
            .filter(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()))
            .and_then(|d| d.parse().ok())
            .map(Fig::Rail)
            .ok_or_else(|| format!("図の参照「{s}」は 図2-<n>・図1・図3・全段 のどれでもない")),
    }
}

/// 図の参照の行き先（href・item-row の字面・対応表の字面）。
#[derive(Debug, PartialEq)]
pub(crate) struct Where {
    pub(crate) href: Option<String>,
    long: String,
    pub(crate) short: String,
}

/// 図の参照を解く。`rail` は段の n と escape した what・`verdicts` は verdicts の節が在るか。
fn resolve(s: &str, rail: &[(u64, String)], verdicts: bool) -> R<Where> {
    Ok(match parse_fig(s)? {
        Fig::Rail(n) => {
            let what = rail
                .iter()
                .find(|(k, _)| *k == n)
                .map(|(_, w)| w)
                .ok_or_else(|| format!("図の参照「{s}」の段 {n} が rail に無い"))?;
            Where {
                href: Some(format!("#rail-{n}")),
                long: format!("図 2 {n} {what}"),
                short: format!("図 2 {n}"),
            }
        }
        Fig::Context => Where {
            href: Some("#fig-context".to_string()),
            long: "図 1".to_string(),
            short: "図 1".to_string(),
        },
        Fig::Verdicts => Where {
            href: verdicts.then(|| "#fig-verdicts".to_string()),
            long: "図 3".to_string(),
            short: "図 3".to_string(),
        },
        Fig::All => Where {
            href: None,
            long: "全段に掛かる".to_string(),
            short: "全段".to_string(),
        },
    })
}

/// 段の 2 つの枠（上 = 持ち主の側・下 = 道具の側）。`node` は組み立て済みの rail-node。
fn slots(owner: bool, node: &str) -> (String, String) {
    if owner {
        (
            format!("<div class=\"rail-slot filled\">{node}</div>"),
            "<div class=\"rail-slot\"><span class=\"lane-hint\">folio は待つ</span></div>"
                .to_string(),
        )
    } else {
        (
            "<div class=\"rail-slot\"><span class=\"lane-hint\">あなたの出番なし</span></div>"
                .to_string(),
            format!("<div class=\"rail-slot filled\">{node}</div>"),
        )
    }
}

// ── 骨格 ──

fn head(o: &mut Vec<String>, ctx: &Ctx<'_>, m: &X<'_>, stamp: &str) -> R<()> {
    let version = m.ef("version")?;
    let status = m.f("status")?.lookup(DOC_STATUS, "文書の状態")?;
    ctx.frame.head(
        o,
        &format!("folio2 — 要件書（{version}）"),
        &m.ef("generated")?,
        &version,
        status,
        stamp,
    );
    Ok(())
}

/// 「<数> 件（<最初の id>–<最後の id>）」。
fn range(items: &[Item<'_>]) -> String {
    match (items.first(), items.last()) {
        (Some(a), Some(b)) => format!("{} 件（{}–{}）", items.len(), a.id, b.id),
        _ => "0 件".to_string(),
    }
}

fn meta_span(k: &str, v: &str) -> String {
    format!("<span class=\"m\"><span class=\"k\">{k}</span><span class=\"v\">{v}</span></span>")
}

fn cover(o: &mut Vec<String>, ctx: &Ctx<'_>, m: &X<'_>) -> R<()> {
    let title = m.ef("title")?;
    o.push(format!(
        "<header {}>",
        ctx.frame.dc(Component::DocCoverBand)
    ));
    o.push(format!(
        "<p class=\"cover-eyebrow\"><span class=\"doc-type\">要件書 (SRS)</span> <span>folio2 — {title}</span></p>"
    ));
    o.push(format!("<h1>{title}</h1>"));
    o.push(format!(
        "<div class=\"summary-card\"><span class=\"ic\">要</span><div><p class=\"lab\">この文書が約束すること（1 文）</p><p class=\"txt\">{}</p></div></div>",
        m.ef("promise")?
    ));
    o.push("<div class=\"cover-meta\">".to_string());
    for (k, href, rows) in [
        ("機能要件", "s3", &ctx.fr),
        ("非機能要件", "s4", &ctx.nfr),
        ("受入基準", "s5", &ctx.acs),
        ("制約", "s6", &ctx.cons),
    ] {
        o.push(meta_span(
            k,
            &format!("<a href=\"#{href}\">{}</a>", range(rows)),
        ));
    }
    let mut figs = vec![
        "<a href=\"#fig-context\">図 1</a>",
        "<a href=\"#fig-rail\">図 2</a>",
    ];
    if ctx.verdicts.is_some() {
        figs.push("<a href=\"#fig-verdicts\">図 3</a>");
    }
    o.push(meta_span("図", &figs.join(" · ")));
    o.push(meta_span(
        "版",
        &format!("{} / {}", m.ef("version")?, m.ef("generated")?),
    ));
    o.push("</div>".to_string());
    let state = if is_effective(m)? {
        let mut when = None;
        for row in m.f("approval")?.seq()? {
            if row.f("role")?.v.as_str() == Some("承認") {
                when = Some(row.ef("when")?);
            }
        }
        match when {
            Some(w) => format!("発効・拘束力あり（承認 {w}）"),
            None => "発効・拘束力あり".to_string(),
        }
    } else {
        "未承認のため拘束力なし → 持ち主の承認で発効".to_string()
    };
    o.push(format!(
        "<p class=\"cover-status\"><span class=\"k\">状態</span><span>{state}（<a href=\"#approval\">承認欄へ</a>）</span></p>"
    ));
    o.push("</header>".to_string());
    Ok(())
}

/// 各章の h2 の字面（01〜08）。
fn chapter_h2(ctx: &Ctx<'_>, n: usize) -> String {
    match n {
        1 => "「これができたら成功」を先に決める".to_string(),
        2 => "誰が登場し、何を作り、何を作らないか".to_string(),
        3 => format!("{} の機能要件 — いつ・何をするか", ctx.fr.len()),
        4 => format!("{} の非機能要件 — 数で測れる約束", ctx.nfr.len()),
        5 => format!(
            "{} の受入基準 — 何を見せられたら「できた」か",
            ctx.acs.len()
        ),
        6 => "設計の前に決まっていること".to_string(),
        7 => {
            "どの要件が、どのゴールのためにあり、どの受入基準で確かめ、図のどこにあるか".to_string()
        }
        _ => "本文に出てくる専門語のやさしい説明".to_string(),
    }
}

fn toc(o: &mut Vec<String>, ctx: &Ctx<'_>) {
    let mut heads = (1..=CHAPTERS.len())
        .map(|n| (CHAPTERS[n - 1].to_string(), chapter_h2(ctx, n)))
        .collect::<Vec<_>>();
    // 図の章は 1 枚以上のときだけ（図なしの面は便 33 までと byte 不変）
    let n = ctx.figures.len();
    if n > 0 {
        heads.push((FIGURES_CHAPTER.to_string(), format!("{n} 枚")));
    }
    ctx.frame.toc(o, &heads, "作成 / レビュー / 承認");
}

pub(crate) fn band(o: &mut Vec<String>, ctx: &Ctx<'_>, n: usize, lead: Option<&str>) {
    ctx.frame
        .band(o, n, CHAPTERS[n - 1], &chapter_h2(ctx, n), lead);
}

fn figure_open(
    o: &mut Vec<String>,
    ctx: &Ctx<'_>,
    id: &str,
    fn_: &str,
    title: &str,
    legend: Option<&str>,
) {
    o.push(format!(
        "<figure {} data-role=\"diagram\" id=\"{id}\">",
        ctx.frame.dc(Component::FigurePanel)
    ));
    let legend = legend.map_or_else(String::new, |l| hint("凡例", l));
    o.push(format!(
        "<div class=\"fig-title\"><span class=\"fn\">{fn_}</span>{title} <span class=\"fig-tools\">{legend}<button class=\"zoom-btn\" type=\"button\">拡大</button></span><button class=\"zoom-close\" type=\"button\">✕ 閉じる</button></div>"
    ));
}

fn figure_close(o: &mut Vec<String>, fn_: &str, m: &X<'_>) -> R<()> {
    o.push(format!(
        "<figcaption><span class=\"ver\">{fn_} · {} {} · srs.yaml</span></figcaption>",
        m.ef("version")?,
        m.ef("generated")?
    ));
    o.push("</figure>".to_string());
    Ok(())
}

/// 図の中の要件へのリンク。
fn fig_req(item: &Item<'_>) -> String {
    format!(
        "<a class=\"fig-req\" href=\"#{}\"><span class=\"no\">{}</span>{}</a>",
        anchor(item.id),
        item.id,
        item.title
    )
}

/// 憲法の条への xref（字面は「<前置き><id><後置き>」）。
fn article_link(id: &str, text: &str) -> String {
    format!(
        "<a class=\"xref\" href=\"constitution.html#{}\">{text}</a>",
        anchor(id)
    )
}

fn xref(id: &str, text: &str) -> String {
    format!("<a class=\"xref\" href=\"#{}\">{text}</a>", anchor(id))
}

// ── 章 ──

fn goals_chapter(o: &mut Vec<String>, ctx: &Ctx<'_>) -> R<()> {
    band(o, ctx, 1, None);
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!(
        "<div {} style=\"--band-n:{}\">",
        ctx.frame.dc(Component::SectionLeadCallout),
        ctx.goals.len()
    ));
    for g in &ctx.goals {
        o.push(card(
            "card accent",
            Some(&anchor(g.id)),
            g.id,
            &format!(
                "<p class=\"ct\">{}</p><p class=\"cd\">{}</p>",
                g.title,
                g.x.ef("text")?
            ),
        ));
    }
    o.push("</div>".to_string());
    o.push("</div>".to_string());
    Ok(())
}

fn band_limit(x: &X<'_>, what: &str, n: usize) -> R<()> {
    if n > MAX_PER_BAND {
        return Err(format!(
            "{}: {what}の帯が {n} で上限 {MAX_PER_BAND}（部品目録の context-band の max_per_band）を超える",
            x.at
        ));
    }
    Ok(())
}

/// 一覧の値を「 ／ 」で繋ぐ。
fn joined(x: &X<'_>) -> R<String> {
    Ok(x.seq()?
        .iter()
        .map(X::e)
        .collect::<R<Vec<_>>>()?
        .join(" ／ "))
}

fn scope_chapter(o: &mut Vec<String>, ctx: &Ctx<'_>, s: &X<'_>, m: &X<'_>) -> R<()> {
    let actors_x = s.f("actors")?;
    let actors = actors_x.seq()?;
    let mut inputs = Vec::new();
    let mut tools = Vec::new();
    for a in &actors {
        if a.f("role")?.text()? == TOOL_ROLE {
            tools.push(a);
        } else {
            inputs.push(a);
        }
    }
    if tools.len() != 1 {
        return Err(format!(
            "{}: role が「{TOOL_ROLE}」の actor が {} で 1 つでない",
            actors_x.at,
            tools.len()
        ));
    }
    let outputs_x = s.f("outputs")?;
    let outputs = outputs_x.seq()?;
    band_limit(&actors_x, "入れる側", inputs.len())?;
    band_limit(&outputs_x, "出る側", outputs.len())?;

    band(o, ctx, 2, None);
    o.push("<div class=\"chapbody\">".to_string());
    figure_open(o, ctx, "fig-context", "図 1", "誰が使い、何が出るか", None);
    o.push(format!(
        "<div {} style=\"--band-n:3\">",
        ctx.frame.dc(Component::ContextBand)
    ));
    o.push("<div class=\"band\"><h4>入れる側</h4>".to_string());
    for a in &inputs {
        o.push(format!(
            "<div {} id=\"c-{}\"><p class=\"nt\">{}</p><span class=\"edge\"><span class=\"arrow\"></span>{}</span></div>",
            ctx.frame.dc(Component::BandNode),
            a.f("id")?.id()?,
            a.ef("name")?,
            a.ef("role")?
        ));
    }
    o.push("</div>".to_string());
    o.push("<div class=\"band-arrow\"><span class=\"arrow\"></span></div>".to_string());
    o.push("<div class=\"band tool\"><h4>道具</h4>".to_string());
    for a in &tools {
        o.push(format!(
            "<div {} id=\"c-{}\"><p class=\"nt\">{}</p></div>",
            ctx.frame.dc(Component::BandNode),
            a.f("id")?.id()?,
            a.ef("name")?
        ));
    }
    o.push("</div>".to_string());
    o.push("<div class=\"band-arrow\"><span class=\"arrow\"></span></div>".to_string());
    o.push("<div class=\"band\"><h4>出る側</h4>".to_string());
    for x in &outputs {
        o.push(format!(
            "<div {} id=\"c-{}\"><p class=\"nt\">{}</p><p class=\"fig-reqs\">{}</p></div>",
            ctx.frame.dc(Component::BandNode),
            x.f("id")?.id()?,
            x.ef("name")?,
            fig_req(ctx.req(&x.f("from")?)?)
        ));
    }
    o.push("</div>".to_string());
    o.push("</div>".to_string());
    figure_close(o, "図 1", m)?;

    o.push("<h3>作るもの / 作らないもの</h3>".to_string());
    for (key, label) in [("scope", "M0"), ("scope_m1", "M1")] {
        let sc = s.f(key)?;
        let note = if key == "scope_m1"
            && let Some(n) = sc.g("note")?
        {
            format!(" {}", hint("注", &n.e()?))
        } else {
            String::new()
        };
        o.push(format!(
            "<div {} style=\"--band-n:2\">",
            ctx.frame.dc(Component::SectionLeadCallout)
        ));
        o.push(card(
            "card accent ok",
            None,
            &format!("{label} で作る"),
            &format!("<p class=\"cd\">{}</p>", joined(&sc.f("build")?)?),
        ));
        o.push(card(
            "card accent",
            None,
            &format!("{label} では作らない <span class=\"pill\">対象外</span>"),
            &format!("<p class=\"cd\">{}{note}</p>", joined(&sc.f("not_build")?)?),
        ));
        o.push("</div>".to_string());
    }
    o.push("</div>".to_string());
    Ok(())
}

/// 凡例の 1 行（言葉 = 強度と型の名札・確かめ方 = 確かめ方の名札）。強度と型は導出した型の ALL の順（= 憲法の値域の file の順）。
fn legend_line() -> String {
    let words = ce::Strength::ALL
        .iter()
        .map(|s| format!("{} = {}", strength_label(*s), strength_meaning(*s)))
        .chain(
            ce::Pattern::ALL
                .iter()
                .map(|p| pattern_label(*p).to_string()),
        )
        .collect::<Vec<_>>()
        .join("／");
    let methods = METHOD
        .iter()
        .map(|(_, label)| *label)
        .collect::<Vec<_>>()
        .join("／");
    format!(
        "<div class=\"legend-line\"><span>凡例:</span>{}{}</div>",
        hint("言葉", &words),
        hint("確かめ方", &methods)
    )
}

fn fr_chapter(o: &mut Vec<String>, ctx: &Ctx<'_>, m: &X<'_>) -> R<()> {
    band(o, ctx, 3, None);
    o.push("<div class=\"chapbody\">".to_string());
    o.push(legend_line());

    // 図 2
    let count = ctx.rail.len();
    let legend = "<div class=\"fig-legend\"><span class=\"lg\"><span class=\"sw ok\"></span>あなたの手番</span><span class=\"lg\"><span class=\"sw neutral\"></span>folio がやる</span><span class=\"lg\"><span class=\"sw line\"></span>番号の順に進む</span><span class=\"lg\">枠線が点線 = 定める要件が無い段</span></div>";
    figure_open(
        o,
        ctx,
        "fig-rail",
        "図 2",
        &format!("folio が 1 回で通す {count} 段 — 各段を決めている要件"),
        Some(legend),
    );
    o.push(format!(
        "<ol {} style=\"--rail-n:{count}\">",
        ctx.frame.dc(Component::PipelineRail)
    ));
    for st in &ctx.rail {
        let x = &st.x;
        let nt = match x.g("note")? {
            Some(n) => format!("{} {}", st.what, hint_q(&n.e()?)),
            None => st.what.clone(),
        };
        let basis = match x.g("basis")? {
            Some(b) => b
                .seq()?
                .iter()
                .map(|q| {
                    let (id, _) = ctx.article(q)?;
                    Ok(article_link(id, &format!("憲法 {id}")))
                })
                .collect::<R<Vec<_>>>()?,
            None => Vec::new(),
        };
        let reqs = x
            .f("reqs")?
            .seq()?
            .iter()
            .map(|q| Ok(fig_req(ctx.req(q)?)))
            .collect::<R<Vec<_>>>()?;
        let fig_reqs = if reqs.is_empty() {
            let refs = if basis.is_empty() {
                String::new()
            } else {
                format!("（{}）", basis.join("・"))
            };
            format!("<span class=\"fig-req none\">この段を定める要件は無い{refs}</span>")
        } else {
            reqs.concat()
        };
        let node = format!(
            "<article {}{} id=\"rail-{}\"><span class=\"actor\">担当: {}</span><p class=\"nt\">{nt}</p><p class=\"fig-reqs\">{fig_reqs}</p></article>",
            ctx.frame.dc(Component::RailNode),
            if st.owner { " class=\"tone-ok\"" } else { "" },
            st.n,
            st.who
        );
        o.push(format!(
            "<li class=\"rail-col\"><span class=\"rail-step\"><span class=\"no\">{}</span></span>",
            st.n
        ));
        let (upper, lower) = slots(st.owner, &node);
        o.push(upper);
        o.push(format!("{lower}</li>"));
    }
    o.push("</ol>".to_string());
    figure_close(o, "図 2", m)?;

    o.push("<div class=\"stack\">".to_string());
    for it in &ctx.fr {
        item_row(o, ctx, it, false)?;
    }
    o.push("</div>".to_string());

    // 図 3（verdicts の節が在るときだけ）
    if let Some(verdicts) = &ctx.verdicts {
        let mut legend = String::new();
        let mut nodes = Vec::new();
        for v in verdicts {
            let tone_x = v.f("tone")?;
            let class = tone_x.lookup(TONE, "図の色")?;
            let tone = tone_x.e()?;
            let name = v.ef("name")?;
            legend.push_str(&format!(
                "<span class=\"lg\"><span class=\"sw {tone}\"></span>{name}</span>"
            ));
            nodes.push(format!(
                "<li {} class=\"{class}\" id=\"v-{}\"><p class=\"nt\">{name}</p><span class=\"cond\">ここへ来る条件</span><p class=\"np\">{}</p></li>",
                ctx.frame.dc(Component::StateNode),
                v.f("id")?.id()?,
                v.ef("cond")?
            ));
        }
        figure_open(
            o,
            ctx,
            "fig-verdicts",
            "図 3",
            &format!(
                "検査の答えは {} つ — どういうときにその答えになるか",
                verdicts.len()
            ),
            Some(&format!("<div class=\"fig-legend\">{legend}</div>")),
        );
        o.push(format!(
            "<ul {} style=\"--state-n:{}\">",
            ctx.frame.dc(Component::StateStrip),
            verdicts.len()
        ));
        o.extend(nodes);
        o.push("</ul>".to_string());
        figure_close(o, "図 3", m)?;
    }
    o.push("</div>".to_string());
    Ok(())
}

fn nfr_chapter(o: &mut Vec<String>, ctx: &Ctx<'_>) -> R<()> {
    band(o, ctx, 4, None);
    o.push("<div class=\"chapbody\">".to_string());
    o.push(legend_line());
    o.push("<div class=\"stack\">".to_string());
    for it in &ctx.nfr {
        item_row(o, ctx, it, true)?;
    }
    o.push("</div>".to_string());
    o.push("</div>".to_string());
    Ok(())
}

fn item_row(o: &mut Vec<String>, ctx: &Ctx<'_>, it: &Item<'_>, nfr: bool) -> R<()> {
    let x = &it.x;
    let pattern = x.f("pattern")?;
    let ears = pattern_label(pattern.parse(ce::Pattern::from_name, "型")?);
    let strength = x.f("strength")?;
    let s = strength.parse(ce::Strength::from_name, "強度")?;
    let (kw, prio) = (strength_label(s), strength_prio(s));
    let milestone = x.g("milestone")?;
    o.push(format!(
        "<article {}{} id=\"{}\">",
        ctx.frame.dc(Component::ItemRow),
        if nfr { " class=\"kind-nfr\"" } else { "" },
        anchor(it.id)
    ));
    let pill = match &milestone {
        Some(ms) => format!("<span class=\"pill\">{}</span>", ms.e()?),
        None => String::new(),
    };
    o.push(format!(
        "<div class=\"ir-head\"><span class=\"rid\">{}</span><h3 class=\"rt\">{}</h3><span class=\"badges\"><span class=\"prio {prio}\">{kw}</span><span class=\"ears\">{ears}</span>{pill}</span></div>",
        it.id, it.title
    ));
    let when = x.ef("when")?;
    let shall = x.ef("shall")?;
    let norm = if pattern.v.as_str() == Some("ubiquitous") {
        shall
    } else {
        format!("<span class=\"ew\">{when}</span> {shall}")
    };
    o.push(format!("<p class=\"norm\">{norm}</p>"));
    o.push(format!(
        "<div class=\"plain\"><span class=\"pk\">やさしく言うと</span>{}</div>",
        x.ef("plain")?
    ));

    // 小窓（確かめ方・根拠・注）と図の参照
    let verify = x.f("verify")?;
    let method = verify.f("method")?;
    let mut how = format!(
        "<span class=\"vbadge\">{}</span> {}",
        method_label(&method)?,
        verify.ef("how")?
    );
    let acs = verify
        .f("ac")?
        .seq()?
        .iter()
        .map(|q| {
            let a = ctx.ac(q)?;
            Ok(xref(a.id, a.id))
        })
        .collect::<R<Vec<_>>>()?;
    if !acs.is_empty() {
        how.push_str(&format!("（{}）", acs.join("・")));
    }
    let mut chips = vec![hint("確かめ方", &how)];
    let mut basis = Vec::new();
    for q in x.f("goals")?.seq()? {
        let g = ctx.goal(&q)?;
        basis.push(xref(g.id, &format!("{}（{}）", g.id, g.title)));
    }
    for q in x.f("basis")?.seq()? {
        let (id, title) = ctx.article(&q)?;
        basis.push(article_link(id, &format!("憲法 {id}（{title}）")));
    }
    if let Some(rules) = x.g("rules")? {
        for q in rules.seq()? {
            let id = ctx.rule(&q)?;
            basis.push(article_link(id, &format!("rules {id}")));
        }
    }
    if !basis.is_empty() {
        chips.push(hint("根拠", &basis.join("・")));
    }
    if let Some(note) = x.g("note")? {
        chips.push(hint("注", &note.e()?));
    }
    for f in x.f("figures")?.seq()? {
        let w = ctx.fig(&f)?;
        chips.push(match w.href {
            Some(h) => format!("<a class=\"rq-where\" href=\"{h}\">{}</a>", w.long),
            None => w.long,
        });
    }
    o.push(format!("<p class=\"meta-chips\">{}</p>", chips.concat()));

    // 機械のための面（正本の値のまま）
    let mut dl = format!(
        "<dt>pattern</dt><dd>{}</dd><dt>strength</dt><dd>{}</dd><dt>when</dt><dd>{when}</dd><dt>verify</dt><dd>{}</dd>",
        pattern.e()?,
        strength.e()?,
        method.e()?
    );
    if let Some(ms) = &milestone {
        dl.push_str(&format!("<dt>milestone</dt><dd>{}</dd>", ms.e()?));
    }
    o.push(format!(
        "<details class=\"machine\" data-audience=\"machine\"><summary>機械のための面</summary><dl>{dl}</dl></details>"
    ));
    o.push("</article>".to_string());
    Ok(())
}

/// 受入基準の章の凡例の 1 行（札「まだ分からない」が何を言うか・便 64）。部品と class は §3 の凡例（`legend_line`）と同じ。
/// folio はまだ受入の結果を測らない（P-4.2）——測るのは便の検証の歯で、その結果はこの面に写していない。
fn ac_legend_line() -> &'static str {
    "<div class=\"legend-line\"><span>凡例:</span><span>まだ分からない = この基準の合否を folio はまだ数えていません（合否を測るのは便の検証の歯で、その結果はこのページに写していません）</span></div>"
}

fn ac_chapter(o: &mut Vec<String>, ctx: &Ctx<'_>, s: &X<'_>) -> R<()> {
    band(o, ctx, 5, None);
    o.push("<div class=\"chapbody\">".to_string());
    o.push(ac_legend_line().to_string());
    o.push(format!(
        "<div {} style=\"--band-n:2\">",
        ctx.frame.dc(Component::SectionLeadCallout)
    ));
    for a in &ctx.acs {
        let x = &a.x;
        // folio はまだ受入の結果を測らない（P-4.2）
        let mut cid = format!(
            "{} <span {}>まだ分からない</span>",
            a.id,
            ctx.frame.dc(Component::AcStateChip)
        );
        if let Some(ms) = x.g("milestone")? {
            cid.push_str(&format!(" <span class=\"pill\">{}</span>", ms.e()?));
        }
        let verifies = x
            .f("verifies")?
            .seq()?
            .iter()
            .map(|q| {
                let r = ctx.req(q)?;
                Ok(xref(r.id, r.id))
            })
            .collect::<R<Vec<_>>>()?;
        let red = x.f("red_test")?;
        let mut cd = format!(
            "{} を確かめる。{}",
            verifies.join("・"),
            hint(
                "RED の歯",
                &format!("{}／fixture: {}", red.ef("sentence")?, red.ef("fixture")?)
            )
        );
        if let Some(note) = x.g("note")? {
            cd.push_str(&hint("注", &note.e()?));
        }
        o.push(card(
            "card accent",
            Some(&anchor(a.id)),
            &cid,
            &format!(
                "<p class=\"ct\">{}</p><div class=\"plain\"><span class=\"pk\">やさしく言うと</span>{}</div><p class=\"cd\">{cd}</p>",
                a.title,
                x.ef("plain")?
            ),
        ));
    }
    o.push("</div>".to_string());
    o.push(format!(
        "<div class=\"callout warn\"><span class=\"ck\">凍結しないもの</span><p>{}</p></div>",
        s.ef("not_frozen")?
    ));
    o.push("</div>".to_string());
    Ok(())
}

fn con_chapter(o: &mut Vec<String>, ctx: &Ctx<'_>) -> R<()> {
    band(o, ctx, 6, None);
    o.push("<div class=\"chapbody\">".to_string());
    o.push("<div class=\"tbl-wrap\"><table class=\"tbl\">".to_string());
    o.push(
        "<thead><tr><th>id</th><th>制約</th><th>中身</th><th>根拠</th><th>出所</th></tr></thead>"
            .to_string(),
    );
    o.push("<tbody>".to_string());
    for con in &ctx.cons {
        let x = &con.x;
        let mut refs = Vec::new();
        for q in x.f("basis")?.seq()? {
            let (id, _) = ctx.article(&q)?;
            refs.push(article_link(id, id));
        }
        if let Some(rules) = x.g("rules")? {
            for q in rules.seq()? {
                let id = ctx.rule(&q)?;
                refs.push(article_link(id, id));
            }
        }
        let refs = if refs.is_empty() {
            "—".to_string()
        } else {
            refs.join("・")
        };
        o.push(format!(
            "<tr id=\"{}\"><td class=\"id\" data-k=\"id\">{}</td><td data-k=\"制約\">{}</td><td data-k=\"中身\">{}</td><td data-k=\"根拠\">{refs}</td><td data-k=\"出所\">{}</td></tr>",
            anchor(con.id),
            con.id,
            con.title,
            x.ef("text")?,
            x.ef("source")?
        ));
    }
    o.push("</tbody>".to_string());
    o.push("</table></div>".to_string());
    o.push("</div>".to_string());
    Ok(())
}

/// 自前の図の数 = 2 + 〔verdicts の節が在れば 1〕（図 1〜3）。章 09 の図の番号はこの続き（便 35）・帯の h2 / toc /
/// foot の数は章 09 の図の数のまま。
fn own_figures(ctx: &Ctx<'_>) -> usize {
    2 + usize::from(ctx.verdicts.is_some())
}

/// 章 09（図・便 34）。用語集の後・承認欄の前。図ごとに共有の図の枠（`face::figure_panel`）を置き、根拠（refs）の
/// リンクは `Ctx::ref_link` で解く。図が 1 枚でも導出できなければ Err（面全体が「まだ分からない」・前の面は残る）。
fn figures_chapter(o: &mut Vec<String>, ctx: &Ctx<'_>, dir: &Path) -> R<()> {
    ctx.frame.band(
        o,
        CHAPTERS.len() + 1,
        FIGURES_CHAPTER,
        &format!("{FIGURES_CHAPTER} {} 枚", ctx.figures.len()),
        None,
    );
    o.push("<div class=\"chapbody\">".to_string());
    for (n, fig) in (own_figures(ctx) + 1..).zip(&ctx.figures) {
        let drawn = face::figure_body(dir, fig)?;
        let caption = fig.ef("caption")?;
        let refs = match fig.g("refs")? {
            Some(rs) => rs
                .seq()?
                .iter()
                .map(|q| ctx.ref_link(dir, q))
                .collect::<R<Vec<_>>>()?,
            None => Vec::new(),
        };
        face::figure_panel(o, &ctx.frame, n, &drawn, &caption, &refs);
    }
    o.push("</div>".to_string());
    Ok(())
}

fn approval(o: &mut Vec<String>, ctx: &Ctx<'_>, m: &X<'_>) -> R<()> {
    let status = m.f("status")?.lookup(DOC_STATUS, "文書の状態")?;
    let lead = match m.g("status_note")? {
        Some(n) => format!("{status} — {}", n.e()?),
        None => status.to_string(),
    };
    ctx.frame.approval_band(o, "作成 / レビュー / 承認", &lead);
    o.push("<div class=\"chapbody\">".to_string());
    o.push(format!("<div {}>", ctx.frame.dc(Component::ApprovalBlock)));
    for row in m.f("approval")?.seq()? {
        let role = row.f("role")?;
        let mut sign = format!(
            "<div class=\"sign\"><span class=\"role\">{}</span><span class=\"who\">{}</span><span class=\"when\">{}</span>",
            role.e()?,
            row.ef("who")?,
            row.ef("when")?
        );
        if let Some(version) = row.g("version")? {
            sign.push_str(&format!("<span class=\"when\">版 {}</span>", version.e()?));
        }
        if let Some(verbatim) = row.g("verbatim")? {
            sign.push_str(&format!(
                "<span class=\"when\">逐語「{}」</span>",
                verbatim.e()?
            ));
        }
        let class = if role.v.as_str() == Some("承認") {
            "stamp"
        } else {
            "stamp self"
        };
        sign.push_str(&format!(
            "<span class=\"{class}\">{}</span></div>",
            row.ef("stamp")?
        ));
        o.push(sign);
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
    if let Some(ev) = m.g("effective_version")? {
        dl.push_str(&format!("<dt>effective_version</dt><dd>{}</dd>", ev.raw()?));
    }
    let items = ctx
        .fr
        .iter()
        .chain(&ctx.nfr)
        .chain(&ctx.acs)
        .chain(&ctx.cons)
        .map(|i| i.id)
        .collect::<Vec<_>>()
        .join(" / ");
    dl.push_str(&format!("<dt>items</dt><dd>{items}</dd>"));
    // 図の数は 1 枚以上のときだけ（図なしの面は便 33 までと byte 不変）
    if !ctx.figures.is_empty() {
        dl.push_str(&format!("<dt>figures</dt><dd>{}</dd>", ctx.figures.len()));
    }
    ctx.frame.foot(o, &version, &generated, &dl);
    Ok(())
}

#[cfg(test)]
mod face_srs_tests {
    use super::*;

    fn rail() -> Vec<(u64, String)> {
        vec![(1, "相談を受ける".to_string()), (2, "決める".to_string())]
    }

    #[test]
    fn face_srs_figure_refs_resolve_to_the_rail_and_the_figures() {
        assert_eq!(
            resolve("図2-1", &rail(), false).unwrap(),
            Where {
                href: Some("#rail-1".to_string()),
                long: "図 2 1 相談を受ける".to_string(),
                short: "図 2 1".to_string(),
            }
        );
        let context = resolve("図1", &rail(), false).unwrap();
        assert_eq!(context.href.as_deref(), Some("#fig-context"));
        assert_eq!(context.long, "図 1");
        let with = resolve("図3", &rail(), true).unwrap();
        assert_eq!(with.href.as_deref(), Some("#fig-verdicts"));
        let without = resolve("図3", &rail(), false).unwrap();
        assert_eq!(without.href, None);
        assert_eq!(without.long, "図 3");
        let all = resolve("全段", &rail(), true).unwrap();
        assert_eq!(
            (all.href, all.long.as_str(), all.short.as_str()),
            (None, "全段に掛かる", "全段")
        );
        for bad in ["図4", "図2-0", "図2-9", "図2-", "図2-x", "図 1", ""] {
            assert!(resolve(bad, &rail(), true).is_err(), "{bad}");
        }
    }

    #[test]
    fn face_srs_owner_takes_the_upper_slot_and_folio_the_lower() {
        let (upper, lower) = slots(true, "N");
        assert_eq!(upper, "<div class=\"rail-slot filled\">N</div>");
        assert!(lower.contains("folio は待つ"));
        let (upper, lower) = slots(false, "N");
        assert!(upper.contains("あなたの出番なし"));
        assert_eq!(lower, "<div class=\"rail-slot filled\">N</div>");
    }

    #[test]
    fn face_parts_are_all_allowed_on_the_srs_face() {
        for part in PARTS {
            assert!(
                part.faces().contains(&"srs"),
                "{} は要件書の面に置けない",
                part.name()
            );
        }
        assert!(!PARTS.contains(&Component::LaneChip));
        let mut names: Vec<&str> = PARTS.iter().map(|p| p.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 18);
    }

    /// 帯の上限の関数（上限は便 52 から部品目録の導出・値の一致は parts.rs の歯が見る）。
    #[test]
    fn face_srs_band_limit_rejects_one_more_than_the_limit() {
        let v = crate::yaml::Value::Null;
        let x = X::root(&v, "srs.yaml.actors");
        assert!(band_limit(&x, "入れる側", MAX_PER_BAND).is_ok());
        assert!(band_limit(&x, "入れる側", MAX_PER_BAND + 1).is_err());
    }

    /// 強度の 3 つの名札（規範の語・意味・色）の揃いは、便 50 から同じ型への網羅の場合分けになり組み立てが保つ。
    #[test]
    fn face_srs_label_tables_are_aligned() {
        let keys = |t: &[(&str, &str)]| t.iter().map(|(k, _)| k.to_string()).collect::<Vec<_>>();
        assert_eq!(keys(TONE), ["ok", "bad", "neutral", "warn"]);
        for (k, class) in TONE {
            assert_eq!(*class, format!("tone-{k}"));
        }
        let both = crate::yaml::Value::Str("test+inspection".into());
        assert_eq!(
            method_label(&X::root(&both, "m")).unwrap(),
            "実際に動かして確かめる（Test） + 目で見て確かめる（Inspection）"
        );
        let bad = crate::yaml::Value::Str("review".into());
        assert!(method_label(&X::root(&bad, "m")).is_err());
    }
}
