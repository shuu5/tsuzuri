# 設計: 便 119 — 導出物の独立の命令 folio derive（契約表から器の形の導出物を組み、置き場の導出物との差分を数える）と、床の定数の命令の名の直し（FR11・ADR-16 決定 (5) の ⑤）

- 要件: FR11（導出物を生成し、差分 0 を数える・第 1.32 版・契機は「導出物の検査の命令〔配信の組み立て folio build から切り離した独立の命令・面の生成器も様式の file も呼ばない・名は便で決める〕が実行されたとき」）/ AC9（導出物を 1 字変えると --check が非 0 で終わる）/ FR19（床の定数から欄の決まりの file の生成区間へ写す＝本便は設計ノートの欄の決まりの写しの値を変える）。規範文はどれも変えない。契約表の行の req は FR11 と FR19 の 2 つで、AC9 は入れない（器の要件面は受入基準の id を読まず、起草役が作業ツリーで撃った表の検査が requirement-missing を出した）。
- 条: P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する）/ P-18.2（事後の検査を編集の時点で止めることの代わりにしない）/ P-4.1（実行できなかった結果を異常なしとして扱わない）/ P-4.2（判定できないものは「まだ分からない」として表に出す）/ P-10.1（独立した凍結 anchor を 1 本以上持つ）/ P-10.2（生成物どうしの突き合わせを唯一の合格判定にしない）/ P-5.6（実装の型付きの定数の写しを設計文書の置き場へ決定的に導出する）/ P-1.2（採否の最終判断を代行する機能を持たない＝足す命令は生成と検査だけ）/ N-1.1（管理下の対象を回復不能に削除しない）
- 出所: 判断の記録 **ADR-16**（発効 2026-09-23 17:39 JST・持ち主の承認・対話面 R-8・逐語「裁定は両方承認する」・裁定 id = 台帳 f2-648 notes 2026-09-23 17:39 JST）の決定 (5)「導出物を組む口と差分を数える口は、配信の組み立て（folio build）から切り離した独立の命令とし、面の生成器も様式の file も呼ばない（名は便で決める・案 folio derive --check）」「要件 FR11 の契機（when）と、設計ノートの欄の決まりの床の定数（導出物の検査の命令の名）は、この独立の命令を名指すように直す」と、決定 (7) の便の列の **⑤「導出物の独立の命令（床の定数の命令の名の直しを含む・中・1〜2 便）」**。FR11 の契機は要件書 第 1.31 版（版 A）で既に直っている。規則の表の開発規律行 D-11（P-5.6 が掛かる定数の値を変える便は根拠の判断の記録か裁定 id を名指す）は、この行と契約表の行の title が ADR-16 と裁定 id を名指すことで満たす。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `dr` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 16 本。**新しい file は 5 本で、先頭に `+` を付けて宣言する**（src 1・歯 1・凍結の対 2・folio2 自身の導出物 1）。**縮む file も消す file も無く、新しい dir も作らない**（凍結の対は既に在る dir `tests/fixtures/design-note/` に、folio2 自身の導出物は既に在る dir `contracts/` に置く）。
- 門: 本便は `design-intent/design-note/schema.yaml`（設計ノートの欄の決まり）の生成区間を書き換えるので天井の門の対象である。起草役が write-set 16 本をそのまま `folio ceiling --gate --write-set …` に渡して実測すると **2（まだ分からない・断りの字は 印が古い）**（2026-09-24・本便の後の binary）。base の binary に設計ノートの欄の決まり 1 本だけを渡しても同じ 2 なので、印は本便の前から古い。src の 1 本だけを渡すと 0（通す・設計文書の正本を書き換えない便）。規則の表の開発規律行 D-12 の扱いは §1 (g)。
- 前の便: 判断の記録 ADR-16 決定 (7) の前提 = ADR-15 の列と一括 14（着地済み）と要件書の版 A（第 1.31 版・発効済み）。便 117（行 `dp`・main a34c6b2）と便 118（行 `dq`・main 0b0aafd）と f2-648.162（main 41ed484・歯の file `crates/folio/tests/check.rs` と `crates/folio/tests/face_constitution.rs` だけ）は着地済み。**base = main 41ed484。この契約の数はすべて base 41ed484 の実測（参考値）である**（規則の表の行 D-13）。0b0aafd から 41ed484 までの差は歯の file 2 本だけで src は同じなので、src の行数・変異・配信の組み立ての差・門の実測は 0b0aafd に本便の patch を当てた木でも同じ値だった。
- 並行の便との重なり: 一括 15（枝 docs/batch15・4c1134c・承認待ち・改訂 f〜h で要件 FR11 と FR13 の注の「s2-07l.473 は着地」を直し、問 7 を足した）は判断の記録・憲法・規則の表・要件書・語彙と `docs/design/` だけを書き換え、本便の write-set 16 本とは 1 本も重ならない。
- **前提（一括 15 の 🔴 問 7）。** 本便は一括 15 の問 7 の **案 A（要件 FR11 の保留の解除を維持し、folio2 の側の口は器の口 s2-07l.473 の着地を待たずに起こす）** を前提にする。**受付は問 7 の持ち主の裁定の後。** 案 B（FR11 を保留に戻す）なら本便は出さずに park する。案 C（導出物の形を判断の記録で改める）なら、形を改めた判断の記録の発効の後に、本便の形と歯 1・4・6 の期待を直して起こし直す。
- 分割: 1 便で運ぶ（§1 (h) の余地の表のとおり、触る src の 4 本はどれも size M の見積 300 を上回る余地を持つ）。ADR-16 決定 (7) の ⑤ の見積「1〜2 便」の 1 便の側である。
- 改訂: 改訂 c（2026-09-24・席）= 検証役の再検証（4bff3da・792/792・変異 19 通り一致・contracts/ への --check 0・通す）の文面 2 つを当てた（§1 の「この 1 つを除いて」の言い過ぎ・§5 に後の便が contracts/example.toml を write-set に入れる申し送り）。区間・write-set・verify・size・done は変えていない。
- 改訂: 改訂 b（2026-09-24 07:3x JST・起草役）= 検証役（~/.local/share/folio2/handoff-2026-09-24/d119-verify.md・直してから・blocking 1・文面 9）と席の裁定 3 つを当てた。blocking = verify の 6 行目を `--bin folio` に（folio は実行 file だけの crate）。席の裁定 = (1) §0 の前提に一括 15 の問 7 を書いた・(2) --check は導出元を持つ導出物だけで差分を数え、導出元の無い .toml は数えずに名を出す形に変えた（前は 1）・(3) folio2 自身の導出物 `contracts/` の下の example.toml を本便の write-set に `+` で足し、着地の後の main で --check が 0 になることを done に入れた。文面 = 歯 4 と歯 6 を実の置き場の本数に依らない性質の形に・器の台帳の今（s2-07l.473 は再開・open）と一括 15 の今（4c1134c）・命令の名の言い過ぎ・変異 M5 を組める形で撃ち直した・門の裁定の名指し・命令の名を床の定数と命令の口の 1 か所で結んだ（DERIVED_SUBCOMMAND）・歯の無かった断り 7 つに歯を足した（歯 2 と歯 3）・参考値の断り 3 つ・歯の file の頭の注を §1 (e) に。直したもの = write-set（15 → 16）・verify（6 行目）・done・title・歯の本数は 6 のまま（歯 2・3・4・6 の中身を広げた）・size M のまま・base を 41ed484 に。

