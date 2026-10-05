//! close と開き直しの理由を器の読み手が読める形にする歯（行 e-reason-id・接頭辞 rsid_）。
//! 器の読み手は空白で割った 1 語目が 裁定 なら 2 語目だけを裁定 id と読む。偽の bd は作業場の out.json を標準出力へ出す script、
//! 偽の bdw は撃たれた回ごとの argv（語ごとに NUL で終える）を記録の置き場に書き、最初の引数が create なら作った id を出す script。
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

use crate::common::{bead, rid, script};
use tsuzuri_boundary::server::batch::{self, BATCH_PREFIX};
use tsuzuri_boundary::server::ledger::{Source, parse_bd};
use tsuzuri_boundary::server::policy;
use tsuzuri_boundary::server::ruling::{
    self, POLICY_MARK, REASON_HEAD, Revoked, Writer, policy_reason, reason,
};
use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::{POLICY_SCOPE_LABEL, QUESTION_LABEL};
use tsuzuri_contract::surface::{
    BatchItem, BatchRequest, ItemOutcome, RevokeRequest, RevokeResponse, RulingRequest,
    latest_ruling, revocable,
};
use tsuzuri_contract::wire;
use tsuzuri_core::delivery::{Pending, undelivered};
use tsuzuri_core::question::open_questions;

/// 起草の時の main 4ad0b18f の契約表の verify の filter の語を畳んだ 247 語（字のまま・空白で区切る）。
const FILTER_WORDS: &str = concat!(
    "aaround_ abss_ abst_ accept_ acchold_ account_ acctcore_ acctdoc_ accthb_ accthome_ acctled_ ",
    "acctlook_ acctpcore_ acctproj_ acctsess_ acctwin_ acctwire_ aface_ afocus_ alean_ aord_ aown_ ",
    "apark_ apop_ areread_ askcard_ athr_ batchpanel_ bhalf_ board_min_ bport_ brand_ btuck_ cadl_ ",
    "cadopt_ cadq_ cdorm_ cexcl_ cfsplit_ cg9_ cgdom_ cishard_ cmark_ cnote_ cnret_ contract_form_ ",
    "cround_ csled_ cspk_ ctick_ cupd_ cupdlist_ denv_ dnedge_ dngrp_ dnrow_ dnskip_ dretry_ dstg_ ",
    "ecache_ eheld_acct_ eheld_design_ eheld_held_ eheld_marks_ eheld_vessel_ elazy_ epolq_ eretry_ ",
    "esig_ evkind_ f123_ f159_ f212_ f2ret_ fdlt_ fdlv_ flight_ fmark_ fprem_ frame_ fserve_ fstop_ ",
    "fundl_ fxpre_ g3g7_ gacct_ gapspage_ gatt_ gbnote_ gchip_ gcoach_ gext_ gfix_ gfresh_ ghb_ gins_ ",
    "gjst_ glabel_ gmret_ gmretw_ gnav_ gpface_ gpill_ gpulse_ graph_ gsum_ gtuck_ gview_ gwv_ ",
    "hacols_ harest_ hasplit_ hbconf_ hbmark_ hbon_ hbpost_ hbproc_ hbroute_ hcard_ hcled_ hcnx_ ",
    "hcproj_ hcsess_ hdchip_ hfig_ hnunk_ hook_ hruling_ hsblock_ hsderive_ hshist_ hspage_ hspk_ ",
    "hsym_ hthr_ http_ hwstore_ iclose_ ilink_ inject_ jcount_ jrun_ kcli_ kdeny_ kg9_ kindlab_ ",
    "klink_ ksum_ lateface_ launch_ lcard_ ledgerblock_ lgrp_ lhome_ lidle_ lkind_ lresume_ lsnap_ ",
    "lspark_ lstg_ lstore_ mapview_ mkeys_ mlink_ mqask_ mqface_ mstore_ mtips_ mtree_ nact_ nbatch_ ",
    "ncard_ nextstep_ nodepage_ nstall_ nsum_ nsumw_ ntc_ ntime_ nxact_ nxorg_ parts_ pci_ pclosed_ ",
    "pfold_ pgsw_ pgz_ pipe_ pkac_ pkview_ plimit_ pmisfit_ pmore_ popapp_ pquest_ pqueue_ project_ ",
    "ptitle_ pubfp_ pubscan_ punmap_ pwhole_ qblock_ qgate_ qkey_ qsig_ question_ rbusy_ relay_ ",
    "rhold_ runsdoc_ rvk_ saxis_ sclosed_ seatblock_ seatcard_ server_ sesplit_ sgrace_ shb_ ",
    "skeleton_ smore_ stage_ stats_ stbp_ stcli_ steady_ sthr_ stnfy_ stskill_ sttgt_ sxaxis_ tgall_ ",
    "tgown_ ticker_ tipx_ tkad_ tlic_ topbar_ topfit_ tz_ udacct_ udash_ unow_ urpanel_ uword_ ",
    "wstrip_",
);

