//! account board の各 project の席からの知らせの block（見本に無い block・HOME の頭の段・行 i-11・要件 FR16）。
//! project board の block「席からの知らせ」と同じ口と同じ電文を読み（`crate::project::notice`）、project ごとの最新の 1 つを
//! 電文の順（新しい順）に並べ、記録の読めない project の名を最後の 1 行に出す。
//! 題の link はその project の名前つきの窓で開く（windows の `win_name`・前に出す命令は撃たない）。

use crate::frame::Block;
use crate::project::Body;
use crate::project::notice::{Line, doc, line};
use crate::view::Fetched;

/// 読みの口（project の notice の module の定数を写す・account の下の file は口の字を持たない）。
pub use crate::project::notice::PATH;

pub const BLOCK: Block = Block {
    id: "notices",
    heading: "notices",
    class: "panel",
};

/// 知らせも読めない記録も無いときの 1 行。
pub const NONE_LINE: &str = "どの project の席からも知らせはまだ無い";

/// 読めない記録の行の頭。
const UNREAD_HEAD: &str = "知らせの記録が読めない project";

/// block の中身（電文の順の知らせの行と、記録の読めない project の名）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rows {
    pub lines: Vec<Line>,
    pub unread: Vec<String>,
}

/// block の中身: 読めなければ測れていない・知らせも読めない記録も無ければ `NONE_LINE`・ほかは行の列と読めない名。
/// 行の時刻は電文を組んだ時刻との差。
pub fn content(fetched: &Fetched) -> Body<Rows> {
    let doc = match doc(fetched) {
        Ok(doc) => doc,
        Err(reason) => return Body::Unmeasured(reason),
    };
    if doc.latest.is_empty() && doc.unread.is_empty() {
        return Body::Empty(NONE_LINE);
    }
    Body::Filled(Rows {
        lines: doc.latest.iter().map(|n| line(n, doc.at)).collect(),
        unread: doc.unread,
    })
}

/// 記録の読めない project の行の字（空の列は None・名は「・」で継ぐ）。
pub fn unread_line(names: &[String]) -> Option<String> {
    (!names.is_empty()).then(|| format!("{UNREAD_HEAD}: {}", names.join("・")))
}

#[cfg(target_arch = "wasm32")]
pub fn view() -> leptos::prelude::AnyView {
    dom::view()
}

/// 各 project の席からの知らせの DOM（wasm の target のときだけ）。
#[cfg(target_arch = "wasm32")]
mod dom {
    use leptos::prelude::*;

    use super::{BLOCK, PATH, Rows, content, unread_line};
    use crate::account::windows::win_name;
    use crate::project::notice::Line;
    use crate::project::{Body, body_view, section, unmeasured};

    pub fn view() -> AnyView {
        let fetched = crate::net::read(PATH);
        let body = move || match fetched.with(content) {
            Body::Unmeasured(reason) => unmeasured(reason),
            Body::Empty(line) => body_view(Body::Empty(line)),
            Body::Filled(rows) => rows_view(rows),
        };
        section(BLOCK, ().into_any(), body.into_any())
    }

    fn rows_view(rows: Rows) -> AnyView {
        let lines = rows.lines.into_iter().map(line_view).collect_view();
        let unread = unread_line(&rows.unread).map(|t| view! { <li class="small muted">{t}</li> });
        view! { <ul class="items">{lines}{unread}</ul> }.into_any()
    }

    /// 1 行（project の名・題〔link の先が在ればその project の窓で開く a・無ければ字だけ〕・時刻）。
    fn line_view(l: Line) -> AnyView {
        let title = match l.href {
            Some(href) => {
                view! { <a class="ttl" href=href target=win_name(&l.project)>{l.title}</a> }.into_any()
            }
            None => view! { <span class="ttl">{l.title}</span> }.into_any(),
        };
        view! { <li><b>{l.project}</b>{title}<span class="small muted num">{l.when}</span></li> }
            .into_any()
    }
}
