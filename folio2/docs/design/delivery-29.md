# 設計: 便 29 — 入口の棚から設計ノートの各ページへ辿れるようにし、`folio build` の出力に設計ノートの面を足す（便 26 と同じ形・5 面目の導線）

- 要件: FR9（設計ノートを 1 つの型で生成する: 生成した面を入口の棚から辿れ、組み立てに含める）/ FR7（組み立てて見せる: build の出力に設計ノートの面を含める）/ NFR2（部品目録に無い class は 0）
- 条: P-2.1・P-2.2（人が読むページは 1 つの生成器から・手書きのページを入口の導線に置かない）/ P-6.3（数と一覧は生成のたびに正本から数える・入口の正本には書かない）/ P-4.1・P-4.2（読めなかった正本を「無い」と扱わない・判定できないものは「まだ分からない」）/ P-10.1（独立した凍結 anchor）
- 判断の記録: ADR-3 決定 (1)（設計ノートは 1 つの型・1 文書 = YAML 1 file）。ADR-5 決定 (2)（入口の案内文の正本・数は生成のたびに数える）。ADR-7 決定 (1) の形（一覧は入口の棚の行から出し、棚の行から各ページへ辿れる・入口の正本は改訂しない）を設計ノートにも同じ形で当てる。便 28（f2-648.43）の後に直列で置く。
- 裁定: 持ち主 2026-09-18「推奨でよい」（設計ノートの面は手書きのまま・ADR-5 / ADR-7 の撤退条件 ①）・「これでよい」（見本 v1 の方向・便 28）。walk 承認（AC5 の形・生成した設計ノートの面と入口の棚を持ち主が開いて読む）は本便の着地後に planner が tailnet の内側で取る（本便の契約の外）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 ad が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。

## 1. 目的と中身

便 28 で設計ノート 1 本につき 1 枚の面（`note-<文書 id>.html`・`folio face --face note --id <文書 id>`）が生成できるようになった。本便は (1) 入口の面の棚の「設計ノート」の行を、設計ノートが 1 本以上あるときは「読める」側の形にし、各設計ノートのページへのリンクを出す、(2) `folio build` の出力を 3 面 + 様式 2 本 + 判断の記録の面 + 設計ノートの面（設計ノートの数だけ）にする、(3) 歯と凍結 fixture を便 26 と同じ規律で足す、の 3 つ。入口の正本 `design-intent/index.yaml` は改訂しない（ADR-7 帰結と同じ）。

planner の実測（2026-09-19・main fee0b5b）: 設計ノートの正本は `design-intent/design-note/` の下に `example.yaml`（status example・generated 2026-09-17）の 1 本と、欄の決まり `schema.yaml`（設計ノートではない・床 note.rs は「dir の直下の `.yaml` から `schema.yaml` を除く」で数える）。入口の面の棚の design-note の行は face.rs の SHELF_DOCS で place `shelf-d`・face None（面なし）・card は grid の中の article（class `shelf-d is-absent`）・sc-row の state は「○ まだ無い」・文は index.yaml の absent「実装の直前に書く」・sc-none「—（まだページはありません）」。棚の見出しの「いま読めるのは <数> 面」は関数 readable_faces（= 1 + face のある文書 2 + 記録の数・unit の歯は 0 本で 3・7 本で 10）。shelf-head の「これから増える文書 <数>（<型>）」は関数 missing（face の無い文書のうち、記録が 1 本以上なら adr を除く）で、実の正本では 1（設計ノート）。status-line の「まだ無い」の v も missing から組む（実の正本では「設計ノート（実装の直前に書く）」）。関数 readable_types は 読める文書の型を並べ、記録が 1 本以上なら「判断の記録<tail>」を末尾に足す。foot の sources は const SOURCES の 7 本（`adr/` を含み `design-note/` を含まない）。site.rs の build_all は OUTPUTS 5 本 → 判断の記録の面（`face_index::records` の順）の順に組み、stdout は動的な数（実の正本では 12 file）。face_note.rs の関数 derive は同じ dir の憲法・rules・要件書と、契約表の節が在れば正本の置き場の親 dir の `contracts/schema.toml` を読む。状態の表 STATUS（draft 下書き・effective 発効・retired 廃止・example 見本）と id の形の関数 is_doc_id は face_note.rs の非公開。凍結 fixture `tests/fixtures/face/design-note/full.yaml`（status draft・generated 2026-09-18・契約表の節を持つ）と `expected-note.html`（便 28）が base に在る。入口の面を写しから導出する歯は 3 か所で全部（tests/face_index.rs の index_fixture_copy・tests/site.rs の fixture_copy・tests/serve.rs の built_site。entrance.rs は design-intent の写し全部・parts.rs と render.rs は実の正本・face_adr.rs / face.rs / face_note.rs は入口を導出しない・grep 「index」・folio_build・built_site で列挙）。組み立ての写し（site.rs の 6 file + adr/ADR-2.yaml）に `design-note/full.yaml` と親 dir の `contracts/schema.toml` を足して `folio face --face note --id full` を撃つと `expected-note.html` と byte 一致（planner の実測 2026-09-19・面の入力は設計ノートと憲法・rules・要件書・導出 file だけなので、写しの他の file には依らない）。正規化した行数（幅 120）: face_index.rs 1,169（余地 331）・tests/face_index.rs 1,146（余地 354）・site.rs 201・tests/site.rs 373・tests/serve.rs 342・main.rs 389・face_note.rs 997。

