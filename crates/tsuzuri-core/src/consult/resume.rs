//! 話す窓の撃ち直しの会話の続き（判断の記録 ADR-55 決定 (1)(3)）。
//! 撃ち直しで続ける会話の id を、作業場の会話の印の最後の id と席が名指した id から決める（`pick`）。

use tsuzuri_contract::consult::Form;

use super::stamp::is_uuid;

/// 撃ち直しで続ける会話の id（`last` は作業場の会話の印の最後の id・`named` は席が名指した id・断りは理由の字）。
/// 続けるのは話す窓の撃ち直しだけで、名指しは会話の印を持たない窓だけに受ける。続けない時は None。
pub fn pick(
    form: Form,
    again: bool,
    last: Option<&str>,
    named: Option<&str>,
) -> Result<Option<String>, &'static str> {
    let talk = again && form == Form::Talk;
    match (named, last) {
        (Some(_), _) if !again => Err("--session は --again と一緒に渡す"),
        (Some(_), _) if form == Form::Ask => Err("問う窓は会話を続けない（--session を外す）"),
        (Some(id), _) if !is_uuid(id) => Err("--session の値が会話の id の形（uuid）でない"),
        (Some(_), Some(_)) => {
            Err("窓は会話の印を持つ（印の最後の会話の id で続くので --session を外す）")
        }
        (Some(id), None) => Ok(Some(id.to_string())),
        (None, Some(id)) if talk && is_uuid(id) => Ok(Some(id.to_string())),
        (None, Some(_)) if talk => {
            Err("会話の印の最後の会話の id が uuid の形でない（印の file を確かめる）")
        }
        (None, _) => Ok(None),
    }
}
