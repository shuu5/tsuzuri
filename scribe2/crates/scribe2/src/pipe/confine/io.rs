//! 囲いごとの装置への正味の書き（行 xp-io-bytes）。
//!
//! 測りの源は包みの `sh` の `/proc/$$/io`（`write_bytes − cancelled_write_bytes`）である。user の scope には io の
//! controller が委ねられない host が在り（`io.stat` が無い）、`/proc` の io は子を刈った親へ数が積もるので、行の
//! 終端で `sh` 自身の io を読めば刈った子孫の書きの和になる。刈られずに親を離れた子孫（孤児）の書きは入らない＝下限。
//! 読めない周（file が無い・欄が欠ける・数でない）は字 `unmeasured` と書き 0 と書かない（C10）。

use super::{read_usage, Confinement};

/// 終端行の epilogue が `__io`（読む file の path）から `__wb`（正味の byte か字 `unmeasured`）を組む shell の字。
///
/// 欄は `write_bytes:` と `cancelled_write_bytes:` の 2 つで、どちらかが欠けるか数でない周は `unmeasured` のまま。
pub(super) const NET_WRITE: &str = "__wb=unmeasured\n\
     __w=\n\
     __c=\n\
     [ -r \"$__io\" ] && while read -r __k __v; do case \"$__k\" in write_bytes:) __w=$__v;; cancelled_write_bytes:) __c=$__v;; esac; done < \"$__io\"\n\
     case \"$__w\" in ''|*[!0-9]*) ;; *) case \"$__c\" in ''|*[!0-9]*) ;; *) __wb=$((__w - __c));; esac;; esac\n";

/// 包めた周だけ stdout の終端行の書きを読む（包めなかった周の stdout は測定として読まない・`None`）。
pub fn written(confinement: &Confinement, stdout: &str) -> Option<u64> {
    if confinement.confined() {
        read_usage(stdout).write_bytes
    } else {
        None
    }
}

/// 測れない周の字。
pub const UNMEASURED: &str = "unmeasured";

/// 書きの字（測れた周は 10 進・測れない周は [`UNMEASURED`]）。
pub fn word(bytes: Option<u64>) -> String {
    bytes.map_or_else(|| UNMEASURED.to_owned(), |found| found.to_string())
}

/// 消費の event の `detail` に置く語（`write:<word>`）。
pub fn detail(bytes: Option<u64>) -> String {
    format!("write:{}", word(bytes))
}

