# 設計: 便 88 — 要件・判断・受入基準の id の消失と改番を数える口（id の一覧の凍結 anchor・NFR3 / FR5）

- 要件: NFR3（参照は必ずつながる・出所に P-7 を持つ）/ FR5（検査結果を必ず 3 値で返す・実行できなかった検査を合格にしない）
- 条: P-7.1（要件・条文・判断・受入基準の id を再利用・改番しない）/ P-7.2（廃止は状態で表し番号は空けたままにする）/ P-4.2（判定できないものは まだ分からない として表に出す）/ P-10.1（検査は独立した凍結 anchor を 1 本以上持つ）/ P-10.3（anchor が維持できない検査は まだ分からない に落とす）/ P-6.2（生成物を手で直さない）
- 出所: 台帳 f2-648.114（bug・P3）。憲法 P-7 の mechanism の注が「要件・判断・受入基準の id の消失と改番は今は数える口が無い＝まだ分からない（台帳へ起票して塞ぐ）」と自認している。baseline を持つのは憲法だけで（design-intent/anchors/ の constitution-v*.yaml・実装 anchor.rs）、要件書の FR / NFR / AC と判断の記録 ADR の id は、消しても付け替えても床が黙る。本便はその穴を塞ぐ。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 ck が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（新規は + を付けた 4 本）。
- 門: 本便は design-intent の下の正本（constitution.yaml の注）と anchors/ を書き換えるので天井の門の対象だが、持ち主の裁定 D-12（2026-09-21 23:45 JST「推奨で進めて」）により門の外で受ける。

## 1. 目的と中身

憲法の条 P-7.1 は 要件・条文・判断・受入基準 の 4 種の id を縛るが、機械の baseline を持つのは条文（憲法）だけである。要件書 design-intent/srs.yaml の FR / NFR / AC と、判断の記録 design-intent/adr/ADR-n.yaml の id は、行ごと消しても、同じ本文を別の番号に付け替えても、床の口が 1 つも無い。本便は憲法の凍結 anchor と同じ流儀で、ただし写し全部ではなく id とその規範文の要約値だけを持つ軽い anchor を足し、床に (i) 消失 (ii) 改番 (iii) baseline が無い の 3 つの判定を置く。

### (a) 実測（2026-09-22・main ec71e98）

- 器の受付の行数の式（空行を含む全行を数え、1 行の字数が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す）で測った値と余地（1500 − 値）。src = anchor.rs 1004・余地 496／check.rs 778・余地 **722**／freeze.rs 345・余地 **1155**／main.rs 535・余地 **965**／lineage.rs 863・余地 637／adr.rs 1243・余地 **257**。歯 = tests/check.rs 862・余地 638／tests/freeze.rs 309・余地 1191／tests/floor_cases.rs 760・余地 740／tests/schema.rs 1403・余地 **97**／tests/anchor.rs 123・余地 1377。
- 要件書の id の数と行の形。requirements は block の行（先頭が 2 字下げの - id: FR1 の形）で 20 本（FR1 から FR20）・nonfunctional は同じ形で 3 本（NFR1 から NFR3）・acceptance は flow の行（- {id: AC1, title: …, plain: …} の形）で 18 本（AC1 から AC18）。判断の記録は design-intent/adr/ の ADR-n.yaml が 12 本（ADR-1 から ADR-12。同じ置き場の schema.yaml は記録ではないので数えない＝adr::check_adr の records がすでにその形で外してある）。合わせて 53 本。
- 歯の土台 tests/fixtures/floor_base/design-intent（凍結した day-1 の写し・要件書は v1.8）の同じ数え = FR 19・NFR 3・AC 17・ADR 10 の合わせて 49 本。
- 歯の関数名の接頭辞。grep -rn 'fn f88_' crates/folio/tests は今 0 本。
- 版管理の照合（gitcheck.rs）は anchors/ の直下の yaml を file 名で見ており、constitution- の接頭辞で絞るのは履歴の中身の照合だけ（gitcheck.rs 316 行）。作業ツリーに在って追跡されていない anchor・追跡されているのに作業ツリーに無い anchor・履歴に在ったのに作業ツリーに無い anchor は、file 名を問わず違反になる。本便が anchors/ に足す file もこの守りの内側に入る（下の (g) の撤退条件はこの実測に従う）。
- 語彙の検査 R-9 の母集団は、憲法の条の title / plain / 規範文と前文、rules 行の what、要件書の requirements / nonfunctional / acceptance / constraints / goals の本文だけ（vocab.rs の population）。条の mechanism の注は母集団の外なので、本便の注の書き換えで語彙 design-intent/vocabulary.yaml を触る必要は無い（実測）。

### (b) id の一覧の凍結 anchor（新しい file）

