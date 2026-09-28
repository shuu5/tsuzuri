# 設計: 便 184 — 設計ノートの面に計画の設計ノートの節の型 3 つを描く（判断の記録 ADR-31 の便 B の後半）

- 要件: FR9（設計ノートの面・節の型の閉じた一覧に無い型を持つ正本は生成しない）・FR27（計画の設計ノートの 3 つの節の型・要件書 第 1.53 版）。どちらも本流の要件書に在る id で、字は変えない。
- 条: P-2.1（人が読むページは 1 つの生成器から）・P-6.1（正本から逐語で生成）・P-2.4（部品目録の外の class を持たない）・P-10.1（期待の字は歯の側の手書き）。
- 出所: 判断の記録 ADR-31（持ち主の承認 2026-09-28 10:29 JST・対話面 R-8・逐語「全部承認する」・台帳 f2-648.267）の決定 (5)（便 B は面の描き方を含み、差分が審査の上限を超えるなら型・床・書く命令の便と面の便に割る）と (7)（3 つの節の型の面の描き方は M3 の外で FR9 の範囲）。前半は便 183（行 gd・docs/design/delivery-183.md）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `ge` が指す §1 だけ。write-set は 2 本（src 1・歯の file 1）。新しい file・縮む file・消す file・新しい dir は無い。
- 門: 対象外・0（設計文書の正本を書き換えない便・本流 c2e59c3 の組み立てで `folio ceiling --gate --dir design-intent --write-set crates/folio/src/face_note.rs crates/folio/tests/face_note.rs` は 0「通す（設計文書の正本を書き換えない便）」）。
- 前提: **base = 便 183（行 gd）が本流 c2e59c3 の上に着地した後の main**（便 183 の見本 366f96f〔改訂 a〕と同じ中身）。この契約の数は 1f8ea15 と 366f96f の写しの実測（参考値・規則の表の行 D-13）。便 183 が見本と違う中身で着地したら、その main で数え直す。
- 実装の見本: origin の枝 `impl/d184`（commit **3398d0b** = 見本 eaa6152〔親 1f8ea15〕に便 183 の改訂 a〔366f96f〕を取り込んだ merge）。`git diff 366f96f 3398d0b` が便の全体の差分（2 file・+99 −7・9,429 byte・`git diff 1f8ea15 eaa6152` と同じ）。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout 3398d0b -- <write-set の file>`）。write-set の外は変えない。
- 並行の便との重なり: 便 183 とは write-set が重ならないが、便 183 の節の型の定数が無いと歯の写しの床の前提が違うので、便 183 の着地の後に受け付ける。

## 1. 設計

### (a) いま起きていること（base 1f8ea15 の実測・参考値）

1. **面は 3 つの型を知らない。** 便 183 の後、床（`folio check`）は節の型 9 つを数えるが、設計ノートの面の生成器（`crates/folio/src/face_note.rs`）の名札の表 TYPES は 6 つのままで、行の索引・計画だけの行・判断の表の節を持つ設計ノートは面を導出しない（`folio face` と `folio build` は 2「まだ分からない: … 節の型「row-index」は閉じた一覧に無い」）。tsuzuri の写し（c43cae0）に移行を模した commit を当てると、床は合格 0/0 だが build は 2（起草の記録）。
2. **folio2 自身は 3 つの型の節を持たない**（名札の行を置かない・持ち主の裁定）＝本便の後も folio2 の `folio build` の出力は byte で同じ（36 file・sum ac76ec3ce9403fbc）。
3. **base の歯（参考値）。** workspace の nextest 1035 / 1035・clippy 0 警告・床 4 本 rc 0。`git grep -n f184_ -- crates` は 0 件・行 id `ge` は 0 件。

### (b) 直す先 — `crates/folio/src/face_note.rs`

1. **名札の表 TYPES を 9 つに**（行の索引・計画だけの行・判断の表・並びは欄の決まりの type_enum と同じ）と、型ごとの節の数 by_type を 9 つに。
2. **table_chapter。** 行の索引の rows が印だけ（YAML では null）なら 0 行で描く（ほかの型は今のまま rows を求める）。行の索引の行は所属の設計ノートの面へ（class xref の a 要素で、行き先は note-<文書 id>.html・字は文書 id）。計画だけの行は what を題に・size を pill に・depends を hint「依存」（「・」で繋ぐ）・files を hint「書く file」（`<code>` を `<br>` で繋ぐ）・ruling を hint「拠る」・note を hint「注」に。判断の表の行は text を題に・ruling を「裁定」の欄に。値は escape して逐語（α）。
3. **頭の注**に 1 項。
4. **変えないもの。** ほかの 6 つの型の描き方・部品の一覧（新しい class を足さない）・床・書く命令・folio2 自身の面。

### (c) 歯（f184_・base で 0 件）

1. **f184_the_three_plan_types_are_drawn_with_their_fields（`crates/folio/tests/face_note.rs`）。** 面の fixture の写しに、歯の中の最小の手書きの計画のノート plan.yaml（status example・行の索引 1 行〔doc full〕・計画だけの行 2 行〔size・depends・files 2 つ・ruling・note・what に `<` と `>`〕・判断の表 1 行）を置き、`folio face --face note --id plan --write` が 0 で、3 つの章の名札・索引の行の面への link・escape した what・size の pill・files の `<code>` の並び・拠る裁定・注・判断の字と裁定の字を持つ（期待の字は歯の側の手書き）。面は `folio parts --check` で 0。行の索引の rows が印だけの写しでも 0 で、索引の link を持たない。**base では 2（節の型「row-index」は閉じた一覧に無い）＝RED**（red-184.log）。

### (d) 採らなかった形

1. **便 183 と 1 便で運ぶ。** 和が 118,760 byte で審査の上限に近い（便 183 の (d) の 1）。
2. **行の索引の行を契約表の行の章へ結ぶ（`note-<文書 id>.html#s<章>-<行 id>`）。** 所属の面の章の番号は、その面の正本を読まないと決まらない（別の面の組み立ての中身を読む）。面の単位の link に留めた。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **既存の歯。** 本便の差分を 1f8ea15 に当てた写しで workspace の nextest **1036 / 1036**・clippy 0・床 4 本 rc 0・`folio build --write` 36 file（base と byte で同じ）。落ちる既存の歯は 0（直した既存の歯も 0）。便 183 の改訂 a を取り込んだ 3398d0b では clippy 0 と `--test face_note --test plan` の 41 / 41 を撃った（workspace は便 183 の改訂 a の 1036 に本便の 1 本を足した 1037 の見込み）。
2. **突然変異（mut-183.log の M12・M13）。** 判断の表の裁定の字を出さない・索引の行を面へ結ばない、はどちらも f184_ が落ちる（生き残り 0）。
3. **外の置き場（tsuzuri の写し 1eb50a7・参考値）。** 便 183 の (e) の 3 の手順（tz-run.sh）で移行した写しで、本便の binary の `folio build --write` は 0・37 file・surface-plan の面の §8 行の索引 126 行・§9 計画だけの行 62 行・§10 判断の表 30 行。

