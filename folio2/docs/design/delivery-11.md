# 設計: 便 11 — 読み物の生成器 `scripts/render_preview.py` を `folio render` に写し、出力を byte 一致させる

- 要件: FR4（人が読むページは 1 つの生成器から出す）/ FR5（3 値・実行できなかった生成を合格にしない）
- 条: P-2.1（人が読むページはすべて 1 つの生成器から出力する）/ P-6.1・P-6.2（正本は YAML・生成物を手で直さない）/ P-6.3（同じ内容を 2 面が持つとき一方を正本とし他方は導出する）/ P-4.1（実行できなかった結果を異常なしにしない）/ P-10.1・P-10.2（独立した凍結 anchor・生成物どうしの突き合わせを唯一の判定にしない）
- 判断の記録: ADR-1（帰結「M0 で folio が同じ検査を出せた時点で床の script を退役」）。便 10（f2-648.20）の後に直列で置く。
- 裁定: 持ち主 2026-09-17「進めて良い」と前もっての確認（f2-648 notes）。day-1 の script の削除は本便に含めない（A-1・別に問う）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 l が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。

## 1. 目的と中身

day-1 の読み物 `design-intent/preview/readable.html`（正本 4 file と判断の記録から導出した 1 面・`.vessel.toml` の requirements が指す暫定の要件面）を作る生成器は、いま Python の `scripts/render_preview.py`（142 行）だけである。これを Rust の副命令 `folio render` に写し、同じ入力から 1 byte も違わない出力を出す。planner の実測（2026-09-17・main 512c132）: script の出力と版管理の readable.html は byte 同一（194830 byte）。 要件との対応: FR4 のうち本便が運ぶのは「単一の生成器・単一の design token（css 1 本）」へ生成の口を寄せる段だけで、入口・憲法・要件書の 3 面そのものは M0 の後続の便が出す（本便の出力は day-1 の読み物 1 面）。FR5 は (a) の 3 値と (d) で満たす。

(a) 命令の形: `folio render --dir <正本の置き場・既定 design-intent> --out <出力先・既定 preview/readable.html> --write|--check`。
- `--out` が相対なら `--dir` からの相対（script が `--dir` へ移ってから開くのと同じ）・絶対ならそのまま。
- `--write` と `--check` はどちらか 1 つを必ず指定する（便 2 の `folio inject` と同じ ArgGroup の形）。script はどちらも無いとき write に倒すが、folio は倒さない（黙って書かない）。
- 終了コードと文言（3 値・字面は script と同じ）:
  - check で出力先が無い = 2・標準エラー「render: 読み物が無い（未生成）」
  - check で不一致 = 1・標準エラー「render: DRIFT — 読み物 N byte ≠ 導出 M byte（手で直したか正本が変わった）」（N = 今の file の byte 数・M = 導出の byte 数）
  - check で一致 = 0・標準出力「render: OK — 読み物は正本と一致（N byte）」
  - write = 0・標準出力「rendered bytes N」（N の前は半角空白 1 つ）
  - 導出できない（下の (d)）= 2・標準エラー「folio render: まだ分からない: <理由>」。このとき `--write` でも出力先に 1 byte も書かない。
- check の比較は byte 列の一致（script は読むとき改行を読み替えるが、folio は読み替えない＝厳しい側）。出力先が UTF-8 として読めない file でも byte 比較なので 1（不一致）になる。