## 1. 設計

### (a) いま起きていること（実測・base main 41ed484）

1. **導出物を組む口も差分を数える口も無い。** `crates/folio/src/main.rs` の命令の一覧は 12 本（check・inject・parts・face・figure・build・intake・hello・ceiling・schema・graph・serve）で、契約表から器の形の file を書く命令は無い。`crates/folio/src/site.rs`（配信の組み立て）も導出物を組まず数えない（toml の字は 1 つも無い）。
2. **導出物の形は床の定数に在る。** 設計ノートの欄の決まりの床の定数（`crates/folio/src/floor_note.rs` の FLOOR）の derived の節が形を定める: 形式 toml-subset・拡張子 .toml・1 文書 1 file・先頭 schema = 1（定数 EXTERNAL_HEAD）・配列 contract・行の欄は器の導出 file の欄の名そのまま + goal・goal は行の section が指す節の body の逐語を単一行に写す（各行を trim し空行を落とし空白 1 つで繋ぐ・引用符と逆斜線は escape しない）・section は文字列・空の一覧は欄名ごと省く・値の文法は text / list-of-text / number / bool・置き場は消費側が宣言する。
3. **床の定数の命令の名は配信の組み立てを指している。** 同じ derived の節の check の command の値が字 folio build --check である。写しは 4 か所に在る: `design-intent/design-note/schema.yaml` の生成区間（`folio schema --write` が書く）・凍結 anchor `tests/fixtures/schema/note-region.txt`（生成区間と byte 一致・`crates/folio/src/note.rs` の単体の歯と `crates/folio/tests/schema.rs` が読む）・床の凍結の土台 `tests/fixtures/floor_base/design-intent/design-note/schema.yaml`（floor_cases・freeze・ids・graph の歯が写して撃つ・床は生成区間の注でない欄を FLOOR と突き合わせる）・その土台から独立の script が組んだ節点の要約値の anchor `tests/fixtures/schema/node-digest-anchor.txt` の残差の 2 行（土台の全 byte を数える）。組み直す手順:

   ```
   git grep -n 'folio build --check' -- crates tests design-intent
   git grep -n 'derived-diff-zero' -- crates tests design-intent
   ```

4. **生成区間の注 3 つが未実装と書いている。** 同じ FLOOR の folio_check_note・derived_note・guards_note の 3 つが、導出物の差分 0 と導出の口を「未実装である」と書き、`crates/folio/tests/schema.rs` の歯 schema_design_note_region_matches_the_frozen_anchor_and_says_unimplemented（便 57）が 3 つの注に字「未実装である」が在ることを固定している。
5. **設計ノートと器の導出 file の読み手は床に 1 本ずつ在る。** `crates/folio/src/note.rs` の load_notes（design-note/ の直下の yaml を名の昇順に読む・重複キーは違反・読めないものは「まだ分からない」）と load_external（置き場の親 dir の `contracts/schema.toml` を行走査で読み、欄の名・要否・形を宣言の順に返す・読めない・知らない値は「まだ分からない」）と has_contract_table。どれも非公開である。面の生成器（`crates/folio/src/face_note.rs`）は同じ読みの写しを別に持つ（ADR-16 決定 (2)(キ) が便 ④ で 1 つにする）。
6. **命令の一覧は閉じている。** `crates/folio/tests/check.rs` の歯 p1_commands_closed_list（憲法 P-1 の機構）が folio --help の命令の一覧を 12 本の閉じた一覧と全数で突き合わせる。憲法 P-1 の機構の注は一覧を列挙しておらず、命令の名を持たない（注の字は変えなくてよい）。
7. **層の割り当ても閉じている。** `crates/folio/tests/modules.rs` の表 LAYERS（0 土台〜5 配る）が src の全 file を持ち、file は自分より下の層だけを名指せる（面の生成器は層 4）。
8. **器の読み手の今（folio2 の外の事実・判定点 ③ の前提）。** 起草役が器の repo（scribe2 main 596989b の写し）の契約表の読み手（table の parse の read_rows・pipe を撃たない小さな例の program）に、本便の導出器が書いた file を読ませると **欄 goal を「未知の key goal」で断る**（goal の行を除くと 1 行として読める）。器の欄の正本（table の FIELDS・18 欄）に goal は無い。器の台帳の s2-07l.473（導出 toml の goal を受ける口）は 2026-09-22 に着地せずに閉じられ（閉じた理由の字 = FR11 は M3 の入口で再判定・導出 toml の形は未定・要るときは folio2 から契約の形を添えて再要望）、folio2 の席の再要望で **2026-09-24 07:15 JST に再開された（open・未着地・契約未確定）**（器の台帳の読み取り・2026-09-24）。判断の記録 ADR-16 の文脈 (6) の「着地済み」はこの事実と食い違い、一括 15 の問 7 がその扱いを持ち主に問う（§0 の前提・§1 (i) の 4）。
9. **器の導出 file の今。** 器の repo の `contracts/schema.toml` は欄 18 本で、verify と done の要否が conditional、ほかに約束の行の欄の表を持つ。folio2 の repo の写しは 16 本で conditional を持たない。器の repo の写しを置き場の親に置いて本便の導出器を撃つと、床と同じ読み手が「欄 verify の need conditional を知らない」で「まだ分からない」を返す（§1 (i) の 5）。
10. base の workspace の nextest は全部緑（参考値 786 本・base 41ed484・`cargo nextest run --workspace --no-tests=fail --no-fail-fast`）。
11. **folio2 の版管理に導出物は無い。** `contracts/` の下は器の導出 file の写し schema.toml の 1 本だけで、folio2 で契約表を持つ設計ノートは見本 example.yaml（状態 example）の 1 本だけ（参考値・base 41ed484・`git ls-files contracts design-intent/design-note`）。folio2 の便の契約表は `docs/design/` の Markdown の区間で、導出の命令は読まない。

### (b) 直す先 — 独立の命令 folio derive

**命令の口。**

```
folio derive --dir <design-intent> --out <置き場> --write
folio derive --dir <design-intent> --out <置き場> --check
```

- --dir は既定 design-intent（ほかの命令と同じ）。--out は**既定なし**で必須（導出物の置き場は消費側が宣言する・床の定数の placement のとおり）。相対なら --dir からの相対・絶対ならそのまま（folio build と同じ）。--write と --check はどちらか 1 つが要る（clap の group）。
- 入力 = `<dir>/design-note/` の直下の設計ノート（床と同じ load_notes）と、置き場の親 dir の `contracts/schema.toml`（床と同じ load_external）。契約表の節（型 contract-table）を 1 つ以上持つ設計ノートだけを導出する（文書の状態 status では選ばない）。契約表を持つ設計ノートが 0 本なら器の導出 file を読まない。
- 出力 = 契約表を持つ設計ノート 1 本につき 1 file、名は `<文書 id>.toml`（文書 id は file 名の stem・床と同じ）。形は (a) の 2 の derived の節のとおりで、次の順で書く: 1 行目に schema = 1、次に行ごとに空行 1 つと見出し [[contract]]、続けて**器の導出 file の欄の宣言の順**に値の在る欄を 1 行ずつ（字の値は二重引用符で囲む・一覧は 1 行の角括弧の中に二重引用符の値を「, 」で繋ぐ）、行の最後に goal。無い欄・null・空白だけの字・空の一覧は欄名ごと省く。契約表の節が 2 つ以上在れば節の順に行を続ける。行の中の欄の書き順には依らない。
- goal = 行の section の値と n が等しい散文の節（型 prose）の body を、行ごとに trim（Rust の str の trim・全角の空白も落ちる）し、空行を落とし、空白 1 つで繋いだ 1 行。
- 見本（凍結の対の 2 行目・この節は二重引用符を置けないので「」で書く。実物は二重引用符）:

  ```
  [[contract]]
  id = 「b」
  title = 「差分を数える口」
  req = [「FR11」, 「AC9」]
  section = 「2」
  verify = [「cargo nextest run -p folio --test derive f119_」, 「cargo clippy --workspace --all-targets -- -D warnings」]
  size = 「S」
  done = 「1 字の差で非 0 になる」
  depends = [「a」]
  goal = 「差分を数える。」
  ```

