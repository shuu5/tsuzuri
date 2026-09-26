# 設計: 便 154 — 面と生成物に焼かれた名 folio2 を、置き場の憲法の meta.id から導く名に替える（群 A の 3 本目・台帳 f2-648.191）

- 要件: FR7（全面を配信先へ生成する）・FR4（面の単一の生成器）・FR1（支度表）・FR24（始まりの凍結の 3 file）・FR20（天井の印）。規範文・確かめ方・受入基準は変えない。どの規範文も、面や生成物に出す名の字を定めていない。
- 条: P-5.1（名の導き方は型付きの定数と関数で持つ）/ P-6.3（名を 2 か所に持たない＝既に置き場の名である憲法の meta.id から導く）/ P-10.1・P-10.2（凍結 fixture の測り直しは生成器を通さない置き換えの閉じた一覧）/ N-3.1（旗も例外の口も足さない）。
- 出所: 台帳 **f2-648.191**（外の置き場の実測 `.local/share/folio2/handoff-2026-09-24/folio2-v3-entry-notes.md` §3 の 1〜3）。持ち主の直命 2026-09-26「folio2 の道具としての完成を目指す」の群 A。
- **席の判断が要る 1 点: 凍結 fixture の測り直し（P-10）。** 歯の置き場の憲法の名は fixture-constitution なので、本便の後の面と印の名は fixture になる。凍結 fixture 8 本と歯の中の字 2 か所が変わる（§1 (e) の 1）。どの形で名を導いても folio2 以外の名の置き場の字は変わるので、避ける形は無い。
- 置き場: 審査の材料は行 `fa` が指す §1 だけ。write-set は 22 本（src 11・歯の file 3〔新規 1〕・凍結 fixture 8）。新しい dir は無い。
- 門: 対象外。作業ツリー planner-d154 の一番上で base の binary に write-set 22 本を渡すと **0（通す・設計文書の正本を書き換えない便）**。
- 前の便: **base = main d6d84fa（便 152・ADR-27・便 153 の着地の後）。数は base の写しの実測（参考値・行 D-13）**で、受付の時点の main が違えば数え直す。
- 改訂 b（独立の検証の非 blocking の提案 1〜5 を採る）: 名の無い置き場の印と 5 種の面の歯 2 本・狭い幅の名札の例 k8s・置き換えの一覧を字のまま・運ばない字に owner の値・名の形の確かめの注。

## 1. 設計

### (a) いま起きていること（base d6d84fa の実測・参考値）

1. **焼かれた箇所は 10 file の 19 か所**（`git grep` の全数）。面の題・表紙・見出し・名札が face.rs と face_*.rs 5 本に 13、生成物の頭の注が freeze.rs 2（凍結 anchor・索引）・ids.rs・stamp.rs・sheet.rs 1 ずつ、支度表の id が sheet.rs の SHEET_ID = folio2-intake-sheet。
2. **外の置き場の答え。** 一時 dir で git の init → `folio init --dir design-intent` → 憲法と入口の meta.id を kumo-constitution・kumo-index に替える → 様式 3 本を repo の design-intent/preview/ から写す → commit、の後に撃った。右の欄は本便を当てた写しの binary。

| 出力 | base | 本便の後 |
| --- | --- | --- |
| build --write の面 4 枚（入口・憲法・要件書・ADR-1） | 題・表紙・名札が folio2（1 枚に folio2 が 3〜4 回・狭い幅の名札 f2 が 1 回） | 題「kumo — 要件書（v0.1）」の形・名札 kumo / k・folio2 は 0 回 |
| intake --write の支度表 | 頭の注「# folio2 支度表」・id folio2-intake-sheet | 「# kumo 支度表」・kumo-intake-sheet・folio2 は 0 回 |
| 骨格のまま（憲法の meta.id は 未記入） | 上と同じ folio2 | 題「要件書（v0.1）」の形・名札なし・「# 支度表」・intake-sheet |

   build は床が凍結の基準の不在で まだ分からない なので 2 のまま（FR5）。外の置き場は列の根の表に行が無く凍結が走らない（FR23）ので、凍結の 3 file の名は単体の歯で数える。天井の束の faces/ は面の写しで、面と一緒に直る。
