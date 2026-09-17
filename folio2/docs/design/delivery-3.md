# 設計: 便 3 — `folio inject` の歯から day-1 の script を外す（削除の前段）

- 要件: FR6（憲法を AI の手元に届ける・--write と --check）/ FR5（3 値・実行できなかった検査を合格にしない）/ AC4（生成区間を 1 字変えると --check が非 0）
- 条: P-14（inject の決定的な導出・--check の差分検出）/ P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する＝導出器を 2 本並べたままにしない）/ P-10.1（検査は生成側からも検査側からも独立した凍結 anchor を持つ）
- 判断の記録: ADR-3（folio2 は器・導出・差分 0）。便 2（main b6db144）の上に置く。
- 裁定: 持ち主 2026-09-17「全部承認する」（A-1・day-1 の script `scripts/inject_check.py` の削除と歯の作り直しを便 3 に載せる・f2-648 notes 10:4x JST）と前もっての確認（毎便問わない）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 d が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・既存 file 4 本＝触る 2 本 + verify で回す歯の file 2 本・新規 file なし・fixture は触らない）。

## 1. 目的と中身

便 2 で `folio inject` が着地し、CLAUDE.md の生成区間の導出器が day-1 の script `scripts/inject_check.py` と Rust の 2 本並んでいる。持ち主は script の削除を承認した（A-1・2026-09-17）。器の契約表の write-set には file を消す形が無い（接頭辞の形は 新規 と 縮む面 の 2 つ・消した file は着地後の契約表の検査で解けなくなる）ので、この便は削除そのものを運ばず、script を呼んでいる 2 file から呼び出しを外す。着地の後、planner が `scripts/inject_check.py` を別の PR で消す（この便の外・この文書 §2）。

(a) `folio inject` の歯 `crates/folio/tests/inject.rs`。便 2 の突き合わせ 6 入力（day-1 の script と folio の終了コードを比べていた）を、script を呼ばない形に作り直す。入力と期待の終了コードは便 2 の §1 の値をそのまま歯に固定する（期待は script でなく数で持つ）: 正本の `design-intent/constitution.yaml`・`design-intent/rules.yaml`・`CLAUDE.md` を一時 dir へ写し、変異を 1 つ当て、folio だけを掛ける。(1) 変異なし --check → 0 / (2) 変異なし --print → 0 で、標準出力の先頭に改行 1 つを足したものが、写した CLAUDE.md の begin の印の直後から end の印の直前までの byte 列と一致する（便 2 の §1 の 区間の中身 = 改行 + 本文 + 改行・--print = 本文 + 改行。main b6db144 の実測: 標準出力 7,631 byte・区間 7,632 byte・一致）/ (3) CLAUDE.md の区間の本文の 1 文字を変える --check → 1 / (4) CLAUDE.md の end の印を消す --check → 2 / (5) constitution.yaml の P-1 の最初の規範文の strength を maybe にする --check → 2 / (6) CLAUDE.md の区間の外の末尾に「これは規範とする。」の 1 行を足す --check → 1。歯の名前は便 2 のもの（`inject_parity_…`）を保ってよいし `inject_pinned_…` に改めてもよいが、6 本の入力と期待は変えない。`script_inject` と、script の出力を受ける側の引数・戻り値は消す。便 2 の他の歯（正本と今の CLAUDE.md で --check 合格・fixture 5 組・--write の往復・mode は 1 つだけ）と `tests/fixtures/inject/` の 5 組は触らない。

(b) day-1 の床 `scripts/check_draft.py` の inject の節（`import inject_check` で導出の本数と 前文 + 規範文 の本数を比べていた 1 節・script の冒頭の検査の一覧の `inject` の 1 行を含む）を外す。この節は導出器が 2 本あったときの相互検査で、Rust では --write・--check・--print が同じ 1 つの導出関数を使うため対応物が無い。他の節（schema / refs / rules / vocab / polarity / adr / anchor）と終了コードの 3 値・`--emit-amends`・`--freeze-anchor` は 1 字も変えない。`folio check` との突き合わせの歯 crates/folio/tests/parity.rs（2026-09-18 に scripts/retired/ へ退役） は check_draft.py を回すので、この変更で緑のまま（parity の 9 入力は inject の節に触れない）。

判定と終了コードは便 2 の §1 のまま（合格 0 / 違反 1 / まだ分からない 2）。`crates/folio/src/inject.rs`（unit の歯の名が inject を含む）と crates/folio/tests/parity.rs（2026-09-18 に scripts/retired/ へ退役） は verify で回す歯の file として write-set に在るが触らない（本文も期待も変えない）。`crates/folio/src/` の実装は変えない。外部 crate は増やさない（`Cargo.toml` と `Cargo.lock` は触らない）。正規表現は使わない。CI は Python と pyyaml を持ち続ける（parity の歯が check_draft.py を回す）。

## 2. 範囲

- 入れる: `crates/folio/tests/inject.rs` の突き合わせ 6 入力の作り直し（script を呼ばず期待を数で固定）・`scripts/check_draft.py` の inject の節の撤去。
- 入れない: `scripts/inject_check.py` の削除そのもの（着地後に planner が別 PR で消す・持ち主の承認は済み）・`crates/folio/src/` の変更・fixture の変更・語彙 R-9 の検査（便 4）・判断の記録と anchor の検査（便 5 以降）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| pinned | 固定の期待 | 6 入力の終了コードと --print の byte 列を script でなく数と区間の写しで持つ |
| floor-trim | 床の節の撤去 | check_draft.py から導出器の相互検査の節を外す |

## 4. 検査（歯）

§1 のとおり（inject の歯 6 入力 + 便 2 の歯そのまま・parity の歯が check_draft.py を回して緑）。共通の検証は `.vessel.toml` の common-verify。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。CI の Python 3 と pyyaml は parity の歯のために残す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "d"
title = "folio inject の歯から day-1 の script を外す（削除の前段）"
req = ["FR6", "FR5"]
section = "1"
write-set = ["crates/folio/tests/inject.rs", "scripts/check_draft.py", "crates/folio/src/inject.rs", "crates/folio/tests/parity.rs"]
verify = ["cargo nextest run -p folio inject", "cargo nextest run -p folio parity", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "inject の歯 6 入力が script を呼ばず §1 の終了コードと --print の区間一致で緑、parity の歯が inject の節を外した check_draft.py で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
