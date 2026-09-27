//! 画面の中身を決める純粋な関数（便 g-min）: 並べ方・状態の印・測れていないの判定・時刻の字。
//! DOM と通信に触らないので host の cargo test で試す（描くのは board・読むのは net）。

use std::cmp::Ordering;

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{LedgerList, LedgerRow};
use tsuzuri_contract::wire;

/// 問いの bead の種類。
pub const QUESTION_KIND: &str = "question";

/// epic の bead の種類。
pub const EPIC_KIND: &str = "epic";

/// 台帳の一覧の口から読んだ結果（net が作る）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fetched {
    /// 口が 200 で返した本文。
    Body(String),
    /// 口に届かない・200 でない・本文が読めない。
    Failed,
}

/// 1 つの epic とその下の bead（epic の外の bead は `epic` が None の組に入る）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpicGroup {
    pub epic: Option<LedgerRow>,
    pub children: Vec<LedgerRow>,
}

/// 読めた台帳から組んだ画面の中身（問いの一覧と台帳の一覧）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub questions: Vec<LedgerRow>,
    pub groups: Vec<EpicGroup>,
}

/// 画面の状態。台帳が読めなければ `Unknown`（0 件と区別する・要件 NFR2）。
#[derive(Debug, Clone, PartialEq)]
pub struct Screen {
    pub board: Reading<Board>,
    /// 最後に読めた時刻（まだ 1 度も読めていなければ None）。
    pub updated_at: Option<EpochSecs>,
}

impl Screen {
    /// 起動の直後（まだ読んでいない＝測れていない）。
    pub fn initial() -> Self {
        Self {
            board: Reading::Unknown,
            updated_at: None,
        }
    }

    /// 一覧の口を読んだ後の状態。読めなければ測れていないにし、最終更新は前の値を残す。
    pub fn after_read(&self, fetched: &Fetched, at: EpochSecs) -> Self {
        match read_rows(fetched) {
            Reading::Known(rows) => Self {
                board: Reading::Known(board(&rows)),
                updated_at: Some(at),
            },
            Reading::Unknown => self.after_lost(),
        }
    }

    /// 変化の知らせが切れた後の状態（届かない間は今の一覧が正しいと言えない）。
    pub fn after_lost(&self) -> Self {
        Self {
            board: Reading::Unknown,
            updated_at: self.updated_at,
        }
    }
}

/// 口の本文を台帳の行に読む（届かない・電文が読めない・台帳が読めないは Unknown）。
pub fn read_rows(fetched: &Fetched) -> Reading<Vec<LedgerRow>> {
    match fetched {
        Fetched::Body(body) => match wire::decode::<LedgerList>(body) {
            Ok(list) => list.rows,
            Err(_) => Reading::Unknown,
        },
        Fetched::Failed => Reading::Unknown,
    }
}

/// 台帳の行から画面の中身を組む。
pub fn board(rows: &[LedgerRow]) -> Board {
    Board {
        questions: questions(rows),
        groups: ledger_groups(rows),
    }
}

/// 問いの一覧: open の question を古い順（更新時刻の昇順・同じ時刻は id の順）。
pub fn questions(rows: &[LedgerRow]) -> Vec<LedgerRow> {
    let mut out: Vec<LedgerRow> = rows
        .iter()
        .filter(|r| r.kind == QUESTION_KIND && r.status == "open")
        .cloned()
        .collect();
    out.sort_by(|a, b| {
        a.updated_at
            .cmp(&b.updated_at)
            .then_with(|| id_order(a.id.as_str(), b.id.as_str()))
    });
    out
}

/// 台帳の一覧: epic ごとの組（epic の id の順）に、その下の bead を task・memo・その他の順で並べる。
/// 親は id の階層（`a.1` の親は `a`）で決め、いちばん近い epic の祖先の下に置く。
/// epic の祖先の無い bead は最後の組（epic が None）に入る。question は問いの一覧が持つので入れない。
pub fn ledger_groups(rows: &[LedgerRow]) -> Vec<EpicGroup> {
    let mut epics: Vec<&LedgerRow> = rows.iter().filter(|r| r.kind == EPIC_KIND).collect();
    epics.sort_by(|a, b| id_order(a.id.as_str(), b.id.as_str()));
    let mut groups: Vec<EpicGroup> = epics
        .iter()
        .map(|e| EpicGroup {
            epic: Some((*e).clone()),
            children: Vec::new(),
        })
        .collect();
    let mut outside = Vec::new();
    for row in rows
        .iter()
        .filter(|r| r.kind != EPIC_KIND && r.kind != QUESTION_KIND)
    {
        let home =
            ancestors(row.id.as_str()).find_map(|a| epics.iter().position(|e| e.id.as_str() == a));
        match home {
            Some(i) => groups[i].children.push(row.clone()),
            None => outside.push(row.clone()),
        }
    }
    for group in &mut groups {
        sort_children(&mut group.children);
    }
    if !outside.is_empty() {
        sort_children(&mut outside);
        groups.push(EpicGroup {
            epic: None,
            children: outside,
        });
    }
    groups
}