3. **置き場の名は既に在る。** 憲法の meta.id は、`crates/folio/src/adr.rs` の関数 place_name が「置き場の憲法の名」として読み、列の根の表（FR23）を引く鍵である。folio2 の正本の meta.id は 6 本とも「folio2-<種類>」、歯の置き場は fixture-constitution、骨格は 未記入、tsuzuri は tsuzuri-constitution。
4. **base の歯。** workspace の nextest 959 / 959・clippy 0 警告・床 4 本 rc 0・`folio build --write` の出力 34 file。`f154_` と行 id `fa` は 0 件。

### (b) 直す先

1. **名の導き方（adr.rs・place_name の隣）。** 型付きの定数 NAME_SUFFIX = -constitution と関数 4 つを置く。
   1. display_name: 憲法の meta.id が `<名>-constitution` で、<名> が契約表の行の id と同じ形（ASCII の小文字で始まり、小文字・数字・ハイフンだけ）なら <名>、それ以外は無し。形の確かめは note.rs の関数 is_lower_id（床の定数 ROW_ID_PATTERN）の書き写しで、そこを指す注を置く（片方を変えるなら両方を変える・note.rs は書き換えない）。
   2. name_of: 置き場の dir から place_name を読み display_name に通す（読めなければ無し）。
   3. named: 名が有れば「<名><区切り><残り>」、無ければ残りだけ。
   4. short_name: 狭い幅の名札＝頭の 1 字と末尾の数字の並び（folio2 は f2・tsuzuri は t・中の数字は拾わない＝k8s は k）。
2. **面（face*.rs）。** face.rs の関数 head_dated は題を（名・題）の組で受け、題の頭に「<名> — 」を付け、名札は名が有るときだけ出す。5 面の head と cover は、derive で name_of を 1 回読んで受け、表紙の「<名> — 」・入口の見出し「<名> — 」・憲法の見出し「<名> の憲法 — 」を named で組む。
3. **生成物。** 凍結 anchor と索引の頭の注（freeze.rs・名は凍結の状態が持つ meta.id から）、id の一覧の頭の注（ids.rs・Current に名の欄を足す）、天井の印の頭の注（stamp.rs）、支度表の頭の注と id（sheet.rs・id は named で「<名>-intake-sheet」、名が無ければ intake-sheet）。
4. **変えないもの。** folio2 自身の出力（1 byte も）・digest と要約値の組み方（頭の注は YAML の注で木の外）・床と門・命令の旗と help・init の骨格・様式と部品目録・(i) の 2 の folio2 の字。

### (c) 歯

1. **単体の歯 3 本（既存か新しい tests 区間）。**
   1. f154_the_place_name_comes_from_the_constitution_id（adr.rs）: 4 つの名を導き、形の違う 9 つと無しからは導かない。short_name の 5 例（f2・t・s12・a2・k8s の k）と named の 2 例。
   2. f154_the_ids_header_names_the_place（ids.rs）: id の一覧の頭の注が、名 kumo なら「# kumo 要件・判断…」、無ければ「# 要件・判断…」で始まる。
   3. f154_the_anchor_and_index_headers_name_the_place（freeze.rs・新しい tests 区間）: meta.id が kumo-constitution なら凍結 anchor と索引の頭の注が「# kumo 憲法 v1.0 の凍結 anchor（ADR-2）。」「# kumo 凍結 anchor の索引（」で始まり、未記入なら名なしで始まる。

   **base では関数と欄が無く組み立てが落ちる（7 件）＝RED。**
