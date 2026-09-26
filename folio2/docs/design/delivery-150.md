# 設計: 便 150 — 天井の門が --dir を設計文書の置き場か確かめず、渡し間違いの --dir で黙って通す穴を塞ぐ（置き場の印の file・天井の 41 周目 実態 F-1）

- 要件: FR20（天井の門・要件書 第 1.45 版）。本便は FR20 の規範文も確かめ方も受入基準 AC18 の 8 場合も変えない。門が `--dir` を設計文書の置き場かどうか確かめずに 0（通す）を返していた穴を、既存の「判定が実行できなかったら まだ分からない」の族（印が読めない・引き金の要約値が測れない・印の節点の表が読めない と同じ・判断の記録 ADR-24 決定 (2)）に収めて塞ぐ。契約表の行の req は FR20 の 1 つ（main に在る id・便 73・126・129・142 と同じ）。受入基準 AC18 は器の要件面に無い id なので req に書かない（便 142 の受付の先撃ちの実測）。
- 条: P-4.1（検査が実行できなかった結果を異常なしとして扱わない＝置き場でない `--dir` で 通す と言わない）/ P-4.2（判定できないものは まだ分からない として表に出す＝理由の行に撃ち直し方を書く）/ P-5.1（置き場の印の file と理由の字は型付きの定数で持つ）/ P-15.2 の向き（門の判定の式は 1 つの関数のまま＝設計文書の判定 is_design_source は変えず、その前に置き場の確かめを置く）/ P-10.1（期待の字は歯の側の手書きで持つ＝`tests/fixtures/ceiling/` は 1 byte も変えない）/ N-3.1（例外の口を足さない＝旗を足さない）。
- 出所: 天井の 41 周目 実態 F-1（止める・反証 支持）。台帳 **f2-648.224**。席が本流で再現した（`--dir design-intnet` と `--dir design-intent/adr` で、write-set に `design-intent/srs.yaml` が在っても 0 通す）。要件書 第 1.46 版（一括 24・承認待ち）の FR20 の注は、この穴を「門の今の既知の穴」と書く。依頼は席から起草役へ（2026-09-26）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `ew` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 3 本（書き換える 2 本 + 本文が変わらない verify の scope 1 本）。新しい file も縮む file も消す file も無く（`+` も `-` も当たらない）、新しい dir も作らない。
- 改訂 b（2026-09-26・独立の検証 d150-verify.md〔支持・一致 22・blocking 0〕の非 blocking 1・2・席の裁定）: (c) の 2 の歯に判定の順を縛る 1 通り（今の dir の外で置き場でもない --dir は 今の dir の下に無い の理由）を足し、(e) の 4 の表に M10（置き場の確かめを根の突き合わせより前へ移す）を足した。(i) の 1 に、実装だけの write-set の 2 は規範文の字どおりの読みより厳しい側である旨を 1 文足した。実装・write-set・verify の行・size・余地は変えない。
- 門: 本便は設計文書の正本（`design-intent/` の下）を 1 本も書き換えないので、天井の門の対象外である。起草役が作業ツリー planner-d150 の一番上で write-set 3 本を base の binary（写しの組み立て）で `folio ceiling --gate --dir design-intent --write-set …` に渡すと **0（通す・断りの字 = 設計文書の正本を書き換えない便）**。便を当てた写しの binary でも同じ字で 0。
- 前の便: 前提の着地は無い。**base = main 5d56658（便 149 の着地の後）。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。起草役は `git archive 5d56658` の写しを組み立てて測った（本流の作業ツリーの binary は再現の対照にだけ使った）。
- 並行の便との重なり: base の時点で、門（`crates/folio/src/gate.rs`）と歯の file `crates/folio/tests/gate.rs`・`crates/folio/tests/stamp.rs` を書き換える便の契約は無い（一括 24 の枝 docs/batch24 は `design-intent/` と文書だけを書く）。共通の検証は同時に撃たない逐次を勧める。

## 1. 設計

