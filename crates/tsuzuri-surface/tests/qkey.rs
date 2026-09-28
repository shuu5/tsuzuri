//! 行 g-keyed の歯: 問いの card とまとめて承認の行の鍵（番号を除いた中身）・block の形と列の読み・
//! 行の番号の読み直し・2 つの block の DOM の鍵つきの一覧の字の並び。

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::ledger::BeadId;
use tsuzuri_contract::question::QuestionList;
use tsuzuri_contract::wire;
use tsuzuri_surface::project::{Body, NOT_READ, ask, batch};
use tsuzuri_surface::view::Fetched;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{rel} を読む: {e}"))
}

fn fixture_text() -> String {
    read("../../tests/fixtures/surface/question-list.json")
}

fn fixture() -> Fetched {
    Fetched::Body(fixture_text())
}

fn bead(id: &str) -> BeadId {
    BeadId::new(id).expect("bead の id")
}

/// fixture の一覧を替えて本文に戻す。
fn altered(edit: impl FnOnce(&mut QuestionList)) -> Fetched {
    let mut list: QuestionList = wire::decode(&fixture_text()).expect("fixture の電文");
    edit(&mut list);
    Fetched::Body(wire::encode(&list).expect("電文"))
}

fn known(list: &mut QuestionList) -> &mut Vec<tsuzuri_contract::question::QuestionCard> {
    match &mut list.cards {
        Reading::Known(cards) => cards,
        Reading::Unknown => panic!("fixture の cards は Known"),
    }
}

fn without_qa2() -> Fetched {
    altered(|l| known(l).retain(|c| c.id.as_str() != "qa.2"))
}

fn qa10_with(edit: impl FnOnce(&mut tsuzuri_contract::question::QuestionCard)) -> Fetched {
    altered(|l| {
        let card = known(l)
            .iter_mut()
            .find(|c| c.id.as_str() == "qa.10")
            .expect("qa.10");
        edit(card);
    })
}

fn filled(fetched: &Fetched) -> Vec<ask::Card> {
    match ask::body(fetched) {
        Body::Filled(cards) => cards,
        other => panic!("Filled でない: {other:?}"),
    }
}

fn find<'a>(cards: &'a [ask::Card], id: &str) -> &'a ask::Card {
    cards.iter().find(|c| c.id.as_str() == id).expect("card")
}

/// 鍵の型は Hash と Eq を持つ（鍵つきの一覧の鍵の fn に渡せる）。
fn hashed<K: Hash + Eq>(key: &K) -> u64 {
    let mut h = DefaultHasher::new();
    key.hash(&mut h);
    h.finish()
}

/// 6 つの読み（NotRead・Failed・本文 not json・cards が Unknown・cards が空・fixture）。
fn six() -> Vec<Fetched> {
    vec![
        Fetched::NotRead,
        Fetched::Failed,
        Fetched::Body("not json".to_string()),
        Fetched::Body(
            wire::encode(&QuestionList {
                cards: Reading::Unknown,
            })
            .expect("電文"),
        ),
        Fetched::Body(
            wire::encode(&QuestionList {
                cards: Reading::Known(Vec::new()),
            })
            .expect("電文"),
        ),
        fixture(),
    ]
}

fn after<'a>(src: &'a str, mark: &str) -> &'a str {
    let i = src.find(mark).unwrap_or_else(|| panic!("字 {mark} が無い"));
    &src[i..]
}

fn between<'a>(src: &'a str, from: &str, to: &str) -> &'a str {
    let rest = after(src, from);
    let end = rest.find(to).unwrap_or_else(|| panic!("字 {to} が {from} より後に無い"));
    &rest[..end]
}

