//! stylesheet の部品（行 g-style-parts・判断の記録 ADR-58）。基の style.css は index.html の link のまま配る。
//! style の dir の部品は組み立ての script が名の byte の順の表 PARTS にし、wasm が面を描く前に head の末尾へ
//! `<style>` 1 つとして入れる（基の後に効く）。selector の字は基と部品の全部の中で 1 つの file だけが持つ。

include!(concat!(env!("OUT_DIR"), "/style_parts.rs"));

/// 部品を名の順につないだ字（部品ごとに名の注の 1 行を前に置き、末尾に改行が無ければ足す）。
pub fn joined() -> String {
    let mut out = String::new();
    for (name, text) in PARTS {
        out.push_str("/* ");
        out.push_str(name);
        out.push_str(" */\n");
        out.push_str(text);
        if !text.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

/// 部品をつないだ字を head の末尾へ `<style>` 1 つとして入れる（面を描く前に 1 度だけ呼ぶ・head が無ければ入れない）。
#[cfg(target_arch = "wasm32")]
pub fn inject() {
    let document = leptos::prelude::document();
    let (Ok(style), Ok(Some(head))) = (
        document.create_element("style"),
        document.query_selector("head"),
    ) else {
        return;
    };
    style.set_text_content(Some(&joined()));
    if head.append_child(&style).is_err() {
        leptos::logging::warn!("stylesheet の部品を head に入れられない");
    }
}