### (a) いま起きていること（実測・base main 5d56658・数は参考値）

1. **門の判定の順。** `folio ceiling --gate`（`crates/folio/src/gate.rs` の関数 run）は、根の突き合わせ（関数 dir_parts で `--dir` が今の dir の下か・関数 other_root で write-set の path が同じ根からの相対か・便 142）の後、関数 is_design_source で write-set に `--dir` の下の file が在るかを見る。1 つも無ければ、印を読まずに 0（通す・設計文書の正本を書き換えない便）を返す。この 3 つは、どれも `--dir` と write-set の path の **字だけ** で決まる。`--dir` が dir として在るか、設計文書の置き場か（正本の file を持つか）は、どこも確かめていない。
2. **穴の再現（repo の写しの根・base の binary）。** 写しの根を今の dir にし、write-set を `design-intent/srs.yaml crates/folio/src/gate.rs`（要件書を書き換える便の形）にして `--dir` だけを変えた答え。

| 名 | --dir | base の答え | 本便の後の答え |
| --- | --- | --- | --- |
| A | design-intent（対照） | 0 通す（印が 4 観点とも合格…正本の要約値が同じ） | 同じ |
| B | design-intnet（打ち間違い・実在しない） | **0 通す（設計文書の正本を書き換えない便）** | 2 置き場でない |
| C | design-intent/adr（置き場の下の dir） | **0 通す（同上）** | 2 置き場でない |
| D | 空の dir | **0 通す（同上）** | 2 置き場でない |
| E | design-intnet・write-set が実装の file だけ | 0 通す（同上） | 2 置き場でない |
| F | design-intent・write-set が実装の file だけ（対照） | 0 通す（同上） | 同じ |
| G | ./design-intent/（対照） | 0 通す（印が 4 観点とも合格…） | 同じ |
| H | design-intent/srs.yaml（file を --dir に） | **0 通す（設計文書の正本を書き換えない便）** | 2 置き場でない |
| I | design-intent/anchors（置き場の下の dir） | **0 通す（同上）** | 2 置き場でない |
| J | 今の dir の下の実在しない絶対 path | **0 通す（同上）** | 2 置き場でない |

   B・C・D・H・I・J が穴で、判定が実行できない所で 0 を返す（P-4.1 / P-4.2 に反する）。本便の後の 2 の理由の行は、例えば B で「まだ分からない（--dir が設計文書の置き場でない（design-intnet・constitution.yaml が無い）・作業ツリーの一番上から撃つ）」。main の `target/debug/folio`（本流の build）と base の写しの binary は 10 通りとも同じ字を返した。作業ツリー planner-d150 の一番上でも、base の binary は B・C の形で 0、本便の binary は 2 を返した。

3. **今の撃ち方は当たらない。** 席の受付の手順 admit.sh と一括の取り込みの手順は、本流の一番上で `--dir design-intent` で撃つ（A・F の形）。歯も同じ形（`crates/folio/tests/gate.rs` は一時 dir を今の dir にして `--dir design-intent`・`crates/folio/tests/stamp.rs` は周の一時 dir を今の dir にして `--dir src`）。穴に当たるのは `--dir` を渡し間違えたときである。
4. **置き場の印の候補の実測。** 候補 3 つ（constitution.yaml・ceiling.yaml・index.yaml）が在る dir を、repo の `design-intent/` と `tests/fixtures/` の下で数えた。
   1. constitution.yaml は置き場の一番上にだけ在る（本物の置き場・門の歯の置き場 `tests/fixtures/ceiling/bundle/source/`・床の置き場 `tests/fixtures/floor_base/design-intent/` を含む 25 か所）。置き場の下の dir には無い（`design-intent/anchors/` の凍結 anchor は constitution-v1.0.yaml などの版付きの名）。`folio check` が最初に読む正本である（`crates/folio/src/check.rs` の正本 7 file の一覧 FILES の先頭）。
   2. ceiling.yaml も置き場の一番上にだけ在る（21 か所）。門自身が読む天井の正本（関数 documents と `ceiling_src::load`）。
   3. index.yaml は置き場の下の dir にも在る（`design-intent/anchors/index.yaml`・`tests/fixtures/floor_base/design-intent/anchors/index.yaml`・`tests/fixtures/anchor/root-digest-drift/anchors/index.yaml`）。印にすると `--dir design-intent/anchors`（I の形）を置き場と読み、穴が残る。