置き場は憲法の anchor と同じ design-intent/anchors/（新しい dir は作らない）。file 名は ids- に要件書 meta.version をつないだ yaml（本便が作る最初の 1 本は ids-v1.24.yaml）。中身は folio が書く生成物で、手では書かない（P-6.2）。欄は次のとおり。

- kind = ids-anchor（憲法の anchor の constitution-anchor とは別の値。anchor.rs の読み手は anchors/ の直下の constitution- で始まる file だけを列に載せるので、両者は互いを読まない＝実測）
- digest_algo = 憲法の anchor と同じ床の定数（sha256-json-1）を adr::floor_val から読む
- version = 凍結した時点の要件書 meta.version（由来の記録。下の (c) のとおり、現行の版と一致していることは求めない）
- projection = 写しの取り方。節の一覧（requirements・nonfunctional・acceptance・adr）と、節ごとにどの欄を要約したかの対（requirements と nonfunctional は shall・無ければ title／acceptance は title／adr は title）
- ids = 行の一覧。1 行は id と 節 と 要約値 の 3 欄で、節と id の順に並べる
- digest = ほかの全欄を正規化した木の sha256（憲法の anchor と同じ anchor::digest_of をそのまま使う）

要約値の取り方は、その欄の字面の UTF-8 の byte 列の sha256 の 16 進とする（前後の空白は落とさず、末尾の改行も付けない）。正規化した木の要約値（digest_algo の方式）ではなく素の字面にするのは、人も OS の道具も同じ値を独立に出せるようにするため＝下の (e) の凍結 anchor がそれで立つ。

### (c) 床の側の判定（新しい module crates/folio/src/ids.rs）

anchors/ の直下の ids- で始まる yaml を名前順に全部読み、各 file の digest 欄を検算する。検算の合わない file は違反（手で直した anchor）として名指し、baseline には混ぜない。読めない file は「まだ分からない」（P-10.3）。読めた anchor の ids の**和**を baseline とする。和を取るのは、id は増えるだけ（P-7.2）なので古い baseline が偽の違反を出すことが無く、最新の 1 本だけを消して列を浅くする細工も効かないからである。索引（anchors/index.yaml の形の追記 file）は本便では作らない。憲法の索引は根の要約値を床の定数で留めた 1 本の列の順序を定めるものだが、id の一覧の anchor は列ではなく独立した baseline の束で、順序を持たない。消失と差し替えは (a) で実測した版管理の照合がすでに受け持つので、2 本目の追記 file を足しても検出は増えない。

現行の id は、要件書の 3 節と adr/ の記録から (b) と同じ取り方で組み、baseline と突き合わせて次を出す。

1. baseline に在って現行に無い id で、その要約値が現行のどの id にも無いもの = 種別 P-7 の違反（番号を消した）。
2. baseline に在って現行に無い id で、baseline がその id に記録した要約値が現行の別の id に在るもの = 種別 P-7.1 の違反（同じ本文を別の番号に付け替えた）。文言には消えた番号と現れた番号の両方を出す。
3. 1 件の消えた id が出す違反は 1 件だけとする（2 に当たれば 2 だけを出す）。1 つの誤りを 2 件に数えると、床の件数が直し方の手数と合わなくなる。
4. 現行にだけ在る id（足した id）は違反にしない（P-7.2 は増やすことを許す）。要約値が変わっただけの id も違反にしない。P-7 が縛るのは番号であって本文ではなく、本文の改訂は判断の記録と要件書の版が受け持つ。
5. ids- の anchor が 1 本も無いときは違反を出さず、「まだ分からない」を 1 件立てる（P-4.2・P-10.3）。文言は、要件・判断・受入基準の id の消失と改番は baseline が無いので測れないこと、folio check --freeze-ids で凍結できることを言う。下の (d) の旗が立っているときはこの 1 件を立てない（凍結の前提の検査に替わる＝憲法の anchor が --freeze-anchor で (i) を掛けないのと同じ形）。

呼ぶ場所は check.rs の check_dir で、判断の記録を読めた後（adr::check_adr が Some を返した後）・anchor::check_anchor の次。freeze.rs の after に id の側の結果を 1 つ足して渡す。

### (d) 凍結の口（folio check --freeze-ids）

main.rs の Check に旗 --freeze-ids を足し、既存の --emit-amends・--freeze-anchor のどちらとも同時に立てられないと宣言する（1 回の実行で旗は高々 1 つ・今の断りの形と同じ）。freeze.rs の Flag に値を 1 つ足し、after の場合分けに 1 本足す。書くのは、全検査が 0 違反で「まだ分からない」も無いときだけ（憲法の凍結と同じ前提）。同じ名前の file が既に在れば書かずに断る（同じ版は上書きしない・N-1.1）。書いた後は結果の 1 行に、書いた file と載せた id の本数と、版管理に commit することを出す。