/// 囲いの書きの和。1 つでも測れない囲いが在る周と囲いが 0 の周は `None`（部分の和を全体と書かない）。
pub fn total(parts: impl IntoIterator<Item = Option<u64>>) -> Option<u64> {
    let mut sum: Option<u64> = None;
    for part in parts {
        sum = Some(sum.unwrap_or(0).checked_add(part?)?);
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::super::{read_usage, script, Confinement, Reason};
    use super::{detail, total, word, written, NET_WRITE};
    use crate::fleet::store::{read_all, LockPolicy};
    use crate::fleet::{Cost, CostSource, Usage};
    use crate::pipe::{record_cost, record_cost_with};
    use std::process::Command;

    /// fixture の io の字を `__io` に置いて [`NET_WRITE`] を撃ち、`__wb` の字を返す（host の `/proc` を読まない）。
    fn net_of(io: Option<&str>) -> String {
        let dir = std::env::temp_dir().join(format!("xpio-{}-{}", std::process::id(), io.map_or(0, str::len)));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("io");
        let _ = std::fs::remove_file(&path);
        if let Some(text) = io {
            let _ = std::fs::write(&path, text);
        }
        let line = format!("__io='{}'\n{NET_WRITE}printf %s \"$__wb\"", path.display());
        let out = Command::new("sh").arg("-c").arg(line).output();
        let _ = std::fs::remove_dir_all(&dir);
        out.map(|found| String::from_utf8_lossy(&found.stdout).into_owned()).unwrap_or_default()
    }

    /// 正味は write_bytes から cancelled_write_bytes を引いた数で、ほかの欄（rchar・wchar・read_bytes）は数えない。
    #[test]
    fn xpio_net_write_subtracts_cancelled_bytes() {
        let io = "rchar: 900\nwchar: 800\nsyscr: 1\nsyscw: 2\nread_bytes: 4096\nwrite_bytes: 1052672\ncancelled_write_bytes: 4096\n";
        assert_eq!(net_of(Some(io)), "1048576");
    }

    /// 欄が欠ける・数でない・file が無い周は字 unmeasured（0 と書かない）。
    #[test]
    fn xpio_net_write_is_unmeasured_without_both_fields() {
        let whole = "write_bytes: 8192\ncancelled_write_bytes: 0\n";
        assert_eq!(net_of(Some(whole)), "8192", "対照");
        assert_eq!(net_of(Some("write_bytes: 8192\n")), "unmeasured", "cancelled の欠け");
        assert_eq!(net_of(Some("cancelled_write_bytes: 0\n")), "unmeasured", "write の欠け");
        assert_eq!(net_of(Some("write_bytes: 8x92\ncancelled_write_bytes: 0\n")), "unmeasured", "数でない");
        assert_eq!(net_of(None), "unmeasured", "file が無い");
    }

    /// 終端行は自分の `sh` の `/proc/$$/io` を読み、peak と oom の後に字 write_bytes= を足す。
    #[test]
    fn xpio_epilogue_reads_own_proc_io() {
        let body = script("true", "u1");
        assert!(body.contains("__io=/proc/$$/io\n"), "{body}");
        assert!(body.contains(NET_WRITE), "{body}");
        assert!(body.contains("confine-usage peak_bytes=%s oom_kill=%s write_bytes=%s\\n"), "{body}");
        assert!(body.contains("\"${__oom:--}\" \"$__wb\""), "{body}");
    }

    /// 終端行の write_bytes= は数の周だけ値を持ち、unmeasured・負・欠けは `None`。
    #[test]
    fn xpio_read_usage_reads_write_bytes() {
        let found = read_usage("confine-usage peak_bytes=1048576 oom_kill=0 write_bytes=5368709120\n");
        assert_eq!(found.write_bytes, Some(5_368_709_120));
        assert_eq!(found.peak_mb, Some(1), "peak は不変");
        let unread = read_usage("confine-usage peak_bytes=1048576 oom_kill=0 write_bytes=unmeasured\n");
        assert_eq!(unread.write_bytes, None);
        assert_eq!(read_usage("confine-usage peak_bytes=1 oom_kill=0 write_bytes=-4096\n").write_bytes, None, "負");
        assert_eq!(read_usage("confine-usage peak_bytes=1 oom_kill=0\n").write_bytes, None, "旧い行");
    }

    /// 包めた周だけ終端行の書きを読み、包めなかった周は素の行の字を測定として読まない。
    #[test]
    fn xpio_written_reads_only_confined_runs() {
        let stdout = "runner: rc=0 records=1\nconfine-usage peak_bytes=1 oom_kill=0 write_bytes=4096\n";
        let confined = Confinement::Confined { unit: "u1".to_owned() };
        assert_eq!(written(&confined, stdout), Some(4096));
        assert_eq!(written(&Confinement::Unconfined(Reason::NoTool), stdout), None);
    }

    /// 語は write: と数か字 unmeasured・和は 1 つでも測れない囲いか囲い 0 の周は測れない。
    #[test]
    fn xpio_detail_and_total_keep_unmeasured() {
        assert_eq!(detail(Some(0)), "write:0");
        assert_eq!(detail(None), "write:unmeasured");
        assert_eq!(word(Some(12)), "12");
        assert_eq!(total([Some(1), Some(2)]), Some(3));
        assert_eq!(total([Some(1), None]), None);
        assert_eq!(total(Vec::<Option<u64>>::new()), None);
    }

    /// 消費の行は detail に語を運び、読み返しても同じ字で、detail の無い口は key を書かない（旧い行と同じ形）。
    #[test]
    fn xpio_cost_event_carries_write_detail() {
        let dir = std::env::temp_dir().join(format!("xpio-ev-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);
        let Ok(policy) = LockPolicy::embedded() else {
            panic!("lock の既定を読めない");
        };
        let usage = Usage::from_words("usage=in:1,out:2,cache_read:3,cache_create:4 turns=5 wall_ms=6");
        let cost = usage.map(|found| Cost { source: CostSource::Runner, usage: found });
        assert_eq!(record_cost_with(&dir, ("r1", "b1"), cost, Some(detail(Some(4096))), policy), None);
        assert_eq!(record_cost(&dir, ("r1", "b1"), cost, policy), None);
        let events = read_all(&dir).unwrap_or_default();
        let _ = std::fs::remove_dir_all(&dir);
        let details: Vec<Option<&str>> = events.iter().map(|event| event.detail.as_deref()).collect();
        assert_eq!(details, [Some("write:4096"), None]);
        assert!(events.iter().all(|event| event.cost == cost), "6 値は不変");
    }
}