(a) 入口の面（`crates/folio/src/face_index.rs`）。設計ノートの読みを足す: 関数 notes（pub・`records` と同じ形）が `design-note/` の直下で名が `.yaml` で終わり `schema.yaml` でない file を名の昇順に読み（dir が無ければ 2「design-note/: 読めない」・記録の adr/ と同じ扱い・P-4.1）、各 file を face.rs の関数 load で読んで meta の id・title・status・generated を取る（id は file 名の stem と一致しなければ 2「欄 meta.id「<id>」が file 名「<stem>」と違う」・id の形は face_note.rs の関数 is_doc_id で見て違えば 2・title は非空・status は face_note.rs の表 STATUS に無ければ 2「設計ノートの状態」・generated は非空）。並びは id の字（byte）の昇順（file 名が一意なので同じ id は無い）。struct Note は id・状態の名札・generated（escape 済み）を持ち、関数 file は `note-<id>.html`（id は英小文字・数字・ハイフンだけなので escape は要らない）、関数 id は正本の id。face_note.rs の STATUS と is_doc_id は `pub(crate)` にして face_index.rs から使う（表を二重に持たない・P-6.3・face_note.rs の変更はこの 2 行だけ）。
- 棚の design-note の card: 設計ノートが 1 本以上のとき article の class は `shelf-d`（is-absent を外す・grid の中の置き場は変わらない）・sc-row 1 行目は state ok「● <数> 本」と要約「<状態の名札> <数>（STATUS の表の順・0 の種類は出さない・「・」で区切る）」（id は番号でないので範囲は出さない）・sc-row 2 行目は「更新 <最大の generated>」と、各設計ノートへのリンクを id の順に「・」で区切って並べる（a 要素・class xref・href `note-<id>.html`・字は id）+ sc-open「開く →」と sc-hit（href は generated が最大の 1 本・同じ日付が 2 本以上なら id の順で後の 1 本）。0 本のときは今の形（`shelf-d is-absent`・「○ まだ無い」・absent の文・sc-none）。判断の記録の card（adr_rows）は変えない。
- 棚の見出し「いま読めるのは <数> 面」の数 = 1 + face のある文書 2 + 記録の数 + 設計ノートの数（関数 readable_faces は 2 つの数を取る・unit の歯は (0, 0) で 3・(7, 1) で 11）。続く型の列挙（readable_types）は棚の文書の順（憲法・要件書・設計ノート・判断の記録）で、設計ノートと判断の記録はそれぞれ 1 本以上のときだけ並べ、tail（status-line では「 <数> 本」・見出しでは空）を型の直後に付ける（今の adr だけの末尾追加を、棚の順の走査に改める）。関数 missing は 設計ノートが 1 本以上なら design-note も除く。missing が 0 のとき shelf-head の sub は「これから増える文書 なし ／ 付録 <数>（憲法の中）」（括弧を出さない）、status-line の「まだ無い」の v は「なし」（行は残す・部品の数は変えない）。minimap の設計ノートは 1 本以上なら state ok。`--shelf-n` は 3 のまま（readable() は face のある文書だけを数える・棚の置き場は css で固定）。status-line の「揃っている」の v は「憲法・要件書・設計ノート <数> 本・判断の記録 <数> 本が読める（付録の…）」（各 1 本以上のとき）。foot の sources（const SOURCES）は `adr/` の次に `design-note/` を足して 8 本。他の節（cover・読む順番・相談窓口・支度表の節）は変えない。
- 名札の表: 状態の名札は face_note.rs の STATUS をそのまま（β・表に無い値は 2）。

