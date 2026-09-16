# 設計: 便 2 — `folio inject`（憲法の規範文を CLAUDE.md の生成区間へ書く・検査する）

- 要件: FR6（憲法を AI の手元に届ける・--write と --check）/ AC4（生成区間を 1 字変えると --check が非 0）/ FR5（3 値・実行できなかった検査を合格にしない）
- 条: P-14（inject の決定的な導出・--check の差分検出・R-2 の上限）/ P-6（正本から導出・生成物を手で直さない）/ N-2 ②（生成区間の外に規範を置かない）
- 判断の記録: ADR-3（folio2 は器・導出・差分 0）。便 0（main 53b4c83）と便 1（main 6e5b226）の上に置く。
- 裁定: 持ち主 2026-09-17「進めて良い」と前もっての確認（f2-648 notes・毎便問わない）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 c が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。fixture は最小の手書き（正本の写しは使わない・便 1 の教訓）。

## 1. 目的と中身

day-1 の暫定 script `scripts/inject_check.py`（憲法の規範文を CLAUDE.md の生成区間へ書く・検査する）を、命令 `folio inject` として Rust に写す。この便では script を消さず残す（突き合わせの歯の相手にする・消すのは別の便で持ち主の確認の後）。CLAUDE.md の生成区間の本文は 1 byte も変えない（導出が同じ式なら --write は何も書き換えない）。

命令の形: `folio inject --dir <正本の置き場> --claude-md <CLAUDE.md の path>` に、`--write`・`--check`・`--print` のどれか 1 つを添える（2 つ以上・0 個は引数の断り）。`--dir` の既定は `design-intent`・`--claude-md` の既定は `CLAUDE.md`。正本の置き場からは `constitution.yaml` と `rules.yaml` の 2 file だけを読む（語彙と要件書は読まない・正本の形の検査 `folio check` は呼ばない）。

導出（決定的・--write と --check と --print は同じ 1 つの関数を使う）: 母集団 = 憲法の articles の各条の statements の全規範文（strength 欄が must・must-not・should のどれか。それ以外の値が 1 つでも在れば「まだ分からない」）。行の形 = `規範文の id` + `: ` + `本文`（本文は前後の空白を落とし、連続する空白を半角空白 1 つに畳む。末尾が「。」でなければ「まだ分からない」）。先頭に `順位: ` + precedence.text（同じ畳み方）を 1 行置く（precedence.text が無いか空なら置かない）。行どうしは空行 1 つで区切る（改行 2 つ）。区間の中身 = 改行 + 本文 + 改行。規範文が 0 本なら「まだ分からない」。

区間: CLAUDE.md の中の `<!-- constitution:begin -->` と `<!-- constitution:end -->` の間。begin と end がそれぞれ 1 本ずつで begin が先にあるときだけ区間が定まり、それ以外（0 本・2 本以上・逆順）と CLAUDE.md が無い場合は「まだ分からない」。

上限（rules 行 R-2）: rules.yaml の thresholds の R-2 の value 欄から「byte」の直前の数（桁区切りの「,」を除く）を読む。読めなければ「まだ分からない」。本文の byte 数がこれを超えれば 違反（day-1 の script は終了コード 3 で断るが、folio は 3 値に揃えて 違反 = 1 にする。突き合わせの入力にはしない）。

判定と終了コード（合格 0 / 違反 1 / まだ分からない 2・便 0 と同じ 3 値）:
- `--print`: 本文を標準出力へ書き、末尾に改行 1 つ。規範文の本数と byte 数を標準エラーへ。上限の検査は行う（超えれば 1）。
- `--write`: 区間の中身を導出で置き換えて書く。差が無ければ書かない。上限を超えれば書かずに 1。
- `--check`: (1) 区間の外の行に、前後の空白を落として「する。」か「ない。」で終わる行（`<!--` で始まる行は除く）が 1 行でも在れば 違反。(2) 区間の中身が空（空白だけ）なら「まだ分からない」（未注入を合格にしない）。(3) 区間の中身が導出と byte 一致しなければ 違反。全部通れば 合格。

`folio inject` 自身の歯（`crates/folio/tests/inject.rs`・binary 経由・`tests/check.rs` と同じ形）の fixture は `tests/fixtures/inject/` の 5 組で、各組は `constitution.yaml`（条 2 つ・規範文 3 本・precedence あり）・`rules.yaml`（R-2 の行 1 つ）・`CLAUDE.md`（marker 1 対）の 3 file の最小の手書き。組と期待: `ok/` = 区間が導出と一致 → --check 0 / `drift/` = `ok/` の区間の本文を 1 byte だけ変えた → --check 1（AC4）/ `no-marker/` = begin の marker が無い → --check 2 / `outside/` = 区間は一致するが区間の外に「する。」で終わる行が 1 行 → --check 1 / `over-limit/` = rules.yaml の R-2 の値が本文より小さい → --check 1 と --write 1（書かない）。加えて歯は `drift/` の CLAUDE.md を一時 dir に写して --write（0）→ --check（0）を通し、写しの中身が `ok/CLAUDE.md` と byte 一致することを見る。