/// 受付の時刻（2026-09-28T04:41:30Z）。
const AT: u64 = 1_790_570_490;

fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// 台帳の 1 本。
struct Bead {
    id: &'static str,
    kind: &'static str,
    status: &'static str,
    labels: Vec<&'static str>,
    parent: Option<&'static str>,
    description: &'static str,
    notes: String,
    close_reason: String,
}

fn bead_of(id: &'static str, kind: &'static str, status: &'static str, labels: &[&'static str]) -> Bead {
    Bead {
        id,
        kind,
        status,
        labels: labels.to_vec(),
        parent: Some("fx-r"),
        description: "",
        notes: String::new(),
        close_reason: String::new(),
    }
}

fn s(text: &str) -> String {
    wire::encode(&text).expect("字の電文")
}

impl Bead {
    fn json(&self) -> String {
        let labels: Vec<String> = self.labels.iter().map(|l| s(l)).collect();
        let mut out = format!(
            "{{\"id\":{},\"title\":{},\"status\":{},\"priority\":2,\"issue_type\":{},\"created_at\":\"2026-09-28T01:00:00Z\",\"updated_at\":\"2026-09-28T01:00:00Z\",\"labels\":[{}]",
            s(self.id),
            s(&format!("題 {}", self.id)),
            s(self.status),
            s(self.kind),
            labels.join(",")
        );
        if let Some(p) = self.parent {
            out.push_str(&format!(",\"parent\":{}", s(p)));
        }
        if !self.description.is_empty() {
            out.push_str(&format!(",\"description\":{}", s(self.description)));
        }
        if !self.notes.is_empty() {
            out.push_str(&format!(",\"notes\":{}", s(&self.notes)));
        }
        if !self.close_reason.is_empty() {
            out.push_str(&format!(",\"close_reason\":{}", s(&self.close_reason)));
        }
        out.push('}');
        out
    }
}

fn ledger(beads: &[Bead]) -> String {
    let lines: Vec<String> = beads.iter().map(Bead::json).collect();
    format!("[\n{}\n]\n", lines.join(",\n"))
}

/// 根の epic fx-r。
fn root() -> Bead {
    let mut root = bead_of("fx-r", "epic", "open", &[]);
    root.parent = None;
    root
}

/// open の問い。
fn open_question(id: &'static str) -> Bead {
    let mut q = bead_of(id, "task", "open", &[QUESTION_LABEL]);
    q.description = "概要 = 画面の色を 2 つに減らしてよいか";
    q
}

/// 閉じた問い（notes の定型行と close の理由つき）。
fn closed_question(id: &'static str, notes: String, close_reason: &str) -> Bead {
    let mut q = bead_of(id, "task", "closed", &[QUESTION_LABEL]);
    q.description = "概要 = 閉じた問い";
    q.notes = notes;
    q.close_reason = close_reason.to_string();
    q
}

/// 歯ごとの作業場（repo・偽の bd と bdw・記録の置き場）。
struct Place {
    root: PathBuf,
    repo: PathBuf,
    log: PathBuf,
}

impl Place {
    fn new(name: &str, ledger_text: &str) -> Place {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("rsid").join(name);
        let _ = fs::remove_dir_all(&root);
        let (repo, log) = (root.join("repo"), root.join("log"));
        fs::create_dir_all(repo.join(".beads")).expect("repo の置き場");
        fs::create_dir_all(&log).expect("記録の置き場");
        fs::write(root.join("out.json"), ledger_text).expect("偽の bd の出力");
        script(
            &root.join("bd"),
            &format!("exec cat '{}'", root.join("out.json").display()),
        );
        let log_dir = log.display();
        script(
            &root.join("bdw"),
            &format!(
                "n=$(( $(cat '{log_dir}/bdw.count' 2>/dev/null || echo 0) + 1 ))\n\
                 echo \"$n\" > '{log_dir}/bdw.count'\n\
                 for a in \"$@\"; do printf '%s\\000' \"$a\"; done > '{log_dir}/bdw.'\"$n\"'.args'\n\
                 if [ \"$1\" = create ]; then printf 'fx-r.9\\n'; fi\n\
                 exit 0"
            ),
        );
        Place { root, repo, log }
    }