版管理の照合は (a) の実測のとおり、作業ツリーに在って追跡されていない anchor を違反にする（免除は憲法の列の最新の 1 本だけ）。したがって --freeze-ids で書いた直後の素の床は 1 で終わり、commit してから 0 になる。これは仕様として文言に出す（凍結した anchor は commit する）。gitcheck.rs は触らない。

### (e) 歯（関数名は f88_ で始める・新しい file crates/folio/tests/ids.rs）

土台は tests/fixtures/floor_base/design-intent の写しを一時 dir に作り、器の導出 file contracts/schema.toml を写しの根に置き、git init と 1 commit を行う形（tests/freeze.rs の Work と同じ作り）。

1. f88_lost_id_is_a_violation — 写しの要件書から FR の行を 1 本まるごと消して素の床 → 終了コード 1・種別 P-7 の行に消した番号が出る。
2. f88_renumbered_id_is_a_violation — 写しの要件書で FR の番号だけを未使用の番号に替え、shall は 1 字も変えずに素の床 → 終了コード 1・種別 P-7.1 の行に消えた番号と現れた番号の両方が出る。
3. f88_added_id_is_green — 写しの要件書に欄のそろった FR を 1 本足すだけなら 終了コード 0（増やすのは違反でない）。
4. f88_without_an_anchor_the_result_is_unknown — 写しの anchors/ から ids- の file を消して素の床 → 終了コード 2・「まだ分からない」の行に id の消失と改番が測れないことが出る。
5. f88_freeze_writes_the_anchor_once — 4 と同じ写しで --freeze-ids → 終了コード 0・anchors/ に ids- の file が 1 本でき、その file を commit してから素の床 → 終了コード 0。続けてもう一度 --freeze-ids → 終了コード 1・断りの行に 上書きしない が出る。
6. f88_the_frozen_literal_pins_the_summary — 凍結 anchor（P-10.1）。実の design-intent/anchors/ids-v1.24.yaml を読み、ids が 53 本（FR 20・NFR 3・AC 18・ADR 12）で、FR1 の行の要約値が次の字面と 1 字も違わないことを見る: 01391bf27709cf47e6424f5b3f4e363a877c2d1fcc069f6da753b1fab650fa2c。この値は生成器からも検査側からも独立に、OS の道具 sha256sum に FR1 の shall の字面（150 byte・58 字）を末尾の改行なしで与えて出した（席の実測 2026-09-22）。同じ字面が floor_base の要件書 v1.8 の FR1 にも在るので、土台の写しの側でも同じ値が立つ。本便の前の main では ids-v1.24.yaml が存在しないので、この歯は赤い歯になる。

土台の写しに載せる ids- の anchor（tests/fixtures/floor_base/design-intent/anchors/ids-v1.8.yaml）は本便が足す。手では書かず、floor_base の写しを一時 dir に取り、器の導出 file を置き、git init と 1 commit をしてから --freeze-ids を回して出た file をそのまま fixture へ戻す（生成物・P-6.2）。この 1 本が無いと、土台を使う既存の歯（tests/freeze.rs の 5 本・tests/floor_cases.rs の 141 件）は (c) 5 の「まだ分からない」を拾って終了コードが 2 へ倒れる。載せれば、終了コード 0 を期待する 21 件は全部 増やすだけの変異（FR を 1 本足す・判断の記録を新しく作る）なので、期待は 1 件も動かない（実測で全件を読んだ）。

### (f) 大きさ

新しい src は crates/folio/src/ids.rs（新規・見込み 300 行ほど・余地 1500）。行が増える既存の src は 3 本で、crates/folio/src/check.rs（余地 722・+ 6 行ほど）／crates/folio/src/freeze.rs（余地 1155・+ 10 行ほど）／crates/folio/src/main.rs（余地 965・+ 12 行ほど）。size **M**（触る src の各 file の余地はどれも 300 以上）。外部 crate は増やさない（要約値は既存の sha256.rs・木の読み書きは既存の yaml.rs・床の定数は adr::floor_val）。様式・部品目録・面の生成器・CI の yml は触らない。

### (g) 本便が運ばないもの・撤退条件