### (f) 大きさ・余地・verify と done の対応

1. **write-set の印。** どれも印なし。差分 9,429 byte（`git diff 1f8ea15 eaa6152 | wc -c`）。
2. **余地。** `crates/folio/src/face_note.rs` は base 1023（余地 477）→ 本便の後 1063（+40・余地 437）（python と awk の 2 実装で一致・cap-183.log）。
3. **size は S。** 増分 +40 は S の見積 100 の内、余地 477 は S の 100 を超える。
4. **verify は 3 行**で、done の 3 つの塊と 1 対 1。便の後の写しで 3 行とも rc 0。
   1. `cargo nextest run -p folio --test face_note f184_` = (c) の 1。
   2. `cargo nextest run -p folio --test face_note` = 設計ノートの面の既存の歯の全部（凍結 fixture との byte 一致を含む）。
   3. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。

### (g) 門と受付

1. **門。** 冒頭のとおり対象外・0。
2. **受付。** 便 183 の着地の後に受け付ける。precheck は本流 c2e59c3 の上の枝 docs/d183 で契約に起因する断り 0（起草の記録の precheck-184.log）。着地の後、席は tsuzuri へ「移行（名札の行と surface-plan の 3 つの節の型）は本便の binary から撃てる・手順は tz-run.sh と tz-migrate.py（今の HEAD で撃ち直す）」を返す。

