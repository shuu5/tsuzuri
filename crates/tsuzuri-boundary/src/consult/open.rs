//! tz consult open（判断の記録 ADR-29 決定 (3)(4)(5)(11)）: 窓の作業場を用意する。
//! 引数: [--topic <題>] [--form talk|ask] [--model <名>] [--effort <段>] --by seat|chat|button [--said <分>]
//! [--request <頼みの id>] [--via 見張り|hook|一覧] [--question <file>] と共通の --repo・--bd・--bdw。
//! 環境の `TZ_PLUGIN_VERSION` が中核の `PLUGIN_VERSION` と同じでなければ断る（plugin の解き方を通さない撃ちと版のずれ）。
//! 起こし手が持ち主のチャットなら --said に器が記録した発話の分の字、持ち主の button なら --request に頼みの id が要る。
//! 頼みの時は台帳の相談の頼みの行の形と model を使い（題は --topic が無ければ頼みの題）、相談の受けの行を経路
//! （既定 見張り）で頼みの題の置き場に書く（台帳に無い頼みと受けた頼みは断る・受けの行は作業場より先に書く）。
//! 作業場は起草の置き場の下に `consult-cw<n>` を原子に作り（n は在る窓と退いた窓の最大 + 1）、git init し、
//! `DIRS` の dir・窓の控え・控えの書き場 notes.md・束を置く。標準出力は「窓 cw<n> を用意した」と作業場の絶対 path の 2 行。
//! 開きの行は起動の口が書く。

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use tsuzuri_contract::consult::{
    Form, RequestId, Starter, Via, WindowFile, WindowId, Word, minute_ok,
};
use tsuzuri_contract::ledger::LedgerItem;
use tsuzuri_core::consult::launch::{VERSION_ENV, version_ok};
use tsuzuri_core::consult::lines::{Line, Subject};

use super::bundle::{self, Inputs};
use super::{
    COMMON, Ctx, DIRS, FAIL, GIT_TIMEOUT, NOTES, Refused, UNKNOWN, append, ctx, flags, ledger,
    lines_of, minute_now, refuse, windows, write_window,
};
use crate::acct::GIT;
use crate::out::emit;
use crate::server::ledger::capture;

/// 既定の model（制約 CON6）。
pub const MODEL: &str = "fable";

/// 既定の念入りさ（制約 CON6）。
pub const EFFORT: &str = "xhigh";

/// 作業場の dir を作り損ねた時に次の番号で撃ち直す回の上限。
const TRIES: u32 = 64;

/// 窓の控えの材料（引数か頼みの行から）。
struct Want {
    form: Form,
    topic: Option<String>,
    model: String,
    effort: String,
    starter: Starter,
    uttered: Option<String>,
    request: Option<RequestId>,
    via: Via,
}

/// tz consult open の残りの引数を受けて終了 code を返す。
pub fn run(rest: &[&str]) -> u8 {
    let mut values = COMMON.to_vec();
    values.extend([
        "--topic",
        "--form",
        "--model",
        "--effort",
        "--by",
        "--said",
        "--request",
        "--via",
        "--question",
    ]);
    let made = flags(rest, &values, &[], &[]).and_then(|f| {
        if !f.pos.is_empty() {
            return Err((FAIL, format!("知らない引数 {}", f.pos.join(" "))));
        }
        if !version_ok(std::env::var(VERSION_ENV).ok().as_deref()) {
            return Err((
                FAIL,
                format!("{VERSION_ENV} が tz の版と違う（plugin の tz の解き方で撃つ）"),
            ));
        }
        let want = want(&f)?;
        let c = ctx(&f)?;
        let (text, items) = ledger(&c)?;
        let want = receive(&c, &items, want)?;
        let question = f.get("--question");
        make(&c, &want, &Inputs::seat(&c, &text, None, question))
    });
    match made {
        Ok((id, ws)) => {
            emit(&format!("窓 {id} を用意した"));
            emit(&ws.display().to_string());
            0
        }
        Err(e) => refuse("open", e),
    }
}

