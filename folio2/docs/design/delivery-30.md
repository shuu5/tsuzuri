# 設計: 便 30 — 設計ノートの図 1 枚を型付き記述から図の道具で描く `folio figure`（FR15 / AC12・ADR-4 決定 (2)(3)(5)(6)）

- 要件: FR15（図は型付き記述から生成し、通らない図は出さない: 道具を子の処理として呼び、検査と描画を仕上がりの段で掛け、出力から図の本体（SVG）だけを取り、通らない図は生成せず前の生成物も上書きしない、凍結 anchor が落ちたら「まだ分からない」）
- 条: P-2.1（人が読むページの生成器は folio 1 つ・道具は内側の部品を作る道具）/ P-4.1・P-4.2（実行できなかった結果を「異常なし」にしない・判定できないものは「まだ分からない」）/ P-10.1・P-10.3（独立した凍結 anchor・落ちたら「まだ分からない」）/ P-11.1（同じ図の往復の上限は rules 行 R-7・超えたら持ち主へ）/ A-3.1（依存を足す確認）
- 判断の記録: ADR-4（決定 (2) 子の処理で検査と描画・仕上がりの段は最上段固定（rules 行 R-14）・通らない図は生成せず前の生成物も上書きしない／決定 (3) 出力から図の本体だけを取り閲覧の仕掛けは捨てる・意味の属性は残す／決定 (5) 版の固定（rules 行 R-15）・通信する命令は使わない／決定 (6) 凍結 anchor = 型付き記述 1 本と図の本体の写し／決定 (7) 往復の記録は台帳）。便 29（f2-648.44）の後に直列で置く。設計ノートの面へ図を埋めるのは便 31（別の行）。
- 裁定: 持ち主 2026-09-19「承認する」（A-3.1・図の道具 archify 2.16.0 と実行環境 Node.js 18 以上を build 時の依存として足す・f2-648 notes 01:0x JST）。方針は ADR-4 の発効（2026-09-16「いずれも承認する」）と 2026-09-15「受け入れで良い」。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 ae が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。

## 1. 目的と中身

設計ノートの正本（`design-intent/design-note/<文書 id>.yaml`）の図の節（figures）の 1 枚を、型付き記述（spec）から図の道具で検査と描画に掛け、図の本体（SVG）だけを 1 file に書く命令 `folio figure` を足す。本便は命令と生成器と歯だけで、設計ノートの面への埋め込み・folio.css の図の class の様式・parts --check への図の class（便 31）は含まない。

planner の実測（2026-09-19・main 8eb7b0e の上に受け皿 f2-648.45 を置いた後）: 図の道具の写しは `vendor/archify/`（43 file・2.1 MB・deliver に要る file だけ = assets/template.html・bin/・renderers/・schemas/・scripts/check-render-output.mjs・package.json・LICENSE・vendor/README.md に出所と規則）。道具の版は package.json の version が 2.16.0 で、rules 行 R-15 の value にその版と commit c826e6c と要約値（sha256・写しの全 file を path の byte 順に連結した byte 列の値 = 8e974e5dc0808408fc125ed8845ce339776dd6c2732baee69d2cc6fc5fdba8d4）が在り status は 凍結（裁定 = 2026-09-19 の A-3.1 の承認）。道具の命令は `node vendor/archify/bin/archify.mjs deliver <型> <spec.json> <out.html> --quality showcase --json`（型は architecture・workflow・sequence・dataflow・lifecycle の 5 つ）で、通れば終了 0 で out.html を書き stdout に JSON 1 つ（ok true）、通らなければ終了 1 で out.html を書かず stdout に JSON 1 つ（ok false・error 欄に診断の文・diagnostics 配列）。出力 html には `<svg` が 1 つだけ在り、その `<svg` から `</svg>` までの byte 列が凍結 anchor `tests/fixtures/figure/anchor/body.svg`（16,804 byte・sha256 fcab70af…）と一致する（anchor/spec.json を 2 度 deliver して同じ byte・render と deliver の出力も同じ・planner の実測）。`tests/fixtures/figure/fails-showcase.json` は終了 1 で error 欄に「adjust labelDx/labelDy/labelSegment or set labelAt」を含む。実の正本 example.yaml の fig-1 と凍結 fixture full.yaml の fig-1 は受け皿で layout（grid・cols・gapX・gapY・cellW・cellH）を足して deliver を通した（往復 1 回・R-7 の内・台帳に記帳）。実行環境は node 18.19.1（ホスト）・CI は setup-node 20。正本の読み手 yaml.rs の Value は Null・Bool・Int（字面）・Float・Date（字面）・Str・Seq・Map（書かれた順）を持つ。face_note.rs の関数 is_doc_id は pub(crate)（便 29）。sha256.rs（便 7・手書き）が在る。main.rs は 389 行（余地 1,111）。CI の共通の検証（common-verify）は workspace の歯を全部回すので、本便の歯が node を呼ぶ分は setup-node で足りる。