/// id の祖先（近い順: `a.b.c` なら `a.b`・`a`）。
fn ancestors(id: &str) -> impl Iterator<Item = &str> {
    std::iter::successors(parent(id), |p| parent(p))
}

fn parent(id: &str) -> Option<&str> {
    id.rsplit_once('.').map(|(p, _)| p)
}

/// epic の下の順: task・memo・その他、同じ種類の中は id の順。
fn sort_children(children: &mut [LedgerRow]) {
    children.sort_by(|a, b| {
        kind_rank(&a.kind)
            .cmp(&kind_rank(&b.kind))
            .then_with(|| id_order(a.id.as_str(), b.id.as_str()))
    });
}

fn kind_rank(kind: &str) -> u8 {
    match kind {
        "task" => 0,
        "memo" => 1,
        _ => 2,
    }
}

/// id の順（`.` で区切った段ごとに、数なら数の大小・字なら字の順: `a.2` は `a.10` より前）。
pub fn id_order(a: &str, b: &str) -> Ordering {
    let mut xs = a.split('.');
    let mut ys = b.split('.');
    loop {
        match (xs.next(), ys.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => {
                let o = match (x.parse::<u64>(), y.parse::<u64>()) {
                    (Ok(m), Ok(n)) => m.cmp(&n).then_with(|| x.cmp(y)),
                    _ => x.cmp(y),
                };
                if o != Ordering::Equal {
                    return o;
                }
            }
        }
    }
}

/// 状態の印（字の印・日本語の 1 語・class の名）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mark {
    pub glyph: &'static str,
    pub word: &'static str,
    pub class: &'static str,
}

/// bd の状態の語を印にする（知らない語は「?」の印で、語は title に残す）。
pub fn mark(status: &str) -> Mark {
    let (glyph, word, class) = match status {
        "open" => ("○", "未着手", "st-open"),
        "in_progress" => ("◐", "作業中", "st-in-progress"),
        "blocked" => ("⊘", "止まり", "st-blocked"),
        "deferred" => ("◌", "先送り", "st-deferred"),
        "closed" => ("●", "閉じた", "st-closed"),
        _ => ("?", "知らない状態", "st-other"),
    };
    Mark { glyph, word, class }
}

/// 測れていないの字。
pub const UNMEASURED: &str = "測れていない";

/// 問いの件数の字（測れていなければ 0 件と書かない）。
pub fn questions_label(screen: &Screen) -> String {
    match &screen.board {
        Reading::Known(b) => format!("問い {} 件", b.questions.len()),
        Reading::Unknown => format!("問い {UNMEASURED}"),
    }
}

/// 台帳の件数の字（epic も数える・測れていなければ 0 件と書かない）。
pub fn ledger_label(screen: &Screen) -> String {
    match &screen.board {
        Reading::Known(b) => {
            let n: usize = b
                .groups
                .iter()
                .map(|g| g.children.len() + usize::from(g.epic.is_some()))
                .sum();
            format!("台帳 {n} 件")
        }
        Reading::Unknown => format!("台帳 {UNMEASURED}"),
    }
}

/// 最終更新の字。
pub fn updated_label(screen: &Screen) -> String {
    match screen.updated_at {
        Some(at) => format!("最終更新 {}", clock(at)),
        None => "最終更新 まだ無い".to_string(),
    }
}

/// epoch 秒を UTC の字にする（`2026-09-27 07:39:00 UTC`）。
pub fn clock(at: EpochSecs) -> String {
    let days = at / 86_400;
    let secs = at % 86_400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02} UTC",
        secs / 3600,
        secs % 3600 / 60,
        secs % 60
    )
}

/// 1970-01-01 からの日数を (年, 月, 日) にする（先発グレゴリオ暦）。
fn civil_from_days(days: u64) -> (u64, u64, u64) {
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z % 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + u64::from(m <= 2);
    (y, m, d)
}