**導出できない行（まだ分からない＝2・置き場は変えない）。** 次のどれか 1 つでも在れば、どの file も書かずに 2 で終わり、理由を 1 行で出す（全部か無しか・P-4.1）。どれも黙って飛ばさない。

1. `<dir>/design-note/` が dir として無い（symlink を含む）。
2. 設計ノートの読み手が違反（重複キー）か「まだ分からない」を積んだ。
3. 契約表を持つ設計ノートが在るのに器の導出 file が読めない・知らない値を持つ（床と同じ断り・FR10）。
4. 行が欄の表でない・行の欄が器の導出 file に無い・要否 required の欄が無いか空・欄の形が text / list と合わない・一覧の要素が字でない・同じ文書に同じ行 id が 2 度在る。
5. section が同じ文書の、本文を持つ散文の節を指さない。
6. 値（goal を含む）に二重引用符・逆斜線・改行が在る（escape しない形では書けない。床の定数の derived_note に 1 文で書く）。

**終了コード（3 値・P-4.2）。**

| 口 | 0 | 1 | 2 |
| --- | --- | --- | --- |
| --write | 書いた（違う file だけを書く・同じ file は書かない・導出元の無い .toml は消さずに名を出す） | （無い） | 導出できない・置き場の親 dir が無い・書けない |
| --check | 導出元を持つ導出物がすべて置き場の file と byte 一致する | 導出元を持つ導出物のうち 1 byte でも違う・置き場に無い（全部を名指す） | 導出できない・置き場が dir として無い |

- **差分を数えるのは導出元（契約表を持つ設計ノート）を持つ導出物だけ**（席の裁定）。置き場の直下に在る導出元の無い .toml（器の導出 file の写し schema.toml など）は差分に数えず、--write でも --check でも標準出力に「folio derive: 数えない: <名>（導出元なし）」を 1 file 1 行で出す（黙って落とさない・P-4.1）。終了コードは導出元を持つ file だけで決まる。ほかの拡張子の file は数えず、名も出さず、消さない（N-1.1）。これで置き場を器の導出 file と同じ dir（`contracts/`）にしても --check は 0 に届く。
- 置き場に無い導出物を 1 にするのは、版管理に置かれていないことが差分そのものだからである（配信の組み立ての --check は配信先に無い面を 2 にするが、面は置き場が消費側の宣言ではない）。
- 比較は byte 列の一致で、改行の読み替えをしない。

**folio2 自身の導出物（席の裁定）。** 本便の実装役は、実の置き場に `folio derive --dir design-intent --out ../contracts --write` を撃って folio2 自身の導出物を書き、版管理に置く。置き場は既に在る dir `contracts/`（器の導出 file の写しと同じ dir）、file 名は `contracts/<文書 id>.toml`（文書 id = `design-intent/design-note/` の下の設計ノートの file 名の stem）。base 41ed484 で契約表を持つ設計ノートは example.yaml だけなので、書くのは `contracts/` の下の example.toml の 1 本で、write-set に `+` で挙げる（参考値）。着地の後の main で `folio derive --dir design-intent --out ../contracts --check` が 0（schema.toml は数えないの 1 行）になることを done に入れ、歯 6 が版管理の導出物の集合と導出元を持つ file の集合の一致を見る。folio2 は器の受付に自分の導出物を読ませない（folio2 の契約表は Markdown の区間）ので、この導出物は差分 0 の検査を folio2 の中で回し続けるための材料である。

**置き場（module）と層。** 新しい file `crates/folio/src/` の下の derive.rs（層 3 導出する）に置く。持つもの = 口の列挙 Mode（Write / Check）・結果 Outcome（3 値と標準出力の行と標準エラーの行）・入口 run・全文書の導出 derive_all・1 文書の導出 derive_doc と行の導出・goal の単一行化・値の引用・導出元の無い .toml の列挙・書く口と数える口。`crates/folio/src/main.rs` に命令 Derive（旗 --dir・--out・--write・--check）と振り分けの枝と mod の 1 行を足す（build の次に置く＝一覧の順）。命令の名は `crates/folio/src/floor_note.rs` に足す命令の名の定数（DERIVED_SUBCOMMAND・§1 (c) の 1）を clap の name の属性で引く。`crates/folio/src/note.rs` は読み手 3 つ（load_notes・has_contract_table・load_external）と、読んだ設計ノートの型と器の導出 file の 1 欄の型（とその欄）を pub(crate) にするだけで、読みの本体と断りの字は 1 字も変えない（床と導出物が同じ読み手・同じ断りを持つ＝ADR-16 決定 (2)(キ) の便 ④ が読みの置き場を版管理の根へ改めれば導出物も同じに従う）。

**面の生成器も様式の file も呼ばないこと（辺の走査で確かめる）。** 新しい file は層 3 で、`crates/folio/tests/modules.rs` の表 LAYERS に 1 行（層 3）を足す。この歯（p106_edges_point_down）は src の全 file の crate の名指しを走査し、層 3 から層 4 の面の生成器（face_note ほか）や層 5 の配信の組み立て（site）を名指せば落ちる。様式の file（preview/ の下）を読まないことは歯 4 が、置き場から preview/ を外しても同じ byte を書くことで確かめる。

### (b2) folio build との関係

- 配信の組み立て（`crates/folio/src/site.rs` と `crates/folio/src/main.rs` の Build の枝）は 1 字も変えない。今も導出物を組まず数えないので、外す差分は無い。全部か無しか（面が 1 つでも導出できなければ配信先へ 1 byte も書かない）はそのまま。
- 導出物の検査を配信の組み立てに入れない（ADR-16 決定 (5)）。章の上限 12 を超えて面が導出できない設計ノートでも導出物は作れる（歯 4）。
- 起草役の実測: 実の置き場を base の binary と本便の後の binary でそれぞれ `folio build --write` へ別の配信先に書き、`diff -r` で比べて **差 0**（参考値 23 file・床 = 合格）。本便の後の binary を base の置き場（生成区間を直す前）に撃つと床が 1（違反 1 = 欄の決まりの写しの差）で書かない＝生成区間の直しは同じ便で運ぶ要がある。

### (c) 床の定数の命令の名の直しと写し