5. **要件書との関係。** FR20 の規範文（第 1.45 版）は、照らせない形（`--dir` が今の dir の下に無い・write-set に根からの相対で読めない path が在る）を一覧で持ち、照らせるときに write-set に置き場の file が 1 つも無ければ 0 と言う。「--dir が置き場でない」はこの一覧に無い。規範文の「設計文書の置き場の file」は `--dir` が置き場であることを前提にしており、置き場でない `--dir` では 0 の場合にも他の 7 場合にも当たらず、判定が実行できない。門は既にこの族の まだ分からない を持つ（印が読めない・天井の正本が読めない・引き金の要約値が測れない・正本の要約値が測れない・印の節点の表が読めない・印の後に変わった節点が数えられない）。本便の 2 もこの族に置く（ADR-24 決定 (2)）。(i) の 1 を参照。
6. **base の歯（参考値）。** workspace の nextest 949 / 949・clippy 0 警告・床 4 本（check・inject --check・schema --check・derive --check）rc 0・`folio build --write` の出力 31 file。歯の file の本数は tests/gate.rs 16・tests/stamp.rs 13・gate.rs の単体 2（gate_classifies_design_sources・f142_the_dir_and_the_write_set_share_one_root）。`git grep -n f150_ -- crates` は 0 件。行 id `ew` は repo の docs/design に 0 件。

### (b) 直す先 — 根の突き合わせの直後に置き場の確かめを置く（`crates/folio/src/gate.rs` の 1 file）

1. **置き場の印の file を型付きの定数で持つ。** 定数 PLACE_MARK = constitution.yaml（`folio check` が最初に読む正本・(a) の 4 の 1）。関数 is_place を足す: `--dir` の直下に PLACE_MARK が file として在れば真（在れば `--dir` は dir として実在する）。PLACE_MARK が dir なら偽。印の file の中身は読まない（読めるかどうかは、この後の引き金の要約値の測りと床が見る）。
2. **関数 run の、根の突き合わせの直後・設計文書の判定（通す・設計文書の正本を書き換えない便）の前に置く。** is_place が偽なら、2（まだ分からない）と理由の行「--dir が設計文書の置き場でない（<--dir の字>・constitution.yaml が無い）・作業ツリーの一番上から撃つ」。字の頭 --dir が設計文書の置き場でない は型付きの定数 UNKNOWN_NOT_A_PLACE、印の file の名は PLACE_MARK、末尾の撃ち直し方は既存の定数 FROM_THE_TOP を使う。`--dir` の字は渡された字のまま（相対なら相対・絶対なら絶対）。標準出力は今の形のまま 1 行（folio ceiling: まだ分からない（<理由>））。
3. **判定の順（§1 (c) の 2 の族）は変えない。** 今の dir の下に無い（2）→ 根が違う（2）→ **置き場でない（2・本便）** → 設計文書の file が無い（0）→ 印が無い / 読めない（2）→ …の順で、最初に当たったもので決まる。置き場でないことは、write-set が実装の file だけでも 2 にする（E の形）。置き場でない `--dir` では、write-set のどれが置き場の file かを照らせないから。
4. **変えないもの。** 関数 dir_parts・other_root・is_design_source・印の読み方・引き金の要約値と正本の要約値の関数（印の側 `crates/folio/src/stamp.rs` と面の名札 `crates/folio/src/face.rs` も同じ関数を呼ぶ）・3 値の優先・通すときの節点の数の字・命令の口（旗を足さない）・凍結 anchor（`tests/fixtures/ceiling/`）・admit.sh と一括の手順・`folio check` の側・設計文書。
5. **置き場のときの答えは変わらない。** `--dir` が置き場（constitution.yaml を持つ dir）なら、判定は base と同じ関数で同じ順に進む。本流の撃ち方（`--dir design-intent`）と歯の撃ち方（一時 dir の design-intent・周の一時 dir の src）は全部これに当たる。