(b) 組み立て（`crates/folio/src/site.rs`）。build_all は 5 本 → 判断の記録の面 → 設計ノートの面（face_index.rs の関数 notes の順・名は `note-<id>.html`・中身は face_note.rs の関数 derive）の順に足す。1 本でも導出できなければ全体を 2（今と同じ「全部か無しか」・設計ノートに契約表の節が在って導出 file が無いときもここで 2 になる）。書く・検査する・stdout の「<数> file」は動的な数（実の正本では 3 + 2 + 7 + 1 = 13）。unit の歯 2 本（OUTPUTS は 5・derive の 3 面）は据え置き。file の doc の 1 行目と `crates/folio/src/main.rs` の Build の doc 3 か所「3 面 + 判断の記録の面 + 様式 2 本」を「3 面 + 判断の記録の面 + 設計ノートの面 + 様式 2 本」に直す（旗は変えない）。serve は触らない（配信先の file を名で出すだけ）。

(c) 写しの揃え。入口を導出する写し 3 か所（(実測) tests/face_index.rs の index_fixture_copy・tests/site.rs の fixture_copy・tests/serve.rs の built_site）に `design-note/full.yaml` を `design-note/full.yaml` の名で足す。組み立てを回す 2 か所（site.rs の fixture_copy・serve.rs の built_site）は、設計ノートの面の導出が契約表の節を読むので、repo の `contracts/schema.toml` を一時 dir（写しの src/ の親）の `contracts/schema.toml` へも写す（tests/face_note.rs の fixture_copy と同じ形）。入口の写し（index_fixture_copy）は面の meta しか読まないので導出 file は写さない。

(d) 凍結 fixture。入口の写しに設計ノートが 1 本入るので `tests/fixtures/face/expected-index.html` と `expected-index-sheet.html` を (a) の形で再凍結する（差分は棚の design-note の card・見出しの数と型の列挙・shelf-head・minimap・status-line・foot の sources だけ＝他の節は byte 不変を歯で見る）。組み立ての凍結は 7 本（6 本 + `note-full.html`）で、`note-full.html` の期待は便 28 の `tests/fixtures/face/expected-note.html` をそのまま使う（上の実測で byte 一致・面の入力が写しの他の file に依らないため独立 anchor は保たれる・新しい凍結は足さない）。`expected-site-adr-2.html` と `expected-adr.html` は変えない（判断の記録の面は設計ノートを読まない）。

(e) 歯。`crates/folio/tests/face_index.rs`（関数名はすべて face_index を含める・`--test face_index` の scope）: 既存の歯は写しの変化に追従して期待の数だけ直す（記録 1 本の歯の「いま読めるのは」4 → 5・記録 0 本の歯の 3 → 4・census の歯の `3 + adr` → `3 + adr + notes`・「まだ無い」の検査は設計ノートの有無で分ける）。次を足す。
1. 写し（記録 1 本 + 設計ノート 1 本）で棚の design-note の card が is-absent を持たず「● 1 本」「下書き 1」と `note-full.html` へのリンク（xref・sc-open・sc-hit の 3 つ）を持つ ∧ card に sc-none が無い ∧ 見出しの数が 5 ∧ 型の列挙が「憲法・要件書・設計ノート・判断の記録」∧ shelf-head が「これから増える文書 なし ／」∧ status-line の「まだ無い」の v が「なし」∧「揃っている」の v が「憲法・要件書・設計ノート 1 本・判断の記録 1 本が読める」で始まる ∧ minimap の設計ノートが state ok ∧ foot の sources に `design-note/` が在る。
2. 写しから `design-note/full.yaml` を消す（design-note/ は残す）= 0 ∧ card が `shelf-d is-absent` ∧「○ まだ無い」∧ absent の文「実装の直前に書く」∧ sc-none ∧ 見出しの数が 4 ∧ shelf-head が「これから増える文書 1（設計ノート）」∧「まだ無い」の v に「設計ノート」。
3. 写しの設計ノートの status を「final」に = 2 ∧「状態」。
4. 写しの設計ノートの meta.id を「fill」に（file 名は full.yaml のまま）= 2 ∧「file 名」。
5. 写しの `design-note/` を dir ごと消す = 2 ∧「design-note/」。
6. 実の正本で入口を生成し、設計ノート（`design-intent/design-note/` の `.yaml` から `schema.yaml` を除いた id の字の昇順・実測 1 = example）の各 id が xref のリンク（`note-<id>.html`）として順に在り、要約が「見本 1」∧ 見出しの数が 3 + 記録の数 + 設計ノートの数 ∧ parts --check 合格（既存の census の歯に足す）。
7. 写し（記録 1 本 + 設計ノート 1 本）と、そこから設計ノートを消した写しの 2 面で、棚の図と status-line の外（cover・読む順番・相談窓口・foot）が byte で同じ（既存の関数 outside_the_shelf を使う・foot の sources は両方に `design-note/` が在るので同じ）。
`crates/folio/tests/site.rs`（関数名は site を含める・`--test site` の scope）: 凍結の歯を 7 本に（SITE_FILES に `note-full.html`・期待に expected-note.html）・stdout の「7 file」・check の 3 値の歯の drift の対象に `note-full.html` を 1 つ足す・実の正本の歯で build の出力が 3 + 2 + 記録の数 + 設計ノートの数（実測 13）file で `note-example.html` が在り、`parts --check` に note の面（`note-<最後の id>.html`）を --page で渡して = 0 ∧ 各設計ノートの `face --face note --id <id> --check` = 0 を足す（設計ノートの id は歯の中で `design-intent/design-note/` を読んで数える・schema.yaml を除く）。全部か無しかの歯に「写しの設計ノートの status を『final』にすると 2 で配信先に 1 byte も書かない」を 1 本足す。
`crates/folio/tests/serve.rs`（関数名は serve を含める・`--test serve` の scope）: built_site の写しに (c) の 2 点を足すだけ・期待は不変（serve の歯は数を数えない）。
unit は face_index.rs の readable_faces の値を直すだけ（site.rs の unit 2 本は不変）。`crates/folio/tests/face_note.rs`（`--test face_note` の scope）: 本文も期待も変えない（verify の scope の置き場として write-set に載せる・face_note.rs の pub(crate) の 2 行で歯が不変に緑なことを見る）。

