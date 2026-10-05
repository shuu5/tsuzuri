//! tz consult show <所見 id か頼み id> --via 見張り|完了|hook|一覧（席が読む口・判断の記録 ADR-29 決定 (8)・受入 AC16 の台帳の 2 行）。
//! 所見は作業場（退いた作業場も）の findings/<所見 id>.json を読み、全部の欄を人の読む形で標準出力に出し、添え物は
//! 作業場の絶対 path で出す。台帳に相談の所見の行と相談の受けの行（経路の語）が無ければ 1 つずつ書く（同じ id の行が
//! 在れば書かない＝何度撃っても 2 行だけ）。置き場は所見の題から中核の `home_of` で選ぶ memo か根（題が器の引用の形を
//! 含むか なし なら題なし）。頼みは台帳の相談の頼みの行を出し、開く命令の字を添える（受けの行は open --request が書く）。

use std::path::{Path, PathBuf};

use tsuzuri_contract::consult::{Finding, FindingId, RequestId, Via, Word};
use tsuzuri_contract::wire;
use tsuzuri_core::consult::lines::{Line, Subject, WORD_MAX, cited, free};

use super::{
    COMMON, Ctx, FAIL, Refused, append, ctx, flags, ledger, lines_of, minute_now, refuse, retired,
    tzw, workspace,
};
use crate::out::emit;

/// tz consult show の残りの引数を受けて終了 code を返す。
pub fn run(rest: &[&str]) -> u8 {
    let mut values = COMMON.to_vec();
    values.push("--via");
    let made = flags(rest, &values, &[], &[]).and_then(|f| {
        let [id] = f.pos.as_slice() else {
            return Err((FAIL, "所見か頼みの id を 1 つ渡す".to_string()));
        };
        let via = f.get("--via").and_then(Via::from_word).ok_or((
            FAIL,
            "--via は 見張り・完了・hook・一覧 のどれか".to_string(),
        ))?;
        let c = ctx(&f)?;
        match (FindingId::parse(id), RequestId::parse(id)) {
            (Ok(found), _) => show_finding(&c, found, via),
            (_, Ok(rq)) => show_request(&c, &rq),
            _ => Err((FAIL, format!("{id} は所見の id でも頼みの id でもない"))),
        }
    });
    match made {
        Ok(()) => 0,
        Err(e) => refuse("show", e),
    }
}

/// 所見の file の path（作業場か退いた作業場の在る方）。
pub fn finding_path(drafts: &Path, id: FindingId) -> Option<PathBuf> {
    [workspace(drafts, id.window()), retired(drafts, id.window())]
        .into_iter()
        .map(|ws| ws.join("findings").join(format!("{id}.json")))
        .find(|p| p.is_file())
}

/// 所見を読む（無いか読めなければ rc 1）。
pub fn read_finding(drafts: &Path, id: FindingId) -> Result<(PathBuf, Finding), Refused> {
    let path = finding_path(drafts, id).ok_or((FAIL, format!("所見 {id} の file が無い")))?;
    let text =
        std::fs::read_to_string(&path).map_err(|e| (FAIL, format!("所見 {id} が読めない: {e}")))?;
    let finding = wire::decode(&text).map_err(|e| (FAIL, format!("所見 {id} の形が違う: {e}")))?;
    Ok((path, finding))
}

/// 所見の題（行に置く字・なし と空と器の引用の形を含む字は None）。
pub fn topic_of(f: &Finding) -> Option<String> {
    let t = f.topic.trim();
    (!t.is_empty() && t != "なし" && !cited(&free(t, WORD_MAX))).then(|| t.to_string())
}

/// 所見を人の読む形の行の列にする（添え物は作業場 `ws` の下の絶対 path）。
pub fn render(f: &Finding, ws: &Path) -> Vec<String> {
    let or_none = |v: &[String]| {
        if v.is_empty() {
            "なし".to_string()
        } else {
            v.join(", ")
        }
    };
    let mut out = vec![
        format!(
            "所見 {}（窓 {}・題 {}・時刻 {}・model {}）",
            f.id, f.window, f.topic, f.date, f.model
        ),
        format!("結論: {}", f.conclusion),
        format!("推奨: {}", f.recommendation),
        "候補:".to_string(),
    ];
    for o in &f.options {
        let mark = if o.verdict == tsuzuri_contract::consult::Adoption::Adopted {
            "採る"
        } else {
            "採らない"
        };
        out.push(format!(
            "- {} {}（{mark}）: {}・理由: {}",
            o.id, o.name, o.text, o.reason
        ));
    }
    out.push("主張:".to_string());
    for c in &f.claims {
        let word = wire::encode(&c.confidence)
            .unwrap_or_default()
            .replace('"', "");
        out.push(format!("- [{word}] {}（{}）", c.text, c.how));
    }
    out.push(format!("根拠: {}", or_none(&f.basis)));
    out.push(format!("持ち主に問う: {}", or_none(&f.ask_owner)));
    out.push(format!("触る節点: {}", or_none(&f.touches)));
    out.push(format!("束の要約値: {}", f.bundle_digest));
    out.push(format!("囲いの断り: {}", or_none(&f.sandbox_denials)));
    out.push("添え物:".to_string());
    for a in &f.attachments {
        let kind = wire::encode(&a.kind).unwrap_or_default().replace('"', "");
        out.push(format!(
            "- {kind} {}（{}）",
            ws.join(&a.path).display(),
            a.note
        ));
    }
    out
}

/// 所見を出し、所見の行と受けの行の無い方を書く。
fn show_finding(c: &Ctx, id: FindingId, via: Via) -> Result<(), Refused> {
    let (path, f) = read_finding(&c.drafts, id)?;
    let ws = path.parent().and_then(Path::parent).unwrap_or(&c.drafts);
    let (_, items) = ledger(c)?;
    let lines = lines_of(&items);
    let topic = topic_of(&f);
    if !lines
        .iter()
        .any(|l| matches!(l, Line::Finding { id: x, .. } if *x == id))
    {
        let line = Line::Finding {
            id,
            topic: topic.clone(),
            at: f.date.clone(),
        };
        append(c, &items, topic.as_deref(), &line)?;
    }
    let subject = Subject::Finding(id);
    if !lines
        .iter()
        .any(|l| matches!(l, Line::Receipt { subject: s, .. } if *s == subject))
    {
        let line = Line::Receipt {
            subject,
            via,
            at: minute_now(),
        };
        append(c, &items, topic.as_deref(), &line)?;
    }
    render(&f, ws).iter().for_each(|l| emit(l));
    Ok(())
}

/// 頼みの行を出す（台帳に無ければ rc 1）。
fn show_request(c: &Ctx, rq: &RequestId) -> Result<(), Refused> {
    let (_, items) = ledger(c)?;
    let found = lines_of(&items).into_iter().find_map(|l| match l {
        Line::Request {
            id,
            topic,
            form,
            model,
            at,
        } if id == *rq => Some((topic, form, model, at)),
        _ => None,
    });
    let (topic, form, model, at) = found.ok_or((FAIL, format!("頼み {rq} が台帳に無い")))?;
    let topic = topic.unwrap_or_else(|| "題なし".to_string());
    emit(&format!(
        "頼み {rq}（題 {topic}・形 {}・model {model}・時刻 {at}）",
        form.word()
    ));
    emit(&format!(
        "開く: {} consult open --request {rq} --by button",
        tzw()
    ));
    Ok(())
}