2. **binary 経由の歯 5 本（新しい file `crates/folio/tests/place_name.rs` に 4 本・tests/stamp.rs に 1 本）。** 外の置き場は (a) の 2 の手順。期待の字は歯の中の手書き（生成器の関数を呼ばない）。
   1. f154_a_renamed_place_shows_its_own_name_and_never_folio2: kumo の置き場で build --write が 2、面 4 枚すべてに題「<title>kumo — 」・名札 kumo / k・表紙「kumo — 」、入口と憲法の見出し、支度表の頭の注「# kumo 支度表（」と id kumo-intake-sheet。面 4 枚と支度表に字 folio2・f2-・>f2< が 0 回。
   2. f154_the_skeleton_shows_no_name: 骨格のまま、面 4 枚の題と表紙が名なしの字・名札なし・憲法の見出し「<h1>憲法 — 」・支度表の「# 支度表（」と id intake-sheet。同じく 0 回。
   3. f154_folio2_keeps_its_own_name: repo の design-intent の build が 0 で面 4 枚の題「folio2 — 」と名札 folio2 / f2、その写しの支度表が folio2 と folio2-intake-sheet、床の凍結の土台の写しの始まりの凍結の 3 file の頭の注が「# folio2 」で始まる（tests/freeze_root.rs の歯 4 と同じ作り方）。
   4. f154_every_face_of_an_unnamed_place_shows_no_name（改訂 b）: 面の凍結 fixture（tests/fixtures/face）の写しの憲法の meta.id を 未記入 にして build（2）、入口・憲法・要件書・判断の記録 ADR-2・設計ノート full の 5 種の面の題が名で始まらず、名札が無く、字 >folio2< と「folio2 — 」が無い。
   5. f154_an_unnamed_place_stamps_without_a_name（tests/stamp.rs・改訂 b）: 印の歯の束の写しの憲法の meta.id を 未記入 にして --stamp（0）、印の頭の注が「# 天井の印 — 生成物（」で始まる。

   **base では 1・2・4・5 が落ちる＝RED**（3 は base でも緑の退行の歯）。

### (d) 採らなかった形

1. **置き場の dir 名か版管理の根の dir 名から導く。** folio2 自身の出力が作業ツリーの名（`.worktrees/…` や一時 dir の写し）で変わり、同じ正本から同じ面が出なくなる（P-6.1・天井の束の要約値も変わる）。
2. **正本に名の欄を足す。** 欄の決まりの生成区間を変える＝design-intent/ を書く便で、今は門で止まる（行 D-16）。上の (b) で筋が通る。
3. **入口の meta.id から導く・両方の一致を床で数える。** 置き場の名は既に憲法の meta.id（列の根の表の鍵）なので、そこ 1 か所に揃える。一致の床は検査を足す便で範囲外。
4. **名が導けないときの既定を道具の名 folio か dir 名にする。** 道具の名は置き場の名でなく、dir 名は 1 と同じ理由。名を出さない。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **落ちる既存の歯は 18 本で、どれも凍結 fixture の置き場の名が fixture であることによる（起草役の実測）。** 本便は置き換えの閉じた一覧で測り直す（script fix-anchors-154.py・生成器を通さない・P-10.2）。
   1. 面の凍結 fixture 7 本（tests/fixtures/face/ の expected.html・expected-srs.html・expected-index.html・expected-index-sheet.html・expected-adr.html・expected-note.html・expected-site-adr-2.html）に、次の 6 組だけを当てる（字のまま・左 → 右・1 本につき 5〜6 か所）。
      - `<title>folio2 — ` → `<title>fixture — `（7 本）
      - `>folio2</span><span class=` → `>fixture</span><span class=`（名札の長い名・7 本）
      - `>f2</span></span>` → `>f</span></span>`（名札の狭い幅の名・7 本）
      - `<span>folio2 — ` → `<span>fixture — `（表紙・7 本）
      - `<h1>folio2 — ` → `<h1>fixture — `（入口の見出し・expected-index.html と expected-index-sheet.html）
      - `<h1>folio2 の憲法 — ` → `<h1>fixture の憲法 — `（憲法の見出し・expected.html）
   2. 印の凍結 fixture tests/fixtures/ceiling/findings/stamp-expected.yaml に `# folio2 天井の印 — ` → `# fixture 天井の印 — `（1 か所）。tests/stamp.rs の歯が数える byte 数に `(34, 1_342)` → `(34, 1_343)`（1 か所）。
   3. tests/face_srs.rs の歯の中の字に `<title>folio2 — 要件書（v0.4）</title>` → `<title>fixture — 要件書（v0.4）</title>`（1 か所）。
   合計 34 か所（script fix-anchors-154.py の表と同じ字）。名札の組を 1 つの字として数えた改訂 a の 27 か所と、替わる byte は同じ。
   4. 落ちる歯（base に測り直しだけを当てた数と同じ 18 本）: face・face_index 2・face_adr 2・face_note 2・face_srs 4・face_srs_body・face_srs_figure・site・badge・stamp 3。
   5. **測り直しが名だけの差である確かめ（反実仮想）。** 本便の実装だけを当て、fixture を base のまま、fixture の置き場 2 つの憲法の meta.id を folio2-constitution に替えて 18 本の file を撃つと、落ちるのは 3 本（face・site・badge）だけで、差は憲法の面の「機械のための面」が meta.id の字をそのまま出す欄だけ（名の導き方の外）。