### (c) 歯（関数名 f150_・base で 0 件）

1. **f150_the_place_is_a_dir_with_the_mark（単体の歯・`crates/folio/src/gate.rs` の既存の tests の区間 gate_tests の末尾）。** PLACE_MARK が `check.rs` の FILES の先頭に .yaml を足した字と同じであること。一時 dir の中で、is_place が、constitution.yaml を持つ dir を真、実在しない dir・空の dir・constitution.yaml という名の dir を持つ dir・ceiling.yaml と index.yaml だけを持つ dir・constitution.yaml の file そのもの（file を --dir に）を偽とすること。**base では PLACE_MARK と is_place が無く組み立てが落ちる＝RED。**
2. **f150_the_gate_is_unknown_when_the_dir_is_not_a_place（`crates/folio/tests/gate.rs`・binary 経由）。** 既存の Repo（一時 dir の design-intent/ に束の source の写し）に合格で新しい印（stamp-pass.yaml を fresh）を置き、一時 dir に空の dir empty を作る（歯の中で作る・fixture の dir は足さない）。一時 dir を今の dir にして、write-set を design-intent/srs.yaml と crates/folio/src/gate.rs にし、`--dir` を design-intnet・design-intent/adr・empty・design-intent/srs.yaml・一時 dir の下の design-intnet の絶対 path にした 5 通りと、`--dir design-intnet` で write-set を crates/folio/src/gate.rs だけにした 1 通りを撃つ。6 通りとも終了コード 2 で、標準出力が まだ分からない と --dir が設計文書の置き場でない（<渡した --dir の字>・constitution.yaml が無い）・作業ツリーの一番上から撃つ を持つ。加えて判定の順（(b) の 3）を縛る 1 通り: 同じ写しで `--dir ../nope`（今の dir の外で、置き場でもない）を撃つと、終了コード 2 で、標準出力が --dir が今の dir の下に無い を持ち 置き場でない を持たない（根の突き合わせが置き場の確かめより先）。**base では 6 通りとも 0（通す・設計文書の正本を書き換えない便）＝RED**（順の 1 通りは base でも 今の dir の下に無い を返す）。
3. **f150_the_gate_reads_the_place_as_before（同）。** 置き場のときの答えが変わらないこと（GREEN の側の見張り）。印の無い写しで、write-set が実装の file だけなら 0 と 設計文書の正本を書き換えない便、要件書なら 2 と 印が無い（印を読みに行く）。合格で新しい印を置いた後、`--dir design-intent`・`--dir ./design-intent/`・置き場の絶対 path の 3 通りで要件書の write-set を撃つと、どれも 0 と 正本の要約値が同じ。**base でも緑**（base の答えを本便の後も保つことを縛る歯・(e) の 4 の M9 で落ちる）。

fixture は新しい file を足さない。歯は既存の口（Repo・put_stamp・gate・gate_at・code・stdout）だけを使う。凍結 anchor は読むだけで書き換えない。

### (d) 採らなかった形

1. **印を ceiling.yaml にする。** 置き場の一番上にだけ在るのは constitution.yaml と同じで、(a) の 2 の 10 通りの答えも同じになる。採らないのは、ceiling.yaml が門自身の入力（天井の正本）だからである。置き場かどうかの確かめを門の入力の有無に結ぶと、天井の正本が無い置き場の 2 の理由が「置き場でない」に変わり、天井の正本が読めないという今の理由（設計文書の file が在るときの ceiling_src::load の断り）と区別できなくなる。constitution.yaml は `folio check` が最初に読み、`folio init` が最初に書く正本で、置き場を置き場たらしめる file として床と揃う。
2. **印を index.yaml にする。** 置き場の下の凍結 anchor の dir（`design-intent/anchors/`）にも同じ名の file が在り、`--dir design-intent/anchors` を置き場と読んで 0 を返す（(a) の 4 の 3・I の形）。
3. **`--dir` が dir として実在するかだけを見る。** B と J は塞げるが、C・I（置き場の下の dir）と D（空の dir）が 0 のまま残る。
4. **正本 7 file（`check.rs` の FILES）が全部在るかを見る。** 置き場の確かめとしては強いが、門が `folio check` の正本の一覧に依る新しい前提になり、7 file の 1 つが欠けた置き場でも実装だけの便を止める。印の 1 file で足りる（床の欠けは `folio check` が見る）。

