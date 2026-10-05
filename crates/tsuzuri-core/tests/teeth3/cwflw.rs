//! 話す窓を口座の移動に付いて来させる判じと 1 行の歯（接頭辞 cwflw_・設計ノート surface-wave29b 行 cs-follow・判断の記録 ADR-55 決定 (3)）。
#![cfg(test)]

use tsuzuri_contract::consult::{Stamp, StampEvent, WindowId};
use tsuzuri_core::consult::follow::{PROMPT, drifted, idle, input_clear};
use tsuzuri_core::consult::lines::{Event, notice};

fn stamp(event: StampEvent) -> Stamp {
    let sid = "bf1f3252-2ac9-42ce-bfb7-b5897fa42308".to_string();
    Stamp {
        at: 1,
        event,
        sid,
        source: None,
    }
}

/// ずれは印の口座が在り今の口座と違う時だけ。手すきは印の最後の行が会話の始まりか turn の終わりの時だけ（印が無ければ偽）。
#[test]
fn cwflw_drift_and_idle() {
    assert!(drifted(Some("/a"), Some("/b")));
    for (mark, now) in [
        (Some("/a"), Some("/a")),
        (None, Some("/b")),
        (Some("/a"), None),
        (None, None),
    ] {
        assert!(!drifted(mark, now), "{mark:?} {now:?}");
    }
    let [start, prompt, stop] =
        [StampEvent::Start, StampEvent::Prompt, StampEvent::Stop].map(stamp);
    assert!(idle(std::slice::from_ref(&start)));
    assert!(idle(&[prompt.clone(), stop.clone()]) && idle(&[stop.clone(), start.clone()]));
    assert!(!idle(&[]) && !idle(&[start, stop, prompt]));
}

/// 入力欄は `❯` を含む最後の行の右が空白だけの時に空で、字が在れば空でなく、その行が無ければ照らせない（None）。
#[test]
fn cwflw_input_clear_reads_the_last_prompt_line() {
    assert_eq!(PROMPT, '❯');
    assert_eq!(input_clear("● 答え\n\n❯ \n  ? for shortcuts\n"), Some(true));
    assert_eq!(input_clear("❯ 前の発言\n● 答え\n❯\u{a0}\n"), Some(true));
    assert_eq!(input_clear("❯ \n❯ 打ちかけ\n"), Some(false));
    assert_eq!(
        input_clear(" ❯ 1. Yes, I trust this folder\n   2. No, exit\n"),
        Some(false)
    );
    assert_eq!(input_clear("> \n$ \n"), None);
    assert_eq!(input_clear(""), None);
}

/// 口座のずれの 1 行は付いてくる口の命令と見張りの置き直しを持ち、付いてこなかった 1 行は持ち主に問う字で置き直しを持たない。
#[test]
fn cwflw_notice_lines() {
    let w = WindowId::new(1).expect("窓 id");
    assert_eq!(
        notice(&Event::Drift(w), "/x/tzw"),
        "相談: 話す窓 cw1 が前の口座で動いている（/x/tzw consult launch cw1 --follow）・「/x/tzw consult watch」を背景で置き直す"
    );
    assert_eq!(
        notice(&Event::Held(w), "/x/tzw"),
        "相談: 話す窓 cw1 が手すきにならず付いてこなかった（持ち主に問う）"
    );
}