/// card の鍵は番号を 0 にした中身で、番号が詰まっても変わらず、中身が替われば替わる。
#[test]
fn qkey_card_key_rules() {
    let all = filled(&fixture());
    assert_eq!(all.len(), 2);
    for card in &all {
        let key = ask::card_key(card);
        assert_eq!(key.number, 0);
        assert_eq!(
            ask::Card {
                number: card.number,
                ..key.clone()
            },
            *card
        );
        assert_eq!(hashed(&key), hashed(&ask::card_key(card)));
    }
    let qa10 = find(&all, "qa.10");
    assert_eq!(qa10.number, 2);

    let rest = filled(&without_qa2());
    assert_eq!(rest.len(), 1);
    let moved = find(&rest, "qa.10");
    assert_eq!(moved.number, 1);
    assert_eq!(ask::card_key(moved), ask::card_key(qa10));
    assert_eq!(hashed(&ask::card_key(moved)), hashed(&ask::card_key(qa10)));
    assert_eq!(ask::target_number(&rest, "qa.10"), Some(1));

    let digest = filled(&qa10_with(|c| c.digest = "aaaaaaaaaaaaaaaa".to_string()));
    assert_ne!(ask::card_key(find(&digest, "qa.10")), ask::card_key(qa10));
    let title = filled(&qa10_with(|c| c.title = "別の題".to_string()));
    assert_ne!(ask::card_key(find(&title, "qa.10")), ask::card_key(qa10));
    // 替えていない card の鍵は替わらない。
    assert_eq!(
        ask::card_key(find(&title, "qa.2")),
        ask::card_key(find(&all, "qa.2"))
    );
}

/// 形は Filled の中身を捨てた値、列は Filled の中身かほかは空の列（ask と batch で同じ）。
#[test]
fn qkey_outline_and_listed() {
    let want: [Body<()>; 6] = [
        Body::Unmeasured(NOT_READ),
        Body::Unmeasured(ask::REASON),
        Body::Unmeasured(ask::UNREADABLE),
        Body::Unmeasured(ask::CARDS_UNKNOWN),
        Body::Empty(ask::EMPTY),
        Body::Filled(()),
    ];
    let reads = six();
    for (fetched, want) in reads.iter().zip(want.iter()) {
        assert_eq!(ask::outline(fetched), *want, "{fetched:?}");
        assert_eq!(batch::outline(fetched), *want, "{fetched:?}");
    }
    let (last, rest) = reads.split_last().expect("6 つ");
    for fetched in rest {
        assert!(ask::listed(fetched).is_empty(), "{fetched:?}");
        assert!(batch::listed(fetched).is_empty(), "{fetched:?}");
    }
    let Body::Filled(cards) = ask::body(last) else {
        panic!("fixture は Filled");
    };
    assert_eq!(ask::listed(last), cards);
    assert_eq!(ask::listed(last).len(), 2);
    let Body::Filled(rows) = batch::body(last) else {
        panic!("fixture は Filled");
    };
    assert_eq!(batch::listed(last), rows);
    assert_eq!(batch::listed(last).len(), 2);
}

/// 行の鍵は番号を 0 にした行で、番号は id から列の中で読む。
#[test]
fn qkey_batch_keys_and_numbers() {
    let cards = ask::cards(&fixture()).expect("fixture の card");
    let rows = batch::rows(&cards);
    assert_eq!(batch::number_of(&rows, &bead("qa.2")), Some(1));
    assert_eq!(batch::number_of(&rows, &bead("qa.10")), Some(2));
    assert_eq!(batch::number_of(&rows, &bead("qa.99")), None);
    for row in &rows {
        let key = batch::row_key(row);
        assert_eq!(key.number, 0);
        assert_eq!(
            batch::Row {
                number: row.number,
                ..key.clone()
            },
            *row
        );
        assert_eq!(hashed(&key), hashed(&batch::row_key(row)));
    }

    let rest_cards = ask::cards(&without_qa2()).expect("除いた一覧の card");
    let rest = batch::rows(&rest_cards);
    assert_eq!(rest.len(), 1);
    assert_eq!(batch::number_of(&rest, &bead("qa.10")), Some(1));
    assert_eq!(batch::number_of(&rest, &bead("qa.2")), None);
    let full = rows.iter().find(|r| r.id.as_str() == "qa.10").expect("qa.10");
    assert_eq!(batch::row_key(&rest[0]), batch::row_key(full));
    assert_eq!(
        hashed(&batch::row_key(&rest[0])),
        hashed(&batch::row_key(full))
    );
}