2. **全体。** 本便を当てた写しで workspace の nextest **967 / 967**（base 959 + 8）・clippy 0 警告・床 4 本 rc 0。folio2 自身の置き場で `folio build --write` の 34 file・始まりの凍結の 3 file・支度表が base と diff -r で一致。rustfmt --check の差の数は 14 file とも base と同じ。
3. **突然変異（写しの src を 1 通りずつ変え、単体の f154_・place_name・face・face_index・face_adr・face_note・face_srs_body・stamp・sheet・freeze_root を撃つ）。** M1〜M12（起草）と V3・V4・V7（独立の検証で生き残った 3 つ・改訂 b）の 15 通りとも 1 本以上が落ちる（括弧は落ちた本数）。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 名を入口の meta.id（-index）から導く | 単体 2・place_name 2・面と印と支度表の凍結 fixture・freeze_root（19） |
| M2 導けないとき folio2 を既定にする | 単体 2・place_name 2・stamp の名なし（5） |
| M3 名の形の確かめを外す | 単体（1） |
| M4 狭い幅の名札を名そのものにする | 単体・面の凍結 fixture・place_name 2（11） |
| M5 名札を出さない | 面の凍結 fixture・place_name 2（10） |
| M6 入口の表紙が名を使わない | 入口の凍結 fixture 2・place_name（3） |
| M7 凍結 anchor と索引の頭の注に folio2 を焼き戻す | 単体 freeze（1） |
| M8 id の一覧の頭の注から名を落とす | 単体 ids・freeze_root の歯 4・place_name の folio2（3） |
| M9 天井の印に folio2 を焼き戻す | stamp（4） |
| M10 支度表の id に folio2 を焼き戻す | place_name 2（2） |
| M11 支度表の頭の注から名を落とす | place_name 2・支度表の凍結 anchor 3（5） |
| M12 憲法の見出しから名を落とす | 憲法の凍結 fixture・place_name（2） |
| V3 天井の印で、名が無ければ folio2 | stamp の名なし（1） |
| V4 設計ノートの面で、名が無ければ folio2 | place_name の 5 種の面（1） |
| V7 狭い幅の名札を頭の 1 字 + 全部の数字に | 単体の k8s（1） |

   M3・M7・V7 は単体の歯だけが落とす（形の違う名・folio2 以外の名の凍結・中に数字を持つ名は、今の歯の置き場の出力に現れない）。id の一覧の名を check_ids の側で folio2 に倒す変異は、名の無い置き場では凍結が必ず断られるので届かない（等価の変異・検証役の V9）。

### (f) 大きさ・verify と done の対応

1. **write-set 22 本。** src 11 本（adr.rs・face.rs・face_adr.rs・face_constitution.rs・face_index.rs・face_note.rs・face_srs.rs・freeze.rs・ids.rs・sheet.rs・stamp.rs）・歯の file 3 本（`+crates/folio/tests/place_name.rs`・tests/face_srs.rs・tests/stamp.rs）・凍結 fixture 8 本（(e) の 1）。
2. **余地（CapHeadroom）。** 各行 ceil(字数 / 120)・空行は 1。python と awk の 2 実装で一致。

| file（crates/folio/src/） | base（参考値） | 余地 | 模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| adr.rs | 958 | 542 | 1034 | 466 |
| face.rs | 1113 | 387 | 1119 | 381 |
| face_adr.rs | 1080 | 420 | 1099 | 401 |
| face_constitution.rs | 1166 | 334 | 1178 | 322 |
| face_index.rs | 794 | 706 | 798 | 702 |
| face_note.rs | 1012 | 488 | 1031 | 469 |
| face_srs.rs | 1058 | 442 | 1061 | 439 |
| freeze.rs | 458 | 1042 | 508 | 992 |
| ids.rs | 343 | 1157 | 365 | 1135 |
| sheet.rs | 571 | 929 | 580 | 920 |
| stamp.rs | 353 | 1147 | 354 | 1146 |