(a) 命令の口（`crates/folio/src/main.rs`）。Command に Figure を足す: `--doc <文書 id>`（設計ノートの文書 id・必須）・`--id <図 id>`（figures の行の id・必須）・`--dir`（既定 design-intent）・`--out`（既定なし・相対なら --dir からの相対・絶対ならそのまま）・`--write` / `--check`（group mode・どちらか必須・face と同じ形）。doc は「設計ノートの図 1 枚を型付き記述から図の道具（rules 行 R-15）で検査と描画に掛け、図の本体（SVG）を出力先へ書く（--write）・検査する（--check）」。mod figure を足す（置き場は他の mod と同じ並び）。dispatch は face と同じ 3 値（Outcome の verdict を終了コードに・stdout / stderr は 1 行ずつ）。

(b) 生成器（`+crates/folio/src/figure.rs`・関数 derive の引数は dir・文書 id・図 id・戻り値は図の本体の String）。
- 読み: 文書 id は face_note.rs の関数 is_doc_id で形を見て違えば 2。face.rs の関数 load で `design-note/<文書 id>.yaml` を読み（無ければ 2）、meta.id が --doc と違えば 2。figures の一覧から id が --id と一致する行を 1 つ取る（無ければ 2「図「<id>」が figures に無い」・2 つ以上なら 2）。行の type は閉じた表（β）で道具の型に写す: archify-architecture → architecture・archify-workflow → workflow・archify-sequence → sequence・archify-dataflow → dataflow・archify-lifecycle → lifecycle。表に無い型（既存 3 型 pipeline-rail・context-band・state-strip を含む）は 2「図の型「<型>」は図の道具の型でない」。行の spec は Map でなければ 2。
- 型付き記述の書き出し（関数 to_json・Value → JSON の文字列・決定的）: Null → null・Bool → true / false・Int → 字面そのまま・Float → Rust の Display の字面・Date → 引用符付きの字面・Str → JSON の escape（二重引用符と逆斜線と U+0000〜U+001F を escape し、他は逐語・非 ASCII はそのまま）・Seq → `[` 要素を `,` で `]`・Map → `{` 鍵は Str のときだけ（それ以外は 2「型付き記述の鍵が文字列でない」）鍵を引用符で囲み「:」と値を続けた組を `,` で `}`。空白と改行を入れない（1 行）。順は正本に書かれた順。
- 道具の呼び出し: 道具の置き場は正本の置き場の親 dir の `vendor/archify/bin/archify.mjs`（便 28 の導出 file と同じ「親 dir」の規則・symlink は認めない・無ければ 2「図の道具が無い」）。一時 dir（std の temp_dir の下・名は folio-figure-<pid>-<nanos>）を作り spec.json と out.html の名を決め、`node <道具> deliver <型> <spec.json> <out.html> --quality showcase --json` を子の処理で起動する（仕上がりの段は showcase 固定・R-14・旗を持たない）。起動できなければ 2「図の道具を起動できない（node）: <理由>」。終了が 0 でなければ 2 で、stderr の 1 行は「図「<id>」は図の道具の検査を通らない: <stdout の JSON の error 欄の値の先頭 2 行>」（error 欄は JSON の鍵 error の値の文字列を取り、escape された改行で行に分け、先頭 2 行を空白 1 つで繋ぐ・2 行目が無ければ 1 行だけ・欄が無ければ stdout の先頭 200 字。fails-showcase の error 欄は 1 行目が「Architecture layout validation failed:」で 2 行目に「adjust labelDx/labelDy/labelSegment or set labelAt」を含む＝診断の中身が持ち主に見える・admin の実測 2026-09-19）。終了 0 なら out.html を読み、`<svg` の出現が 1 つでなければ 2「図の本体が 1 つでない（<数>）」、`<svg` から最初の `</svg>` の直後までを図の本体として返す（改行を足さない・byte はそのまま）。一時 dir は結果に関わらず消す（消せなくても結果は変えない）。道具の他の命令（preview・brands capture・--open）は呼ばない。往復（座標を直して撃ち直す）は folio の外＝planner が台帳に記帳する（ADR-4 決定 (7)）。
- 3 値（関数 run・face.rs の run と同じ形）: --write は derive → 出力先へ書く（0・stdout「folio figure: 書いた（<byte> byte）」）。derive が Err なら 2 で出力先に触れない（前の生成物が残る = AC12）。--check は出力先が無ければ 2「図が無い（未生成）」・byte 不一致なら 1「DRIFT」・一致なら 0「OK」。出力先の親 dir が無ければ 2。