### (e) 既存の歯のうち落ちるもの・突然変異・面の変化

1. **落ちる既存の歯は 0 本（起草役の実測）。** 本便の差分を base に当てた写しで、workspace の nextest **952 / 952**（base 949 + 単体 1 + gate 2）・clippy 0 警告・床 4 本 rc 0・`folio build --write` の出力 31 file（base と diff -r で全 file が一致）・tests/gate.rs 18 / 18（既存 16 を含む）・tests/stamp.rs 13 / 13。既存の歯の置き場（束の source の写し・周の一時 dir の src）は、どれも constitution.yaml を持つ。便 126 の歯 f126_the_gate_is_unknown_when_the_trigger_cannot_be_measured は constitution.yaml の字を壊すが file は残すので、置き場の確かめを通って今までどおり 引き金の要約値が測れない で 2 になる（緑）。凍結 anchor（`tests/fixtures/ceiling/` の下）は 1 byte も変わらない（差分は `crates/folio/src/gate.rs` と `crates/folio/tests/gate.rs` の 2 file だけ）。
2. **回帰なし（本流の撃ち方・起草役の実測）。** repo の写しの根で `--dir design-intent` を使い、base と本便の binary に同じ write-set を 7 通り渡した（便 149 の write-set・本便の write-set・要件書と語彙・./ 付き・+ 付きの新しい判断の記録・preview と retired だけ・docs/design だけ）。7 通りとも終了コードと標準出力が byte で同じだった。(a) の 2 の A・F・G（対照）も base と同じ字。
3. **RED（起草役の実測）。** 歯だけを base に当てた写しで、単体の歯は組み立てが落ち（is_place と PLACE_MARK が無い）、gate の f150_the_gate_is_unknown_when_the_dir_is_not_a_place は落ちる（1 通り目の答えが 0 の 通す）。f150_the_gate_reads_the_place_as_before は base でも緑（見張りの歯）。
4. **突然変異（起草役の実測・本便を当てた写しの `crates/folio/src/gate.rs` を 1 通りずつ変える）。** 撃つ歯は f150_ の 3 本・単体の gate_classifies_design_sources と f142_・tests/gate.rs と tests/stamp.rs の全部（34 本）。10 通りとも f150_ の 1 本以上が落ちる。M9 のほかは既存の歯は緑のまま。

| 変異 | 単体 f150_ | gate f150_ 置き場でない | gate f150_ 置き場は今までどおり |
| --- | --- | --- | --- |
| M1 置き場の確かめを消す（base の形） | 緑 | 落ちる | 緑 |
| M2 置き場の確かめを 通す（設計文書の正本を書き換えない便）の判定の後へ移す | 緑 | 落ちる | 緑 |
| M3 置き場でないとき 通す を返す | 緑 | 落ちる | 緑 |
| M4 印を見ず dir の実在だけを見る | 落ちる | 落ちる | 緑 |
| M5 印が dir でも在ると読む | 落ちる | 緑 | 緑 |
| M6 印を ceiling.yaml にする | 落ちる | 落ちる | 緑 |
| M7 印を index.yaml にする | 落ちる | 落ちる | 緑 |
| M8 理由の字から --dir の字を落とす | 緑 | 落ちる | 緑 |
| M9 確かめの向きを逆にする（置き場なら まだ分からない） | 緑 | 落ちる | 落ちる（既存の歯 17 本も落ちる） |
| M10 置き場の確かめを根の突き合わせ（dir_parts）より前へ移す | 緑 | 落ちる（`--dir ../nope` の理由が 置き場でない になる） | 緑 |

