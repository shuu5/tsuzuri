# 設計: 便 152 — 骨格のままで注入と面の生成器が落ちないようにし、憲法の雛形を置き場に依らない形にする（群 A の 1 本目・台帳 f2-648.189 と .196）

- 要件: FR22（骨格の命令）を主に、骨格の上で回す口の FR6（注入）・FR4（3 面）・FR16（判断の記録の面）。規範文・確かめ方・受入基準 AC19 の書く file 11 本の閉じた一覧は変えない。
- 条: P-4.1（面の生成器が断る欄を骨格に黙って欠かせない）/ P-5.1・P-6.3（行 R-2 と R-16 の字は folio2 の正本から型付きで読む）/ P-10.1（歯の期待は歯の側の手書き）/ N-3.1（旗を足さない）。
- 出所: 外の置き場の実測（`.local/share/folio2/handoff-2026-09-24/folio2-v3-entry-notes.md` §1〜§3）と台帳 **f2-648.189**・**.196**。持ち主の直命 2026-09-26「folio2 の道具としての完成を目指す」の群 A。ADR-21 の決定 (1)(2)(4) と帰結は、骨格のままで落ちる命令と本来の憲法を持たない利用者向けの雛形を v3 の入口の便（M3 の列の外）の論点とした。
- 置き場: 審査の材料は行 `ey` が指す §1 だけ。write-set は 2 本で、新しい file も dir も無い。
- 門: 対象外。作業ツリー planner-d152 の一番上で、base の binary・本便の binary・本流の target/debug/folio に write-set 2 本を渡すと、どれも **0（通す・設計文書の正本を書き換えない便）**。
- 前の便: **base = main 6093b8d（便 151 の着地の後）。数は base の写しの実測（参考値・行 D-13）**で、受付の時点の main が違えば数え直す。
- 割り方: 様式の file は **便 153 に割る**（§1 (i) の 4）。

## 1. 設計

### (a) いま起きていること（base 6093b8d の実測・参考値）

1. **骨格のままの命令の答え。** 一時 dir で git の init → `folio init --dir design-intent` → commit の後、手直しなしで撃った。右の欄は本便を当てた写しの binary。

| 命令 | base | 本便の後 |
| --- | --- | --- |
| init | 0 | 0（11 file のまま） |
| check | 2（違反 0・まだ分からない 2 = 凍結の基準の不在だけ） | 同じ |
| inject --write・--check（CLAUDE.md に印の対） | **2（rules の R-2 が読めない）** | 0・0 |
| schema --check | 0 | 0 |
| build --write | **2（srs.yaml.meta: 欄 counts が無い）** | 2（preview/folio.css が読めない・配信先に書かない） |
| face adr --id ADR-1 | **2（欄 requirements が無い）** | 0 |
| face constitution・srs・index | **2（欄 goals・rail・counts が無い）** | 0・0・0 |
| parts --check | 2（parts.json が読めない） | 同じ（範囲外） |
| graph --digest・--print・intake --print・hello | 0 | 0 |

2. **面の生成器が要る欄（本便の骨格から 1 つずつ抜いた実測）。** 床はどれを抜いても 2 のままで、面だけが落ちる。4 面とも要る = requirements・nonfunctional・acceptance。入口の面以外が要る = goals・constraints。要件書の面だけが要る = scope・scope_m1・actors（役が 道具 の行がちょうど 1 つ）・outputs・rail・not_frozen・glossary_pointer・meta.promise。入口と要件書の面が要る = meta.counts。verdicts・sources・figures は要らない。
3. **憲法の雛形（.196）。** 名は floor-declaration（床の受付の宣言）で、前文と条 P-1 の規範文が利用者の本来の憲法を指し、根拠の種別は scribe2-article。改訂の欄は字「未記入」で、憲法の面は「amendment: 表でない」で断る。
4. **注入。** `crates/folio/src/inject.rs` の r2_limit は行 R-2 を名で引く。骨格の規則の表は R-8 と R-16 だけなので、上限の検査を飛ばさず 2 を返す（P-4.1 の向きで正しい）。
5. **base の歯。** workspace の nextest 954 / 954・clippy 0 警告・床 4 本 rc 0・`folio build --write` の出力 33 file。`--test init` は 18 本（f125_ 7 本と、歯の file が読み込む yaml・sha256 の単体 11 本）。`f152_` と行 id `ey` は 0 件。

### (b) 直す先 — `crates/folio/src/init.rs` の雛形の定数と関数 rules_rows