(c) 名札の表（β）: 図の型の写像（(b) の 5 組）だけ。人が読む字は出さない（図の本体は道具の出力そのまま）。

(d) 凍結 fixture。既存の `tests/fixtures/figure/anchor/spec.json`・`anchor/body.svg`・`fails-showcase.json` をそのまま使う（変えない）。設計ノートの写しは `tests/fixtures/face/design-note/full.yaml`（受け皿で fig-1 に layout 済み）。新しい凍結は足さない。

(e) 歯（`+crates/folio/tests/figure.rs`・関数名はすべて figure を含める・`--test figure` の scope）。写しは tests/face_note.rs の fixture_copy と同じ形（正本 4 file + design-note/full.yaml を src/ へ・導出 file は要らない）に加えて、repo の `vendor/archify/` を一時 dir（src/ の親）の `vendor/archify/` へ写す（43 file・dir を再帰で写す）。anchor の記述を写しの設計ノートに埋める補助: anchor/spec.json の本文を 1 行ずつ 6 字下げて `spec:` の下に置いた figures の行（id fig-anchor・type archify-architecture・caption 任意）を full.yaml の figures に足す（JSON は YAML の流れの表として読める）。fails-showcase.json も同じ形で id fig-fails。この埋め方は実測済み: 実の design-intent の写しの example.yaml の figures に anchor/spec.json（109 行）を 6 字下げの流れの表として埋めた図を足すと folio check が合格（違反 0・まだ分からない 0）し、folio face の設計ノートの面に図の card が増える（17,203 byte・図 3 枚）。true・null・escape した引用符と改行・小数・整数を持つ 2 行に跨る流れの表も同じく読め、流れの表の「,」を 1 つ消すと check は「まだ分からない 1」で黙って読み飛ばさない（yaml.rs は yaml-rust2 の低水準 event から木を組み、Value は style で型付けする・admin の実測 2026-09-19・main 3aeb761）。
1. fig-anchor を --write = 0 ∧ 出力先の byte が anchor/body.svg と一致（凍結 anchor・P-10.1）∧ stdout が「folio figure: 書いた（16804 byte）」。
2. fig-anchor を 2 度 --write して同じ byte（決定的）。
3. 出力先に前の生成物（任意の byte 列）を置いた上で fig-fails を --write = 2 ∧ stderr に「図の道具の検査を通らない」と「labelDy」∧ 出力先の byte が前の生成物のまま（AC12 の形）。
4. --check の 3 値: 未生成 2「図が無い」・1 byte 変えて 1「DRIFT」・一致 0「OK」。
5. 図の型を pipeline-rail に = 2 ∧「図の道具の型でない」／ --id を無い id に = 2 ∧「figures に無い」／ --doc を大文字始まりに = 2 ／ spec の鍵の 1 つを数字（`1: x`）に = 2 ∧「文字列でない」／ spec を文字列に = 2。
6. 写しの親 dir の `vendor/archify/bin/archify.mjs` を消す = 2 ∧「図の道具が無い」／ 環境変数 PATH を空の dir だけにして撃つ = 2 ∧「起動できない」。
7. 実の正本: `--doc example --id fig-1 --write` = 0 ∧ 図の本体に `<svg` が 1 つ ∧ 行内の様式（style の属性）が 0 ∧ 色の直書き（fill の属性の値が # で始まる）が 0（意味の class だけ・ADR-4 決定 (3)）∧ class の語が全部 design-intent/preview/parts.json の figure_body_classes の値（node_kind 等の全一覧の和）に在る（歯の中で parts.json を読むだけ・write-set 外で変えない・figure_body_classes は base に在り鍵は source・type_ids・node_kind・edge_kind・arrowhead・text_role・sigil・semantic_attrs・note の 9 つで、class の一覧は node_kind・edge_kind・arrowhead・text_role・sigil の 5 つ（semantic_attrs は data 属性の一覧で class ではない）・planner の実測 2026-09-19）。
8. 版の固定: `vendor/archify/package.json` の version の字が rules.yaml の R-15 の value に在る ∧ vendor/archify/ の全 file を path の byte 順に連結した byte 列の sha256（歯は `sha256sum` を子の処理で呼ぶ・無ければ panic でなく「まだ分からない」の形で失敗の理由を出す）の 16 進が R-15 の value に在る（R-15 の value は歯の中で yaml で読む）。
9. figure.rs の unit（`--test figure` の外・cfg(test)・verify の外で CI の common-verify が回す・done には数えない＝振る舞いは歯 1〜6 が binary 経由で測る・便 23 の教訓）: to_json の 8 種（Null・Bool・Int・Float・Date・Str の escape（引用符・逆斜線・改行・制御文字・非 ASCII 逐語）・Seq・Map の順）と鍵が Str でない Map の Err、`<svg` の抽出（0 個・2 個は Err・1 個は byte そのまま）、error 欄の先頭 2 行の取り出し（escape された引用符を跨ぐ・escape された改行で行に分けて 2 行を空白 1 つで繋ぐ・1 行しか無ければその 1 行・欄が無いとき先頭 200 字）。