1. **床の定数。** `crates/folio/src/floor_note.rs` に次を足す: 拡張子（.toml）と配列の名（contract）の定数・命令の名の字（derive）を返す小さな macro 1 つ・その macro から引く命令の名の定数 DERIVED_SUBCOMMAND と、同じ macro から字を繋いで作る導出物の差分を数える命令の定数 DERIVED_CHECK_COMMAND（字 folio derive --check）。FLOOR の derived の節の extension・array・check の command の 3 つの葉を、字の literal からこの定数へ替える。拡張子と配列の名は導出の命令と写しが同じ定数を使い、命令の名は `crates/folio/src/main.rs` の命令 Derive の名（clap の name の属性）と写しの command が同じ macro の字から引く＝床の定数と命令の名が 1 か所で結ばれる（P-5.6・P-6.3）。値が変わるのは command だけ（folio build --check から folio derive --check へ）。旗の字 --check は命令 Derive の欄の名から clap が作り、歯 5 が撃って確かめる。
2. **注 3 つ。** 同じ FLOOR の注を次のとおり直す（`_note` の欄は床の突き合わせが読まない説明の注で、写しの生成区間には出る）。
   - folio_check_note の最後の文「このうち導出物の差分 0（derived-diff-zero）は未実装である = …（その間この検査の結果は「まだ分からない」として扱う・P-4.2）」を「このうち導出物の差分 0（derived-diff-zero）は、配信の組み立て（folio build）から切り離した独立の命令 folio derive --check が数える（便 119・判断の記録 ADR-16 決定 (5)）。folio check と folio build はこの検査を回さない」に。
   - derived_note の最後の文「導出物を組む口と差分を数える口は未実装である（…）」を「導出物を組む口（folio derive --write）と差分を数える口（folio derive --check）は便 119 で入った。どちらも面の生成器も様式の file も呼ばず、導出物の置き場（--out）は消費側が宣言する（既定なし・判断の記録 ADR-16 決定 (5)）。値に二重引用符・逆斜線・改行が在る行は、escape しない形では書けないので導出せず「まだ分からない」とする」に。
   - guards_note の「post のうち derived-diff-zero は未実装である（…「まだ無い検査」として寄せる）。」を「post のうち derived-diff-zero は folio derive --check が数える（便 119・事後の検出で、編集の時点で止める仕掛けの代わりにしない）。」に。
3. **写し 4 か所（全数・(a) の 3 の手順で数えた）。**
   - `design-intent/design-note/schema.yaml` の生成区間: `folio schema --write` の導出（4 行が変わる・区間の外は 1 byte も変えない）。書いた後の `folio schema --check` は 0・`folio check` は合格。
   - 凍結 anchor `tests/fixtures/schema/note-region.txt`: **実装から独立に**、字面の置き換え 4 か所（上の 1 の command と 2 の注 3 つ）を script で当てて直し、`folio schema --write` の導出と byte 一致することを確かめる（生成物の写しで anchor を作らない・P-10.2）。
   - 床の凍結の土台の `tests/fixtures/floor_base/design-intent/design-note/schema.yaml`: check の command の 1 行だけを直す（土台の注は前の版の字のまま・床は注を読まない）。直さないと floor_cases・freeze・ids の歯が土台の写しの差で落ちる（起草役の実測・(f)）。
   - 節点の要約値の anchor `tests/fixtures/schema/node-digest-anchor.txt`: 土台が 1 行変わるので、独立の script `tests/fixtures/schema/node-digest.py` の出力のうち**残差の 2 行だけ**が動く（残差の要約値と、合計の byte が 1 増える行・節点の行〔参考値 189 行・base 41ed484〕は 1 つも動かない）。script の出力で置き換え、`crates/folio/tests/graph.rs` の anchor の要約値の定数を直す（行数と byte 数は同じ・参考値 192 行 3,007 byte・base 41ed484・`wc -l -c` で測り直せる）。
4. **FR11 の契機との一致。** 要件 FR11 の契機（第 1.31 版）は命令の名を書かず「独立の命令・面の生成器も様式の file も呼ばない」と書く。歯 5 が、生成区間の check の command の命令の名が folio derive --check（配信の組み立てではない）であることと、その命令に --dir と --out を足せば撃てて 0 になることを見る（--dir と --out は消費側が足す・命令の名そのものには無い）。歯 4 と層の歯が「面の生成器も様式の file も呼ばない」を見る。
5. **写しに出ない変更。** 契約表・節の型・欄の集合・値域・判定の式は変えない。床の判定の結果は変わらない（床は注を読まず、command の値は写しと突き合わせるだけ）。

### (d) 採らなかった形

1. **導出物の検査を folio build --check の中に置いたまま、章の上限を超える面だけを飛ばす。** ADR-16 の選択肢で退けた形（配信の組み立ての全部か無しかが崩れる）。採らない。
2. **folio check（床）に導出物の差分を入れる。** 床は置き場の外（消費側の導出物の置き場）を知らず、--out を持たない。事後の検出を床に混ぜると、置き場を宣言していない repo の床が「まだ分からない」になる。採らない。
3. **導出器が自分で器の導出 file と設計ノートを読む（3 本目の読み手）。** 同じ読みの写しが 3 つになり、便 ④ の読みの置き場の改めが導出物に効かなくなる（P-6.3）。床の読み手を pub(crate) にして共有する。採らない。
4. **--out に既定を置く。** 置き場は消費側が宣言する（床の定数の placement）。ADR-16 決定 (3) が置き場から導出物の path を決めるのは便 ④ の骨格の命令である。本便では既定を持たない。採らない。
5. **二重引用符・逆斜線を escape して書く。** 器の値の読み手は escape を解かない（床の定数の derived_note・器の contract-source.md §2）。escape すると器が読む値が正本と字で違う。採らない（書けない値は 2）。
6. **導出元の無い .toml を --write が消す。** N-1.1。消すのは持ち主の手で、--write は名を出すだけ。採らない。
7. **凍結の対を新しい dir（要件書 AC9 の予定の名 tests/fixtures/contracts/）に置く。** 契約の型（`docs/design/delivery-template.md`）の write-set の書き方は新しい dir を作らない。既に在る `tests/fixtures/design-note/` に置き、AC9 の固定の材料の path の字は後の要件書の版で直す（§1 (i) の 3）。採らない。
8. **status が example の設計ノートを導出しない。** 状態で選ぶ規則は床の定数に無く、足すと黙って飛ばす形になる。契約表を持つ設計ノートは全部導出する。採らない。
9. **導出元の無い .toml を差分に数えて 1 にする（初版の形）。** 置き場を器の導出 file と同じ dir にすると、写しの schema.toml が導出元の無い導出物に数えられて --check が永久に 1 になる（検証役の実測）。消費側の置き場の決め方（便 ④）を縛らないよう、数えずに名を出す形にした（席の裁定）。採らない。

### (e) 歯（新しく 6 本・置き場は新しい file `crates/folio/tests/` の下の derive.rs）

関数名は f119_ で始める（verify の絞り込みの語・base で `git grep -n 'fn f119_'` は 0 件）。**凍結 anchor（P-10.1）** は新しい file 2 本の手書きの対で、生成器から独立している: `tests/fixtures/design-note/` の下の derive-anchor.yaml（契約表を 2 行持つ設計ノート・散文の節 2 つ・部品の表 1 つ。1 行目は空の一覧 depends と接頭辞 + の付いた write-set を持ち、2 行目は欄を器の宣言と違う順に書き、goal の元の本文は字下げ・空行・字 # と & を持つ）と derive-anchor.toml（期待する導出物・手で書いた file・参考値 23 行・`wc -l` で測り直せる）。

**歯の土台。** 一時 dir に design-intent/design-note/ を作って対の設計ノートを写し、repo の `contracts/schema.toml` をその親の contracts/ へ写して、命令を撃つ（一時 dir は消す）。歯 4 は実の design-intent の写し全部を使う。歯 6 は実の置き場を読むだけで、書く先は一時 dir と、版管理の導出物（`contracts/`）への --check だけ（書かない）。