1. **要件書の雛形 SRS。** meta に counts（4 つとも 0）と promise（未記入）を足す。(a) の 2 の要る節を、空の一覧か「未記入」で正本の節の順に足す。actors は id tool・名 未記入・役 道具 の 1 行。verdicts・sources・figures は書かない。
2. **憲法の雛形（置き場に依らない形）。** 頭の注を「憲法 — 正本（folio init の雛形・名と条は利用者が決めて書き換える）」に、meta.id を 未記入 に替える。
   1. 前文の規範文は「段どうしが衝突したら『絶対にやらない ＞ 確認してから ＞ いつも守る』の順で解く。」。骨格が写す schema の節の tier_precedence と同じ順で、新しい規範ではない。根拠の欄は空。
   2. 条 P-1 の題は「承認の受け方と検査の値」、規範文 P-1.1 は「承認は規則の表の行 R-8 の対話面を通ったものだけを受け取り、散文の門は行 R-16 の値で数え、AI の手元へ写す生成区間は行 R-2 の値に収める。」。根拠の欄は空で、relations.rules は R-2・R-8・R-16。
   3. amendment を declaration（未記入）・steps（空）・effective_step（段 0・担当 持ち主・what 未記入・条は空）の表にする。
3. **行 R-2。** rules_rows は、行 R-16 と同じ読み方で folio2 の行 R-2 の what・value・kind を型付きで読む（値「8,000 byte 以下」・種別 deny）。行は R-2・R-8・R-16 の順で、条 P-1・状態 仮・裁定 未記入。what に出る claude.md を語彙の雛形の識別子に足す（無いと床の行 R-9 が違反 1）。
4. **骨格自身の番号。** OWN_IDS に R-2 を足す（括弧を落とす式が R-2 を folio2 固有と数えない）。頭の注の「注入の対象にしない」を「CLAUDE.md は書かない」に直す（init は今も書かない）。
5. **変えないもの。** 書く file の一覧 FILES と断りの一覧 REFUSE・命令の旗・入口と相談窓口と天井の写し方・ADR-1 の雛形・生成区間の導出・面の生成器・床・注入・folio2 自身の正本と面。

### (c) 歯（関数名 f152_・`crates/folio/tests/init.rs`・binary 経由）

1. **f152_the_skeleton_runs_every_command_without_hand_edits（群 A の通しの歯）。** 既存の口 Work の一時の根の直下の design-intent（v3 と同じ形）に init をして commit し、順に撃つ。check は 2 で「違反 0・まだ分からない 2」。根の一時の CLAUDE.md に印の対を置き、inject の --write と --check が 0 で区間に P-1.1 と R-2 が在る。schema --check は 0。build --write は 2 で、標準出力に「床 = まだ分からない（違反 0・まだ分からない 2）」、標準エラーはちょうど 1 行で「preview/folio.css: 読めない」、配信先の dir を作らない。face の adr（ADR-1）・constitution・srs・index が 0 で、書いた file が HTML の宣言で始まる。後続の便はこの期待を延ばす（便 153 は build の答えを 6 file を書いた 2 に替える）。**base では inject が 2＝RED。**
2. **f152_the_constitution_template_stands_on_its_own（.196）。** 骨格の憲法の生成区間と schema の節（値域に scribe2-article が在る）を除いた字に「本来の憲法」「scribe2」「床の受付の宣言」「floor-declaration」が無い。meta.id が 未記入、前文と各条の根拠の欄が空、前文の規範文が手書きの期待（(b) の 2 の 1 の文・段の順）と等しく、条の規範文に「憲法」が無い（外の憲法を指さない）、amendment が 3 欄の表で発効の段が 段 0・担当 持ち主。**base では「本来の憲法」が在る＝RED。**
3. **f152_no_folio2_words_in_the_skeleton。** 骨格の 11 本の数える範囲（生成区間と憲法の schema の節を除く＝f125_no_folio2_ids_in_the_skeleton と同じ範囲）に folio2 固有の字（folio2・台帳の id の頭 f2-・器の名 scribe2）が無く、どの語も folio2 自身の同じ範囲には在る（数えが効いている）。folio2 の要件 id はその f125 の歯が縛る（(e) の 3 の M16）。生成区間に焼かれた名は .191。**base では憲法の雛形の scribe2-article が在る＝RED。**
4. **置き換える既存の歯 2 本。** 骨格自身の番号の定数 OWN に R-2 を足す（f125_no_folio2_ids_in_the_skeleton）。f125_sources_match_folio2 の「条 P-1 は R-8 と R-16 を縛る」を R-2・R-8・R-16 に替え、骨格の行 R-2 の what・value・kind が folio2 の行 R-2 と同じことと、骨格の行の状態と裁定が 仮・未記入（folio2 の行の状態と裁定を写さない）ことを足す（base では落ちる）。

