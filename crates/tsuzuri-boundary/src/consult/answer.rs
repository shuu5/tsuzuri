//! tz consult answer <草稿の file>（行 cs-answer・窓の側の所見の口・判断の記録 ADR-10 決定 (3) と ADR-29 決定 (1)(ウ)・(8)・受入 AC16）。
//! 窓の id は環境の `TZ_CONSULT_ID`、作業場は cwd（窓の控えの id が同じこと）。草稿を中核の `finding::check` で検め、
//! 断りは理由の字（欠けた欄の名の列など）を標準出力に 1 行出して rc 1（file を作らない）。添え物は作業場の下に実在する
//! file であること（canonicalize して作業場の下・symlink で外を指す物を断る）。所見は findings/<所見 id>.json に、
//! 一時の名に書いてから既に在る名を上書きしない hard link で置く（番号 k は窓の所見の最大 + 1・在れば次の番号）。
//! 台帳は書かない（窓は台帳を書かない・席が読む口で記帳する）。

use std::fs;
use std::path::Path;

use tsuzuri_contract::consult::{Finding, FindingDraft, FindingId, WindowId};
use tsuzuri_contract::wire;
use tsuzuri_core::consult::finding::check;
use tsuzuri_core::consult::launch::ID_ENV;

use super::{FAIL, Refused, UNKNOWN, findings, flags, minute_now, read_window, refuse};
use crate::out::emit;

/// 番号を進めて撃ち直す回の上限。
const TRIES: u32 = 64;

/// tz consult answer の残りの引数を受けて終了 code を返す。
pub fn run(rest: &[&str]) -> u8 {
    let made = flags(rest, &[], &[], &[]).and_then(|f| {
        let [file] = f.pos.as_slice() else {
            return Err((FAIL, "草稿の file を 1 つ渡す".to_string()));
        };
        let id = std::env::var(ID_ENV)
            .ok()
            .and_then(|v| WindowId::parse(&v).ok())
            .ok_or((
                FAIL,
                format!("環境の {ID_ENV} が窓の id でない（窓の中で撃つ）"),
            ))?;
        let ws = std::env::current_dir().map_err(|e| (UNKNOWN, format!("cwd が読めない: {e}")))?;
        if read_window(&ws).map(|w| w.id) != Some(id) {
            return Err((FAIL, format!("cwd が窓 {id} の作業場でない")));
        }
        let text = fs::read_to_string(ws.join(file))
            .map_err(|e| (FAIL, format!("草稿 {file} が読めない: {e}")))?;
        let draft = match check(&text) {
            Ok(draft) => draft,
            Err(refusal) => {
                emit(&refusal.to_string());
                return Err((FAIL, "草稿を受けない".to_string()));
            }
        };
        attached(&ws, &draft)?;
        place(&ws, id, draft)
    });
    match made {
        Ok(found) => {
            emit(&format!("所見 {found} を置いた"));
            0
        }
        Err(e) => refuse("answer", e),
    }
}

/// 添え物が作業場の下に実在する file か（無い・外を指す・dir を断る）。
pub fn attached(ws: &Path, draft: &FindingDraft) -> Result<(), Refused> {
    let root = ws
        .canonicalize()
        .map_err(|e| (UNKNOWN, format!("作業場が読めない: {e}")))?;
    for a in &draft.attachments {
        let real = ws.join(&a.path).canonicalize();
        if !real.is_ok_and(|p| p.starts_with(&root) && p.is_file()) {
            emit(&format!("添え物が作業場の下の file でない: {}", a.path));
            return Err((FAIL, "草稿を受けない".to_string()));
        }
    }
    Ok(())
}

/// 所見を findings/ に上書きせずに置き、所見の id を返す。
fn place(ws: &Path, id: WindowId, draft: FindingDraft) -> Result<FindingId, Refused> {
    let dir = ws.join("findings");
    let date = minute_now();
    let mut k = findings(ws, id).last().map_or(0, |f| f.k()) + 1;
    let tmp = dir.join(format!(".answer-{}.tmp", std::process::id()));
    for _ in 0..TRIES {
        let found = FindingId::new(id, k).ok_or((FAIL, "所見の番号".to_string()))?;
        let finding = Finding::new(found, &date, draft.clone());
        let text = wire::encode(&finding).map_err(|e| (UNKNOWN, e.to_string()))?;
        fs::write(&tmp, text + "\n")
            .map_err(|e| (UNKNOWN, format!("findings/ に書けない: {e}")))?;
        let linked = fs::hard_link(&tmp, dir.join(format!("{found}.json")));
        let _ = fs::remove_file(&tmp);
        match linked {
            Ok(()) => return Ok(found),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => k += 1,
            Err(e) => return Err((UNKNOWN, format!("所見を置けない: {e}"))),
        }
    }
    Err((UNKNOWN, "所見の番号が尽きた".to_string()))
}