(f) 便 28 までの形との接続: 新規 file は無い。`crates/folio/src/face_index.rs` は (a)（見積 +100 行・余地 331）・`crates/folio/src/face_note.rs` は pub(crate) の 2 行・`crates/folio/src/site.rs` は (b)（+15 行）・`crates/folio/src/main.rs` は doc 3 行・`crates/folio/tests/face_index.rs` は (e)（+150 行・余地 354）・`crates/folio/tests/site.rs` は (e)（+70 行）・`crates/folio/tests/serve.rs` は写しの 2 点（+10 行）・`crates/folio/tests/face_note.rs` は不変（verify の scope）・fixture 2 本の再凍結。`face.rs`（SHELF_DOCS の design-note の face は None のまま）・`face_adr.rs`・`note.rs`・`parts.rs`・`parts.json`・凍結目録・`index.yaml`・`folio.css`・`design-note/example.yaml`・`full.yaml`・`expected-note.html`・`.github/workflows/` は触らない。外部 crate は増えない。正規表現は使わない。size は S = 中身を変える既存の file 1 本あたりの増分は小さい（face_index.rs +100・tests/face_index.rs +150・tests/site.rs +70）。

## 2. 範囲

- 入れる: 棚の design-note の行の「読める」形と各ページへの導線・build の出力に設計ノートの面・写し 3 か所の揃え・fixture 2 本の再凍結・歯。
- 入れない: 入口の正本（index.yaml）の改訂・設計ノートの面そのものの変更（便 28）・設計ノートの一覧のページ（索引の面）・図の生成（FR15・別の便）・serve の変更・walk 承認（着地後・契約の外）・導出物（FR11・便 30・器の s2-07l.473 の後）・暫定の読み物（readable.html）の退役（s2-07l.467 の後に A-1 で問う）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| shelf | 棚の行 | design-note の card の 2 形（読める・0 本） |
| count | 数え | 設計ノートの id・状態・日付を読み並べる |
| build | 組み立て | 5 本 + adr-<数>.html + note-<id>.html |
| copies | 写し | 入口を導出する 3 か所に設計ノート 1 本と導出 file |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 28 の歯は期待不変で全部回る）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ad"
title = "入口の棚から設計ノートの各ページへ辿れるようにし、folio build の出力に設計ノートの面を足す（便 26 と同じ形）"
req = ["FR9", "FR7", "NFR2"]
section = "1"
write-set = ["crates/folio/src/face_index.rs", "crates/folio/src/face_note.rs", "crates/folio/src/site.rs", "crates/folio/src/main.rs", "crates/folio/tests/face_index.rs", "crates/folio/tests/site.rs", "crates/folio/tests/serve.rs", "crates/folio/tests/face_note.rs", "tests/fixtures/face/expected-index.html", "tests/fixtures/face/expected-index-sheet.html"]
verify = ["cargo nextest run -p folio --test face_index face_index", "cargo nextest run -p folio --test site site", "cargo nextest run -p folio --test serve serve", "cargo nextest run -p folio --test face_note face_note", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "face_index の歯（既存の数の追従 + 新規 7 本）が緑、site の歯（7 本の凍結・実の正本で 13 file と note の parts / face --check・drift・全部か無しか 1 本）が緑、serve の歯が期待不変で緑、face_note の歯が本文不変（pub(crate) の 2 行だけ）で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
