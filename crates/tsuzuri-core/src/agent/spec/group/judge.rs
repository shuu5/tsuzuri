//! 群の起こしの門の判じ（判断の記録 ADR-61 決定 (1)(2)(4)(5)・要件 FR22）。
//! 順は、群の行と 2 つの file・型と下書きの種類と引き金と番と数と 1 体の割り・割りの主張の和・生きた群の係の数と頭の予算とモデルと群の合計・
//! 同じ対象のほかの群の 7 日。断りは歯が見る短い鍵と理由の字の中身を持つ。通す時は群の席の札を組む。
//! 同じ群の係どうしの対象の重なりを ADR-59 の重なりの判じから外すのは境界の口（席の札の群の id で外す・決定 (7) の (ア)）。

use tsuzuri_contract::EpochSecs;

use super::{
    CLAIMS_MAX, DRAFTS, K_MAX, KEY, LIST, LIST_MAX, LIST_MIN, LIVE_MAX, Line, MEMBER_NEW, Member,
    PLAN, Plan, Seat, TYPE, UNCOUNTED, WINDOW, list_ids, named, question_of, reached, ruled,
};
use crate::agent::spec::{Head, Spec};

/// 台帳の問い（裁定 id の問いの欄の bead）の字。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ledger {
    Unread,
    Read { notes: String, description: String },
}

/// 群の起こしの門の入力（頭は ADR-59 の 4 行を読めた値・file の字は無ければ None・群の合計は読めなければ None・
/// 台帳は計画の裁定 id の問いを読んだ字で、裁定 id が無ければ None・ほかの群の係は札と群の席の札の組）。
#[derive(Debug, Clone)]
pub struct Ask<'a> {
    pub kind: Option<&'a str>,
    pub name: &'a str,
    pub line: &'a str,
    pub head: &'a Head,
    pub model: Option<&'a str>,
    pub plan: Option<&'a str>,
    pub list: Option<&'a str>,
    pub peers: &'a [(Spec, Seat)],
    pub totals: Option<(u64, u64)>,
    pub ledger: Option<&'a Ledger>,
    pub now: EpochSecs,
}

/// 群の起こしの門の断り（歯が見る短い鍵と、理由の字の中身）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub key: &'static str,
    pub what: String,
}

fn refuse(key: &'static str, what: String) -> Refusal {
    Refusal { key, what }
}

/// 群の行の無い起こしの名を割りに持つ計画（群の id と計画の字の組）が在れば、5 行目の欠けの断り。
pub fn unmarked(name: &str, plans: &[(String, String)]) -> Option<Refusal> {
    let (group, _) = plans.iter().find(|(_, text)| {
        Plan::parse(text).is_some_and(|p| p.members.iter().any(|m| m.name == name))
    })?;
    let what = format!("群 {group} の計画が割りに持つ係 {name} の頭に {KEY}: の行が無い");
    Some(refuse("line-missing", what))
}

/// 群の行と計画の file と一覧の file を読む（欠けと形の誤りと、計画の対象と頭の対象の違いを断る）。
fn files(ask: &Ask) -> Result<(Line, Plan, Vec<String>), Refusal> {
    let Some(line) = Line::parse(ask.line) else {
        let what = format!("{KEY}: の行 {} が <群 id> <i>/<k> の形でない", ask.line);
        return Err(refuse("line-form", what));
    };
    let g = &line.group;
    let Some(text) = ask.plan else {
        return Err(refuse("plan-missing", format!("群 {g} の {PLAN} が無い")));
    };
    let Some(plan) = Plan::parse(text).filter(|p| p.target == ask.head.target) else {
        let what = format!("群 {g} の {PLAN} が計画の形でないか対象が頭と違う");
        return Err(refuse("plan-form", what));
    };
    let Some(text) = ask.list else {
        return Err(refuse("list-missing", format!("群 {g} の {LIST} が無い")));
    };
    let Some(ids) = list_ids(text) else {
        let what = format!("群 {g} の {LIST} の行がタブで区切った 4 欄でないか id が重なる");
        return Err(refuse("list-form", what));
    };
    Ok((line, plan, ids))
}

/// 型と下書きの種類と引き金と番と数と 1 体の割りを判じる（割りは呼びの名の係）。
fn shape<'p>(ask: &Ask, line: &Line, plan: &'p Plan, rows: usize) -> Result<&'p Member, Refusal> {
    let (i, k, draft) = (line.i, line.k, &plan.draft);
    if ask.kind != Some(TYPE) || plan.kind != TYPE {
        return Err(refuse("type", format!("群に入れる型は {TYPE} だけ")));
    }
    if !DRAFTS.contains(&draft.as_str()) {
        let what = format!(
            "下書きの種類 {draft} が {} のどれでもない",
            DRAFTS.join("・")
        );
        return Err(refuse("draft", what));
    }
    if rows < LIST_MIN {
        return Err(refuse(
            "few",
            format!("一覧の行 {rows} が {LIST_MIN} に届かない"),
        ));
    }
    if rows > LIST_MAX {
        return Err(refuse(
            "many",
            format!("一覧の行 {rows} が {LIST_MAX} を越える"),
        ));
    }
    if i > k {
        return Err(refuse("index", format!("番 {i} が数 {k} を越える")));
    }
    if k > K_MAX {
        return Err(refuse("wide", format!("数 {k} が {K_MAX} を越える")));
    }
    let Some(me) = plan.members.iter().find(|m| m.name == ask.name) else {
        let what = format!("群 {} の {PLAN} の割りに係 {} が無い", line.group, ask.name);
        return Err(refuse("plan-form", what));
    };
    if me.claims.len() > CLAIMS_MAX {
        let what = format!(
            "係 {} の割りの主張 {} が {CLAIMS_MAX} を越える",
            me.name,
            me.claims.len()
        );
        return Err(refuse("claims", what));
    }
    Ok(me)
}

