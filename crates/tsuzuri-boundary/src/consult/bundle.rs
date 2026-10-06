//! tz consult bundle <窓 id> [--topic <題>] [--question <file>]（判断の記録 ADR-29 決定 (1)(イ)・束を機械が組む）。
//! 窓の束 bundle/ に手引き brief.md・問い question.md・読む物の一覧 reads.md・台帳の写し ledger.json・要約値 digest を書き、
//! 標準出力に「束 <要約値 16 字>」を 1 行出す。席が撃つ時（環境に窓の id が無い時）は台帳を bd で読んで写しを替え、
//! 窓の中で撃つ時（環境の `TZ_CONSULT_ID` が窓の id で cwd が作業場）は台帳の写しを替えない（囲いの中では bd を撃てない）。
//! 問いは --question の file の字、無くて題が在り question.md が無ければ題（台帳の bead なら題名と本文も）から書く。

use std::io::Read;
use std::path::Path;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::consult::{WindowFile, WindowId};
use tsuzuri_contract::ledger::{LedgerItem, fnv1a64};
use tsuzuri_core::consult::launch::{ID_ENV, brief, read_roots};

use super::plain::{open_plain, plain_file, read_text, write_plain};
use super::{
    COMMON, FAIL, READ_MAX, Refused, UNKNOWN, ctx, flags, ledger, plain_ws, read_window, refuse,
    tz_path, workspace,
};
use crate::out::emit;
use crate::server::ledger::parse_bd;

/// 束の file の名（要約値に入れる順）。
pub const FILES: [&str; 4] = ["brief.md", "question.md", "reads.md", "ledger.json"];

/// 要約値の file の名。
pub const DIGEST: &str = "digest";

/// tz consult bundle の残りの引数を受けて終了 code を返す。
pub fn run(rest: &[&str]) -> u8 {
    let mut values = COMMON.to_vec();
    values.extend(["--topic", "--question"]);
    let f = match flags(rest, &values, &[], &[]) {
        Ok(f) => f,
        Err(e) => return refuse("bundle", e),
    };
    let [id] = f.pos.as_slice() else {
        return refuse("bundle", (FAIL, "窓の id を 1 つ渡す".to_string()));
    };
    let Ok(id) = WindowId::parse(id) else {
        return refuse("bundle", (FAIL, format!("窓の id {id} の形でない")));
    };
    let inside = std::env::var(ID_ENV).ok().as_deref() == Some(id.to_string().as_str());
    let made = if inside {
        std::env::current_dir()
            .map_err(|e| (UNKNOWN, format!("cwd が読めない: {e}")))
            .and_then(|ws| rebuild(&ws, id, f.get("--topic"), f.get("--question")))
    } else {
        ctx(&f).and_then(|c| {
            let (text, _) = ledger(&c)?;
            let ws = workspace(&c.drafts, id);
            let topic = f.get("--topic");
            write(
                &ws,
                id,
                &Inputs::seat(&c, &text, topic, f.get("--question")),
            )
        })
    };
    match made {
        Ok(digest) => {
            emit(&format!("束 {digest}"));
            0
        }
        Err(e) => refuse("bundle", e),
    }
}

/// 束の材料。
#[derive(Debug, Clone, Default)]
pub struct Inputs {
    /// 台帳の写しに置く bd の字（窓の中では None＝写しを替えない）。
    pub ledger: Option<String>,
    /// 題（None なら窓の控えの題）。
    pub topic: Option<String>,
    /// 問いの file の path。
    pub question: Option<String>,
    /// 読む物の一覧の字（窓の中では None＝替えない）。
    pub reads: Option<String>,
}

impl Inputs {
    /// 席が撃つ時の材料（読む根と台帳の字）。
    pub fn seat(c: &super::Ctx, text: &str, topic: Option<&str>, question: Option<&str>) -> Inputs {
        let roots = read_roots(
            &c.repo.display().to_string(),
            &c.state.display().to_string(),
        );
        let mut reads = String::from("# 読む物（読むだけ・作業場の外へは書けない）\n\n");
        for root in &roots {
            reads.push_str(&format!("- {root}\n"));
        }
        reads.push_str("- bundle/ledger.json（台帳の写し・席が束を組んだ時の bd の字）\n");
        Inputs {
            ledger: Some(text.to_string()),
            topic: topic.map(str::to_string),
            question: question.map(str::to_string),
            reads: Some(reads),
        }
    }
}