/// 2 つの block の DOM は鍵つきの一覧で、番号と経過と link を子の中で読み直す。
#[test]
fn qkey_dom_text() {
    let src = read("src/project/ask.rs");
    let dom = after(&src, "mod dom {");
    for word in ["<For", "card_key", "outline", "listed"] {
        assert!(dom.contains(word), "ask の mod dom に {word} が無い");
    }
    let list = between(dom, "let list = move ||", "section(BLOCK");
    assert!(list.contains("<For"), "{list}");
    assert!(!list.contains("collect_view()"), "{list}");
    assert!(!dom.contains(".number"), "ask の mod dom に .number");
    assert!(!dom.contains("now:"), "ask の mod dom に now の引数");
    assert!(!dom.contains("now :"), "ask の mod dom に now の引数");
    let head = between(dom, "Part::Head =>", "Part::Summary =>");
    assert!(head.contains("href=move ||"), "{head}");

    let src = read("src/project/batch.rs");
    let dom = after(&src, "mod dom {");
    for word in ["<For", "row_key", "outline", "listed", "number_of"] {
        assert!(dom.contains(word), "batch の mod dom に {word} が無い");
    }
    assert!(!dom.contains(".number"), "batch の mod dom に .number");
    let filled = between(dom, "fn filled(", "fn row_view(");
    assert!(filled.contains("<For"), "{filled}");
    assert!(!filled.contains(": Vec<Row>"), "{filled}");
}

/// 歯の名はどれも qkey_ で始まり、先の歯の filter の語を含まない。
#[test]
fn qkey_own_names_clean() {
    let text = read("tests/qkey.rs");
    let mut names = Vec::new();
    let mut rest = text.as_str();
    // 属性の行と次の行の `fn `（この字の literal は逆斜線の escape で改行を持たず当たらない）。
    let mark = "\n#[test]\nfn ";
    while let Some(i) = rest.find(mark) {
        let after = &rest[i + mark.len()..];
        let end = after.find('(').expect("fn の名の終わり");
        names.push(after[..end].trim().to_string());
        rest = &after[end..];
    }
    assert_eq!(names.len(), 5, "{names:?}");
    let words = [
        "accept_",
        "account_",
        "acctcore_",
        "acctdoc_",
        "acctframe_",
        "accthb_",
        "accthome_",
        "acctled_",
        "acctlook_",
        "acctpcore_",
        "acctproj_",
        "acctsess_",
        "acctwin_",
        "acctwire_",
        "askcard_",
        "batchpanel_",
        "board_min_",
        "contract_form_",
        "frame_",
        "gapspage_",
        "gquestion_",
        "graph_",
        "gview_",
        "hbconf_",
        "hbpost_",
        "hbproc_",
        "hbroute_",
        "hook_",
        "hsblock_",
        "hsderive_",
        "hspage_",
        "ledgerblock_",
        "mapgraph_",
        "mapview_",
        "nextstep_",
        "nodepage_",
        "parts_",
        "pipe_",
        "project_",
        "question_",
        "seatblock_",
        "seatcard_",
        "server_",
        "skeleton_",
        "stage_",
        "stats_",
        "steady_",
        "topbar_",
        "tz_",
        "mlink_",
        "gnav_",
        "mkeys_",
        "klink_",
        "ilink_",
        "afocus_",
        "nxact_",
        "urpanel_",
        "saxis_",
        "ticker_",
        "hfig_",
        "pmore_",
        "lspark_",
        "apop_",
        "brand_",
        "runsdoc_",
        "nbatch_",
        "kcli_",
        "qgate_",
        "nsum_",
        "hcard_",
        "ntime_",
        "ptitle_",
        "ncard_",
        "hsym_",
        "smore_",
        "lcard_",
        "aaround_",
        "hruling_",
        "sxaxis_",
        "plimit_",
        "bport_",
        "fstop_",
        "nsumw_",
        "fmark_",
        "fserve_",
        "cadopt_",
        "tipx_",
        "hcsess_",
        "hcproj_",
        "sesplit_",
        "mstore_",
        "athr_",
        "qblock_",
        "wsteady_",
        "gsum_",
        "nstall_",
        "pfold_",
        "uword_",
        "cround_",
        "cgdom_",
        "csled_",
        "lhome_",
        "shb_",
        "ghb_",
        "nact_",
        "aord_",
        "mtree_",
        "rhold_",
        "wstrip_",
        "flight_",
    ];
    assert_eq!(words.len(), 110);
    for name in &names {
        let rest = name
            .strip_prefix("qkey_")
            .unwrap_or_else(|| panic!("{name} が qkey_ で始まらない"));
        for w in words {
            assert!(!rest.contains(w), "{name} が {w} を含む");
        }
    }
}