5. **面の変化は無い。** 門は面を書かず、`folio build` の出力は base と byte で同じ（(e) の 1）。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 3 本とも印なし。書き換える 2 本 = `crates/folio/src/gate.rs`・`crates/folio/tests/gate.rs`。本文を変えない 1 本 = `crates/folio/tests/stamp.rs`（verify の `--test stamp` で名指すので入れる・門を `--dir src` で撃つ歯を持つ）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の 1 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/gate.rs` | 523 | 977 | 568（+45） | 932 |

   余地は S の見積 100 を超える。行数の上限の歯（face.rs・face_srs.rs・tests/schema.rs）は本便の write-set に当たらない。src の外の参考値は `tests/gate.rs` 860 → 944・`tests/stamp.rs` 773（不変）。rustfmt の差の数は gate.rs 12 → 12・tests/gate.rs 18 → 18（本便の足した字は fmt に合う・本流も fmt に合わない所を持ち、CI は fmt を見ない）。
3. **size は S。** 触る src は 1 本で、src の増分は +45（関数 1 つ・定数 2 つ・run に 10 行・注 4 行）と単体の歯 1 本。
4. **verify は 5 行**で、done の 5 の塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --bin folio f150_` = (c) の 1。
   2. `cargo nextest run -p folio --test gate f150_` = (c) の 2・3（2 本）。
   3. `cargo nextest run -p folio --test gate` = 門の歯の全部（AC18 の 8 場合と便 126・129・142 の歯を含む・参考値 18 本）。
   4. `cargo nextest run -p folio --test stamp` = 印の歯の全部（周の一時 dir を今の dir にして `--dir src` で門を撃つ歯を含む・参考値 13 本）。
   5. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
   base 5d56658 では 1 と 2 が 0 件で終了コード 4（歯が無い）、3 は 16 本・4 は 13 本で緑、5 は 0 警告である。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は gate・stamp の 2 本で、どちらも write-set に在る。`--bin folio` の絞り込みの語 f150_ を関数名に持つ src は `crates/folio/src/gate.rs` で、write-set に在る。

### (g) 門と受付

1. **門。** 作業ツリー planner-d150 の一番上で `folio ceiling --gate --dir design-intent --write-set crates/folio/src/gate.rs crates/folio/tests/gate.rs crates/folio/tests/stamp.rs` を base の binary で撃つと 0（通す・設計文書の正本を書き換えない便）。本便を当てた写しの binary でも同じ字で 0。
2. **受付。** 受付の先撃ち（precheck）で契約に起因する断りは 0（起草役の実測・(h) の 6）。並行の便は無い。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-26/d150-draft.md`、模擬の差分と script は同じ dir の d150-scripts（repo には入れない）。

1. 再現: repro-150.sh（binary・repo の写しの根）で (a) の 2 の 10 通りを撃つ。作業ツリーの一番上での 3 通りは real-worktree.log。
2. 模擬: base の写しの根で apply-150.py（実装）と apply-150-teeth.py（歯）を撃つ。その差分が c150.patch。workspace の nextest・clippy・床 4 本・`folio build --write` の出力の file 数と base との diff -r・verify の 5 行（verify-150.sh）を撃つ。regress-150.sh（base の binary・本便の binary・repo の写しの根）で (e) の 2 の 7 通り。
3. RED: apply-150-teeth.py の歯だけを base に当てる（r150-teeth.patch）。単体の歯は組み立てで落ちるので、gate の 2 本は `crates/folio/tests/gate.rs` の差分だけを当てて撃つ。
4. 突然変異: mut-150.py（(e) の 4・撃った後は元の gate.rs で組み直す）。
5. 余地: lines-150.py と lines-150.awk（同じ式の 2 実装）を repo の根で write-set の file に当てる。
6. 受付の先撃ち: 契約を commit した後に `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-150.md#ew`。