突き合わせの歯（同じ file `crates/folio/tests/inject.rs` に置く・day-1 の script `scripts/inject_check.py` を `--constitution`・`--claude-md`・`--rules` で回し、`folio inject` と終了コードを比べる。CI は Python と pyyaml を既に持つ）。入力は正本の `design-intent/constitution.yaml`・`design-intent/rules.yaml`・`CLAUDE.md` を一時 dir へ写し（git は要らない）変異を 1 つ当てる: (1) 変異なし --check → 0 と 0 / (2) --print の標準出力が byte 一致（終了コード 0 と 0）/ (3) CLAUDE.md の区間の本文の 1 文字を変える --check → 1 と 1 / (4) CLAUDE.md の end の marker を消す --check → 2 と 2 / (5) constitution.yaml の P-1 の最初の規範文の strength を `maybe` にする --check → 2 と 2 / (6) CLAUDE.md の区間の外の末尾に「これは規範である。」の 1 行を足す --check → 1 と 1。R-2 の超過は終了コードが違う（script 3・folio 1）ので入力にしない。

便 0 が置いた形との接続: `crates/folio/src/main.rs` に subcommand `Inject` を足し、実装は新規 `crates/folio/src/inject.rs` に置く。YAML は既存の `crates/folio/src/yaml.rs` の読み手を使う（欄の取り出しの補助を足してよい）。3 値は既存の `crates/folio/src/verdict.rs` の型を使い、変えない。外部 crate は増やさない（clap と yaml-rust2 のまま・`Cargo.toml` と `Cargo.lock` は触らない）。正規表現は使わない。便 0・便 1 の歯と fixture は触らない。

## 2. 範囲

- 入れる: `folio inject`（--write / --check / --print）・歯 `inject.rs`（fixture 5 組 + 突き合わせ 6 入力）。
- 入れない: script `scripts/inject_check.py` の削除（別の便・A-1）・CLAUDE.md の生成区間の外の文（見出しの「script の生成物」の句の書き換えは着地後に planner が別 PR で行う）・条の抜粋の選別（母集団は全規範文・便 2 では day-1 と同じ）・生成区間の他 file への注入。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| derive | 導出 | 憲法から区間の本文を決定的に作る |
| region | 区間 | CLAUDE.md の marker 1 対を見つけ、中身を読む・置き換える |
| limit | 上限 | rules 行 R-2 の値を読み byte 数と比べる |

## 4. 検査（歯）

§1 のとおり（fixture 5 組・--write の往復・突き合わせ 6 入力）。共通の検証は `.vessel.toml` の common-verify。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。突き合わせの歯は CI が既に持つ Python 3 と pyyaml を使う。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "c"
title = "folio inject — 憲法の規範文を CLAUDE.md の生成区間へ書く・検査する"
req = ["FR6", "FR5"]
section = "1"
write-set = ["+crates/folio/src/inject.rs", "crates/folio/src/main.rs", "crates/folio/src/yaml.rs", "+crates/folio/tests/inject.rs", "+tests/fixtures/inject/ok/constitution.yaml", "+tests/fixtures/inject/ok/rules.yaml", "+tests/fixtures/inject/ok/CLAUDE.md", "+tests/fixtures/inject/drift/constitution.yaml", "+tests/fixtures/inject/drift/rules.yaml", "+tests/fixtures/inject/drift/CLAUDE.md", "+tests/fixtures/inject/no-marker/constitution.yaml", "+tests/fixtures/inject/no-marker/rules.yaml", "+tests/fixtures/inject/no-marker/CLAUDE.md", "+tests/fixtures/inject/outside/constitution.yaml", "+tests/fixtures/inject/outside/rules.yaml", "+tests/fixtures/inject/outside/CLAUDE.md", "+tests/fixtures/inject/over-limit/constitution.yaml", "+tests/fixtures/inject/over-limit/rules.yaml", "+tests/fixtures/inject/over-limit/CLAUDE.md"]
verify = ["cargo nextest run -p folio inject", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "folio inject --check が正本と今の CLAUDE.md で合格（0）を返し、tests/fixtures/inject/ の 5 組で §1 の期待どおりの終了コードを返し、inject の歯（fixture 5 組 + --write の往復 + 突き合わせ 6 入力）が緑で CI が通る"
<!-- contracts:end -->
