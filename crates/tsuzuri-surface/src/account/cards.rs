//! account board の各 project の表の hover の card: 5 つの欄と群の見出しの chip の card を組む純粋な関数
//! （見本の account/index.html の card の nx・ledCard・pcnt・seatCard・gproj・group の枝）。
//! 行 h-acct-split で account の projects から移した（振る舞いは同じ）。
//! 台帳の欄の card は台帳の処理状況の表が、次の一手と口座の欄の card は HOME が、
//! orchestrator の欄の card は session の表も使う。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::account::{AccountDoc, ProjectRow, WindowCap};
use tsuzuri_contract::board::{NextMove, Reading};
use tsuzuri_contract::stats::CheckResult;

use super::home::group_more;
use super::projects::{
    NONE_MARK, RUNS, acc, count_text, current, group_word, is_park, need, wait_of,
};
use crate::project::ledger::{age, fixed1, net};
use crate::project::next::{big, key as next_key};
use crate::project::seat::{NG, PARK_KIND, hm, hmd, park_card, short, state_value};
use crate::project::{UNKNOWN, state_key};
use crate::vocab::label;
use crate::widgets::hover::Card;

/// 次の一手の出所（電文の next を判じる中核の関数）。
pub const NX_SRC: &str = "中核の next_step_seat";

/// 次の一手の card の当たらない種の字（見本の nx の枝）。
pub const NX_MISS: &str = "なし";

/// 分からない値の字（card の中の口座・model・tick・hb）。
const ASK: &str = "?";

/// 次の一手の 6 種（`NextMove::ALL` からなしを除いた順）。
fn nx_kinds() -> impl Iterator<Item = NextMove> {
    NextMove::ALL
        .into_iter()
        .filter(|k| *k != NextMove::Nothing)
}

/// 要対応の欄の card（見本の nx の枝: 題・当たりの種の字・1 行・出所・6 種の結果）。
pub fn nx_card(project: &ProjectRow, at: EpochSecs) -> Card {
    let n = need(project);
    let title = match n.lead {
        Some(_) => format!("{} · ({}) {}", project.name, &n.key[3..], label(n.key)),
        None => format!("{} · {}", project.name, label(n.key)),
    };
    let checks = match &project.next {
        Reading::Known(s) => s.checks.as_slice(),
        Reading::Unknown => &[],
    };
    let check = |kind: NextMove| checks.iter().find(|c| c.kind == kind);
    let mut hits: Vec<&str> = nx_kinds()
        .filter(|k| check(*k).is_some_and(|c| c.result == CheckResult::Hit))
        .map(|k| &next_key(k)[3..])
        .collect();
    if n.lead == Some(NextMove::Nothing) {
        hits.push(&next_key(NextMove::Nothing)[3..]);
    }
    let hits = if hits.is_empty() {
        NONE_MARK.to_string()
    } else {
        hits.join(" ")
    };
    let more = nx_kinds()
        .map(|k| {
            let tail = match check(k) {
                Some(c) if c.result == CheckResult::Hit => format!("· {}", big(k, Some(c)).what),
                Some(c) if c.result == CheckResult::Miss => NX_MISS.to_string(),
                _ => NONE_MARK.to_string(),
            };
            format!("({}) {} {tail}", &next_key(k)[3..], label(next_key(k)))
        })
        .collect();
    Card {
        title,
        kind: format!("{} · {hits}", label("next_all")),
        value: n.line.unwrap_or_else(|| NONE_MARK.to_string()),
        src: format!("{NX_SRC} · ◷ {}", hm(at)),
        more,
    }
}

/// 台帳の数の出所（account の server が anchor ごとに読む台帳の出力）。
pub const LED_SRC: &str = "bd list --all の bead";