3. **size は M。** src 11 本で +221（単体の歯を含む）。最小の余地は face_constitution.rs の 334（M の 300 以上）。
4. **verify は 5 行**で、done の 5 つの塊と 1 対 1。
   1. `cargo nextest run -p folio --bin folio f154_` = (c) の 1（3 本）。
   2. `cargo nextest run -p folio --test place_name f154_` = (c) の 2 の 1〜4（4 本）。
   3. `cargo nextest run -p folio --test stamp` = 印の歯の全部（(c) の 2 の 5 と測り直しの byte 数を含む）。
   4. `cargo nextest run -p folio --test face_srs` = 要件書の面の歯の全部（歯の中の字を測り直した file）。
   5. `cargo clippy --workspace --all-targets -- -D warnings`。

   base では 1 が 0 件で終了コード 4、2 が歯の file が無く終了コード 101、3・4 は緑（13・25 本）、5 は 0 警告。本便の後は 3・4・14・25 本。面の凍結 fixture を読むほかの歯（(e) の 1 の 4）は共通の検証（workspace の nextest）が撃つ。

### (g) 門・受付・並行の便

1. 門は対象外で 0（冒頭）。受付の先撃ち（precheck）で契約に起因する断りは 0（起草役の実測）。
2. **便 153（行 ez）は base に着地済み**（write-set は重ならない）。153 の歯は名の字を持たず、本便の外の置き場の歯は様式 3 本を置き場に写すので、焼いた様式の有無に依らず同じ答え。

### (h) 数え直す手順（行 D-13）

記録は `.local/share/folio2/handoff-2026-09-27/d154-draft.md`、script は同じ dir の d154-scripts（repo の外）。base-main.sh（受付の時点の main の clone）→ suite-154.sh（build-sim-154.sh が実装・測り直し・歯を当てて組み、nextest・clippy・余地・rustfmt・folio2 自身の出力の diff -r・外の置き場・RED・反実仮想を撃つ）→ mut-154.py → `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-154.md#fa`。

### (i) 着地の後に残るもの・撤退条件

1. **利用者の置き場（tsuzuri）。** 次に組んだ面・印・支度表の名が tsuzuri になる（生成物・P-6.2）。凍結 anchor の頭の注は digest の外で床は注を数えないので、既に在る anchor は書き換えず落ちもしない。
2. **運ばない folio2 の字（次の節目の材料・design-intent/ の file か道具を指す字）。** 様式 folio-ui.js の記憶の鍵 folio2.fs（配信先へ写る・画面に出ない）。欄の決まりの生成区間へ写る床の定数の注（道具としての folio2 を指す）と、その注の台帳の id（f2-648.2・.100・.131・.132・f2-648 notes）。設計ノートの欄の決まりの床の定数の値 `owner: folio2`（row_id の欄・骨格にも写る）。憲法の値域の値 folio2-ruling。凍結の断りの「行を足すのは folio2 の便」（道具の repo を指し、どの置き場でも正しい）。
3. **撤退条件。** (1) 本便の後に (e) の 1 の 18 本の外の既存の歯が 1 本でも落ちたら、歯も fixture も直さずに止めて席へ返す。(2) 着地の後の main で folio2 自身の床 4 本の結果か `folio build` の出力が着地の直前と 1 byte でも違ったら、止めて席へ返す。(3) 測り直しの差分が (e) の 1 の置き換えの一覧の外の字を 1 字でも含んだら、止めて席へ返す。

## 2. 範囲

- 入れる: §1 (b) の 1〜3、(c) の歯 8 本、(e) の 1 の測り直し。
- 入れない: 設計文書の正本・様式と部品目録・床と門・init の骨格・命令の旗と help・(i) の 2・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| name | 置き場の名 | adr.rs の NAME_SUFFIX と関数 4 つ |
| faces・outputs | 面と生成物の名 | face.rs と 5 面・freeze.rs・ids.rs・stamp.rs・sheet.rs |
| anchors | 測り直し | 凍結 fixture 8 本と歯の中の字 2 か所 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify。

