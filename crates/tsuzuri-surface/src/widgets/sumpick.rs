//! 表示の型で概要を 1 つ選ぶ部品と、長い概要の畳み（行 g-sum-pick・判断の記録 ADR-30 決定 (2)(3)・規則の行 R-19）。
//! 個別の頁・質問の窓の card・hover の card・吹き出しが同じ関数で選ぶ。純な関数だけを持ち host でも組む。

use crate::frame::Mode;

/// 吹き出しの概要を畳む字数（規則の行 R-19・Unicode のスカラー値で数える）。
pub const POP_SUM_MAX: usize = 200;

/// 畳んだ頭の後ろに置く印。
pub const ELLIPSIS: char = '…';

/// 非エンジニア向けの概要の語の鍵。
pub const PLAIN_KEY: &str = "summary_plain";

/// エンジニア向けの概要の語の鍵。
pub const ENG_KEY: &str = "summary_eng";

/// 本文の頭の 1 行の語の鍵（2 つの概要が両方無い時の印）。
pub const BODY_KEY: &str = "sum_body";

/// 選んだ概要（どの字かの語の鍵・字・表示の型の側の概要でない時の印を出すか）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picked {
    pub key: &'static str,
    pub text: String,
    pub marked: bool,
}

/// 空か空白だけの字は無いと見る。
fn given(text: Option<&str>) -> Option<&str> {
    text.filter(|t| !t.trim().is_empty())
}

/// 表示の型の側の概要の鍵と字・もう一方の鍵と字（初心者は非エンジニア向けが側）。
fn sides<'a>(
    mode: Mode,
    plain: Option<&'a str>,
    eng: Option<&'a str>,
) -> [(&'static str, Option<&'a str>); 2] {
    match mode {
        Mode::Beginner => [(PLAIN_KEY, plain), (ENG_KEY, eng)],
        Mode::Expert => [(ENG_KEY, eng), (PLAIN_KEY, plain)],
    }
}

/// 表示の型の概要を 1 つ選ぶ（字は切らない）。表示の型の側が無ければもう一方を印つきで、
/// 両方無ければ本文の頭の 1 行を印つきで返し、それも無ければ None。
pub fn pick(
    mode: Mode,
    plain: Option<&str>,
    eng: Option<&str>,
    excerpt: Option<&str>,
) -> Option<Picked> {
    let [own, other] = sides(mode, plain, eng);
    [(own, false), (other, true), ((BODY_KEY, excerpt), true)]
        .into_iter()
        .find_map(|((key, text), marked)| {
            given(text).map(|t| Picked {
                key,
                text: t.to_string(),
                marked,
            })
        })
}

/// 表示の型の側でないもう一方の概要（表示の型の側が在る時だけ・個別の頁が概要の下に畳んで置く）。
pub fn other(mode: Mode, plain: Option<&str>, eng: Option<&str>) -> Option<Picked> {
    let [(_, own), (key, text)] = sides(mode, plain, eng);
    given(own)?;
    given(text).map(|t| Picked {
        key,
        text: t.to_string(),
        marked: true,
    })
}

/// 畳んだ字（頭と、畳んだ残り・畳まない時は残りが None）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folded {
    pub head: String,
    pub rest: Option<String>,
}

impl Folded {
    /// 畳んだ形の字（頭と印・畳まない時は全文）。
    pub fn shut(&self) -> String {
        match self.rest {
            Some(_) => format!("{}{ELLIPSIS}", self.head),
            None => self.head.clone(),
        }
    }

    /// 開いた形の字（頭と残りをつないだ全文）。
    pub fn whole(&self) -> String {
        format!("{}{}", self.head, self.rest.as_deref().unwrap_or_default())
    }
}

/// `max` 字以下なら畳まず、越えれば頭の `max - 1` 字と残りに分ける（畳んだ形は頭と印の `max` 字）。
pub fn fold(text: &str, max: usize) -> Folded {
    if text.chars().count() <= max {
        return Folded {
            head: text.to_string(),
            rest: None,
        };
    }
    let cut = text
        .char_indices()
        .nth(max.saturating_sub(1))
        .map_or(text.len(), |(i, _)| i);
    Folded {
        head: text[..cut].to_string(),
        rest: Some(text[cut..].to_string()),
    }
}
