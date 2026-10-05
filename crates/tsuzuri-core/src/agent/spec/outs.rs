//! 出す物の字の形（判断の記録 ADR-59 決定 (4)・ADR-63 決定 (4)・要件 FR21）。
//! 終える前の門は札の出す物の字を係の出力の dir `<名>/w/` に join して探すので、出す物の字は w/ からの相対が正本である。
//! 起こしの門は、`w/` か `/` で始まる字と `..` の段を持つ字を断り、出す物の行だけを直した頭の見本を返す。

use super::{HOLES, fields, fixed, outputs};

/// 出す物の字が w/ からの相対でないか（`w/` か `/` で始まるか、`..` の段を持つ）。
fn stray(item: &str) -> bool {
    item.starts_with('/') || item.starts_with("w/") || item.split('/').any(|s| s == "..")
}

/// 出す物の字のうち w/ からの相対でない字（並びのまま）。
pub fn astray(outputs: &[String]) -> Vec<String> {
    outputs.iter().filter(|o| stray(o)).cloned().collect()
}

/// 直した字。`/` で始まるか `..` の段を持つ字は最後の `/w/` の後にし、それでも直らなければ末の段にする。頭の `w/` は外す。
/// w/ からの相対の字はそのまま。
pub fn mend(item: &str) -> String {
    if !stray(item) {
        return item.to_string();
    }
    let loose = |s: &str| s.starts_with('/') || s.split('/').any(|p| p == "..");
    let mut s = item;
    if loose(s) {
        s = s.rfind("/w/").map_or(s, |i| &s[i + 3..]);
    }
    if loose(s) {
        s = s.rsplit('/').next().unwrap_or_default();
    }
    while let Some(rest) = s.strip_prefix("w/") {
        s = rest;
    }
    s.to_string()
}

/// 断りの理由の字（断った字と次の一手と、出す物の行だけを直した頭の見本・直して空になる字は落とし、全部落ちれば書き方）。
pub fn reason(astray: &[String], prompt: &str) -> String {
    let mended: Vec<String> = fields(prompt)[3]
        .map(outputs)
        .unwrap_or_default()
        .iter()
        .map(|o| mend(o))
        .filter(|o| !o.is_empty())
        .collect();
    let line = if mended.is_empty() {
        HOLES[3].to_string()
    } else {
        mended.join(",")
    };
    let head: Vec<String> = fixed(prompt)
        .lines()
        .map(|l| {
            if l.starts_with("出す物: ") {
                format!("出す物: {line}")
            } else {
                l.to_string()
            }
        })
        .collect();
    format!(
        "係の起こしの門は止める（出す物の字 {} が出力の dir w/ からの相対でない・w/ か / で始まるか .. の段を持つ） 次の一手 = prompt の頭に次の 4 行を置き、空の行の後に頼みを書く\n{}",
        astray.join("・"),
        head.join("\n")
    )
}