fixture は足さない。歯は既存の口（Work・git・folio・id_range・typed・seq・str_of）だけを使う。

### (d) 採らなかった形

1. **面の生成器の側で欄を任意にする。** 欄を読む口は 4 面の生成器の 6 file 以上に散り、face_srs.rs の余地は 442（参考値）。欠けを まだ分からない にしても今の答え 2 と同じで、build と face は通らない。init の 1 本で足すほうが小さい。床の欄の決まりと面の必須を 1 つの一覧に揃える仕事は台帳 f2-648.189 に残す。
2. **行 R-2 のために条を足す。** 骨格が新しい規範を書く（ADR-16 決定 (2)(オ) の懸念）。条 P-1 が縛る行を 3 つにするのが最小。
3. **scribe2 の形の雛形を残し、旗で選ばせる。** 本来の憲法を別に持つ利用者は今いない（scribe2 の移行は取りやめ・ADR-22 文脈 (エ)）うえ、命令の口が増える。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **落ちる既存の歯は (c) の 4 で置き換える 1 本だけ。** 本便を当てた写しで workspace の nextest **957 / 957**・clippy 0 警告・床 4 本 rc 0・`folio build --write` は 33 file で base と diff -r 一致・`--test init` 21 / 21。
2. **RED。** 歯だけを base に当てると、f152_ の 3 本と置き換えた f125_sources_match_folio2 が落ち、ほかの 17 本は緑。
3. **突然変異（写しの init.rs を 1 通りずつ変え、`--test init` を撃つ）。** 16 通りとも 1 本以上が落ちる。表の 通し・雛形・固有の字 は (c) の 1・2・3、床 は f125_init_writes_the_skeleton_and_the_floor_passes、出所 は f125_sources_match_folio2、id は f125_no_folio2_ids_in_the_skeleton。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 行 R-2 を書かない | 通し・床・出所 |
| M2 要件書の meta.counts を書かない | 通し |
| M3 改訂の欄を字「未記入」に戻す | 通し・雛形 |
| M4 前文の根拠を scribe2-article に戻す | 雛形・固有の字 |
| M5 行 R-2 の value を 上限なし に替える | 通し・出所 |
| M6 道具の役者の行を書かない | 通し |
| M7 条 P-1 が行 R-2 を縛らない | 通し・床・出所 |
| M8 語彙の雛形から claude.md を外す | 通し・床 |
| M9 憲法の名を floor-declaration に戻す | 雛形 |
| M10 前文の段の順を逆にする | 雛形 |
| M11 条 P-1 の規範文の頭に 利用者の上位の憲法の条に従い、 | 雛形 |
| M12 要件書の promise に folio2 の字 | 固有の字 |
| M13 道具の役者の名を folio2 | 固有の字 |
| M14 行 R-2 の状態を 凍結 | 出所 |
| M15 発効の段の番号を 9 | 雛形 |
| M16 要件書の promise に FR22 | 通し・床・id |

   生き残る 4 通りは出力に効かないか等価: OWN_IDS から R-2 を外す（写す入口・天井・相談窓口の正本に R-2 が 0 回で括弧を落とす式の答えが変わらない・歯の側の OWN と揃える足し）・行 R-2 の value か kind を読まずに今の字を焼く（今は同じ字で出力が等しい）・行の順を R-8・R-16・R-2 にする（床と注入と面は行を id で引く）。

### (f) 大きさ・verify と done の対応

1. **write-set。** 2 本とも印なし（書き換えるだけ）: `crates/folio/src/init.rs`・`crates/folio/tests/init.rs`。
2. **余地（CapHeadroom）。** 各行 ceil(字数 / 120)・空行は 1。python と awk の 2 実装で一致。