## 5. 依存

- 外部 crate と新しい dir は無い。前提の着地は無い（便 152・153 は着地済み）。
- 着地の後に席が見ること: 本流の target/debug/folio を組み直す。台帳 f2-648.191 を閉じるか（(i) の 2 を次の節目へ控える）を決める。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fa"
title = "群 A の 3 本目（台帳 f2-648.191）: 面の題・表紙・見出し・名札と、凍結 anchor・索引・id の一覧・天井の印・支度表の頭の注と支度表の id に、名 folio2 が焼かれていて、外の置き場で生成しても folio2 と出る。crates/folio/src/adr.rs に、置き場の憲法の meta.id が <名>-constitution で <名> が行の id と同じ形のときその <名> を置き場の名とする型付きの定数と関数を置き、face.rs と 5 面・freeze.rs・ids.rs・stamp.rs・sheet.rs はその名を出し、導けなければ名を出さない（支度表の id は <名>-intake-sheet か intake-sheet）。folio2 自身の出力は 1 byte も変えない。歯の置き場の名が fixture になるので、凍結 fixture 8 本と歯の中の字 2 か所を置き換えの閉じた一覧で測り直す（P-10）。歯は単体の f154_ 3 本と binary 経由の f154_ 5 本（tests/place_name.rs 4・tests/stamp.rs 1）。門の対象外。base = main d6d84fa"
req = ["FR7", "FR4", "FR1", "FR24", "FR20"]
section = "1"
write-set = ["crates/folio/src/adr.rs", "crates/folio/src/face.rs", "crates/folio/src/face_adr.rs", "crates/folio/src/face_constitution.rs", "crates/folio/src/face_index.rs", "crates/folio/src/face_note.rs", "crates/folio/src/face_srs.rs", "crates/folio/src/freeze.rs", "crates/folio/src/ids.rs", "crates/folio/src/sheet.rs", "crates/folio/src/stamp.rs", "+crates/folio/tests/place_name.rs", "crates/folio/tests/face_srs.rs", "crates/folio/tests/stamp.rs", "tests/fixtures/ceiling/findings/stamp-expected.yaml", "tests/fixtures/face/expected.html", "tests/fixtures/face/expected-srs.html", "tests/fixtures/face/expected-index.html", "tests/fixtures/face/expected-index-sheet.html", "tests/fixtures/face/expected-adr.html", "tests/fixtures/face/expected-note.html", "tests/fixtures/face/expected-site-adr-2.html"]
verify = ["cargo nextest run -p folio --bin folio f154_", "cargo nextest run -p folio --test place_name f154_", "cargo nextest run -p folio --test stamp", "cargo nextest run -p folio --test face_srs", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "単体の f154_ 3 本（adr.rs の名の導き方と狭い幅の名札と名を付けた字・ids.rs の id の一覧の頭の注・freeze.rs の凍結 anchor と索引の頭の注が、名が有ればその名で始まり無ければ名なしで始まる）が緑、tests/place_name.rs の f154_ 4 本（憲法と入口の meta.id を kumo に替えた外の置き場の面 4 枚と支度表が kumo の名を出して folio2・f2-・>f2< を 1 つも持たず、骨格のままなら名を出さず支度表の id が intake-sheet で、面の凍結 fixture の写しの meta.id を 未記入 にした置き場の 5 種の面に名札も folio2 の名も無く、folio2 自身の面と支度表と始まりの凍結の 3 file は folio2 の名のまま）が緑、tests/stamp.rs の歯の全部（名の無い置き場の印の頭の注に名が無い f154_・測り直した印の凍結 fixture と byte 数）が緑、tests/face_srs.rs の歯の全部が緑、clippy が 0 警告で、workspace の nextest が全部緑（凍結 fixture を読む face・face_index・face_adr・face_note・face_srs_body・face_srs_figure・site・badge の歯を含む）で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main と file 数も byte も変わらない"
<!-- contracts:end -->
