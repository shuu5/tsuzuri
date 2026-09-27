//! 台帳の読み（bd の読み取りの口を子 process で 1 本撃つだけ・書かない・便 e-src）。
//! 撃つ形は `bd --readonly list --all --limit 0 --json`（cwd は repo の置き場・標準入力は空・標準エラーは捨てる）。
//! 起動できない・rc が 0 でない・JSON として読めない・5 秒を超えて返さない、のどれでも
//! 一覧は 0 件でなく「まだ分からない」（Reading::Unknown）にする。
//! .beads の issues.jsonl は変化の印（更新時刻と長さ）として見るだけで、中身は読まない。
//! 同じ `Source` とその clone の読みは、走っている 1 本の子 process を分け合う（`coalesce`・便 e-coalesce）。

use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{BdLine, BeadId, LedgerItem, LedgerList};
use tsuzuri_contract::wire;

use super::coalesce::{Coalesce, GRACE};

/// 子 process を撃つ部品と時刻の読みは `proc` と `clock` に在り、今までの名のまま再公開する（行 hb-proc）。
pub use super::clock::epoch_secs;
pub use super::proc::{KILL, capture, stop_with};

/// 既定の program の名（引数 --bd で替える）。
pub const BD: &str = "bd";

/// bd に渡す引数の列（読み取りだけ・closed を含む全部・件数の上限なし・JSON）。
pub const BD_ARGS: [&str; 6] = ["--readonly", "list", "--all", "--limit", "0", "--json"];

/// bd が返すまでの上限（要件 NFR2 の上限・規則の行 R-21 の値）。越えれば止めて Unknown。
pub const BD_TIMEOUT: Duration = Duration::from_secs(5);

/// 走っている読みに合流した呼び出しが待つ上限（`BD_TIMEOUT` に 1 秒を足す・便 e-coalesce）。
pub const BD_WAIT: Duration = BD_TIMEOUT.saturating_add(GRACE);

/// 台帳の読みの出所（repo の置き場と bd の program）。
/// clone は読みの合流の場を分け合う（比べるのは repo と bd だけ）。
#[derive(Debug, Clone)]
pub struct Source {
    pub repo: PathBuf,
    pub bd: OsString,
    shared: Coalesce<String>,
}

impl PartialEq for Source {
    fn eq(&self, other: &Source) -> bool {
        (&self.repo, &self.bd) == (&other.repo, &other.bd)
    }
}

impl Eq for Source {}

impl Source {
    pub fn new(repo: impl Into<PathBuf>, bd: impl Into<OsString>) -> Source {
        Source {
            repo: repo.into(),
            bd: bd.into(),
            shared: Coalesce::new(),
        }
    }

    /// 変化の印の 2 つの file（席の書きで動く issues.jsonl と、器の close を含む状態の変更で動く interactions.jsonl）。
    pub fn marks(&self) -> Vec<PathBuf> {
        let beads = self.repo.join(".beads");
        vec![beads.join("issues.jsonl"), beads.join("interactions.jsonl")]
    }

    /// bd を撃って台帳を読む。
    pub fn read(&self) -> Reading<Vec<LedgerItem>> {
        self.text().map_or(Reading::Unknown, |text| parse_bd(&text))
    }

    /// bd を撃ち、返した字をそのまま返す（導出グラフと指標の入力・便 e-read）。
    /// 起動できない・rc が 0 でない・UTF-8 でない・`BD_TIMEOUT` を越える、のどれでも None。
    /// 走っている読みが在れば新しく撃たず、その終わりを `BD_WAIT` まで待って同じ結果を返す（便 e-coalesce）。
    pub fn text(&self) -> Option<String> {
        self.shared.share(BD_WAIT, || self.text_alone())
    }

    /// 走っている読みを分け合わず、新しい子 process で bd を撃つ（裁定の受付の読み直し・便 e-ask）。
    pub fn text_alone(&self) -> Option<String> {
        let out = capture(&self.bd, BD_ARGS, &self.repo, BD_TIMEOUT)?;
        String::from_utf8(out).ok()
    }
}

/// bd の出力（bead の JSON の配列）を読む（server の読みの経路はこれだけ）。
/// 1 本でも読めなければ全体を Unknown にする（壊れた台帳の一部だけを見せない）。
pub fn parse_bd(text: &str) -> Reading<Vec<LedgerItem>> {
    match wire::decode::<Vec<BdLine>>(text) {
        Ok(lines) => items(lines.into_iter().map(Ok::<_, wire::Error>)),
        Err(_) => Reading::Unknown,
    }
}