| file | base の正規化行数（参考値） | 余地 | 模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/init.rs` | 772 | 728 | 803（+31） | 697 |

   src の外は `tests/init.rs` 770 → 957。rustfmt --check の差の数は init.rs 11 → 11・tests/init.rs 26 → 25（足した字は fmt に合う）。
3. **size は S。** src は 1 本で +31（雛形の字と読み方 1 つ）。
4. **verify は 3 行**で、done の 3 の塊と 1 対 1。
   1. `cargo nextest run -p folio --test init f152_` = (c) の 1〜3。
   2. `cargo nextest run -p folio --test init` = 骨格の歯の全部（AC19・AC26・AC27 の歯と (c) の 4 を含む・参考値 21 本）。
   3. `cargo clippy --workspace --all-targets -- -D warnings`。

   base では 1 が 0 件で終了コード 4、2 は 18 本で緑、3 は 0 警告。`--test init` の歯の file は write-set に在り、`--bin` の行は無い。

### (g) 門と受付

門は対象外で 0（冒頭）。受付の先撃ち（precheck）で契約に起因する断りは 0（(h) の 5）。

### (h) 数え直す手順（行 D-13）

記録は `.local/share/folio2/handoff-2026-09-27/d152-draft.md`、script は同じ dir の d152-scripts と改訂 b の d152b-scripts（repo の外）。

1. 再現: repro-152.sh で (a) の 1、probe-152.py で (a) の 2。
2. 模擬: build-sim-152b.sh が main の clone に apply-152.py（実装）・apply-152-teeth.py と apply-152b-teeth.py（歯）を当てて組む（c152.patch）。suite-152b.sh（nextest・clippy）・floor-152.sh（床 4 本と build の diff -r）・verify-152.sh。
3. RED: red-152b.sh（歯だけを base に当てて戻す・r152-teeth.patch）。
4. 突然変異と余地: mut-152b.py（撃った後は元に戻して組み直す）・lines-152.py と lines-152.awk。
5. 受付の先撃ち: `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-152.md#ey`。

### (i) 要件との関係・運ばないもの・撤退条件・割り方

1. **要件との関係（正本は書き換えない）。** FR22 の規範文が足すもの（行 R-8 と R-16・その 2 行を縛る最小の条・最初の判断の記録）は全部残り、本便は行 R-2 と要件書の欄と改訂の表を足す。書く file は 11 本のままで、AC19・AC26・AC27 の歯は緑。本便は ADR-21 決定 (2) が v3 の入口の便へ残した論点（本来の憲法を持たない利用者向けの雛形）に答えて雛形を 1 つに替え、決定 (1) が同じ便へ回した注入の落ちを直す。着地の後に古くなる字は次のとおり。① FR22 の平易文の「決まりの 2 行」。② FR22 の注と ADR-16 決定 (3) の、骨格の憲法を「床の受付の宣言」と題して本来の憲法の条を指す形（scribe2-article）の段と、FR22 の注の「利用者の宣言の条」の字。③ FR22 の注と ADR-21 決定 (2) の、その形を本来の憲法を別に持つ利用者に当てる字と「骨格の命令の振る舞いと雛形は変えない」の文、ADR-21 の帰結の雛形は scribe2 の形という文。本便の後は道具がその形を書けない（当たる利用者は今いない・ADR-22 文脈 (エ)）。④ FR22 の注と ADR-16 決定 (3) の「骨格は注入の対象にしない（M3 の外）」。本便は ADR-21 決定 (1)(4) の v3 の入口の便として M3 の外で注入を通す。⑤ FR22 の規範文の「その 2 行を縛る最小の条」と ADR-16 決定 (2)(オ) の「条は行 R-8・R-16 を縛る 1 本」。条 P-1 は R-2 も縛る（規範文は足すものを閉じないので反しないが、数え上げが合わなくなる）。⑥ FR22 の注と ADR-16 決定 (2)(オ) の、N-3 に当たる条を本来の憲法に指す先が在るときだけ置く段。これらは設計文書の直しで、この便では書かない（次の節目の材料・行 D-16・台帳 f2-648.189 に席が控え済み）。
2. **運ばないもの。** 様式の file と部品目録（便 153）・焼かれた名 folio2（.191）・規則の表の引用符と索引（.190）・凍結の罠（.193）・行 R-17 の黙り（.194）・列の根の表 ROOT_DIGESTS（.233）・(d) の 1 の揃え・設計文書の正本・台帳への記帳・外部 crate。
3. **撤退条件。** (1) (c) の 4 のほかの既存の歯が 1 本でも落ちたら、歯も fixture も直さずに止めて席へ返す。(2) 着地の後の main で folio2 自身の床 4 本の結果か `folio build` の出力が着地の直前と 1 byte でも違ったら止めて席へ返す。(3) 受付の時点の main で init.rs の雛形の定数か rules_rows か、tests/init.rs の口 Work が base と違えば、(h) で数え直してから運ぶ。
4. **割り方（便 153・席の裁定を待つ）。** 様式の file を利用者の置き場へ届ける仕事は ADR-16 決定 (2)(エ)（部品目録は repo ごとに持たない・様式の file を置き場の下から写す形を含め、面の生成を利用者に広げるときは別の判断の記録で扱う）に当たる。依頼の推奨の形（init が焼いた 1 つの正本から preview/ の 3 file を書く）は、書く file を 14 本にして AC19 の 11 本の閉じた一覧と食い違い、parts.json を置き場ごとに持たせる。代わりの形（build が、置き場に様式の file が無いときだけ焼いた様式を出す・`crates/folio/src/site.rs` に +13 行）は、写しで build が 6 file を書くまで届いた。どちらも先に判断の記録か席の裁定が要る。本便の要件書の欄と注入は (エ) に当てない。(エ) の主語は部品目録と様式の file で、ADR-21 決定 (1) は骨格のままで落ちる命令を v3 の入口の便で扱うとし、決定 (4) はその便（.189〜.196）を M3 の列の外に置く。