### (i) 要件との関係・本便が運ばないもの・言えないこと・撤退条件

1. **要件との関係（正本は書き換えない）。** 本便は、置き場でない `--dir` の 2 を、印が読めない・引き金の要約値が測れない と同じ「判定が実行できないときの まだ分からない」の族として実装に足すだけである（ADR-24 決定 (2)）。FR20 の規範文・確かめ方と、受入基準 AC18 の 8 場合は動かさない（天井の引き金を動かさない）。規範文の照らせない形の一覧か注に「--dir が置き場でない」の断りを足すかは、次の一括の材料である（席が台帳 f2-648.224 に控える）。なお (a) の 2 の E の形（実在しない --dir・write-set が実装の file だけ）は、規範文（第 1.45 版）の照らせない形の一覧に無く、字どおりには「置き場の file が 1 つも無いので 0」と読める。本便は --dir が置き場であることを規範文の前提と読み、字どおりの読みより厳しい側（2）を返す。この前提を規範文か注へ上げるのは次の一括の材料である（台帳 f2-648.224 に控え済み）。要件書 第 1.46 版（一括 24）の FR20 の注の「門の今の既知の穴…未着地」の段は、本便の着地の後に着地の字へ直す材料になる（注は設計文書の正本で、便では触れない）。
2. **運ばないもの。** 要件書 FR20 の規範文・注と受入基準 AC18 の場合。設計ノート ceiling-gate.md の字。`folio check` の側（正本の欠けの判定）。根の突き合わせ（便 142）・設計文書の判定 is_design_source・印・要約値の関数。admit.sh と一括の手順（今の撃ち方のまま答えが変わらない）。台帳への記帳（席）。外部 crate。
3. **言えないこと。** 印が示すのは「constitution.yaml を持つ dir である」ことだけで、write-set が指す置き場と同じ置き場であることは言えない。例えば repo の根で `--dir tests/fixtures/ceiling/bundle/source`（別の置き場の写し）を渡すと、置き場と読み、write-set の `design-intent/srs.yaml` はその下に無いので 0（通す）のまま（起草役の実測）。印の file の中身は読まず、symlink も解く（`is_file` は symlink を辿る）。symlink の印や、中身の壊れた印でも置き場と読む（中身の壊れは引き金の要約値の測りと床が見る）。
4. **撤退条件。** (1) 本便の後に、置き換えも足しもしない既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(2) 本便の後に、着地の直前の main の撃ち方（repo の根で `--dir design-intent`）で (e) の 2 の 7 通りの write-set の答えが、着地の直前の main の binary の答えと終了コードか標準出力で違うか、`folio build` の出力か folio2 自身の床 4 本の結果が 1 byte でも変わったら、止めて席へ返す。(3) 受付の時点の main で `crates/folio/src/gate.rs` の関数 run か `crates/folio/tests/gate.rs` の口 Repo・gate_at が base と違えば、(a)(b)(e) を (h) の手順で数え直してから運ぶ（穴が既に閉じていたら止めて席へ返す）。

## 2. 範囲