1. **書いた導出物が凍結の対と byte 一致する。** --write → 0・置き場の file はちょうど derive-anchor.toml の 1 本・中身が対の toml と byte 一致。もう 1 度 --write → 0（書いた 0・変わらない 1）。--check → 0。**base では命令が無く落ちる＝RED。**
2. **1 byte の差と置き場に無い file は 1、導出元の無い file は数えない。** 置き場が無い → --check は 2。対の toml を置く → 0。行 a の id の字を 1 byte 変える → 1（DRIFT と file 名）。file を消す → 1（置き場に無い）。戻して、導出元の無い old.toml と器の導出 file の写し schema.toml と拡張子の違う keep.txt を置く → --check は 0 で、標準出力に「数えない: old.toml（導出元なし）」と「数えない: schema.toml（導出元なし）」が在り、keep.txt の名は無い。導出物をまた 1 byte 変える → 1（DRIFT と数えないの行の両方）。--write → 0（書いた 1 file と数えないの行）で、置き場の 4 本は消えず、導出物は対と byte 一致に戻る。置き場の親 dir が無い置き場へ --write → 2（親 dir が無い）で、親 dir を作らない。**base では落ちる＝RED。**
3. **書けない行は 2 で何も書かない。** 対の設計ノートに字面の変異を 1 か所だけ当てた 9 通り（title に二重引用符・title に逆斜線・title に改行・行 b に器の導出 file に無い欄・行 b の id を a に〔同じ行 id〕・行 a の section を部品の表の節へ・行 b の size を落とす・行 a の size を一覧に〔形 text の違い〕・行 a の req の要素を表に〔一覧の要素が字でない〕）で、名の昇順で先に来る書ける設計ノート（対の設計ノートの写し a-ok.yaml）が同じ置き場に在っても、--write → 2（まだ分からない と理由の字）・置き場には前から在る keep.txt だけ・--check も 2。器の導出 file を消した置き場 → --write は 2（contracts/schema.toml と 読めない）で置き場を作らない。設計ノートの置き場 design-note/ を symlink にした置き場 → --write は 2（design-note/ が dir として無い）で置き場を作らない。**base では落ちる＝RED。**
4. **面の生成器も様式の file も要らない。** 実の置き場の写しに、散文の節 13 本と契約表の節 1 つを持つ設計ノート big を足す（節 14 本＝面の章の上限 12 を超える）→ `folio face --face note --id big --write` は 2（上限 12）・folio derive --write は 0 で、置き場に big.toml が在り、その末尾は section 13 と goal の字（本文 13。）。置き場から preview/（様式の file）を外し、導出物の置き場も消してもう 1 度 --write → 0 で big.toml が前と byte 一致。**base では落ちる＝RED。**
5. **床の定数の命令の名は folio derive --check で、--dir と --out を足せば撃てる。** 実の `design-intent/design-note/schema.yaml` の生成区間の check の行から command の字を取り、字 folio derive --check と一致・先頭の 2 語が folio と derive・残りの語（--check）に対の置き場の --dir と --out を足して撃って 0。**base では字が folio build --check で落ちる＝RED。**
6. **実の置き場から導出でき、版管理の導出物と一致する。** 実の design-intent を一時 dir の置き場へ --write → 0・置き場に example.toml が在る・置き場の各 file は導出元の設計ノート（design-note/ の下の同じ stem の yaml）を持ち、schema = 1 と空行と [[contract]] で始まり、[[contract]] の数と goal の行の数が等しく 1 以上・--check → 0。続けて、版管理の `contracts/` の下の .toml から schema.toml を除いた集合が一時 dir の置き場の集合と一致し、`contracts/` への --check が 0（数えない: schema.toml（導出元なし）の行を持つ）。実の置き場の本数には依らない。**base では落ちる＝RED。**

歯の効き（起草役が `crates/folio/` の下の derive.rs に変異を 1 つずつ当てて測った・base 41ed484 と同じ src の木に本便の patch・撃つのは歯の file derive.rs と modules.rs・script は d119-draft-mut.py・逐語の記録は d119-draft-mut.out。組めない変異は compile-error と出して数えない形にしてあり、今の 19 通りに組めないものは無い）:

| 当てた形 | 落ちる歯 |
| --- | --- |
| 本便の形 | なし（6 本と層の歯が緑） |
| M1 空の一覧と空の値も書く | 1・2・5・6 |
| M2 欄を器の導出 file の逆順に書く | 1・2・4・5・6 |
| M3 goal の各行を trim しない | 1・2・5 |
| M4 導出元の無い .toml を差分に数える（初版の形） | 2・6 |
| M5 --check が置き場に無い file を数えない | 2 |
| M6 二重引用符を断らない | 3 |
| M7 --write が導出元の無い file を消す | 2 |
| M8 器の導出 file に無い欄を断らない | 3 |
| M9 必須の欄の欠けを断らない | 3 |
| M10 面の生成器（face_note の derive）で確かめてから導出する | 1・2・3・4・5 と層の歯 p106_edges_point_down |
| M11 導出元の無い .toml の名を出さない | 2・6 |
| M12 .toml 以外も導出元の無い file に数える | 2 |
| M13 設計ノートの置き場の symlink を断らない | 3 |
| M14 同じ行 id を断らない | 3 |
| M15 逆斜線を断らない | 3 |
| M16 改行を断らない | 3 |
| M17 置き場の親 dir が無くても作る | 2 |
| M18 一覧の要素が字でないのを黙って飛ばす | 3 |
| M19 形 text / list の突き合わせを外す | 3 |
| 生成区間の command を直さない（床の定数だけ直す・別の script で測った） | 5 と床の実の置き場の歯（`crates/folio/tests/check.rs` と `crates/folio/tests/schema.rs` の多数） |

### (f) 既存の歯のうち落ちるもの・凍結 anchor が動くか・直し方

起草役が、本便の src と写しと対だけを当てて歯の file を base のままにした木で workspace の nextest を撃つと、**落ちる既存の歯は 7 本**（参考値・base 41ed484 の 786 本のうち）。どれも本便の write-set の歯の file で直す。

| 歯（file・名） | 落ちる理由 | 直し方 |
| --- | --- | --- |
| `crates/folio/tests/check.rs` の p1_commands_closed_list | 命令の一覧が 13 本になる | 閉じた一覧に derive を build の次に足す（12 → 13）。採否を決める口の禁止の語の検査は変えない |
| `crates/folio/tests/modules.rs` の p106_layers_cover_every_module | 新しい file が層の表に無い | 表 LAYERS に derive を層 3 で 1 行 |
| `crates/folio/tests/schema.rs` の schema_design_note_region_matches_the_frozen_anchor_and_says_unimplemented | 注 3 つから「未実装である」が消える | 名を schema_design_note_region_matches_the_frozen_anchor_and_names_the_derive_command に改め、注 3 つが「未実装である」を含まず字 folio derive --check を含むことを見る形に。頭の注の 20 の行に便 119 の 1 行 |
| 同じ file の schema_check_matches_the_real_design_note_file_and_its_frozen_digest・schema_check_fails_on_one_byte_drift_inside_the_design_note_region・schema_write_restores_the_design_note_region_and_is_idempotent | 生成区間の byte 数と要約値が変わる | 定数 NOTE_REGION_BYTES と NOTE_REGION_SHA256 を直す（行数は同じ・参考値 137 行・base 41ed484・`wc -l tests/fixtures/schema/note-region.txt`）。値は anchor を sha256sum と wc -c で測り直した値（D-13 のとおり契約には写さない） |
| `crates/folio/tests/graph.rs` の f99_the_independent_script_matches_the_anchor | 土台の 1 行で残差の 2 行が動く | anchor を script の出力で置き換え、定数 F99_ANCHOR_SHA256 を直す（行数と byte 数の assert は同じ値のまま通る）。定数の上に便 119 の注 1 行 |