/// bead を 1 本 1 行に並べた字（空の行は飛ばす）を読む。file は開かない純粋な関数で、
/// 面の歯が fixture を server と同じ規則で読むために残す（server の読みの経路では使わない）。
pub fn parse(text: &str) -> Reading<Vec<LedgerItem>> {
    items(
        text.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(wire::decode::<BdLine>),
    )
}

/// bd の bead の列を中身にする（bd が消した bead は出さない・1 本でも読めなければ Unknown）。
fn items<E>(lines: impl Iterator<Item = Result<BdLine, E>>) -> Reading<Vec<LedgerItem>> {
    let mut items = Vec::new();
    for bd in lines {
        let Ok(bd) = bd else {
            return Reading::Unknown;
        };
        if bd.is_tombstone() {
            continue;
        }
        let Some(at) = epoch_secs(&bd.updated_at) else {
            return Reading::Unknown;
        };
        items.push(bd.into_item(at));
    }
    Reading::Known(items)
}

/// 台帳の一覧（口 GET /api/ledger）。
pub fn list(source: &Source) -> LedgerList {
    LedgerList {
        rows: match source.read() {
            Reading::Known(items) => Reading::Known(items.into_iter().map(|i| i.row).collect()),
            Reading::Unknown => Reading::Unknown,
        },
    }
}

/// 1 本の引き（口 GET /api/ledger/<id>）の結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lookup {
    Found(LedgerItem),
    /// 台帳は読めたが id が無い。
    Missing,
    /// 台帳が読めない。
    Unknown,
}

pub fn item(source: &Source, id: &BeadId) -> Lookup {
    match source.read() {
        Reading::Known(items) => items
            .into_iter()
            .find(|i| &i.row.id == id)
            .map_or(Lookup::Missing, Lookup::Found),
        Reading::Unknown => Lookup::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::{Source, epoch_secs, parse, parse_bd};
    use tsuzuri_contract::board::Reading;

    #[test]
    fn server_min_epoch_secs_reads_rfc3339() {
        assert_eq!(epoch_secs("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(epoch_secs("2026-09-27T07:39:00Z"), Some(1_790_494_740));
        assert_eq!(
            epoch_secs("2026-09-27T16:39:00.123456789+09:00"),
            Some(1_790_494_740)
        );
        assert_eq!(epoch_secs("2024-02-29T12:00:00-00:30"), Some(1_709_209_800));
        for bad in [
            "",
            "2026-09-27T07:39:00",
            "2026-02-29T00:00:00Z",
            "2026-13-01T00:00:00Z",
            "2026-09-27T24:00:00Z",
            "2026-09-27T07:39:00.Z",
            "2026-09-27T07:39:00+0900",
            "1969-12-31T23:59:59Z",
            "２０２６-09-27T07:39:00Z",
        ] {
            assert_eq!(epoch_secs(bad), None, "{bad}");
        }
    }

    #[test]
    fn server_min_parse_refuses_broken_lines() {
        let good =
            r#"{"id":"fx.1","title":"t","status":"open","updated_at":"2026-09-27T07:39:00Z"}"#;
        let gone =
            r#"{"id":"fx.2","title":"t","status":"tombstone","updated_at":"2026-09-27T07:39:00Z"}"#;
        let Reading::Known(items) = parse_bd(&format!("[{good},\n{gone}]\n")) else {
            panic!("読める台帳が Unknown");
        };
        assert_eq!(items.len(), 1);
        assert_eq!(parse_bd("[]"), Reading::Known(vec![]));
        let bad_time = good.replace("07:39:00Z", "07:39:00");
        for broken in [
            format!("[{good},{{"),
            format!("[{bad_time}]"),
            good.to_string(),
            String::new(),
            "not json".to_string(),
        ] {
            assert_eq!(parse_bd(&broken), Reading::Unknown, "{broken}");
        }
        // 1 本 1 行の形（面の歯の fixture の読み）も同じ規則で読む。
        let Reading::Known(items) = parse(&format!("{good}\n\n{gone}\n")) else {
            panic!("読める行が Unknown");
        };
        assert_eq!(items.len(), 1);
        assert_eq!(parse(""), Reading::Known(vec![]));
        for broken in [format!("{good}\n{{"), bad_time, "not json".to_string()] {
            assert_eq!(parse(&broken), Reading::Unknown, "{broken}");
        }
    }

    #[test]
    fn server_src_marks_are_two_files() {
        assert_eq!(
            Source::new("/r", "bd").marks(),
            [
                std::path::Path::new("/r/.beads/issues.jsonl"),
                std::path::Path::new("/r/.beads/interactions.jsonl"),
            ]
        );
    }

    #[test]
    fn server_src_unstartable_bd_is_unknown() {
        let source = Source::new(std::env::temp_dir(), "/nonexistent/tz-no-such-bd");
        assert_eq!(source.read(), Reading::Unknown);
    }
}