- 入れる: `crates/folio/src/gate.rs` の定数 PLACE_MARK と UNKNOWN_NOT_A_PLACE、関数 is_place、関数 run の根の突き合わせの直後の置き場の確かめ、単体の歯 1 本（既存の tests の区間）。`crates/folio/tests/gate.rs` の歯 2 本。
- 入れない: 根の突き合わせ・設計文書の判定・印の読み方・要約値の関数・3 値の優先・命令の旗・凍結 anchor・admit.sh と一括の手順・`folio check`・要件書（FR20 の注を含む）と設計文書・新しい fixture の file・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| placemark | 置き場の印 | `crates/folio/src/gate.rs` の PLACE_MARK（constitution.yaml）と is_place |
| run | 門の置き場の確かめ | `crates/folio/src/gate.rs` の run の根の突き合わせの直後の確かめと理由の字の定数 |
| teeth | 歯 | gate.rs の単体の f150_ 1 本と tests/gate.rs の f150_ 2 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 無し（base = main 5d56658）。
- 並行の便: 無し。
- 本便の着地の後に席が見ること: 本流の `target/debug/folio` を組み直す（admit.sh と一括の手順はこの binary で門を撃つ・組み直すまでは着地の前の binary のままで穴が残る）。台帳 f2-648.224 を閉じる。要件書 FR20 の注の「門の今の既知の穴」の段（第 1.46 版）を着地の字に直すこと、規範文の照らせない形の一覧か注に「--dir が置き場でない」を足すかを、次の一括の材料として控える（§1 (i) の 1）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ew"
title = "天井の 41 周目 実態 F-1（台帳 f2-648.224・P-4.1 / P-4.2）: folio ceiling --gate は --dir が設計文書の置き場かを確かめず、打ち間違いの --dir（design-intnet）・置き場の下の dir（design-intent/adr）・空の dir・file を --dir に渡すと、write-set に design-intent/srs.yaml が在っても 通す（rc 0・設計文書の正本を書き換えない便）を黙って返す。crates/folio/src/gate.rs に置き場の印の型付きの定数 PLACE_MARK（constitution.yaml・folio check が最初に読む正本）と関数 is_place（--dir の直下に印が file として在るか）を足し、run の根の突き合わせの直後・設計文書の判定の前で、置き場でなければ まだ分からない（rc 2・理由 --dir が設計文書の置き場でない（<--dir>・constitution.yaml が無い）・作業ツリーの一番上から撃つ）を返す。置き場のときの判定・根の突き合わせ・is_design_source・印と要約値の関数・凍結 anchor・命令の旗・要件書は変えない（FR20 の 8 場合は増やさず、判定が実行できないときの まだ分からない の族に置く・ADR-24 決定 (2)）。歯は単体の f150_ 1 本と tests/gate.rs の f150_ 2 本。設計文書の正本を書き換えないので門の対象外。base = main 5d56658・受付の時点の main で数え直す"
req = ["FR20"]
section = "1"
write-set = ["crates/folio/src/gate.rs", "crates/folio/tests/gate.rs", "crates/folio/tests/stamp.rs"]
verify = ["cargo nextest run -p folio --bin folio f150_", "cargo nextest run -p folio --test gate f150_", "cargo nextest run -p folio --test gate", "cargo nextest run -p folio --test stamp", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "単体の歯 f150_（PLACE_MARK が folio check の正本の一覧の先頭の constitution.yaml で、is_place が印を file として持つ dir を真、実在しない dir・空の dir・印の名の dir を持つ dir・ceiling.yaml と index.yaml だけの dir・印の file そのものを偽とする）が緑、tests/gate.rs の f150_ の 2 本（design-intnet・design-intent/adr・空の dir・design-intent/srs.yaml・実在しない絶対 path の --dir と、実装の file だけの write-set の design-intnet が rc 2 で --dir が設計文書の置き場でない（<--dir>・constitution.yaml が無い）・作業ツリーの一番上から撃つ を出す 、今の dir の外で置き場でもない ../nope は rc 2 で --dir が今の dir の下に無い を出し 置き場でない を出さない / 置き場の --dir では実装だけの write-set が rc 0 で 設計文書の正本を書き換えない便、印の無い置き場の要件書が rc 2 で 印が無い、合格で新しい印の置き場は design-intent・./design-intent/・絶対 path の 3 通りとも rc 0 で 正本の要約値が同じ）が緑、tests/gate.rs の歯の全部（受入基準 AC18 の 8 場合と便 126・129・142 の歯を含む）が緑、tests/stamp.rs の歯の全部（--dir src で門を撃つ歯を含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main の出力と file 数も byte も変わらず、repo の根で --dir design-intent の門の答えは §1 (e) の 2 の 7 通りで着地の直前の main の binary と同じである"
<!-- contracts:end -->
