//! 憲法の面の読み手（便 116・docs/design/delivery-116.md §1・ADR-15 決定 (2)(3)・責務の層 4 面に出す・面の中の割り）。`face_constitution.rs`
//! から条と文脈と改訂の段の型・読みと検査（context・値域の突き合わせ・数の突き合わせ）・欠番・関係の欄のリンクの列・改訂の段の読みを字を変えずに降ろした。
//! 面の口 `derive` と HTML を書く側と枠と名札と数の口は `face_constitution.rs` に残る。面の口と字面の逃がしを呼び、関係の欄からリンクの列も
//! 組むので層 1 には置かない。見え方は書く側が名指すものだけを広げた。

use crate::constitution_enums as ce;
use crate::cursor::{R, X, esc};
use crate::face::{Tier, anchor, section_anchor, split_dash, tier_label, tier_of};

/// 条 1 つ（id・escape した title・段）。
pub(crate) struct Art<'a> {
    pub(crate) x: X<'a>,
    pub(crate) id: &'a str,
    pub(crate) title: String,
    pub(crate) tier_key: String,
    pub(crate) tier: Tier,
}

/// 導出の文脈（段の順・条・参照の先）。
pub(crate) struct Ctx<'a> {
    pub(crate) tiers: Vec<(String, Tier)>,
    pub(crate) arts: Vec<Art<'a>>,
    /// 要件書の 5 節の id → escape した title
    reqs: Vec<(String, String)>,
    rule_ids: Vec<String>,
}

impl<'a> Ctx<'a> {
    /// 条 id を指す値 → その条（無ければ Err）。
    fn article(&self, x: &X<'_>) -> R<&Art<'a>> {
        let id = x.id()?;
        self.arts
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| format!("{}: 条 id「{id}」が無い", x.at))
    }
}

// ── 読みと検査 ──

pub(crate) fn context<'a>(c: &X<'a>, r: &X<'a>, s: &X<'a>) -> R<Ctx<'a>> {
    let enums = c.f("schema")?.f("enums")?;
    let mut tiers: Vec<(String, Tier)> = Vec::new();
    for t in enums.f("tier")?.seq()? {
        let key = t.text()?;
        let tier = tier_of(&key).map_err(|e| format!("{}: {e}", t.at))?;
        if tiers.iter().any(|(k, _)| *k == key) {
            return Err(format!("{}: 段「{key}」が 2 度ある", t.at));
        }
        tiers.push((key, tier));
    }
    if tiers.len() != ce::Tier::ALL.len() {
        return Err("constitution.yaml.schema.enums.tier: 段の表の 3 つ全部でない".to_string());
    }
    // 組み立てた版と読んでいる版のずれの検査を、置き場の schema.enums に在る鍵の全部へ（便 50 (d)・段は上で見た）
    for (key, x) in enums.pairs()? {
        if key != "tier" {
            enum_skew(key, &x)?;
        }
    }

    let mut arts = Vec::new();
    for a in c.f("articles")?.seq()? {
        let id = a.f("id")?.id()?;
        let title = a.ef("title")?;
        let tier_x = a.f("tier")?;
        let tier = tier_label(tier_x.parse(ce::Tier::from_name, "段")?);
        let tier_key = tier_x.text()?;
        arts.push(Art {
            x: a,
            id,
            title,
            tier_key,
            tier,
        });
    }

    let mut reqs = Vec::new();
    for sec in [
        "goals",
        "requirements",
        "nonfunctional",
        "acceptance",
        "constraints",
    ] {
        for x in s.f(sec)?.seq()? {
            reqs.push((x.f("id")?.text()?, x.ef("title")?));
        }
    }

    let mut rule_ids = Vec::new();
    for sec in ["thresholds", "discipline"] {
        for x in r.f(sec)?.seq()? {
            rule_ids.push(x.f("id")?.id()?.to_string());
        }
    }
    Ok(Ctx {
        tiers,
        arts,
        reqs,
        rule_ids,
    })
}

/// 読んでいる置き場の憲法の値域 1 つ（schema.enums の鍵 `key`）が、この folio を組み立てた版の憲法の値域
/// （`constitution_enums::ENUMS`）と集合で一致する = 各値が導出した型に在る・2 度無い・数が同じ（順は問わない・段と同じ強さ）。
/// 組み立てた版に無い鍵が置き場に在るときも Err。置き場に無い鍵は見ない（便 50 §1 (d)）。
fn enum_skew(key: &str, x: &X<'_>) -> R<()> {
    let skew = |why: String| {
        format!(
            "constitution.yaml.schema.enums.{key}: {why}・組み立て時の憲法の値域と違う（組み立て直す）"
        )
    };
    let names = ce::ENUMS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, names)| *names)
        .ok_or_else(|| skew("組み立てた版に無い鍵".to_string()))?;
    let mut seen: Vec<String> = Vec::new();
    for v in x.seq()? {
        let s = v.text()?;
        if !names.contains(&s.as_str()) {
            return Err(skew(format!("値「{s}」が組み立てた版に無い")));
        }
        if seen.contains(&s) {
            return Err(skew(format!("値「{s}」が 2 度ある")));
        }
        seen.push(s);
    }
    if seen.len() != names.len() {
        return Err(skew(format!("組み立てた版の {} 値全部でない", names.len())));
    }
    Ok(())
}