(b) 導出の規則: `scripts/render_preview.py` の 8 行目〜136 行目を上から順にそのまま写す。見出し・文言・HTML の字面・順序・行の区切り（各要素を改行 1 つで繋ぎ、末尾に改行を置かない）は script のとおり。写すときに同じにする点:
- 読み: 正本 4 file（constitution.yaml・rules.yaml・vocabulary.yaml・srs.yaml）と `adr/` の下で名が ADR- で始まり .yaml で終わる file を、便 7 の型付きの読み手 `yaml::parse_typed` で読む（表は書かれた順を保つ＝Python の表の順と同じ）。重複キーは `yaml::parse` の duplicates で数え、1 つでも在れば (d) の 2。`adr/schema.yaml` は読まない。
- 判断の記録の順: 各 file の欄 id を「-」で割った 2 番目を ASCII の数字列として読み、その数の昇順。数字列でない・同じ数が 2 本以上（script では並びが dir の列挙順に依り決まらない）は 2。
- 凍結 anchor の列: `anchors/index.yaml` が無ければ「なし（または索引が空）」の文。在れば entries の各要素の version を文字列化して「 → 」で繋ぐ。entries が空の一覧でも「なし」の文。
- CSS: `scripts/render-preview.css` の本文を `include_str!` で binary に埋め込む（実行時に file を探さない・css は読むだけ・write-set の外・本文は変えない）。
- escape: script の `html.escape`（quote 込み）と同じ 5 字（& → `&amp;`・< → `&lt;`・> → `&gt;`・二重引用符 → `&quot;`・一重引用符 → `&#x27;`）。script が escape している所だけ escape し、していない所（条 id・規範文 id・要件 id・ゴール id・制約 id をそのまま差し込む箇所）はしない。
- 文字列化: script が Python の文字列化（str） を通す所は scalar を `Value::py_str`（None・True・False・整数の字面・日付の字面・小数）で写す。script が `html.escape` に値をじかに渡す所と「・」や「 ／ 」でじかに繋ぐ所は文字列だけを受け、文字列でなければ 2（script は traceback で落ちる）。`%d` の所は整数だけを受け、それ以外は 2。
- 一覧と表の文字列化（Python の repr の狭い写し）: script の Python の文字列化（str） に一覧か表が渡る所が正本に 1 か所在る（要件書 meta の changes_from_v1_1 の 1 番目の要素が表として読まれている＝正本の側の書き損じだが、読み物は今その repr を出しており byte 一致のために写す）。表 = 「{」+ 各組を「キーの repr: 値の repr」にして「, 」で繋ぐ +「}」・一覧 = 「[」+「, 」繋ぎ +「]」・文字列 = 一重引用符で囲む・null と真偽と整数は py_str と同じ。文字列の中身は次の字だけを受ける: ASCII の 0x20〜0x7e のうち一重引用符・二重引用符・逆斜線を除いたもの／Rust 標準の字の判定 is_alphanumeric が真の字／次の閉じた一覧の記号「、。・「」（）〔〕→：／—＝＋〜」。それ以外の字を含む文字列・日付・小数が一覧や表の中に在れば 2（Python の repr が escape や別の字面を出す形を当て推量で写さない・fail-closed）。
- 真偽の判定: script の `if 値:` と `値 or 既定` は Python の偽（null・false・整数 0・小数 0.0・空の文字列・空の一覧・空の表）で倒れる。同じ判定の関数を 1 つ置いて使う。`x.get(k) is not None` の所は null かどうかだけを見る。
- 表引き: 段・縛る相手・型・強さ・機構・出所の種別・判断の記録の状態・案の判定の表引き（TIER・BIND・PAT・STR・MECH・KIND・AST・VERD）で表に無い値は 2（script は KeyError）。LIVE と DOCST は script と同じく、表に無ければ値の文字列化をそのまま出す。
- 閾値の値の読める形（script の関数 val）: 表 = 欄ごとに `<br>` 区切りで全角空白の字下げ（深さ d 個）+ 太字のキー + 値（値が表なら `<br>` を挟んで 1 段深く）・一覧 = 文字列の要素は「」で括り、それ以外は同じ関数へ・「・」繋ぎ・null = `<code>null</code>`・それ以外 = 文字列化して escape。
- 要件書の版ごとの変更点: meta のキーのうち changes_from_ で始まるものを文字列の昇順に並べ、見出しは接頭辞を除いて「_」を「.」に替える。

(c) 歯 `crates/folio/tests/render.rs`（binary 経由・歯の関数名はすべて render を含める＝verify の filter 語）。入力は parity の歯と同じ作り（`design-intent/` を丸ごと一時 dir へ写す・preview/ 込み・git は要らない）。版管理の `design-intent/preview/readable.html` は書き換えない・再生成して commit しない（歯の `--out` は必ず一時 file か写しの中）。script を起動するときは `--dir` も `--out` も絶対 path で渡す（script は `--dir` へ移ってから開く）。
- 凍結 anchor との一致（P-10.1）: 写しに `--out <一時 file> --write` → 終了 0 ∧ 出力が版管理の `design-intent/preview/readable.html` と byte 一致。
- check の 3 値: 写しそのままで `--check` = 0 ∧ 標準出力に「render: OK」／写しの srs.yaml の meta の title の字を 1 つ変えて `--check` = 1 ∧ 標準エラーに「DRIFT」／写しの preview/readable.html を消して `--check` = 2 ∧ 標準エラーに「読み物が無い」／その後 `--write` = 0 → `--check` = 0。
- script との突き合わせ（oracle・P-10.2 により唯一の判定にしない＝上の anchor と併せる）: 写しに変異を 1 つ当て、`python3 scripts/render_preview.py --dir <写し> --out <絶対 path A> --write` と `folio render --dir <写し> --out <絶対 path B> --write` の出力が byte 一致、かつ変異前の出力と違うこと（変異が効いていない突き合わせを緑にしない）。変異は今の読み物が通していない分岐に当てる 4 つ: 判断の記録 1 本に amends の 1 要素（version・target・field・previous_text・new_text）を置く／`anchors/index.yaml` を消す／srs.yaml の meta に status_note を足す／判断の記録 1 本に superseded_by を足す。python3 を起動できなければ歯を落とす（飛ばして緑にしない）。
- 導出できない入力 = 2 で何も書かない: rules.yaml に重複キーを足す／constitution.yaml の north_star を消す／条の tier を表に無い値にする／changes_from_v1_1 の表の値に逆斜線を足す。どれも `--out <一時 file> --write` = 2 ∧ 標準エラーに「まだ分からない」∧ 一時 file が出来ていない。
- unit の歯（`src/render.rs` の中・名に render を含む module の下）: escape の 5 字・真偽の判定・repr の狭い写し（受ける形 1 つと断る形 3 つ: 一重引用符入り・日付・全角空白）・判断の記録の順（10 が 2 の後）。