### (h) 数え直す手順

起草の記録と script は便 183 と共通（`.local/share/folio2/handoff-2026-09-28/d183-draft.md`・d183-scripts）。RED は red-183.sh の後半、突然変異は mut-183.py の M12・M13、余地は cap-183.sh、外の置き場は tz-migrate.py。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 床・書く命令（便 183）・書き出し（便 C）・現在地（器）・外の置き場の書き直し・台帳への記帳・外部 crate・新しい dir。
2. **言えないこと。** 行の索引の行 id が文書をまたいで重なると、面の行の属性 id（`s<章>-<行 id>`）が重なる（床は重なりを数えない・tsuzuri の写しの 121 行には重なり 0）。
3. **撤退条件。** (1) 本便が要件書 FR9 / FR27 の字を変えないと書けないと分かったら、止めて席へ返す。(2) 受付の時点で本流の `face_note.rs` が 1f8ea15 と違えば止めて席へ返す。(3) 本便の後に既存の歯が落ちるか、folio2 自身の `folio build` の出力が 1 byte でも変われば、止めて席へ返す。

## 2. 範囲

- 入れる: `face_note.rs` の TYPES・by_type・table_chapter の 3 型・頭の注・歯 f184_ の 1 本。
- 入れない: 床・書く命令・部品目録・folio2 自身の面・外の置き場。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| types | 名札の表 | `face_note.rs` の TYPES（9 つ） |
| rows | 行の描き方 | table_chapter の行の索引・計画だけの行・判断の表 |
| teeth | 歯 | f184_ の 1 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない。新しい dir は無い。
- 前提の着地: 便 183（行 gd）。
- 本便の着地の後に席が見ること: 台帳の本便の件を閉じる。tsuzuri へ移行の手順（起草の記録の tz-run.sh と tz-migrate.py）と 1 行を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ge"
title = "設計ノートの面に計画の設計ノートの節の型 3 つを描く（判断の記録 ADR-31 の便 B の後半・決定 (5)(7)・要件書 FR9 と FR27・便 183 の後）: crates/folio/src/face_note.rs の節の型の名札の表 TYPES と型ごとの節の数を 9 つにし、table_chapter で行の索引の行を所属の設計ノートの面へ結び（rows が印だけの null なら 0 行）、計画だけの行の what・size・depends・files・ruling・note と、判断の表の行の text と ruling を escape して逐語で描く。部品の一覧は変えない。歯は f184_ の 1 本（crates/folio/tests/face_note.rs）。実装の見本は origin の枝 impl/d184 の commit 3398d0b（見本 eaa6152 に便 183 の改訂 a を取り込んだ merge）で、作業者は write-set の file をその中身にしてよく、write-set の外は変えない。base = 便 183（行 gd）が本流 c2e59c3 の上に着地した後の main"
req = ["FR9", "FR27"]
section = "1"
write-set = ["crates/folio/src/face_note.rs", "crates/folio/tests/face_note.rs"]
verify = ["cargo nextest run -p folio --test face_note f184_", "cargo nextest run -p folio --test face_note", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "crates/folio/tests/face_note.rs の f184_ の 1 本（計画のノートの 3 つの型の章と、索引の行の面への link・escape した what・size の pill・files の並び・拠る裁定・注・判断の字と裁定の字を持つ面が 0 で書け、folio parts --check が 0 で、索引の rows が印だけでも 0）が緑、tests/face_note.rs の歯の全部（凍結 fixture との byte 一致を含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数と中身は着地の直前の main と同じである"
<!-- contracts:end -->