/// meta の counts のキーの集合 = 段の一覧・値 = 数えた数。
pub(crate) fn check_counts(ctx: &Ctx<'_>, counts: &X<'_>) -> R<()> {
    let pairs = counts.pairs()?;
    let same_keys = pairs.len() == ctx.tiers.len()
        && pairs
            .iter()
            .all(|(k, _)| ctx.tiers.iter().any(|(t, _)| t == k));
    if !same_keys {
        return Err(format!("{}: キーの集合が段の一覧と違う", counts.at));
    }
    for (key, x) in pairs {
        let counted = ctx.arts.iter().filter(|a| a.tier_key == key).count() as u64;
        let written = x.count()?;
        if written != counted {
            return Err(format!("{}: {written} だが数えた条は {counted}", x.at));
        }
    }
    Ok(())
}

/// 条の id の列の欠番（接頭辞ごとに 1 から最大の数まで・接頭辞は正本の初出の順・中は数の小さい順・便 80 §1 (a)）。
/// 番号が符号なしの整数に読めない id は Err（P-4.1）。
pub(crate) fn missing_numbers(ctx: &Ctx<'_>) -> R<Vec<String>> {
    let mut groups: Vec<(&str, Vec<u32>)> = Vec::new();
    for art in &ctx.arts {
        let (prefix, n) = art
            .id
            .rsplit_once('-')
            .and_then(|(p, n)| Some((p, n.parse::<u32>().ok()?)))
            .ok_or_else(|| format!("条の id「{}」の番号が読めない", art.id))?;
        match groups.iter_mut().find(|(p, _)| *p == prefix) {
            Some((_, ns)) => ns.push(n),
            None => groups.push((prefix, vec![n])),
        }
    }
    let mut gaps = Vec::new();
    for (prefix, ns) in &groups {
        let max = ns.iter().copied().max().unwrap_or(0);
        for k in 1..=max {
            if !ns.contains(&k) {
                gaps.push(format!("{prefix}-{k}"));
            }
        }
    }
    Ok(gaps)
}

/// 関係の欄 → リンクの列（reqs・rules・articles・sections の順・群の中は正本の順）。
pub(crate) fn relations(ctx: &Ctx<'_>, rel: &X<'_>) -> R<Vec<String>> {
    let mut links = Vec::new();
    if let Some(reqs) = rel.g("reqs")? {
        for q in reqs.seq()? {
            let id = q.id()?;
            let title = ctx
                .reqs
                .iter()
                .find(|(k, _)| k == id)
                .map(|(_, t)| t)
                .ok_or_else(|| format!("{}: 要件書に id「{id}」が無い", q.at))?;
            links.push(format!(
                "<a class=\"xref\" href=\"srs.html#{}\">要件書 {id}（{title}）</a>",
                anchor(id)
            ));
        }
    }
    if let Some(rules) = rel.g("rules")? {
        for q in rules.seq()? {
            let id = q.id()?;
            if !ctx.rule_ids.iter().any(|k| k == id) {
                return Err(format!("{}: rules 行 id「{id}」が無い", q.at));
            }
            links.push(format!(
                "<a class=\"xref\" href=\"#{}\">rules {id}</a>",
                anchor(id)
            ));
        }
    }
    if let Some(articles) = rel.g("articles")? {
        for q in articles.seq()? {
            let art = ctx.article(&q)?;
            links.push(format!(
                "<a class=\"xref\" href=\"#{}\">{} {}</a>",
                anchor(art.id),
                art.id,
                art.title
            ));
        }
    }
    if let Some(sections) = rel.g("sections")? {
        for q in sections.seq()? {
            let text = q.text()?;
            let to = section_anchor(&text).map_err(|e| format!("{}: {e}", q.at))?;
            links.push(format!(
                "<a class=\"xref\" href=\"#{to}\">{}</a>",
                esc(&text)
            ));
        }
    }
    Ok(links)
}

/// 改訂の段 1 つの読み（n・担当・持ち主か・what の前後・条へのリンクの素）。
pub(crate) struct Step<'a> {
    pub(crate) n: u64,
    pub(crate) who: String,
    pub(crate) owner: bool,
    pub(crate) head: String,
    pub(crate) tail: Option<String>,
    pub(crate) arts: Vec<&'a Art<'a>>,
}

pub(crate) fn step<'a>(ctx: &'a Ctx<'a>, st: &X<'_>) -> R<Step<'a>> {
    let n = st.f("n")?.count()?;
    let who_x = st.f("who")?;
    let who = who_x.e()?;
    let owner = who_x.v.as_str() == Some("持ち主");
    let what = st.f("what")?.text()?;
    let (head, tail) = split_dash(&what);
    let arts = st
        .f("article")?
        .seq()?
        .iter()
        .map(|q| ctx.article(q))
        .collect::<R<Vec<_>>>()?;
    Ok(Step {
        n,
        who,
        owner,
        head: esc(head),
        tail: tail.map(esc),
        arts,
    })
}