- 床の定数の人が読める写し（P-5.6）。id の一覧の anchor の 置き場・綴り・要約の取り方のうち、既に床の定数に在るもの（anchors の dir・digest の方式・版の綴り）は adr::floor_val から読むので写しの外に出ないが、本便が新しく持つ定数（kind の値・file 名の頭・節と欄の対）は ids.rs の型付きの定数に留まり、設計文書の側へは導出しない。導出の口 folio schema が読む木は adr.rs の FLOOR（余地 257＝size M の見積 300 に足りない）か check.rs の SRS_FLOOR で、後者を触ると生成区間の凍結 anchor の行数・byte 数・要約値を持つ tests/schema.rs（余地 97＝size S の見積 100 にも足りない）の改訂が要る。どちらの道も本便には積めないので、写しの導出は後続の便に分け、先に tests/schema.rs の切り出しの便を 1 本置く（台帳へ起票する）。
- 要件書の版上げ。本便は要件書の行を 1 字も変えない（読むだけ）ので meta.version は動かさない。
- 憲法の条文。本便が触るのは条 P-7 の mechanism の注の末尾だけで、id・見出し・規範文・段・縛る相手は 1 字も変えない。注は凍結 anchor の写しの範囲（条の 5 欄）の外なので、憲法の anchor の差分も判断の記録も承認も要らない（A-2.2）。注は「今は数える口が無い」を、本便の口を指す字（baseline は憲法と id の一覧の 2 つ・消失は P-7・同じ要約値の付け替えは P-7.1・anchor が無ければ まだ分からない）に直す。
- goals・rules 行・語彙の見出し語の id。P-7.1 が名指すのは 要件・条文・判断・受入基準 の 4 種なので、本便の一覧もその 4 種に限る。
- 撤退条件: id の一覧の baseline が維持できなくなったとき（要件書の id の付け替えを伴う移行など）は、ids.rs の判定の呼び出しを check.rs から外し、結果を「まだ分からない」へ戻す（P-10.3）。便 1 本で戻せる。anchor の file は anchors/ に残したままにする＝(a) で実測したとおり版管理の照合が anchors/ の下の yaml の消失を file 名を問わず落とすので、file を退役の置き場へ動かす形の撤退は取れない。file を残して口を外す形が、この構成での可逆な戻し方である。

## 2. 範囲

- 入れる: 新しい module 1 本・旗 1 つ・凍結 anchor の形 1 つ・実の id の一覧の anchor 1 本・土台の写しの anchor 1 本・憲法の注の末尾の直し・新しい歯の file と f88_ の歯 6 本。
- 入れない: 床の定数の写しの導出・要件書の版上げ・憲法の条文・goals や rules 行の id・索引 file・gitcheck.rs・面の側の表示。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| module | 新しい module | crates/folio/src/ids.rs（写しの取り方・突き合わせ・凍結） |
| flag | 旗 | folio check --freeze-ids（main.rs の宣言と freeze.rs の場合分け） |
| anchor | 実の baseline | design-intent/anchors/ids-v1.24.yaml（生成物・id 53 本） |
| fixture | 土台の baseline | tests/fixtures/floor_base/design-intent/anchors/ids-v1.8.yaml（生成物・id 49 本） |
| note | 憲法の注 | 条 P-7 の mechanism の注の末尾（条文は不変） |
| teeth | 歯 | 新規 crates/folio/tests/ids.rs の f88_ 6 本 |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は .vessel.toml の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ck"
title = "要件書の FR / NFR / AC と判断の記録 ADR の id の消失（P-7）と同じ本文の付け替え（P-7.1）を床で落とす。id と規範文の要約値だけを持つ軽い凍結 anchor を design-intent/anchors/ids-<要件書の版>.yaml に足し、凍結の口 folio check --freeze-ids を置き、anchor が 1 本も無い写しでは まだ分からない に落とす（P-4.2）。憲法 P-7 の mechanism の注は条文を触らずに新しい口を指す字へ直す"
req = ["NFR3", "FR5"]
section = "1"
write-set = ["+crates/folio/src/ids.rs", "crates/folio/src/check.rs", "crates/folio/src/freeze.rs", "crates/folio/src/main.rs", "design-intent/constitution.yaml", "+design-intent/anchors/ids-v1.24.yaml", "+tests/fixtures/floor_base/design-intent/anchors/ids-v1.8.yaml", "tests/floor_cases.yaml", "+crates/folio/tests/ids.rs", "crates/folio/tests/check.rs", "crates/folio/tests/freeze.rs", "crates/folio/tests/floor_cases.rs"]
verify = ["cargo nextest run -p folio --test ids f88_", "cargo nextest run -p folio --test check --test freeze --test floor_cases", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "f88_ の歯 6 本（消失で 1・改番で 1・足すだけなら 0・anchor 無しで 2・凍結して commit すれば 0 かつ二度目は断り・実の anchor の id 53 本と FR1 の要約値の凍結 literal 一致）が全部緑、tests/check.rs と tests/freeze.rs と tests/floor_cases.rs の既存の歯が全部緑（実の design-intent で終了コード 0・土台の写しの 141 件の期待は 1 件も動かない）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