    fn source(&self) -> Source {
        Source::new(self.repo.clone(), self.root.join("bd"))
    }

    /// 配達の先は無し。
    fn writer(&self) -> Writer {
        Writer {
            repo: self.repo.clone(),
            bdw: self.root.join("bdw").into(),
            delivery: None,
        }
    }

    /// 偽の bdw が撃たれた回ごとの argv。
    fn argvs(&self) -> Vec<Vec<String>> {
        let count: u32 = fs::read_to_string(self.log.join("bdw.count"))
            .map_or(0, |c| c.trim().parse().expect("回の数"));
        (1..=count)
            .map(|n| {
                fs::read_to_string(self.log.join(format!("bdw.{n}.args")))
                    .expect("argv の記録")
                    .split_terminator('\0')
                    .map(str::to_string)
                    .collect()
            })
            .collect()
    }
}

/// argv の 3 語目 `--reason=<理由>` の理由を、器の読み手と同じく空白で割った語。
fn reason_words(argv: &[String]) -> Vec<String> {
    assert_eq!(argv.len(), 3, "{argv:?}");
    argv[2]
        .strip_prefix("--reason=")
        .unwrap_or_else(|| panic!("--reason= で始まらない: {argv:?}"))
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

/// open の問いの見た版の要約値（server が照らす値）。
fn digest_of(text: &str, id: &str) -> String {
    let Reading::Known(questions) = open_questions(text) else {
        panic!("読める台帳が Unknown");
    };
    questions
        .into_iter()
        .find(|q| q.card.id.as_str() == id)
        .unwrap_or_else(|| panic!("{id} は open の問いでない"))
        .card
        .digest
}

#[test]
fn rsid_reason_forms() {
    assert_eq!(REASON_HEAD, "裁定");
    let (one, two) = ("fx-r.1:20260928T0441Z-1", "fx-r.1:20260928T0441Z-2");
    let batch_id = "batch:20260928T0441Z-1";

    let plain = reason(&rid(one), None);
    assert_eq!(plain, format!("裁定 {one}"));
    let batched = reason(&rid(one), Some(&format!("束 {batch_id}")));
    assert_eq!(batched, format!("裁定 {one} 束 {batch_id}"));
    let revoking = reason(&rid(two), Some(&format!("取り消す = {one}")));
    assert_eq!(revoking, format!("裁定 {two} 取り消す = {one}"));
    for (text, id) in [(&plain, one), (&batched, one), (&revoking, two)] {
        let words: Vec<&str> = text.split_whitespace().collect();
        assert_eq!(words[0], "裁定", "{text}");
        assert_eq!(words[1], id, "{text}");
    }

    // 前の束と取り消しの形は、空白で割ると 2 語目が id と字 ・ と次の語の頭をつないだ字になる。
    for (old, id) in [
        (format!("裁定 {one}・束 {batch_id}"), one),
        (format!("裁定 {two}・取り消す = {one}"), two),
    ] {
        let second = old.split_whitespace().nth(1).expect("2 語目");
        assert!(second.contains('・'), "{old}");
        assert_ne!(second, id, "{old}");
        assert_ne!(second, one, "{old}");
    }
}

#[test]
fn rsid_writes_split_id() {
    let mut beads = vec![root()];
    beads.extend(["fx-r.1", "fx-r.2", "fx-r.3"].map(open_question));
    let text = ledger(&beads);

    // 1 問の裁定の受付。
    let place = Place::new("ruling", &text);
    let req = RulingRequest {
        question: bead("fx-r.1"),
        seen_digest: digest_of(&text, "fx-r.1"),
        verbatim: "はい".into(),
    };
    let got = ruling::accept(&req, &place.source(), &place.writer(), AT);
    let ruling::Outcome::Recorded(res) = got else {
        panic!("Recorded でない: {got:?}");
    };
    assert_eq!(res.ruling.as_str(), "fx-r.1:20260928T0441Z-1");
    let argvs = place.argvs();
    assert_eq!(argvs.len(), 2, "{argvs:?}");
    assert_eq!(
        argvs[1],
        ["close", "fx-r.1", "--reason=裁定 fx-r.1:20260928T0441Z-1"]
    );

    // 束の受付。
    let place = Place::new("batch", &text);
    let item = |q: &str| BatchItem {
        question: bead(q),
        seen_digest: digest_of(&text, q),
        verbatim: None,
    };
    let req = BatchRequest {
        items: vec![item("fx-r.2"), item("fx-r.3")],
        verbatim: "まとめてはい".into(),
    };
    let got = batch::accept(&req, &place.source(), &place.writer(), AT);
    let batch::Outcome::Recorded(res) = got else {
        panic!("Recorded でない: {got:?}");
    };
    let batch_id = "batch:20260928T0441Z-1";
    assert_eq!(res.batch.as_str(), batch_id);
    let argvs = place.argvs();
    assert_eq!(argvs.len(), 4, "{argvs:?}");
    for (n, question) in ["fx-r.2", "fx-r.3"].into_iter().enumerate() {
        let want = format!("{question}:20260928T0441Z-1");
        let row = &res.items[n];
        assert_eq!(row.question.as_str(), question);
        assert_eq!(row.outcome, ItemOutcome::Written { ruling: rid(&want) });
        assert_eq!(argvs[2 * n + 1][..2], ["close", question]);
        assert_eq!(
            argvs[2 * n + 1][2],
            format!("--reason=裁定 {want} 束 {batch_id}")
        );
        let words = reason_words(&argvs[2 * n + 1]);
        assert_eq!(words, ["裁定", &want, "束", batch_id]);
    }
}

#[test]
fn rsid_old_form_revokes() {
    let old_batch = "batch:20260928T0100Z-1";
    let old_id = |q: &str| format!("{q}:20260928T0100Z-1");
    let notes = |q: &str| {
        format!(
            "裁定 id = {}・問い = {q}・{BATCH_PREFIX}{old_batch}・逐語 = はい",
            old_id(q)
        )
    };
    let beads = vec![
        root(),
        closed_question(
            "fx-r.4",
            notes("fx-r.4"),
            &format!("裁定 {}・束 {old_batch}", old_id("fx-r.4")),
        ),
        closed_question(
            "fx-r.5",
            notes("fx-r.5"),
            &format!("裁定 {} 束 {old_batch}", old_id("fx-r.5")),
        ),
    ];
    let text = ledger(&beads);
    for question in ["fx-r.4", "fx-r.5"] {
        let place = Place::new(&format!("revoke-{question}"), &text);
        let req = RevokeRequest {
            question: bead(question),
            ruling: rid(&old_id(question)),
            verbatim: "待つ".into(),
        };
        let new_id = format!("{question}:20260928T0441Z-1");
        assert_eq!(
            ruling::revoke(&req, &place.source(), &place.writer(), AT),
            Revoked::Recorded(RevokeResponse {
                ruling: rid(&new_id),
                recorded_at: AT,
                reopened_only: false,
            }),
            "{question}"
        );
        let argvs = place.argvs();
        assert_eq!(argvs.len(), 2, "{argvs:?}");
        assert_eq!(
            argvs[1],
            [
                "reopen".to_string(),
                question.to_string(),
                format!("--reason=裁定 {new_id} 取り消す = {}", old_id(question)),
            ]
        );
        let words = reason_words(&argvs[1]);
        assert_eq!(words[..2], ["裁定".to_string(), new_id]);
    }
}

#[test]
fn rsid_policy_close() {
    assert_eq!(POLICY_MARK, "policy:");
    let id = "fx-r.9:20260928T0441Z-1";
    assert_eq!(policy_reason(&rid(id)), format!("裁定 policy:{id}"));

    let place = Place::new("policy", &ledger(&[root()]));
    let req = tsuzuri_contract::surface::PolicyRequest {
        scope: "all".into(),
        verbatim: "全体に急がない".into(),
    };
    let got = policy::accept(&req, &place.source(), &place.writer(), AT);
    let policy::Outcome::Recorded(res) = got else {
        panic!("Recorded でない: {got:?}");
    };
    assert_eq!(res.policy.as_str(), id);
    let argvs = place.argvs();
    assert_eq!(argvs.len(), 3, "{argvs:?}");
    assert_eq!(argvs[0][0], "create");
    assert_eq!(argvs[2], ["close", "fx-r.9", &format!("--reason=裁定 policy:{id}")]);
    let words = reason_words(&argvs[2]);
    assert_eq!(words, ["裁定".to_string(), format!("{POLICY_MARK}{id}")]);
}

#[test]
fn rsid_policy_not_ruling() {
    let policy_id = "fx-r.9:20260928T0441Z-1";
    let ruling_id = "fx-r.1:20260928T0441Z-1";
    let mut policy_bead = bead_of("fx-r.9", "task", "closed", &[QUESTION_LABEL, "policy-scope:all"]);
    policy_bead.notes = policy::line(&rid(policy_id), "all", "全体に急がない");
    policy_bead.close_reason = policy_reason(&rid(policy_id));
    let ruling_bead = closed_question(
        "fx-r.1",
        ruling::line(&rid(ruling_id), &bead("fx-r.1"), "はい"),
        &reason(&rid(ruling_id), None),
    );
    let text = ledger(&[root(), policy_bead, ruling_bead]);
    assert_eq!(POLICY_SCOPE_LABEL, "policy-scope:");

    let Reading::Known(items) = parse_bd(&text) else {
        panic!("読める台帳が Unknown");
    };
    let find = |id: &str| {
        items
            .iter()
            .find(|i| i.row.id.as_str() == id)
            .unwrap_or_else(|| panic!("{id} が台帳に無い"))
    };
    let (policy_item, ruling_item) = (find("fx-r.9"), find("fx-r.1"));
    assert!(policy_item.row.is_policy());
    assert!(!ruling_item.row.is_policy());
    assert_eq!(latest_ruling(&policy_item.notes), None);
    assert_eq!(latest_ruling(&ruling_item.notes), Some(ruling_id));
    assert!(!revocable(policy_item, &rid(policy_id)));
    assert!(revocable(ruling_item, &rid(ruling_id)));
    assert_eq!(
        undelivered(&text),
        Reading::Known(vec![Pending {
            question: bead("fx-r.1"),
            ruling: rid(ruling_id),
        }])
    );

    let second = |text: String| text.split_whitespace().nth(1).expect("2 語目").to_string();
    let policy_second = second(policy_reason(&rid(policy_id)));
    let ruling_second = second(reason(&rid(ruling_id), None));
    assert!(policy_second.starts_with(POLICY_MARK), "{policy_second}");
    assert!(!ruling_second.starts_with(POLICY_MARK), "{ruling_second}");
    assert_eq!(ruling_second, ruling_id);
}

/// 字 `needle` が `text` に現れる数。
fn count(text: &str, needle: &str) -> usize {
    text.matches(needle).count()
}

#[test]
fn rsid_src_text() {
    let read = |name: &str| {
        fs::read_to_string(manifest().join("src/server").join(name))
            .unwrap_or_else(|e| panic!("{name} を読む: {e}"))
    };
    let (ruling_src, batch_src, policy_src) =
        (read("ruling.rs"), read("batch.rs"), read("policy.rs"));
    assert_eq!(count(&ruling_src, "reason(&id,"), 2);
    assert_eq!(count(&batch_src, "reason(id,"), 1);
    assert_eq!(count(&policy_src, "policy_reason(&id)"), 1);
    for (name, src) in [("ruling.rs", &ruling_src), ("batch.rs", &batch_src)] {
        assert_eq!(count(src, "{id}{ID_END}束"), 0, "{name}");
        assert_eq!(count(src, "{id}{ID_END}{REVOKES}"), 0, "{name}");
    }
    assert_eq!(count(&policy_src, "{TITLE} {id}"), 0);
}

#[test]
fn rsid_own_names_clean() {
    let words: Vec<&str> = FILTER_WORDS.split_whitespace().collect();
    assert_eq!(words.len(), 247, "filter の語の数");
    let text = fs::read_to_string(manifest().join("tests/teeth3/rsid.rs")).expect("tests/teeth3/rsid.rs を読む");
    let lines: Vec<&str> = text.lines().collect();
    let names: Vec<&str> = lines
        .windows(2)
        .filter(|w| w[0].trim() == "#[test]")
        .map(|w| {
            let rest = w[1].trim().strip_prefix("fn ").expect("test の属性の次は fn");
            rest.split('(').next().unwrap_or(rest)
        })
        .collect();
    assert_eq!(names.len(), 7, "歯の数");
    for name in names {
        assert!(name.starts_with("rsid_"), "{name} は rsid_ で始まらない");
        for word in &words {
            assert!(!name.contains(word), "{name} が {word} を含む");
        }
    }
    for word in &words {
        assert!(!word.contains("rsid_"), "{word} が rsid_ を含む");
    }
}
