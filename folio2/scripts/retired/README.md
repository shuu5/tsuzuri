# 退役した day-1 の床（可逆な移動・N-1.2）

2026-09-18 に持ち主の承認（A-1・逐語「承認するし質問も全て推奨で承認する」・台帳 f2-648 notes）で退役した。ここに在る file は動かさない・呼ばない。

- `check_draft.py`: day-1 の Python の床。便 1〜10 で `folio check` に写し終えた。`strict_yaml.py`（`scripts/` に残る）を読む。
- `run_floor_cases.py`: 凍結 fixture `tests/floor_cases.yaml` を Python の床で回す runner。写しは `crates/folio/tests/floor_cases.rs`。
- `parity.rs`: Python の床を物差しに `folio check` を突き合わせた歯（34 本）。物差しが退役したので歯も退役。独立した anchor は `tests/floor_cases.yaml`（134 case・手で凍結した期待）が持つ（P-10.1）。

残っているもの: `scripts/render_preview.py`・`render-preview.css`・`strict_yaml.py` は `folio render` の歯の物差し（要件面の暫定 `design-intent/preview/readable.html`）。scribe2 が要件面の yaml を読めるようになったら（s2-07l.467）別途 A-1 で問う。