- `crates/folio/src/note.rs` の単体の歯 note_floor_derives_the_frozen_anchor_byte_for_byte は、anchor と FLOOR を同じ便で直すので緑のまま（anchor を直さないと落ちる）。
- **凍結 anchor が動くのは 2 本**（`tests/fixtures/schema/note-region.txt` と `tests/fixtures/schema/node-digest-anchor.txt`）と、凍結の土台の 1 行（`tests/fixtures/floor_base/design-intent/design-note/schema.yaml`）。どれも (c) の 3 のとおり、生成器の出力の写しではなく独立の手順（字面の置き換えの script・独立の Python の script）で作り直し、実装の出力と byte 一致を歯が見る。ほかの凍結 anchor（判断の記録の欄の決まりの写し・天井の束・所見・面の凍結 fixture・節点の 189 行）は 1 byte も動かない。
- 本便の後の木で workspace の nextest は全部緑（参考値 792 本 = base 41ed484 の 786 + 新しい歯 6）・clippy 0 警告・`folio check` 合格・`folio schema --check` 9 file 一致・`folio derive --dir design-intent --out ../contracts --check` 0。

### (g) 門（規則の表の開発規律行 D-12）

本便は設計ノートの欄の決まり（`design-intent/design-note/schema.yaml`）の生成区間を書き換えるので天井の門の対象で、実測は **2（まだ分からない・印が古い）**（§0 の 門）。印は本便の前から古い（base の binary に設計ノートの欄の決まり 1 本だけを渡しても 2）。行 D-12 は「止める・まだ分からない のときは出さず、所見の解消か持ち主の裁定を先に取る」と定めるので、**器へ出す前に、席が持ち主の裁定を名指すか、天井の周を回して印を新しくする**。先例は便 101・103（2026-09-22）と便 117（2026-09-24）で、持ち主の裁定（2026-09-22 00:0x JST・対話面 R-8・台帳 f2-648 notes）を行 D-12 の裁定として受け、印が古いまま出した。本便にその裁定を当てるかどうかは席が決め、当てるなら裁定 id を台帳の notes に記帳する（起草役は決めない）。本便が書き換える正本の中身は生成区間の 4 行（導出物の検査の命令の名 1 つと未実装の注 3 つ）だけで、人が書く字は 1 字も変わらない。`contracts/` の下の example.toml は設計文書の置き場の外の生成物である。

### (h) 大きさ・verify と done の対応

1. **write-set の印。** 新しい file 5 本に `+`（`crates/folio/src/` の下の derive.rs・`crates/folio/tests/` の下の derive.rs・`tests/fixtures/design-note/` の下の derive-anchor.yaml と derive-anchor.toml・`contracts/` の下の example.toml）。書き換える 11 本は印なし。`-`（行が減る file）・`~`（着地で消える file）は当たらない。新しい dir は作らない。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の印なしの .rs と新しい .rs。測り方は各行の字数を 120 で割って切り上げ（空行は 1）、全行で足す（`wc -l` ではない・script は §1 (j)）。size M の見積は 1 file あたり 300。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 本便の後（参考値） |
| --- | --- | --- | --- |
| `crates/folio/src/main.rs` | 593 | 907 | 632 |
| `crates/folio/src/note.rs` | 903 | 597 | 903 |
| `crates/folio/src/floor_note.rs` | 473 | 1,027 | 487 |
| 新しい derive.rs | 0 | 1,500 | 330 |

   core の合計の上限（器の行 R-C4-1）に対する見積は 4 本 × 300 で、src の全体（参考値 2 万 6 千行台）に足しても上限の半分に届かない。歯の file は src の外なので余地を測らない（参考に、新しい歯の file は同じ式で 431 行・参考値）。