/// 窓の中の組み直し（台帳の写しと読む物の一覧を替えない）。
fn rebuild(
    ws: &Path,
    id: WindowId,
    topic: Option<&str>,
    question: Option<&str>,
) -> Result<String, Refused> {
    let inputs = Inputs {
        topic: topic.map(str::to_string),
        question: question.map(str::to_string),
        ..Inputs::default()
    };
    write(ws, id, &inputs)
}

/// 題の bead の題名と本文（台帳の写しに在れば）。
fn bead_text(items: &[LedgerItem], topic: &str) -> Option<String> {
    let item = items.iter().find(|i| i.row.id.as_str() == topic)?;
    Some(format!(
        "{}\n\n{}\n",
        item.row.title,
        item.description.trim_end()
    ))
}

/// 題から組む問いの字（台帳の写しは `plain` の照らしで読む）。
fn topic_question(ws: &Path, topic: &str) -> String {
    let items = read_text(ws, "bundle/ledger.json", READ_MAX)
        .ok()
        .flatten()
        .and_then(|t| match parse_bd(&t) {
            Reading::Known(items) => Some(items),
            Reading::Unknown => None,
        })
        .unwrap_or_default();
    let mut text = format!("# 題 {topic}\n\n");
    if let Some(body) = bead_text(&items, topic) {
        text.push_str(&body);
    }
    text
}

/// 束の file を書き、要約値を返す（作業場と窓の控えが無いか書けなければ rc 1）。
pub fn write(ws: &Path, id: WindowId, inputs: &Inputs) -> Result<String, Refused> {
    let window: WindowFile =
        read_window(ws).ok_or((FAIL, format!("窓 {id} の作業場が無い: {}", ws.display())))?;
    if window.id != id {
        return Err((
            FAIL,
            format!("作業場の窓の id が {} で {id} でない", window.id),
        ));
    }
    plain_ws(ws, id)?;
    let dir = ws.join("bundle");
    let put = |name: &str, text: &str| {
        write_plain(ws, &format!("bundle/{name}"), text.as_bytes())
            .map_err(|e| (FAIL, format!("{name} を書けない: {e}")))
    };
    put("brief.md", &brief(&tz_path(), id))?;
    if let Some(text) = &inputs.ledger {
        put("ledger.json", text)?;
    }
    if let Some(text) = &inputs.reads {
        put("reads.md", text)?;
    }
    let topic = inputs.topic.as_deref().or(window.topic.as_deref());
    match (&inputs.question, topic) {
        (Some(file), _) => {
            let text = std::fs::read_to_string(file)
                .map_err(|e| (FAIL, format!("問いの file {file} が読めない: {e}")))?;
            put("question.md", &text)?;
        }
        (None, Some(t)) if inputs.topic.is_some() || !plain_file(ws, "bundle/question.md") => {
            put("question.md", &topic_question(ws, t))?;
        }
        _ => {}
    }
    let digest = digest(&dir);
    put(DIGEST, &format!("{digest}\n"))?;
    Ok(digest)
}

/// 束の要約値（`FILES` の在る普通の file の名と字を順に FNV-1a 64 に通した 16 進の 16 字・`plain` の照らしで読む）。
pub fn digest(dir: &Path) -> String {
    let mut bytes = Vec::new();
    for name in FILES {
        let read = open_plain(dir, name).and_then(|(mut file, _)| {
            let mut text = Vec::new();
            file.read_to_end(&mut text).map(|_| text)
        });
        if let Ok(text) = read {
            bytes.extend_from_slice(name.as_bytes());
            bytes.push(0);
            bytes.extend_from_slice(&text);
            bytes.push(0);
        }
    }
    format!("{:016x}", fnv1a64(&bytes))
}