/// 台帳の欄の card（見本の ledCard: 純減 24h と 7d と closed/日・open の task・出所と時点・memo と stale ほか）。
pub fn led_card(project: &ProjectRow) -> Card {
    match &project.ledger {
        Reading::Known(s) => {
            let n24 = net(s.net_drop_24h);
            let n7 = net(s.net_drop_7d);
            let lead = s.lead.as_ref();
            Card {
                title: format!("{} · {}", project.name, label("ledger_state")),
                kind: format!(
                    "{}{} 24h · {}{} 7d · {} {}",
                    n24.arrow,
                    n24.text,
                    n7.arrow,
                    n7.text,
                    label("l_rate"),
                    fixed1(s.closed_per_day)
                ),
                value: format!(
                    "task {}（ready {} / blocked {}）",
                    s.open.task, s.ready, s.blocked
                ),
                src: format!("{LED_SRC} · 時点 {}", hm(s.at)),
                more: vec![
                    format!("memo {} · stale {}", s.open.memo, s.stale),
                    format!("question {} · epic {}", s.open.question, s.open.epic),
                    format!(
                        "lead p50 {} / p90 {}",
                        age(lead.map(|l| l.p50)),
                        age(lead.map(|l| l.p90))
                    ),
                ],
            }
        }
        Reading::Unknown => Card {
            title: format!("{} · {}", project.name, label("j_none")),
            kind: label("ledger_state"),
            value: label(state_key(UNKNOWN)),
            src: LED_SRC.to_string(),
            more: Vec::new(),
        },
    }
}

/// run の数が読めない行の詳しくの 1 行。
pub const RUNS_UNKNOWN: &str = "state dir か event log が読めない";

/// 決定待ちの「―」の読み方（見本の pcnt の枝）。
pub const WAIT_NOTE: &str = "― = 質問の台帳を読んでいない";

/// run の数の欄の card（見本の pcnt の枝: 4 列の数・決定待ち・出所は state dir ごとの event log）。
pub fn pcnt_card(project: &ProjectRow, at: EpochSecs) -> Card {
    let (kind, mut more) = match &project.runs {
        Reading::Known(c) => {
            let counts = [c.wait, c.run, c.stop, c.land];
            let kind = RUNS
                .iter()
                .zip(counts)
                .map(|((name, _), n)| format!("{name} {n}"))
                .collect::<Vec<_>>()
                .join(" · ");
            let more = ["col_wait", "col_run", "col_stop", "col_land"]
                .into_iter()
                .zip(counts)
                .map(|(k, n)| format!("{} {n}", label(k)))
                .collect();
            (kind, more)
        }
        Reading::Unknown => (label(state_key(UNKNOWN)), vec![RUNS_UNKNOWN.to_string()]),
    };
    let wait = wait_of(project);
    if wait.is_none() {
        more.push(WAIT_NOTE.to_string());
    }
    Card {
        title: format!("{} · {}", label("runs4"), project.name),
        kind,
        value: format!("{} {}", label("waiting_you"), count_text(wait)),
        src: format!("fleet/events.jsonl · ◷ {}", hm(at)),
        more,
    }
}

/// orchestrator の欄の card（見本の seatCard: 状態といつから・口座と tick と hb・model・移動待ち・退避までの残り）。
/// 席が Unknown の行は card を持たない。
pub fn orch_card(doc: &AccountDoc, project: &ProjectRow) -> Option<Card> {
    let Reading::Known(c) = &project.seat else {
        return None;
    };
    let state = label(state_key(state_value(c.state)));
    let kind = match c.since {
        Some(s) => format!("{state} · ◷ {} から", hmd(s, doc.at)),
        None => state,
    };
    let tick = match c.tick_healthy {
        Reading::Known(true) => "healthy",
        Reading::Known(false) => "stale",
        Reading::Unknown => ASK,
    };
    let hb = match c.heartbeat {
        Reading::Known(true) => "on",
        Reading::Known(false) => "off",
        Reading::Unknown => ASK,
    };
    let mut more = vec![format!("model {}", c.model.as_deref().unwrap_or(ASK))];
    let a = acc(doc, project);
    let now = project.group.as_deref().and_then(|g| current(doc, g));
    if let (Some(NG), Some(seat), Some(now)) = (a.mark, a.account.as_deref(), now) {
        more.push(format!("{} {seat} → {now}", label("seat_mismatch")));
    }
    if let Some(until) = project.move_until {
        let s = until.saturating_sub(doc.at);
        more.push(format!("{} {s} 秒", label("move_grace")));
    }
    Some(Card {
        title: c.target.clone(),
        kind,
        value: format!(
            "口座 {} · tick {tick} · hb {hb}",
            c.account.as_deref().unwrap_or(ASK)
        ),
        src: format!("seat/{}/state.jsonl ほか", c.target),
        more,
    })
}

/// 口座の欄の card の出所。
pub const GPROJ_SRC: &str = "doctor の席の行と群の今の記録";