/// 引数から控えの材料を読む（起こし手と添えの組み・形・経路の語を断る）。
fn want(f: &super::Flags) -> Result<Want, Refused> {
    let form = match f.get("--form").unwrap_or("talk") {
        "talk" => Form::Talk,
        "ask" => Form::Ask,
        other => return Err((FAIL, format!("--form は talk か ask（{other}）"))),
    };
    let starter = match f.get("--by") {
        Some("seat") => Starter::Seat,
        Some("chat") => Starter::Chat,
        Some("button") => Starter::Button,
        _ => return Err((FAIL, "--by は seat・chat・button のどれか".to_string())),
    };
    let uttered = f.get("--said").map(str::to_string);
    if (starter == Starter::Chat) != uttered.is_some()
        || uttered.as_deref().is_some_and(|s| !minute_ok(s))
    {
        return Err((
            FAIL,
            "--said は --by chat の時だけ・発話の UTC の分の字".to_string(),
        ));
    }
    let request = match f.get("--request").map(RequestId::parse) {
        None => None,
        Some(Ok(r)) => Some(r),
        Some(Err(_)) => return Err((FAIL, "--request が頼みの id の形でない".to_string())),
    };
    if (starter == Starter::Button) != request.is_some() {
        return Err((
            FAIL,
            "--request は --by button の時だけ（button の時は要る）".to_string(),
        ));
    }
    let via = Via::from_word(f.get("--via").unwrap_or(Via::Watch.word()))
        .filter(|v| *v != Via::Done)
        .ok_or((FAIL, "--via は 見張り・hook・一覧 のどれか".to_string()))?;
    Ok(Want {
        form,
        topic: f.get("--topic").map(str::to_string),
        model: f.get("--model").unwrap_or(MODEL).to_string(),
        effort: f.get("--effort").unwrap_or(EFFORT).to_string(),
        starter,
        uttered,
        request,
        via,
    })
}

/// 頼みなら台帳の頼みの行を既定にし、相談の受けの行を書く（台帳に無い頼みと受けた頼みは断る）。
fn receive(c: &Ctx, items: &[LedgerItem], mut want: Want) -> Result<Want, Refused> {
    let Some(rq) = want.request.clone() else {
        return Ok(want);
    };
    let lines = lines_of(items);
    let subject = Subject::Request(rq.clone());
    if lines
        .iter()
        .any(|l| matches!(l, Line::Receipt { subject: s, .. } if *s == subject))
    {
        return Err((FAIL, format!("頼み {rq} はもう受けた")));
    }
    let Some(Line::Request {
        topic, form, model, ..
    }) = lines
        .iter()
        .find(|l| matches!(l, Line::Request { id, .. } if *id == rq))
    else {
        return Err((FAIL, format!("頼み {rq} が台帳に無い")));
    };
    want.topic = want.topic.or_else(|| topic.clone());
    want.form = *form;
    want.model = model.clone();
    let line = Line::Receipt {
        subject,
        via: want.via,
        at: minute_now(),
    };
    append(c, items, topic.as_deref(), &line)?;
    Ok(want)
}

/// 次の窓の番号（在る窓と退いた窓の最大 + 1）。
fn next(drafts: &Path) -> u32 {
    windows(drafts)
        .iter()
        .map(|(id, _)| id.n())
        .max()
        .unwrap_or(0)
        + 1
}

/// 作業場の dir を原子に作る（在れば次の番号で撃ち直す）。
fn create(drafts: &Path) -> Result<(WindowId, PathBuf), Refused> {
    let mut n = next(drafts);
    for _ in 0..TRIES {
        let id = WindowId::new(n).ok_or((FAIL, "窓の番号".to_string()))?;
        let ws = super::workspace(drafts, id);
        match std::fs::create_dir(&ws) {
            Ok(()) => return Ok((id, ws)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => n += 1,
            Err(e) => return Err((UNKNOWN, format!("作業場を作れない: {e}"))),
        }
    }
    Err((UNKNOWN, "作業場の番号が尽きた".to_string()))
}

/// 作業場を作り、git init と dir と控えと束を置く。
fn make(c: &Ctx, want: &Want, inputs: &Inputs) -> Result<(WindowId, PathBuf), Refused> {
    let (id, ws) = create(&c.drafts)?;
    capture(OsStr::new(GIT), ["init", "-q"], &ws, GIT_TIMEOUT)
        .ok_or((UNKNOWN, "作業場の git init が落ちた".to_string()))?;
    for dir in DIRS {
        std::fs::create_dir_all(ws.join(dir))
            .map_err(|e| (UNKNOWN, format!("{dir} を作れない: {e}")))?;
    }
    let window = WindowFile {
        id,
        form: want.form,
        topic: want.topic.clone(),
        model: want.model.clone(),
        effort: want.effort.clone(),
        starter: want.starter,
        uttered: want.uttered.clone(),
        request: want.request.clone(),
        made: minute_now(),
    };
    write_window(&ws, &window).map_err(|e| (UNKNOWN, format!("窓の控えを書けない: {e}")))?;
    std::fs::write(ws.join(NOTES), format!("# 窓 {id} の控え\n"))
        .map_err(|e| (UNKNOWN, format!("{NOTES} を書けない: {e}")))?;
    bundle::write(&ws, id, inputs)?;
    let ws = ws.canonicalize().unwrap_or(ws);
    Ok((id, ws))
}