3. **size は M。** 変える src は 4 本（うち新しい 1 本が約 330 行）。
4. **verify は 7 行**で、done の 7 つの塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --test derive f119_` = (e) の新しい歯 6 本。
   2. `cargo nextest run -p folio --test schema` = 設計ノートの欄の決まりの生成区間と凍結 anchor の歯（直した 4 本を含む全部）。
   3. `cargo nextest run -p folio --test check p1_commands_closed_list` = 命令の閉じた一覧の歯。
   4. `cargo nextest run -p folio --test graph f99_` = 節点の要約値の独立の script の anchor の歯（直した 1 本を含む f99_ の全部）。
   5. `cargo nextest run -p folio --test modules` = 層の割り当てと辺の歯。
   6. `cargo nextest run -p folio --bin folio note_floor` = `crates/folio/src/note.rs` の単体の歯（folio は実行 file だけの crate なので `--lib` ではなく `--bin folio`・床の定数と凍結 anchor の byte 一致）。
   7. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** verify が `--test` で名指す歯の file 5 本（derive・schema・check・graph・modules）と、`--bin folio` の絞り込みの語 note_floor を関数名に持つ src の file（`crates/folio/src/note.rs` の 1 本だけ・base で `git grep -n 'fn [a-z0-9_]*note_floor' -- crates/folio/src` が 1 件）は、すべて write-set に在る。floor_cases・freeze・ids の歯は本文を変えず、土台の写しの 1 行で緑に戻るので write-set に入れず、verify でも名指さない＝共通の検証（`.vessel.toml` の common-verify の workspace の nextest）が走らせる。

### (i) 本便が運ばないもの・言えないこと・まだ分からない点・撤退条件

1. **運ばないもの。** 要件書・判断の記録・憲法・規則の表・語彙の字（FR11 の注の「この要件の口は未実装である（2026-09-23 時点）」を含む）。器の導出 file の置き場を版管理の根で解くこと（便 ④）。骨格の命令と導出物の既定の置き場（便 ④）。器の導出 file の読み手に要否 conditional を足すこと。憲法 P-6 の機構の注に folio derive --check を足すこと（注は再生成の検査の命令を並べるが、本便は憲法を運ばない・席が版 B で直す）。面・配信の組み立て・天井。外部 crate。
2. **言えないこと。** 歯は、folio2 の読み手と凍結の対の範囲で導出物の形と差分の数えを見る。器の読み手がその file を受けるかは folio2 の歯では言えない（次の 4）。goal の単一行化の trim が全角の空白も落とすことは、凍結の対が全角の空白で始まる行を持たないので歯が拾わない（Rust の str の trim の定義どおり）。§1 (b) の断りと 3 値は、この 1 つと歯の無い 3 つ（(b) の 2 の読み手の違反〔重複キー〕・(b) の 4 の「行が欄の表でない」・表の同じ種の断り）を除いて歯 2 と歯 3 と変異 M1〜M19 が 1 つずつ当てている（歯の無い 3 つは読み手の違反の種で、床の歯が同じ読み手を別に当てている）。
3. **まだ分からない — 要件書 AC9 の固定の材料の字。** AC9 の red_test は固定の材料を tests/fixtures/contracts/（予定の名）の「契約表と期待導出物の対 + 1 字ずらした導出物」と書く。本便は新しい dir を作らないので `tests/fixtures/design-note/` の下の derive-anchor の対に置き、1 字ずらした導出物は file として置かずに歯 2 がその場で作る。AC9 と FR11 の注の字を実物に揃えるのは、着地の後の要件書の版（席の起草・持ち主の承認）である。
4. **まだ分からない — 判定点 ③（器の受付に実物を読ませる）。** folio2 の外の事実なので、folio2 の受入では利用者の台帳の記録が在れば着地、無ければ「まだ分からない」の測定値として書く（ADR-16 決定 (5)・ADR-3 決定 (5) と同じ形）。**起草役の実測では、今の器の読み手は欄 goal を未知の key として断る**（§1 (a) の 8）。器の口 s2-07l.473 は 2026-09-24 07:15 JST に再開された（open・未着地・契約未確定）。その notes が導出物の形へ求める字（欄名は goal のまま・空の一覧の欄名の省略は受け口の側で任意欄にして受ける・先頭の版の宣言の key 名を pin する歯を受け口に持つ）は本便の形と食い違わない（検証役の読み取り）。判断の記録 ADR-16 の文脈 (6) と決定 (5) の「着地」の字は、一括 15 の問 7 が持ち主に問う（§0 の前提）。本便は ADR-3 決定 (4) と要件 FR11 の規範文の形（goal を持つ）のまま作る。
5. **まだ分からない — 判定点 ② の前提（器の導出 file の今の形）。** 器の repo の `contracts/schema.toml` は要否 conditional と約束の行の欄の表を持ち、床と導出器の共通の読み手は conditional を知らない値として「まだ分からない」を返す（§1 (a) の 9）。利用者の repo で床（判定点 ①）も導出物（判定点 ②）も、この読み手を器の今の形に合わせる便（FR10 の読み手・ADR-16 の列の外）が先に要る。folio2 の repo の写しの更新も同じ便で扱う。
6. **撤退条件。** (1) 本便の後に (f) の 7 本のほかに既存の歯が 1 本でも落ちたら、その歯の本文を直さずに止めて席へ返す。(2) `folio build` の出力が本便の前後で 1 byte でも変わったら止めて席へ返す（(b2)）。(3) 受付の時点の main で `crates/folio/src/floor_note.rs` の derived の節か `crates/folio/src/note.rs` の 3 つの読み手の周りが書き換わっていたら、変更の逐語と写しの 4 か所を測り直してから運ぶ。(4) 器の側が goal を受けない形に決まったら（上の 4）、導出物の形を改める判断の記録を先に起こし、本便の歯 1・4・6 の期待を直す便は別に起こす。(5) 一括 15 の問 7 が案 B なら受付に出さずに park し、案 C なら §0 の前提のとおり起こし直す。(6) 実装役が `contracts/` の下に書く導出物が example.toml の 1 本でない（受付の時点の main で契約表を持つ設計ノートが増えた）なら、write-set の `+` の行を実物に合わせてから運ぶ。

### (j) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草役の実測 patch は `~/.local/share/folio2/handoff-2026-09-24/d119-measured.patch`（base 41ed484 に `git apply --check` が通る・repo には入れない）。起草の記録は同じ dir の d119-draft.md、変異の script は d119-draft-mut.py、anchor の置き換えの script は d119-draft-anchor.py、行数の script は d119-draft-lines.py、器の読み手に読ませる例の program は d119-draft-s2-readtoml.rs（器の repo の写しの crates/scribe2/examples/ に置いて cargo build --example で組む）。

1. base の写し: `git worktree add --detach <写し> 41ed484`。
2. RED: 歯の file（`crates/folio/tests/` の下の derive.rs）と凍結の対 2 本だけを当てて `cargo nextest run -p folio --test derive f119_ --no-fail-fast` → 6 本とも落ちる。
3. 落ちる既存の歯: patch のうち `crates/folio/src`・`design-intent`・`tests/fixtures`・`contracts` の差分だけを当てて workspace の nextest → (f) の 7 本だけが落ちる。
4. 全部: patch の全部を当てて `cargo nextest run --workspace --no-tests=fail --no-fail-fast`（全部緑）・`cargo clippy --workspace --all-targets -- -D warnings`（0 警告）・`folio check --dir design-intent`（合格）・`folio schema --dir design-intent --check`（0）・`folio derive --dir design-intent --out ../contracts --check`（0）。
5. anchor の独立: base の `tests/fixtures/schema/note-region.txt` に d119-draft-anchor.py を当て、本便の後の `design-intent/design-note/schema.yaml` の生成区間と byte 一致を見る。`python3 tests/fixtures/schema/node-digest.py tests/fixtures/floor_base/design-intent` の出力と `tests/fixtures/schema/node-digest-anchor.txt` の差が 0。
6. 変異: `python3 d119-draft-mut.py <scratchpad>`（(e) の表）。
7. 出力の差: (b2) の手順（base と後の binary で `folio build --write` を別の配信先へ・`diff -r`）。
8. 門: `folio ceiling --gate --write-set <write-set の 16 本（接頭辞を剥がす）>`。
9. 余地: `python3 d119-draft-lines.py <file…>`。

## 2. 範囲

- 入れる: 新しい file `crates/folio/src/` の下の derive.rs（命令の本体）。`crates/folio/src/main.rs` の命令 Derive と振り分けの枝と mod の 1 行。`crates/folio/src/note.rs` の読み手 3 つと型 2 つ（とその欄）の pub(crate) と頭の注の 1 行の書き直し。`crates/folio/src/floor_note.rs` の定数 4 つと命令の名の macro 1 つと FLOOR の葉 3 つと注 3 つ。`crates/folio/src/main.rs` の命令 Derive の name の属性 1 行。`design-intent/design-note/schema.yaml` の生成区間の 4 行（`folio schema --write`）。`tests/fixtures/schema/note-region.txt` の 4 行。`tests/fixtures/floor_base/design-intent/design-note/schema.yaml` の 1 行。`tests/fixtures/schema/node-digest-anchor.txt` の残差の 2 行。新しい凍結の対 2 本。`folio derive --write` が書く folio2 自身の導出物（`contracts/` の下の example.toml）。新しい歯の file（f119_ の 6 本）。`crates/folio/tests/check.rs` の閉じた一覧 1 語・`crates/folio/tests/modules.rs` の表 1 行・`crates/folio/tests/schema.rs` の定数 2 つと歯 1 本の名と本文と注・`crates/folio/tests/graph.rs` の定数 1 つと注 1 行。
- 入れない: 配信の組み立て（`crates/folio/src/site.rs`）・面の生成器（`crates/folio/src/face_note.rs` の同じ読みの写しを含む）・床の判定と断りの字・器の導出 file の読みの置き場と要否の値域・`contracts/schema.toml` の写し・要件書と判断の記録と憲法（P-6 の機構の注を含む）と規則の表と語彙・天井の正本・ほかの 8 file の生成区間・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| cmd | 導出の命令 | 新しい file `crates/folio/src/` の下の derive.rs と `crates/folio/src/main.rs` の命令 Derive（--write / --check） |
| reader | 読み手 | `crates/folio/src/note.rs` の load_notes・has_contract_table・load_external（床と共有・本便は見え方だけを広げる） |
| floor | 床の定数 | `crates/folio/src/floor_note.rs` の derived の節（拡張子・配列の名・命令の名の定数と注 3 つ） |
| region | 生成区間と写し | `design-intent/design-note/schema.yaml` の生成区間・`tests/fixtures/schema/note-region.txt`・土台の 1 行・`tests/fixtures/schema/node-digest-anchor.txt` |
| pair | 凍結の対 | `tests/fixtures/design-note/` の下の derive-anchor.yaml と derive-anchor.toml（手書き） |
| own | folio2 自身の導出物 | `contracts/` の下の example.toml（`folio derive --write` の出力・手で直さない） |
| teeth | 歯 | 新しい歯の file の f119_ の 6 本と、直す既存の歯 7 本 |

## 4. 検査（歯）

§1 (e)(f) と (h) の 4 のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない（読みは床と同じ行走査と yaml の読み手）。新しい dir は無い。
- 前提の着地: ADR-15 の列・一括 14・要件書の版 A（第 1.31 版）。便 117・118 は着地済みで write-set は重ならない。
- 並行の便: 一括 15（docs/batch15）とは file が重ならない。ADR-16 の便 ①〜④ のうち未起草の ② ③ ④ は、`crates/folio/src/note.rs` の読みの置き場（④）と床の定数（②）を触りうるので、起草のときに本便との重なりを見る。
- 受付の前提: 一括 15 の問 7 の持ち主の裁定（案 A）。§0 の前提のとおり。
- 便 ④ の起草への申し送り: 導出物の既定の置き場を決めるとき、導出元の無い .toml は数えない（§1 (b)）ので器の導出 file と同じ dir でもよい。
- 後の便への申し送り: 本便の後は版管理に folio2 自身の導出物 `contracts/example.toml` が在るので、`design-intent/design-note/example.yaml` の契約表か行 a の section が指す節の本文を変える便と、`contracts/schema.toml` の欄の順を変える便（器の導出 file の写しの更新・§1 (i) の 5 の FR10 の読み手の便を含む）は、`contracts/example.toml` を write-set に入れる（入れないと着地の後の main で `folio derive --check` が 1 を返す）。
- 本便の着地の後に席が見ること: 憲法 P-6 の機構の注（版 B）・§1 (i) の 3（AC9 と FR11 の注の字）・4（器の goal の受け口・ADR-16 の文脈 (6) の字）・5（器の導出 file の要否 conditional）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "dr"
title = "判断の記録 ADR-16 決定 (5) と (7) の ⑤（裁定 id = 台帳 f2-648 notes 2026-09-23 17:39 JST）の 1 便: 導出物の独立の命令 folio derive を足す。新しい file crates/folio/src/derive.rs（層 3）が、床と同じ読み手（crates/folio/src/note.rs の設計ノートと器の導出 file の読み手を pub(crate) にして共有）で契約表を持つ設計ノートを読み、設計ノートの欄の決まりの床の定数の derived の節の形（1 文書 1 file の文書 id.toml・先頭 schema = 1・行ごとに見出し contract・欄は器の導出 file の宣言の順・値の無い欄と空の一覧は省く・行の末尾に section が指す散文の節の body を単一行にした goal）の導出物を --write で置き場（--out・既定なし・消費側が宣言）へ書き（違う file だけ）、--check で導出元を持つ導出物だけを置き場と byte 比較する（一致 0・差分か置き場に無いは 1・導出できないか置き場が無いは 2）。置き場の導出元の無い toml（器の導出 file の写しなど）は差分に数えず消さず、数えない の 1 行で名を出す。値に二重引用符・逆斜線・改行が在る行・器の導出 file に無い欄・同じ行 id・必須の欄の欠け・形の違い・散文でない節を指す section は、どの file も書かずに 2。面の生成器も様式の file も呼ばない（層の歯と、章の上限 12 を超える設計ノートでも導出できる歯）。配信の組み立て folio build は 1 字も変えない。床の定数 crates/folio/src/floor_note.rs の導出物の検査の命令の名を folio derive --check に直し（命令の名は 1 つの macro から命令の口と写しの両方が引く・拡張子と配列の名も定数にする）、未実装と書いていた注 3 つを直し、写し 4 か所（設計ノートの欄の決まりの生成区間・凍結 anchor tests/fixtures/schema/note-region.txt・床の凍結の土台の 1 行・節点の要約値の anchor の残差の 2 行）を揃える。folio2 自身の導出物 contracts/example.toml を folio derive --write で書いて版管理に置く。歯は新しい crates/folio/tests/derive.rs に 6 本と手書きの凍結の対 2 本（tests/fixtures/design-note/derive-anchor.yaml と derive-anchor.toml）で、落ちる既存の歯 7 本（命令の閉じた一覧・層の表・生成区間の定数と注の歯 4 本・節点の要約値の anchor の歯）を直す。受付は一括 15 の問 7 の持ち主の裁定（案 A）の後"
req = ["FR11", "FR19"]
section = "1"
write-set = ["+crates/folio/src/derive.rs", "crates/folio/src/main.rs", "crates/folio/src/note.rs", "crates/folio/src/floor_note.rs", "+crates/folio/tests/derive.rs", "crates/folio/tests/check.rs", "crates/folio/tests/schema.rs", "crates/folio/tests/graph.rs", "crates/folio/tests/modules.rs", "design-intent/design-note/schema.yaml", "tests/fixtures/schema/note-region.txt", "tests/fixtures/floor_base/design-intent/design-note/schema.yaml", "tests/fixtures/schema/node-digest-anchor.txt", "+tests/fixtures/design-note/derive-anchor.yaml", "+tests/fixtures/design-note/derive-anchor.toml", "+contracts/example.toml"]
verify = ["cargo nextest run -p folio --test derive f119_", "cargo nextest run -p folio --test schema", "cargo nextest run -p folio --test check p1_commands_closed_list", "cargo nextest run -p folio --test graph f99_", "cargo nextest run -p folio --test modules", "cargo nextest run -p folio --bin folio note_floor", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "f119_ の歯 6 本（凍結の対の設計ノートから書いた導出物が手書きの derive-anchor.toml と byte 一致し check が 0／1 byte の差と置き場に無い file で check が 1・導出元の無い toml は数えずに名を出し終了コードに効かない・write は他の file を消さない・置き場が無い check と置き場の親 dir が無い write は 2／書けない行 9 通りと器の導出 file の無い置き場と symlink の設計ノートの置き場で write が 2 になり書ける設計ノートが同じ置き場に在っても何も書かない／章の上限を超えて面が 2 になる設計ノートでも導出でき様式の file を外しても同じ byte／設計ノートの欄の決まりの生成区間の check の command が folio derive --check で dir と out を足せば撃てて 0／実の置き場から導出元を持つ file だけを導出でき check が 0 で、版管理の contracts/ の導出物の集合がそれと一致して contracts/ への check が 0）が緑、設計ノートの欄の決まりの生成区間と凍結 anchor の歯が緑、命令の閉じた一覧の歯が 13 本で緑、節点の要約値の独立の script の anchor の歯が緑、層の割り当てと辺の歯が緑、note.rs の床の定数と凍結 anchor の byte 一致の単体の歯が緑、clippy が 0 警告で、workspace の nextest が全部緑（floor_cases・freeze・ids・配信の組み立ての歯を含む）で CI が通り、着地の後の main で folio derive --dir design-intent --out ../contracts --check が 0 を返す"
<!-- contracts:end -->