/// 口座の欄の card（見本の gproj の枝: 席の口座と群の今の口座の一致・席の無い project は群の今の口座だけ）。
pub fn gproj_card(doc: &AccountDoc, project: &ProjectRow) -> Card {
    let now = project.group.as_deref().and_then(|g| current(doc, g));
    match &project.seat {
        Reading::Known(c) if project.group.as_deref().is_some_and(|g| is_park(doc, g)) => {
            // 区画の席は群の今の口座と比べない（行 g-park-view）。
            let a = acc(doc, project);
            Card {
                title: c.target.clone(),
                kind: group_word(project.group.as_deref(), true),
                value: format!(
                    "登録 {} · {PARK_KIND}",
                    a.account.as_deref().unwrap_or(ASK)
                ),
                src: GPROJ_SRC.to_string(),
                more: vec![label(state_key(state_value(c.state)))],
            }
        }
        Reading::Known(c) => {
            let a = acc(doc, project);
            let (kind, rel) = match a.mark {
                Some(s) if s == NG => (format!("{} {}", s.glyph, label("seat_mismatch")), "≠"),
                Some(s) => (format!("{} {}", s.glyph, label("seat_match")), "="),
                None => (label(state_key(UNKNOWN)), ASK),
            };
            Card {
                title: c.target.clone(),
                kind,
                value: format!(
                    "登録 {} {rel} 群 {}",
                    a.account.as_deref().unwrap_or(ASK),
                    now.unwrap_or(ASK)
                ),
                src: GPROJ_SRC.to_string(),
                more: vec![label(state_key(state_value(c.state)))],
            }
        }
        Reading::Unknown => Card {
            title: project.name.clone(),
            kind: label("seat_none"),
            value: format!(
                "{} の今の口座 {}",
                project.group.as_deref().unwrap_or(NONE_MARK),
                now.unwrap_or(ASK)
            ),
            src: GPROJ_SRC.to_string(),
            more: Vec::new(),
        },
    }
}

/// 閾値の行（見本の group の枝の閾値の行・窓ごとに短い字と値と %・読めない値は「?」・空の列は「―」）。
pub fn thr_line(caps: &[WindowCap]) -> String {
    let parts: Vec<String> = caps
        .iter()
        .map(|c| match c.cap {
            Reading::Known(v) => format!("{} {v}%", short(&c.window)),
            Reading::Unknown => format!("{} {ASK}", short(&c.window)),
        })
        .collect();
    let text = if parts.is_empty() {
        NONE_MARK.to_string()
    } else {
        parts.join(" · ")
    };
    format!("{} {text}", label("threshold"))
}

/// 群の見出しの chip の card（見本の group の枝: 今の口座・記録の数・いつからと前の口座・候補と閾値と anchor）。
/// 電文の groups が読めないか群が無ければ None。
/// 名が電文の parks に在れば、群の枠でなく区画の短い card（行 g-park-view）。
pub fn grp_card(doc: &AccountDoc, name: &str) -> Option<Card> {
    if is_park(doc, name) {
        return Some(park_card(name));
    }
    let Reading::Known(cards) = &doc.groups else {
        return None;
    };
    let g = cards.iter().find(|g| g.row.group == name)?;
    let previous = g.previous.as_deref().unwrap_or(NONE_MARK);
    let value = match (g.recorded, g.since) {
        (true, Some(s)) => format!("◷ {} から · ← 前 {previous}", hmd(s, doc.at)),
        _ => format!("{} · ← 前 {previous}", label("no_record")),
    };
    let mut more = vec![
        format!(
            "候補 {} 口座 → {}",
            g.row.candidates.len(),
            label("candidates")
        ),
        thr_line(&doc.caps),
        format!("anchor {} 件", g.members.len()),
    ];
    more.extend(g.members.iter().map(|m| format!("・ {}", m.project)));
    Some(Card {
        title: format!("{name} · {} {}", label("current_account"), g.row.account),
        kind: format!("{} · {}", label("group"), group_more(g, &doc.moves).records),
        value,
        src: format!("groups/{name}.account ほか"),
        more,
    })
}

/// 行の 5 つの欄の card（orchestrator と口座の欄は席の在る行だけ）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowCards {
    pub need: Card,
    pub led: Card,
    pub runs: Card,
    pub orch: Option<Card>,
    pub acc: Option<Card>,
}

pub fn row_cards(doc: &AccountDoc, project: &ProjectRow) -> RowCards {
    RowCards {
        need: nx_card(project, doc.at),
        led: led_card(project),
        runs: pcnt_card(project, doc.at),
        orch: orch_card(doc, project),
        acc: match &project.seat {
            Reading::Known(_) => Some(gproj_card(doc, project)),
            Reading::Unknown => None,
        },
    }
}