/// 割りの主張の id の係どうしの重なりと、割りの和と一覧の行の全部との違い（欠けと余り）を判じる。
fn claims(plan: &Plan, ids: &[String]) -> Result<(), Refusal> {
    let all: Vec<&String> = plan.members.iter().flat_map(|m| &m.claims).collect();
    let twice = all
        .iter()
        .enumerate()
        .find(|(n, c)| all.iter().take(*n).any(|x| x == *c));
    if let Some((_, c)) = twice {
        return Err(refuse("overlap", format!("割りの主張 {c} を 2 体が持つ")));
    }
    let missing: Vec<&str> = ids
        .iter()
        .filter(|i| !all.contains(i))
        .map(String::as_str)
        .collect();
    if !missing.is_empty() {
        let what = format!("一覧の主張 {} が割りに無い", missing.join("・"));
        return Err(refuse("missing", what));
    }
    let extra: Vec<&str> = all
        .iter()
        .filter(|c| !ids.contains(c))
        .map(|c| c.as_str())
        .collect();
    if !extra.is_empty() {
        let what = format!("割りの主張 {} が一覧に無い", extra.join("・"));
        return Err(refuse("extra", what));
    }
    Ok(())
}

/// 同時に生きた群の係の数と、頭の予算と、モデルと、群の合計を判じる。
fn load(ask: &Ask, line: &Line, me: &Member) -> Result<(), Refusal> {
    let live = ask.peers.iter().filter(|(s, _)| s.ended.is_none()).count();
    if live >= LIVE_MAX {
        let what = format!("生きた群の係が {live} 体で、起こすと {LIVE_MAX} を越える");
        return Err(refuse("live", what));
    }
    let budget = ask.head.budget;
    if budget > MEMBER_NEW {
        let what = format!("頭の予算 {budget} が係ごとの上限 {MEMBER_NEW} を越える");
        return Err(refuse("budget", what));
    }
    if ask.model != Some(me.model.as_str()) {
        let got = ask.model.unwrap_or("（無し）");
        let what = format!("呼びのモデル {got} が計画の値 {} と違う", me.model);
        return Err(refuse("model", what));
    }
    let g = &line.group;
    let Some((new, read)) = ask.totals else {
        return Err(refuse("totals-unread", format!("群 {g} の合計が読めない")));
    };
    if reached(new, read) {
        let what = format!("群 {g} の合計（新しい量 {new}・読み直し {read}）が止める線に届いた");
        return Err(refuse("stop", what));
    }
    Ok(())
}

/// 同じ対象のほかの群が 7 日の内に在れば、計画の裁定 id を台帳の問いの字で照らす（照らせない時は断る）。
fn window(ask: &Ask, line: &Line, plan: &Plan) -> Result<(), Refusal> {
    let target = &ask.head.target;
    let Some((_, other)) = ask.peers.iter().find(|(s, seat)| {
        seat.group != line.group
            && s.target == *target
            && ask.now.saturating_sub(s.spawned) < WINDOW
    }) else {
        return Ok(());
    };
    let near = format!("同じ対象 {target} の群 {} が 7 日の内に在り", other.group);
    let Some(ruling) = plan.ruling.as_deref() else {
        return Err(refuse("window", format!("{near}、計画に裁定 id が無い")));
    };
    let q = question_of(ruling);
    if UNCOUNTED.contains(&q) {
        let what = format!("{near}、裁定 id {ruling} の問い {q} は数えない");
        return Err(refuse("uncounted", what));
    }
    let Some(Ledger::Read { notes, description }) = ask.ledger else {
        let what = format!("{near}、台帳の問い {q} が読めない");
        return Err(refuse("ledger-unread", what));
    };
    if !ruled(notes, ruling) {
        let what = format!("{near}、裁定 id {ruling} が問い {q} の裁定の定型行に無い");
        return Err(refuse("ruling-absent", what));
    }
    if !named(description, &line.group) {
        let what = format!(
            "{near}、問い {q} の本文が群 {} を語として名指さない",
            line.group
        );
        return Err(refuse("group-unnamed", what));
    }
    Ok(())
}

/// 群の起こしの門の判じ。通す時は群の席の札（群の id・番・数・呼びの名の係の割り）。
pub fn judge(ask: &Ask) -> Result<Seat, Refusal> {
    let (line, plan, ids) = files(ask)?;
    let me = shape(ask, &line, &plan, ids.len())?;
    claims(&plan, &ids)?;
    load(ask, &line, me)?;
    window(ask, &line, &plan)?;
    Ok(Seat {
        group: line.group,
        i: line.i,
        k: line.k,
        claims: me.claims.clone(),
        paths: me.paths.clone(),
    })
}

/// 群の起こしの断りの理由の字（何が違うかと次の一手）。
pub fn reason(r: &Refusal) -> String {
    format!(
        "群の起こしの門は止める（{}） 次の一手 = 計画の file・一覧の file・頼みの頭を直すか、{KEY}: の行の無い 1 体の起こしにする（群の値と形は判断の記録 ADR-61）",
        r.what
    )
}