(f) 便 29 までの形との接続: 新規は `crates/folio/src/figure.rs`（見積 350 行）と `crates/folio/tests/figure.rs`（見積 400 行）。`crates/folio/src/main.rs` は Command の追加と dispatch（+30 行）。`crates/folio/tests/note.rs` は本文も期待も変えない（器の受付が新しい filter 語の歯の置き場を base に在る歯の file で解くため write-set に載せる・便 27 の tests/site.rs と同じ「載せるが変えない」の形・admin の実測 2026-09-19）。`face.rs`・`face_note.rs`・`note.rs`・`parts.rs`・`parts.json`・`folio.css`・`sha256.rs`・`vendor/`・`tests/fixtures/figure/`・`.github/workflows/`・`.vessel.toml` は触らない。外部 crate は増えない（Node.js と道具の写しは受け皿で足した A-3.1 の依存であり、本便は crate を足さない）。正規表現は使わない。size は M = 新規 2 file の見積が中で、既存 file の増分は main.rs +30 だけ。

## 2. 範囲

- 入れる: 命令 folio figure・型付き記述の JSON 書き出し・道具の子の処理・図の本体の抽出・3 値・歯（anchor・AC12・3 値・型と id と鍵の 2・道具と node の不在・実の正本・版の固定）と verify の外の unit。
- 入れない: 設計ノートの面への図の埋め込みと図の様式（便 31）・folio build への図の追加（便 31・面に埋まるので file は増えない）・座標の往復の自動化（AI が直す・往復の記録は planner が台帳に）・道具の版上げ・図の型の追加・既存 3 型の図の道具化。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| json | 型付き記述の書き出し | Value → JSON（決定的） |
| tool | 道具の呼び出し | node deliver showcase・一時 dir |
| body | 図の本体の抽出 | `<svg` … `</svg>` 1 つ |
| cmd | 命令の口 | folio figure の 3 値 |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 29 の歯は期待不変で全部回る・CI は setup-node 20 で node を持つ）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。実行環境 Node.js 18 以上と図の道具の写し `vendor/archify/`（rules 行 R-15・A-3.1 承認 2026-09-19）は受け皿で足した。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ae"
title = "設計ノートの図 1 枚を型付き記述から図の道具で描く folio figure（検査と描画は showcase 固定・図の本体だけ・通らない図は前の生成物を残す）"
req = ["FR15"]
section = "1"
write-set = ["+crates/folio/src/figure.rs", "+crates/folio/tests/figure.rs", "crates/folio/src/main.rs", "crates/folio/tests/note.rs"]
verify = ["cargo nextest run -p folio --test figure figure", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "figure の歯（anchor の byte 一致・決定的・AC12 の前の生成物・check の 3 値・型と id と鍵と spec の 2・道具と node の不在・実の正本の fig-1・版の固定 R-15）が緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