(d) script と揃えない点（ここに挙げたものだけ）: 正本が読めない・重複キー・必須の欄が無い・型が違う・表引きに無い値・repr の狭い写しが断る形・出力先の親 dir が無い は、script では traceback（終了 1）だが folio は 2「まだ分からない」（P-4.1・`folio check` の読めない側と同じ judgement）。空の文書（注釈だけの file を含む）は 4 file・判断の記録・索引のどれでも 2。`--write` と `--check` の両方無し・両方有りは clap の使い方の誤り（終了 2）。突き合わせの歯はこれらの入力を oracle に掛けない。

(e) 便 10 までの形との接続: 新規は `crates/folio/src/render.rs` と歯 `crates/folio/tests/render.rs`。`crates/folio/src/main.rs` は `mod render;` と副命令 Render の追加だけ（Check と Inject の枝は 1 字も変えない）。使う口はすべて既に pub（`yaml::parse`・`yaml::parse_typed`・`yaml::Value`・`Value::py_str`・`verdict::Verdict`）で、可視性の変更は要らない。`yaml.rs`・`verdict.rs`・他の src は触らない。既存の歯 `crates/folio/tests/check.rs` は write-set に在るが 1 字も変えない（受付の門が歯の置き場を解くための既存の歯の file・期待は変えない）＝verify の 2 行目で便 0 の check の歯が期待不変で緑であることを回帰の門にする。verify の filter render は base の歯の名に当たらない（`lineage.rs` の render_val は歯でない private の関数・実測）。`folio check` に render の検査を組み込まない（floor_cases の 134 case の写しは読み物を直さないので、組み込むと期待が化ける）。`scripts/render_preview.py`・`scripts/render-preview.css`・`scripts/strict_yaml.py`・`design-intent/` の下・`.github/workflows/` は触らない。外部 crate は増やさない（clap と yaml-rust2 のまま・`Cargo.toml` と `Cargo.lock` は触らない）。正規表現は使わない。

## 2. 範囲

- 入れる: 副命令 `folio render`（write / check・3 値）・導出の写し・歯（凍結 anchor・check の 3 値・script との突き合わせ 4 変異・導出できない入力 4 つ・unit）。
- 入れない: 見本 3 面（index / constitution / srs）の生成器（設計の分岐を持ち主に確認してから別便）・`folio check` への組み込み・CI の段の変更・day-1 の script の削除や変更（A-1・別に問う）・要件書 meta の書き損じ（表になっている変更点）の是正（要件書の版上げで別に扱う）・`.vessel.toml` の requirements の付け替え。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| load | 読み | 4 file + 判断の記録 + 索引を型付きで読む・重複キーと空の文書を 2 に |
| pyfmt | Python の字面の写し | 文字列化・真偽・escape・repr の狭い写し |
| page | 導出 | script の順に HTML の字面を組む・CSS の埋め込み |
| mode | write / check | 3 値と文言・導出できないときは書かない |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 10 の歯は期待不変で全部回る）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。突き合わせの歯は CI が既に持つ Python 3 と pyyaml を使う（parity の歯と同じ・本便では変えない）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "l"
title = "読み物の生成器を folio render に写し、出力を byte 一致させる"
req = ["FR4", "FR5"]
section = "1"
write-set = ["+crates/folio/src/render.rs", "+crates/folio/tests/render.rs", "crates/folio/src/main.rs", "crates/folio/tests/check.rs"]
verify = ["cargo nextest run -p folio render", "cargo nextest run -p folio check", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "render の歯（凍結 anchor との byte 一致・check の 3 値・script との突き合わせ 4 変異・導出できない入力 4 つ・unit）が緑、便 0 の check の歯が期待不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
