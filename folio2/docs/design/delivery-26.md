# 設計: 便 26 — 入口の棚から判断の記録の各ページへ辿れるようにし、`folio build` の出力に判断の記録の面を足す（ADR-7 決定 (1)・(5)）

- 要件: FR16（一覧は入口の面の棚から出し、棚から各ページへ辿れる）/ FR7（組み立てて見せる: build の出力に判断の記録の面を含める）/ NFR2（部品目録に無い class は 0）
- 条: P-2.1・P-2.2（人が読むページは 1 つの生成器から・手書きのページを入口の導線に置かない）/ P-6.3（数と一覧は生成のたびに正本から数える・入口の正本には書かない）/ P-4.2（判定できないものは「まだ分からない」）/ P-10.1（独立した凍結 anchor）
- 判断の記録: ADR-7（決定 (1) 一覧は入口の棚の行から出し棚の行から各ページへ辿れる・帰結「入口の正本（index.yaml）は改訂しない・棚の行の無いときの文言は記録が 0 本のときだけ出る」）。ADR-5 決定 (2)（入口の案内文の正本・数は生成のたびに数える）。便 25（f2-648.40）の後に直列で置く。
- 裁定: 持ち主 2026-09-18「承認する」（ADR-7 発効・f2-648.39 notes）。walk 承認（AC5 の形・生成した判断の記録の面を持ち主が開いて読む）は本便の着地後に planner が tailnet の内側で取る（本便の契約の外）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 aa が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`・縮む面は `-`）。

## 1. 目的と中身

便 25 で判断の記録 1 本につき 1 枚の面（`adr-<数>.html`・`folio face --face adr --id ADR-<数>`）が生成できるようになった。本便は (1) 入口の面の棚の「判断の記録」の行を、記録が 1 本以上あるときは「読める」側の形にし、各記録のページへのリンクを出す、(2) `folio build` の出力を 3 面 + 様式 2 本 + 判断の記録の面（記録の数だけ）にする、(3) 入口の歯を `crates/folio/tests/face.rs` から `crates/folio/tests/face_index.rs` へ移して（face.rs は縮む面）本便の歯を足す、の 3 つ。入口の正本 `design-intent/index.yaml` は改訂しない（ADR-7 帰結）。

planner の実測（2026-09-18・main 446f35c）: 入口の面の棚の adr の行は face_index.rs の SHELF_DOCS で face が None（面なし）・棚の card は `shelf-adr` の div の中に class `is-absent` の article・sc-row の state は「○ <数> 本」・文は「ページはまだ無い」・sc-none「—（まだページはありません）」。状態の行（status-line）の「まだ無い」の v に「判断の記録（<数> 本・ページはまだ無い）」。棚の見出しの「いま読めるのは <数> 面」は関数 readable_faces（= 1 + face のある文書の数 = 3・unit の歯が 3 と固定）。shelf-grid の `--shelf-n` は readable の数 + 1 = 3（棚の置き場は css で固定・adr は grid の外の div）。site.rs の OUTPUTS は const の 5 本（unit の歯 site_outputs_are_five_files_in_this_order が 5 と固定）・build は「全部か無しか」で書き check は byte 一致。歯 face.rs は正規化 1,597 行（入口の歯は 861 行目以降の 737 行・関数名は face_index_ で始まる 27 本 + 補助）・site.rs の歯は fixture の adr/ADR-1.yaml（2 行・id と status だけ）を写して 5 本を凍結 fixture と突き合わせる。便 25 の fixture `tests/fixtures/face/adr/ADR-2.yaml`（欄の揃った最小の記録・proposed）と `tests/fixtures/face/expected-adr.html`（その面の凍結）が base に在る。

(a) 入口の面（`crates/folio/src/face_index.rs`）。棚の adr の行の読みを変える: `adr/` の下の `ADR-<数>.yaml` を数えるだけの関数 count_adr を、各 file を face.rs の関数 load で読み id・title・status・date を取る読みに広げる（id の数の昇順・数字列でない id や同じ数が 2 本は 2・便 11 の render.rs の並べ方と同じ・status は便 25 の名札の表と同じ 3 値で表に無ければ 2）。記録が 1 本以上のとき adr の card は「読める」側: article の class は `shelf-adr`（is-absent を外す・div の `shelf-adr` はそのまま）・sc-row 1 行目は state ok「● <数> 本」と要約「<最初の id>〜<最後の id>（発効 <数>・提案中 <数>・廃止 <数>・0 の種類は出さない）」・sc-row 2 行目は「更新 <最新の date>」と、各記録へのリンクを id の順に「・」で区切って並べる（a 要素・class xref・href `adr-<数>.html`・字は id）+ sc-open「開く →」と sc-hit（href は最も新しい番号の記録の面）。記録が 0 本のときは今の形（is-absent・「○ 0 本」・absent の文「まだ戻せない判断は無い」・sc-none）。関数 pages_yet と「ページはまだ無い」の文言は落とす（記録が在ればページも在る）。
- 棚の見出し「いま読めるのは <数> 面」の数 = 1 + face のある文書 2 + 記録の数、続く型の列挙は記録が 1 本以上なら「憲法・要件書・判断の記録」。shelf-head の「これから増える文書 <数>（<型>）」は記録が 1 本以上なら 設計ノートだけ。minimap の判断の記録は記録が 1 本以上なら state ok。`--shelf-n` は 3 のまま（adr は数えない・棚の置き場は css で固定）。status-line の「揃っている」の v は「憲法・要件書・判断の記録 <数> 本が読める（付録の…）」（記録が 1 本以上）、「まだ無い」の v から判断の記録を外す（0 本なら今のまま）。関数 readable_faces は記録の数を引数に取る（unit の歯は 0 本で 3・7 本で 10 に）。他の節（cover・読む順番・相談窓口・支度表の節）は変えない。
- 名札の表: 状態の名札は便 25 と同じ（proposed 提案中・accepted 発効・retired 廃止）。

(b) 組み立て（`crates/folio/src/site.rs`）。OUTPUTS の 5 本は据え置き、build_all が 5 本の後に判断の記録の面を id の数の昇順に足す（名は `adr-<数>.html`・中身は便 25 の判断の記録の面の生成器（行 z の新規 file）の関数 derive）。組の名の型を静的な文字列から所有する文字列に変える。1 本でも導出できなければ全体を 2（今と同じ「全部か無しか」）。書く・検査する・stdout の「<数> file」は動的な数（実の正本では 3 + 2 + 7 = 12）。unit の歯 site_outputs_are_five_files_in_this_order は据え置き（OUTPUTS は 5 のまま）・site_derive_takes_only_the_three_face_names は据え置き（derive の 3 面 + adr は別の口）。`crates/folio/src/main.rs` の Build の doc「3 面と様式 2 本」「5 本」を「3 面 + 判断の記録の面 + 様式 2 本」に直す（旗は変えない）。serve は触らない（配信先の file を名で出すだけ）。

(c) 歯の置き場の移動（`crates/folio/tests/face.rs` → `+crates/folio/tests/face_index.rs`）。face.rs の 861 行目以降（入口の歯 27 本と補助の関数・便 16・20〜22）をそのまま新しい file へ移し、face.rs は 1〜860 行（憲法・要件書の歯）だけにする（縮む面・本文の変更は移動と、移した先で要る補助の関数の写しだけ。補助は移した歯が使う 1〜860 行の関数で、実測 20 = repo_root・fixture・design_intent・temp_dir・fixture_copy・folio_face・code・stdout・stderr・edit・esc の 11 と real_face・load_yaml_at・load_yaml・seq・text・components・assert_same_bytes・real_srs・between の 9・admin の実測 2026-09-18）。移した歯の名は face_index_ で始まるまま（verify の filter 語）。27 本のうち 1 本（3 面をまとめて parts に掛ける歯・名が face_all_three_faces で始まる）だけ filter 語を含まないので、移すときに名の先頭を face_index_all_three_faces に改める（本文は変えない）。

(d) 凍結 fixture。入口の歯の写し（index_fixture_copy）と組み立ての歯の写し（site.rs の fixture_copy）は `adr/ADR-1.yaml`（2 行）でなく `adr/ADR-2.yaml`（便 25 の欄の揃った記録）を `adr/ADR-2.yaml` の名で写す（ADR-1.yaml は消さず触らない・便 3 の教訓）。それにより `tests/fixtures/face/expected-index.html` と `expected-index-sheet.html` を (a) の形で再凍結する（差分は棚の adr の card・見出しの数・shelf-head・minimap・status-line だけ＝他の節は byte 不変を歯で見る）。組み立ての凍結は 6 本（5 本 + `adr-2.html`）で、`adr-2.html` の期待は新しい凍結 `+tests/fixtures/face/expected-site-adr-2.html`（組み立ての写しから導出したものを凍結・P-10.1 の組み立て側の独立 anchor）。便 25 の `expected-adr.html` とは byte が違う（便 25 の歯の写しは ADR-1 の 2 行の stub も持つので basis の ADR-1 が `adr-1.html` へのリンクになるが、組み立ての写しは ADR-2 だけなので「ADR-1（まだ分からない）」になる・admin の実測 2026-09-18）＝同一は求めない。

(e) 歯。`crates/folio/tests/face_index.rs`（関数名はすべて face_index を含める・`--test face_index` の scope）: 移した 27 本（期待は (d) の再凍結に追従・census の本数は ADR-2 の 1 本）に次を足す。
1. 記録 1 本（ADR-2）の写しで棚の adr の card が is-absent を持たず「● 1 本」「ADR-2〜ADR-2（提案中 1）」と `adr-2.html` へのリンク（xref・sc-open・sc-hit の 3 つ）を持つ ∧「ページはまだ無い」が 0 ∧ 見出しの数が 4 ∧ status-line の「まだ無い」に判断の記録が無い。
2. 写しから `adr/ADR-2.yaml` を消す（adr/ は残す）= 0 ∧ card が is-absent ∧「○ 0 本」∧ absent の文 ∧ 見出しの数が 3。
3. 写しの記録の status を「draft」に = 2 ∧「状態」。
4. 写しに `adr/ADR-02.yaml`（同じ番号 2）を足す = 2 ∧「2 本以上」。
5. 実の正本で入口を生成し、判断の記録 7 本の id が xref のリンクとして id の順に在り、要約が「ADR-1〜ADR-7（発効 7）」∧ 見出しの数が 10 ∧ parts --check 合格（既存の census の歯に足す）。
6. (d) の再凍結で棚の節の外（cover・読む順番・相談窓口・foot）が便 22 の凍結と byte で同じ（前の凍結 fixture を git の base から読まず、歯の中で棚の節を切り出して残りを比べる = 前の版の期待を歯に埋め込まない・新旧の面から `<figure` 〜 `</figure>` と status-line の section を除いた残りを比較）。
`crates/folio/tests/site.rs`（関数名は site を含める・`--test site` の scope）: 凍結の歯を 6 本に（SITE_FILES に `adr-2.html`・期待に expected-site-adr-2.html）・stdout の「6 file」・実の正本の歯で build の出力が 12 file で `adr-1.html`〜`adr-7.html` が在り、`parts --check --page adr=<adr-7.html>` = 0 ∧ `face --face adr --id ADR-7 --check` = 0 を足す・check の 3 値の歯の drift の対象に `adr-2.html` を 1 つ足す。
`crates/folio/tests/face.rs`（縮む面）: 残る歯は本文も期待も変えない（write-set に `-` で載せる）。unit は置かない（face_index.rs と site.rs の既存 unit は上のとおり値を直すだけ）。

(f) 便 25 までの形との接続: 新規は `crates/folio/tests/face_index.rs`。`crates/folio/src/face_index.rs` は (a)（見積 +80 行・余地 460）・`crates/folio/src/site.rs` は (b)（+30 行）・`crates/folio/src/main.rs` は doc 3 行・`crates/folio/tests/site.rs` は (e)・`crates/folio/tests/face.rs` は 861 行目以降を外す（縮む）・fixture 2 本の再凍結。便 25 の判断の記録の面の生成器（行 z の新規 file）は読むだけ・`face.rs`・`face_constitution.rs`・`face_srs.rs`・`parts.rs`・`parts.json`・`serve.rs`・`index.yaml`・`folio.css`・`build.rs`・`.github/workflows/` は触らない。外部 crate は増えない。正規表現は使わない。size は S = 中身を変える既存の file 1 本あたりの増分は小さい（face_index.rs +80・site.rs +30・tests/site.rs +60・main.rs 3 行）。

## 2. 範囲

- 入れる: 棚の adr の行の「読める」形と各ページへの導線・build の出力に判断の記録の面・入口の歯の移動と再凍結・歯。
- 入れない: 入口の正本（index.yaml）の改訂・判断の記録の面そのものの変更（便 25）・判断の記録の一覧のページ（索引の面・ADR-7 の撤退条件で問う側）・serve の変更・walk 承認（着地後・契約の外）・暫定の読み物（readable.html）の退役（器の s2-07l.467 の後に A-1 で問う）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| shelf | 棚の行 | adr の card の 2 形（読める・0 本） |
| count | 数え | 記録の id・状態・日付を読み並べる |
| build | 組み立て | 5 本 + adr-<数>.html |
| move | 歯の移動 | face.rs → face_index.rs |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 25 の歯は期待不変で全部回る・face.rs は移した分だけ減る）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "aa"
title = "入口の棚から判断の記録の各ページへ辿れるようにし、folio build の出力に判断の記録の面を足す（入口の歯は face_index.rs へ移す）"
req = ["FR16", "FR7", "NFR2"]
section = "1"
write-set = ["crates/folio/src/face_index.rs", "crates/folio/src/site.rs", "crates/folio/src/main.rs", "+crates/folio/tests/face_index.rs", "-crates/folio/tests/face.rs", "crates/folio/tests/site.rs", "tests/fixtures/face/expected-index.html", "tests/fixtures/face/expected-index-sheet.html", "+tests/fixtures/face/expected-site-adr-2.html"]
verify = ["cargo nextest run -p folio --test face_index face_index", "cargo nextest run -p folio --test site site", "cargo nextest run -p folio --test face face", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "face_index の歯（移した 27 本 + 新規 6 本）が緑、site の歯（6 本の凍結・12 file の実測・drift）が緑、face の歯（残る憲法・要件書の歯）が期待不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
