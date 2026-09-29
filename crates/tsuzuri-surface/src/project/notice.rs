//! block「席からの知らせ」（見本に無い block・home の左の列の頭・行 i-11・要件 FR16）: この project の席が「見て」と言った
//! 最新の 1 つ（題・頁への link・時刻）。端末の知らせを見逃しても board を開けば分かる。
//! 中身は口 /api/notices（契約の型の Notices）の latest から自分の名の 1 つを写すだけ。材料は tz stage notify の記録だけで、
//! 題は書き手が検めた字を写す。link にするのは http と https の URL だけで、同じ tab で開く（窓を前に出さない）。
//! 字と並びは純粋な関数にして host で試し、DOM は wasm の target のときだけ組み立てる。

use tsuzuri_contract::EpochSecs;
use tsuzuri_contract::notice::{Notice, Notices};
use tsuzuri_contract::wire;

use super::{Body, NOT_READ};
use crate::frame::Block;
use crate::view::{Fetched, clock_short};

pub const BLOCK: Block = Block {
    id: "notice",
    heading: "notice",
    class: "panel",
};

/// 読みの口（契約の型の crate の notice の PATH と同じ字）。
pub const PATH: &str = "/api/notices";

/// この file が字を持つ口の path（行 hs-derived）。
pub const PATHS: &[&str] = &[PATH];

/// この file の畳める段の開き閉じの鍵の形（無い・行 hs-derived）。
pub const FOLDS: &[&str] = &[];

/// 口が読めないときの理由。
pub const REASON: &str =
    "席からの知らせの口が読めない（server にまだ無い・届かない・知らせが切れた）";

/// 本文が電文として読めないときの理由。
pub const BAD_BODY: &str = "席からの知らせの口の本文が電文（Notices）として読めない";

/// 自分の記録の file は在るが読めないときの理由。
pub const OWN_UNREAD: &str =
    "この project の知らせの記録は在るが読めない（書きかけか形が違う）ので最新の知らせが分からない";

/// 自分の知らせがまだ無いときの 1 行。
pub const NONE_LINE: &str = "席からの知らせはまだ無い";

/// link にしてよい URL の頭。
const SCHEMES: [&str; 2] = ["http://", "https://"];

/// 知らせの 1 行（project の名・題・link の先〔http と https の URL だけ〕・時刻の字）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub project: String,
    pub title: String,
    pub href: Option<String>,
    pub when: String,
}

/// 口の本文を電文に読む（まだ読んでいない・読めない・電文が読めないは理由）。
pub fn doc(fetched: &Fetched) -> Result<Notices, &'static str> {
    match fetched {
        Fetched::NotRead => Err(NOT_READ),
        Fetched::Failed => Err(REASON),
        Fetched::Body(text) => wire::decode::<Notices>(text).map_err(|_| BAD_BODY),
    }
}

/// link の先（http:// か https:// で始まる URL だけ・ほかの字は None）。
pub fn href(url: &str) -> Option<String> {
    SCHEMES
        .iter()
        .any(|s| url.starts_with(s))
        .then(|| url.to_string())
}

/// 知らせを 1 行にする（時刻は `now` との差で短い日本時間の字・view の clock_short）。
pub fn line(notice: &Notice, now: EpochSecs) -> Line {
    Line {
        project: notice.project.clone(),
        title: notice.title.clone(),
        href: href(&notice.url),
        when: clock_short(notice.at, now),
    }
}

/// block の中身: 読めなければ測れていない・自分の名が unread に在れば `OWN_UNREAD` の測れていない・
/// latest に自分の名の知らせが在ればその 1 行（時刻は電文を組んだ時刻との差）・無ければ `NONE_LINE`。
pub fn content(fetched: &Fetched) -> Body<Line> {
    let doc = match doc(fetched) {
        Ok(doc) => doc,
        Err(reason) => return Body::Unmeasured(reason),
    };
    if doc.unread.contains(&doc.project) {
        return Body::Unmeasured(OWN_UNREAD);
    }
    match doc.latest.iter().find(|n| n.project == doc.project) {
        Some(n) => Body::Filled(line(n, doc.at)),
        None => Body::Empty(NONE_LINE),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    dom::view()
}

/// 席からの知らせの DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;

    use super::{BLOCK, Line, PATH, content};
    use crate::project::{Body, body_view, section, unmeasured};

    pub fn view() -> AnyView {
        let fetched = crate::net::read(PATH);
        let body = move || match fetched.with(content) {
            Body::Unmeasured(reason) => unmeasured(reason),
            Body::Empty(line) => body_view(Body::Empty(line)),
            Body::Filled(line) => view! { <ul class="items">{line_view(line)}</ul> }.into_any(),
        };
        section(BLOCK, ().into_any(), body.into_any())
    }

    /// 1 行（題は link の先が在れば同じ tab で開く a・無ければ字だけ・後に時刻）。
    fn line_view(l: Line) -> AnyView {
        let title = match l.href {
            Some(href) => view! { <a class="ttl" href=href>{l.title}</a> }.into_any(),
            None => view! { <span class="ttl">{l.title}</span> }.into_any(),
        };
        view! { <li>{title}<span class="small muted num">{l.when}</span></li> }.into_any()
    }
}