## 2. 範囲

- 入れる: §1 (b) の 1〜4、(c) の歯 3 本と置き換え 2 本。
- 入れない: 様式の file と部品目録・面の生成器・床と注入の実装・命令の旗・設計文書の正本・新しい fixture と dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| srs | 要件書の雛形 | init.rs の SRS に面の生成器が要る欄 |
| constitution | 憲法の雛形 | init.rs の CONSTITUTION_HEAD / BODY（置き場に依らない形・改訂の表） |
| rules | 行 R-2 | init.rs の rules_rows・OWN_IDS・語彙の雛形 |
| teeth | 歯 | tests/init.rs の f152_ 3 本と置き換え 2 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate と新しい dir は無い。前提の着地は無い（base = main 6093b8d）。並行の便は無い（init.rs と tests/init.rs を書き換える契約は無い）。
- 着地の後に席が見ること: 本流の target/debug/folio を組み直す。.196 を閉じる（.189 は様式の file と §1 (i) の 1 の字が残るので開いたまま）。便 153 の形と判断の記録の要否を決める（§1 (i) の 4）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ey"
title = "群 A の 1 本目（台帳 f2-648.189 と .196）: folio init の骨格のままでは、folio inject が行 R-2 の欠けで、folio face と folio build が要件書の欄の欠けと憲法の改訂の欄が字であることで落ちる（床は受ける）。骨格の憲法の雛形は本来の憲法を指す scribe2 の形である。crates/folio/src/init.rs の雛形に、面の生成器が要る要件書の欄を空か未記入で、行 R-2（what・value・kind を folio2 の正本から型付きで読む）を足し、憲法の雛形を自分の憲法を正本とする形（名 未記入・根拠の欄は空・条 P-1 が行 R-2・R-8・R-16 を縛る・改訂の欄は表）にする。書く file 11 本・命令の旗・面の生成器・床・注入・設計文書の正本は変えない。歯は tests/init.rs の f152_ 3 本（通しの歯・憲法の雛形・folio2 固有の字）と既存 2 本の置き換え。様式の file は便 153 に割る。門の対象外。base = main 6093b8d・受付の時点の main で数え直す"
req = ["FR22", "FR6", "FR4", "FR16"]
section = "1"
write-set = ["crates/folio/src/init.rs", "crates/folio/tests/init.rs"]
verify = ["cargo nextest run -p folio --test init f152_", "cargo nextest run -p folio --test init", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/init.rs の f152_ の 3 本（根の直下の design-intent に init して commit した後、手直しなしで check が rc 2 で 違反 0・まだ分からない 2、一時の CLAUDE.md の印の対への inject --write と --check が rc 0、schema --check が rc 0、build --write が rc 2 で 床 = まだ分からない（違反 0・まだ分からない 2）と 1 行の preview/folio.css: 読めない を出して配信先を作らず、face の adr・constitution・srs・index が rc 0 / 骨格の憲法の生成区間と schema の節の外に 本来の憲法・scribe2・床の受付の宣言・floor-declaration が無く、meta.id が 未記入、根拠の欄が空、前文が段の順の手書きの期待と等しく、条の規範文に 憲法 が無く、amendment が 3 欄の表で発効の段が 段 0 / 骨格の 11 本の生成区間と憲法の schema の節の外に folio2・f2-・scribe2 が無い）が緑、tests/init.rs の歯の全部（AC19・AC26・AC27 の歯と、条 P-1 が R-2・R-8・R-16 を縛り骨格の行 R-2 が folio2 の行 R-2 と同じで骨格の行の状態と裁定が 仮・未記入 という置き換えを含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main と file 数も byte も変わらない"
<!-- contracts:end -->
