# ADR-11 下調べ — 実装の定数と design-intent の YAML の二重の記述（全数）

> 判断の記録 ADR-11（f2-648.63）の下調べ。独立の調査役（opus・読み取り専用）が 2026-09-19 に実測した控えで、設計文書ではない（規則を持たない）。行番号は調査時点のもの。

- 対象: main 50639e3 と同じ src（枝 docs/f2-648.63-adr11 の作業ツリーで実測・2026-09-19）
- 読み取り専用。file の変更・git 操作は行っていない。
- 実測 = grep と Read で確かめた行。**推測**と書いた箇所以外はすべて実測。
- 行番号は調査時点の HEAD のもの。

---

## 0. 前提の実測（分類の土台）

| 正本 file | `schema:` 節（節の一覧を YAML が持つか） | 節の一覧の正本 |
|---|---|---|
| `constitution.yaml` | **在る**（`schema.top_level` 9 個・15 行目） | YAML（類 C） |
| `rules.yaml` | **在る**（`schema.top_level` 3 個・8 行目） | YAML（類 C） |
| `vocabulary.yaml` | **無い** | 実装 `check.rs:63 VOCABULARY_TOP_LEVEL`（類 B） |
| `srs.yaml` | **無い**（最上位キー = meta/goals/scope/actors/outputs/rail/verdicts/requirements/nonfunctional/acceptance/not_frozen/constraints/sources/glossary_pointer/figures の 15 個。`scope_m1` は許すが未使用） | 実装 `check.rs:39 SRS_TOP_LEVEL`（類 B） |
| `index.yaml` | **無い**（meta/audience/shelf/lanes/intake） | 実装 `entrance.rs:18 INDEX_TOP_LEVEL`（類 B） |
| `intake.yaml` | **無い**（meta/answers/targets/questions/sheet） | 実装 `intake.rs:18 INTAKE_TOP_LEVEL`（類 B） |
| `ceiling.yaml` | **無い**（meta/verdicts/weights/documents/viewpoints/finding/record/bundle） | 実装 `ceiling.rs:19 CEILING_TOP_LEVEL`（類 B） |
| `adr/schema.yaml` | 在る＝**生成区間**（9〜127 行に `# folio:schema:begin/end`） | 実装 `adr.rs:97 FLOOR`（解決済み） |
| `design-note/schema.yaml` | 在る＝**生成区間**（20〜156 行） | 実装 `note.rs:132 FLOOR`（解決済み） |
| `preview/parts.json` | 部品目録 | **parts.json が正本**。`crates/folio/build.rs` が `OUT_DIR/parts_catalog.rs` へ enum を導出（`parts.rs:16` の `mod catalog`）＝実装側は導出物（類 C）。ただし手書きの上限 3 本と図の名札 1 本だけが写し（類 A） |

比較に使う関数（実測）:

| 関数 | file:行 | 比較の種類 |
|---|---|---|
| `same_list` | `ceiling.rs:314` | 順つき一致（過不足＋順） |
| `same_set` | `ceiling.rs:336` | 集合一致（過不足） |
| `contains_all` | `ceiling.rs:354` | 包含（床の定数を全部含むか・余分は見ない。ただし actual が空なら黙る） |
| `exact` | `face_index.rs:396` | 行の id が表と過不足なく一致＋重複禁止（順は表が決める） |
| `lookup` | `face.rs:262` | 片方向（YAML の値が表に在るか。表の側の余りは見ない） |
| `floor_diff` | `schema.rs:77` | 床の木と YAML の木の欄ごとの差分（生成区間の機構） |
| `catalog_matches` | `parts.rs:128` | parts.json と組み立て時の導出の順つき完全一致 |

---

## 1. 類 A（二重の記述・床か生成器が一致を確かめている）

### 1-a. `folio check`（床）の中で確かめている — 対 `ceiling.yaml`

天井の正本は「人が書く YAML」で、8 本の定数がその中身を固定している。★ = 行の id の固定（純粋な一覧の写しと区別）。

| # | 定数 | 定数の file:行 | YAML 側の欄の道 | 比較 | 比較の file:行 |
|---|---|---|---|---|---|
| A1 | `VERDICT_VALUES` | `ceiling.rs:31` | `ceiling.yaml verdicts.values` | 順つき一致 | `ceiling.rs:83` |
| A2 | ★`VIEWPOINT_IDS` | `ceiling.rs:34` | `ceiling.yaml viewpoints[].id` | 順つき一致 | `ceiling.rs:136` |
| A3 | ★`DOCUMENT_IDS` | `ceiling.rs:37` | `ceiling.yaml documents[].id` | 集合一致 | `ceiling.rs:111` |
| A4 | `FINDING_REQUIRED` | `ceiling.rs:50` | `ceiling.yaml finding.required` | 包含 | `ceiling.rs:154` |
| A5 | `PLACE_REQUIRED` | `ceiling.rs:53` | `ceiling.yaml finding.place.required` | 包含 | `ceiling.rs:164` |
| A6 | `REFUTE_VALUES` | `ceiling.rs:56` | `ceiling.yaml finding.refute.values` | 集合一致 | `ceiling.rs:175` |
| A7 | `RECORD_REQUIRED` | `ceiling.rs:59` | `ceiling.yaml record.required` | 包含 | `ceiling.rs:188` |
| A8 | `BUNDLE_CONTENTS` | `ceiling.rs:62` | `ceiling.yaml bundle.contents` | 包含 | `ceiling.rs:200` |

★の 2 本（A2・A3）は「行の id の固定」で、A1・A4〜A8 とは性質が違う。A1・A4〜A8 は **値の一覧を 2 回書いている**だけだが、A2・A3 は **行（内容つきの表）の id 集合**を定数が固定している＝YAML 側は `name`・`reader`・`question`・`reads`・`file`・`note` という中身を持ち、その id だけが実装に縛られている。生成区間へ移すなら「行そのもの」ではなく「id の列だけ」を生成区間にする必要がある。

### 1-b. `folio check`（床）の中で確かめている — 対 `constitution.yaml` / `rules.yaml`

| # | 定数 | 定数の file:行 | YAML 側の欄の道 | 比較 | 比較の file:行 |
|---|---|---|---|---|---|
| A9 | `adr.rs FLOOR.enums.retreat_kind`（= `adr.rs:50 RETREAT_KIND`） | `adr.rs:50` / `adr.rs:97 FLOOR` | `constitution.yaml schema.enums.retreat_kind` | 長さ＋字面＋順の一致 | `link.rs:99-131`（fn `retreat_kind`） |
| A10 | `adr.rs FLOOR.anchor.scope_minimum` = `[schema, precedence, articles]` | `adr.rs:353-356` | `constitution.yaml schema.amendment_scope` | 包含（下限） | `link.rs:146-158`（fn `amendment_range`） |
| A11 | `adr.rs FLOOR.enums.surface` = `["R-8"]`（= `adr.rs:52 SURFACE`） | `adr.rs:52` | `rules.yaml thresholds/discipline の行 id 空間` | 存在（包含） | `link.rs:134-146`（fn `surface`） |

A9 の違反の字面は「床の `retreat_kind` […] が憲法の値域 […] と食い違う（**憲法が正・床の定数を直す**）」（`link.rs:157`）＝この 1 本だけは「YAML が正本」と明言されている。にもかかわらず同じ値が `adr/schema.yaml` の生成区間（正本 = 実装の定数）へも書かれる＝**正本が 2 つ在ると読める**。ADR-11 の論点として最も鋭い箇所。

### 1-c. 面の生成器（`folio face`）の中で確かめている — 対 `index.yaml` / `constitution.yaml`

床（`folio check`）ではなく面の生成器が確かめる。落ちたときは違反ではなく「まだ分からない」（面が出ない）。

| # | 定数 | 定数の file:行 | YAML 側の欄の道 | 比較 | 比較の file:行 |
|---|---|---|---|---|---|
| A12 | ★`SHELF_DOCS`（4 行） | `face.rs:598` | `index.yaml shelf.documents[].id` | 過不足なし＋重複禁止（`exact`） | `face_index.rs:499` |
| A13 | ★`ANNEXES`（2 行） | `face.rs:646` | `index.yaml shelf.annexes[].id` | 同上 | `face_index.rs:529` |
| A14 | ★`SHELF_RELATIONS`（4 行） | `face.rs:638` | `index.yaml shelf.relations[].id` | 同上 | `face_index.rs:564` |
| A15 | ★`SHELF_LEGEND`（4 行） | `face.rs:649` | `index.yaml shelf.legend[].id` | 片方向の包含（`lookup`） | `face_index.rs:757` |
| A16 | `TIERS`（3 行） | `face.rs:461` | `constitution.yaml schema.enums.tier` | 各値が表に在る＋件数一致＋重複禁止＝集合一致 | `face_constitution.rs:152-164` |

A12〜A15 はすべて ★（行の id の固定）。`index.yaml` の棚の行は `type` / `use` / `absent` / `label` / `hint` / `text` という人が書く中身を持ち、id だけが `face.rs` の表に縛られている。A12〜A14 は `exact` なので **YAML の行を 1 つ増やしても減らしても面が出ない**。

### 1-d. 命令 `folio ceiling` の中で確かめている — 対 `ceiling.yaml`

| # | 定数 | 定数の file:行 | YAML 側の欄の道 | 比較 | 比較の file:行 |
|---|---|---|---|---|---|
| A17 | `bundle.rs CONTENTS` | `bundle.rs:27` | `ceiling.yaml bundle.contents` | 順つき等値（`!=`） | `bundle.rs:200` |
| A18 | `bundle.rs DIGEST` | `bundle.rs:30` | `ceiling.yaml bundle.digest` | 等値 | `bundle.rs:201` |

A17 は A8（`ceiling.rs BUNDLE_CONTENTS`）と**同じ一覧の 3 枚目の写し**（ceiling.yaml・ceiling.rs・bundle.rs）。しかも比較の厳しさが違う（床は包含・命令は順つき等値）。

### 1-e. 部品目録（`parts.json`）との突き合わせ — 歯（test）だけが確かめている

| # | 定数 | 定数の file:行 | parts.json 側 | 比較 | 比較の file:行 |
|---|---|---|---|---|---|
| A19 | `MAX_RAIL_NODES` = 7 | `face.rs:689` | `components.pipeline-rail.max_nodes`（parts.json:184） | 等値 | `face_constitution.rs:1097` / `face_srs.rs:1321`（src の unit test） |
| A20 | `MAX_STATE_NODES` = 4 | `face.rs:691` | `components.state-strip.max_nodes`（parts.json:211） | 等値 | `face_srs.rs:1321` |
| A21 | `MAX_PER_BAND` = 4 | `face.rs:693` | `components.context-band.max_per_band`（parts.json:198） | 等値 | `face_srs.rs:1321` |
| A22 | `FIGURE_LABELS`（5 組） | `face.rs:921` | `figure_body_classes.type_ids`（parts.json:280） | 順つき完全一致 | `face_note.rs:997`（src の unit test） |

A19〜A22 は `folio check` では数えない＝**歯（`cargo test`）を回さないと食い違いが出ない**。parts.json は人が書く（`build.rs` が読む正本）ので、これは P-6.4 の典型。

### 1-f. 既に生成区間へ移り済み（参考・件数には数えない）

| 定数 | file:行 | 生成先 | 機構 |
|---|---|---|---|
| `adr.rs FLOOR`（判断の記録の欄の決まり一式） | `adr.rs:97` | `design-intent/adr/schema.yaml` 9〜127 行 | `folio schema --write`（`schema.rs:29 TARGETS`）。床の側は `floor_diff`（`adr.rs:457-467`） |
| `note.rs FLOOR`（設計ノートの欄の決まり一式） | `note.rs:132` | `design-intent/design-note/schema.yaml` 20〜156 行 | 同上。床の側は `note.rs:569-576` |

**類 A 小計: 22 本**（うち ★「行の id の固定」= 6 本 = A2・A3・A12〜A15／床が数えるもの = 11 本（A1〜A11）／面の生成器 = 5 本（A12〜A16）／命令 = 2 本（A17・A18）／歯だけ = 4 本（A19〜A22））。生成区間化で解決済みの 2 本は別。

---

## 1′. 類 A′（二重の記述だが一致を確かめていない）— 本調査で足した区分

task の 3 類には入らないが、P-6.4 の観点では類 A より危ない（手で直しても何も落ちない）。区分は**私が足したもの**であることを明記する。

### A′-1. 憲法・rules の `schema.enums` と `face.rs` の名札の表（10 + 3 = 13 本）

`constitution.yaml schema.enums` は 10 本の値域を**一覧として**持つ（36〜46 行）。`face.rs` はその 10 本すべてに対応する表を持つ。**`tier`（A16）と `retreat_kind`（A9・adr.rs 側）の 2 本だけ**が突き合わせを持ち、残り 8 本は片方向の `lookup` だけ＝**YAML の enums 一覧そのものは誰も読んでいない**。

| 憲法 `schema.enums.X` | 実装の表 | file:行 | 突き合わせ |
|---|---|---|---|
| `tier` | `TIERS` | `face.rs:461` | **在る**（A16） |
| `retreat_kind` | `face.rs RETREAT_KIND` / `adr.rs RETREAT_KIND` | `face.rs:542` / `adr.rs:50` | adr.rs 側のみ在る（A9）。face.rs の表は突き合わせ無し |
| `binds` | `BINDS` | `face.rs:521` | **無い** |
| `pattern` | `PATTERN` | `face.rs:514` | **無い** |
| `strength` | `STRENGTH` / `inject.rs STRENGTHS` | `face.rs:509` / `inject.rs:15` | **無い**（実装内でも 2 枚） |
| `rationale_kind` | `RATIONALE_KIND` | `face.rs:537` | **無い** |
| `mechanism_kind` | `MECH_KIND` | `face.rs:522` | **無い** |
| `mechanism_live` | `LIVE` | `face.rs:528` | **無い** |
| `stage` | `STAGE` | `face.rs:535` | **無い** |
| `polarity` | `POLARITY` | `face.rs:536` | **無い** |

`rules.yaml schema.enums`（15〜18 行）も同じ:

| rules `schema.enums.X` | 実装の表 | file:行 | 突き合わせ |
|---|---|---|---|
| `kind` | `RULE_KIND`（4 組） | `face.rs:557` | **無い** |
| `status` | `RULE_STATUS`（3 組・順は YAML と違う） | `face.rs:552` | **無い** |
| `stage` | `STAGE`（憲法と共有） | `face.rs:535` | **無い** |

さらに `render.rs`（退役待ちの `readable.html` の生成器）が同じ値域をもう 1 枚ずつ持つ: `AST`(42) / `VERD`(47) / `DOCST`(48) / `TIER`(53) / `STR`(64) / `MECH`(69) / `LIVE`(75) / `BIND`(82) / `KIND`(83) / `PAT`(88)。これらも突き合わせ無し。

### A′-2. その他の写し（一致の検査が無いもの）

| 定数 | file:行 | もう一方の面 | 状況 |
|---|---|---|---|
| `sheet.rs YES` / `NO` = 「はい」「いいえ」 | `sheet.rs:28-29` | `intake.yaml answers.values: [はい, いいえ]` | 突き合わせ**無し**。`sheet.rs:389-394` は回答の字面をこの 2 値と比べるだけ |
| `intake.rs DEFAULT_RECOMMEND` = "recommend" | `intake.rs:24` | `intake.yaml answers.default: recommend` | 片方向（許す値に足すだけ・`intake.rs:55-57`） |
| `bundle.rs FACE_NAMES`（9 行の doc id） | `bundle.rs:36` | `ceiling.yaml documents[].id`（9 行） | 片方向（`bundle.rs:399` で引けなければ Err）。`ceiling.rs DOCUMENT_IDS` と同じ 9 id の 3 枚目 |
| `figure.rs FIGURE_TYPES`（5 組） | `figure.rs:21` | `parts.json figure_type_enum` の archify 5 型 / `face.rs FIGURE_LABELS` のキー | 突き合わせ**無し**（歯 `figure.rs:471` は件数 5 だけ確かめる） |
| `figure.rs QUALITY` = "showcase" | `figure.rs:32` | `rules.yaml R-14 の value`「最上段（showcase）固定」 | rules 側は自由文なので機械の突き合わせ**不能** |
| `face_note.rs EXTERNAL_*`（4 本） | `face_note.rs:119-122` | `note.rs EXTERNAL_*`（`note.rs:114-117`・こちらは FLOOR 経由で生成区間へ出る） | 同じ字面を実装内で 2 枚（`face_note.rs:117` の注に「note.rs と同じ字面を自前に持つ」と明記） |
| `face_note.rs TYPES`（6 組） | `face_note.rs:80` | `note.rs TYPE_ENUM`（`note.rs:80`・生成区間へ出る） | 実装内 2 枚・突き合わせ無し |
| `face_adr.rs STATUS/VERDICT/RETREAT_KIND` | `face_adr.rs:87/94/97` | `adr.rs STATUS/VERDICT/RETREAT_KIND`（生成区間へ出る） | 実装内 2 枚・突き合わせ無し |
| `face_index.rs ADR_STATUS`（3 組） | `face_index.rs:82` | 同上 | 実装内 3 枚目 |
| `face_note.rs STATUS`（4 組） | `face_note.rs:92` | `note.rs STATUS_ENUM`（`note.rs:69`） | 実装内 2 枚・突き合わせ無し |

**類 A′ 小計: 13（enums 由来）+ 10（その他）= 23 本**（`render.rs` の 10 枚を数えると 33 本。退役待ちなので本体は 23 本で数えた）。

---

## 2. 類 B（規則の値が実装だけに在る）

YAML 側には一覧として書かれておらず、YAML の実体がその規則に従うかを床が確かめるだけのもの。

### 2-a. 最上位の節の閉じた一覧

| 定数 | file:行 | 何を縛るか | 人が読める写し |
|---|---|---|---|
| `SRS_TOP_LEVEL`（16） | `check.rs:39` | `srs.yaml` の最上位の節 | **無し**（srs.yaml に schema 節が無い。面の章立て `face_srs.rs:41 CHAPTERS` は別物 = ページの章名 8 つで、節の一覧ではない） |
| `VOCABULARY_TOP_LEVEL`（3） | `check.rs:63` | `vocabulary.yaml` の最上位の節 | **無し** |
| `INDEX_TOP_LEVEL`（5） | `entrance.rs:18` | `index.yaml` の最上位の節 | **無し** |
| `INTAKE_TOP_LEVEL`（5） | `intake.rs:18` | `intake.yaml` の最上位の節 | **無し** |
| `CEILING_TOP_LEVEL`（8） | `ceiling.rs:19` | `ceiling.yaml` の最上位の節 | **無し** |
| `FINDINGS_TOP_LEVEL`（3） | `findings.rs:28` | 所見 file `findings.yaml` の最上位の欄 | **無し**（所見 file は生成物の置き場） |

### 2-b. 欄の集合・閾値・値域（design-intent 側に一覧が無いもの）

| 定数 | file:行 | 何を縛るか | 人が読める写し |
|---|---|---|---|
| `check.rs FILES`（7） | `check.rs:28` | 床が読む正本 7 file と読む順 | 無し（`face_index.rs:60 SOURCES` は 8 本で別の一覧） |
| `SRS_FIGURE_REQUIRED`（4） | `check.rs:59` | `srs.yaml figures[]` の必須欄 | `adr/schema.yaml` の生成区間（`adr.rs:88 FIGURE_ENTRY` が同形） |
| `SRS_FIGURE_OPTIONAL`（2） | `check.rs:60` | 同上・任意欄 | 同上 |
| `MAX_QUESTIONS` = 5 | `intake.rs:27` | `intake.yaml questions` の行数（要件書 FR1 の規範文の数） | 無し（FR1 の文が根拠だが機械では読まない） |
| `INJECT_TARGET` = "inject" | `intake.rs:21` | 棚に無い行き先の 1 語 | 無し |
| `face_note.rs MAX_CHAPTERS` = 12 | `face_note.rs:72` | 設計ノートの章の数の上限 | 無し |
| `face_note.rs CONTRACT_FIXED`（7） | `face_note.rs:115` | 契約表の固定の欄 | 無し（`hint` へ回さない 7 欄） |
| `note.rs EXTERNAL_NEED`（2） | `note.rs:119` | `contracts/schema.toml` の `need` の値域 | 無し（外部 file・器 scribe2 の導出物） |
| `note.rs EXTERNAL_SHAPE`（2） | `note.rs:120` | 同上 `shape` の値域 | 無し |
| `note.rs EXTERNAL_PATH/HEAD/ROWS_KEY/ROW_FIELDS` | `note.rs:114-117` | 外部 file の読み方 | `design-note/schema.yaml` の生成区間（FLOOR 経由） |
| `findings.rs REFUTE_CONTENTS`（5） | `findings.rs:34` | 反証の束の中身 | 無し（`findings.rs:44` の注に「天井の正本 v0.2 で正本へ移すかは持ち主の裁定の後」とある） |
| `findings.rs RESULT_REQUIRED`（6） | `findings.rs:46` | `result.yaml` の欄 | 無し（同上） |
| `findings.rs REFUTE_RULE`（規則の文・逐語） | `findings.rs:49` | 反証役に渡す規則の文 | 無し（**注に「天井の正本 v0.2 で正本へ移すかは持ち主の裁定の後」と明記**＝既知の宿題） |
| `findings.rs FINDINGS_FILE` / `REFUTE_DIR` / `RESULT_FILE` | `findings.rs:25/31/43` | file と dir の名 | 無し |
| `bundle.rs DIGEST_FILE` | `bundle.rs:33` | 要約値 file の名 | 無し |
| `parts.rs FACES`（5） | `parts.rs:23` | `--page` の左辺・面の名 | parts.json の `components[].faces` の値域と同じ集合（**推測**: 突き合わせは無い） |
| `parts.rs ATTRS`（3） | `parts.rs:26` | 検査する属性 3 つ | 無し |
| `site.rs OUTPUTS`（5） | `site.rs:32` | 配信先へ出す固定の file と順 | 無し |
| `face_index.rs SOURCES`（8） | `face_index.rs:60` | 入口の面の foot に出す正本の一覧 | 面そのもの（人が読む写し = 生成物） |
| `face.rs INDEX_STATUS`（2） | `face.rs:657` | `index.yaml meta.status` の値域 | 無し |
| `face.rs TONE`（4） | `face.rs:577` | `srs.yaml verdicts[].tone` の値域 | 無し |
| `face.rs METHOD`（2） / `STRENGTH_MEANING`（3） / `PRIO`（3） | `face.rs:572/564/570` | 要件書の凡例・色の class | 無し |
| `face.rs stop_anchor` の節 id 規則 | `face.rs:660-672` | `index.yaml lanes.stops[].at` の行き先 | 無し（関数に埋まった規則） |
| `vocab.rs ID_PREFIXES`（10） | `vocab.rs:12` | 免除する id の形の頭 | 無し（免除の**語**の一覧は `vocabulary.yaml identifiers`＝類 C。頭の形だけが実装） |
| `refs.rs SRS_ID_SECTIONS`（7） | `refs.rs:11` | 要件書の id を持つ節 | 無し（`link.rs:19`・`note.rs:745` に同じ 7 個の写しが 2 枚） |
| `refs.rs SRS_ID_PREFIXES`（5） | `refs.rs:28` | 要件 id の頭 | 無し（`prose.rs:30 REQUIREMENT` に 5 個の写し） |
| `refs.rs RELATION_NAMESPACES`（4） | `refs.rs:25` | 憲法 `relations` の名前空間 | 無し |
| `RULE_SECTIONS`（2）× 4 枚 | `refs.rs:22` / `link.rs:30` / `note.rs:742` / `prose.rs:13` | rules の 2 節 | `rules.yaml schema.top_level` が同じ 2 語を持つ（**突き合わせ無し**） |
| `prose.rs ARTICLE`（3）/ `RULE`（2）/ `REQUIREMENT`（5） | `prose.rs:28-30` | 参照 id の接頭辞 | 無し |
| `prose.rs ROW_ID` = "R-16" | `prose.rs:12` | どの rules 行から値を読むか | 無し（値そのものは類 C） |
| `prose.rs HEAD` = 60 | `prose.rs:20` | 文言に載せる字数 | 無し |
| `anchor.rs ANCHOR_PREFIX/SUFFIX/KIND/INDEX_KIND` | `anchor.rs:21-24` | 凍結 anchor の file 名と種別 | `adr/schema.yaml` の生成区間（`adr.rs FLOOR.anchor` が同形の値を持つ。**推測**: 字面の突き合わせは無い） |
| `hello.rs GREETING/QUIET/CONSTITUTION/GREETED` | `hello.rs:13-22` | 挨拶の文と静かにする file | 無し |
| `gitcheck.rs TIMEOUT`（20 秒）/ `NO_GIT` | `gitcheck.rs:22/24` | 版管理の照合の待ち時間・文言 | 無し |
| `serve.rs MAX_HEAD` / `TIMEOUT` / `TAILSCALE_DNS` | `serve.rs:15/17/19` | 配信の上限・宛先（N-6.1） | 無し（**N-6.1 の判定式が実装にだけ在る**） |
| `figure.rs TOOL` / `HEAD_CHARS` | `figure.rs:30/36` | 図の道具の置き場・診断の字数 | `rules.yaml R-15` が置き場を自由文で書く（機械の突き合わせ不能） |
| `schema.rs WIDTH` = 100 | `schema.rs:26` | 生成区間の 1 行の幅 | `adr/schema.yaml` 冒頭の注（自由文） |
| `inject.rs BEGIN/END` / `schema.rs BEGIN/END` | `inject.rs:11-12` / `schema.rs:22-23` | 生成区間の印 | 生成物そのもの |

**類 B 小計: 51 本**（最上位の節 6 + その他 45）。

---

## 3. 類 C（YAML が正本で実装が読む）

| 読む欄 | 正本 file | 使う検査・生成 | file:行 |
|---|---|---|---|
| `schema.top_level` | `constitution.yaml` / `rules.yaml` | 未知の節 | `check.rs:220-232`（fn `schema_top_level`）← `check.rs:320` / `check.rs:368` |
| `schema.enums.retreat_kind` | `constitution.yaml` | 床の定数との一致（類 A9 の相手） | `link.rs:104-108` |
| `schema.enums.tier` | `constitution.yaml` | 段の表との一致（類 A16 の相手） | `face_constitution.rs:154` |
| `schema.amendment_scope` | `constitution.yaml` | 写しの取り方・下限の包含 | `anchor.rs:365`（fn `amendment_scope`）/ `link.rs:88-97` |
| `schema.kind_meaning` | `rules.yaml` | 憲法の面の §5 の凡例（逐語） | `face_constitution.rs:659-666` |
| `schema.excluded` | `rules.yaml` | 退役待ちの読み物の生成 | `render.rs:747` |
| `R-16 の value`（marks / prohibition.word / clause_ends / units） | `rules.yaml` | 設計ノートの散文の門（FR12） | `prose.rs:60-105`（fn `gate`）・呼び手は `note.rs` |
| `terms[].term` / `.en` / `field_terms[]` / `identifiers[].words` | `vocabulary.yaml` | 既知の語の集合・免除（R-9） | `vocab.rs:73-110`（fn `known_words`） |
| 条 id・rules 行 id・要件書 id の全空間 | 4 正本 | 参照 id の解決（R-4） | `refs.rs:43-60`（fn `check_refs` / `known_ids`） |
| `meta.counts`（always/ask-first/never） | `constitution.yaml` | 数えた条の数との一致 | `refs.rs`（件数）/ `face_index.rs:433-440` |
| `weights.values` | `ceiling.yaml` | `weights.refute` の行き先 | `ceiling.rs:95-98` |
| `documents[].id` の集合 | `ceiling.yaml` | 観点 `reads[].doc` の行き先 | `ceiling.rs:110`・`131` |
| `viewpoints` / `documents` / `finding` / `weights` / `verdicts` / `record` の値 | `ceiling.yaml` | 束の材料と `--check` の値域 | `bundle.rs:150-210`（fn `load` / `load_rules`） |
| `answers.values` | `intake.yaml` | `questions[].recommend` の行き先 | `intake.rs:46-58`・`83` |
| `targets[].id`（行 id） | `intake.yaml` | `questions[].yes` / `.no` の行き先 | `intake.rs:72`・`84-85` |
| `shelf.documents[].id` / `shelf.annexes[].id` | `index.yaml` | `intake.yaml targets[].id` / `.with` の行き先 | `intake.rs:61-62`・`180-197` |
| `shelf.documents`＋`shelf.annexes` の id 空間 | `index.yaml` | `relations.from/to`・`annexes.inside`・`lanes.stops[].doc` の行き先 | `entrance.rs:75-96`・`126` |
| `components` / `figure_type_enum` / `shelf_type_enum` / `style_props_allowed` | `preview/parts.json` | 部品の enum を組み立て時に導出（人は型の一覧を書かない） | `crates/folio/build.rs:31-`（fn `derive`）→ `parts.rs:16` |
| 同上（実行時の再突合） | `preview/parts.json` | 組み立て時の写しと順つき完全一致 | `parts.rs:128-155`（fn `catalog_matches`） |
| `figure_body_classes.type_ids` | `preview/parts.json` | 図の型の名札（歯が `FIGURE_LABELS` と突き合わせ＝類 A22） | `face_note.rs:997` |
| 図の型の値域 | `preview/parts.json`（`FigureType`） | 要件書 / 判断の記録 / 設計ノートの図の型 | `check.rs:468-481`・`adr.rs:777`・`note.rs`（`FigureType::from_name`） |
| 様式の定義（`preview/folio.css`）の class 集合 | `preview/folio.css` | 面の class の母集団（R-3） | `parts.rs:186-195` |
| `adr/schema.yaml` の `schema` 節 | 生成区間 | 床の定数との差分（生成区間の機構） | `adr.rs:457-467` |
| `design-note/schema.yaml` の `schema` 節 | 生成区間 | 同上 | `note.rs:569-576` |

**類 C 小計: 24 本**。

---

## 4. tests/ の中の、定数と YAML の一致に依っている歯

file 名と fn 名のみ（実測）。

### `crates/folio/tests/ceiling.rs`（`ceiling.yaml` の 8 定数の針）
`ceiling_canonical_copy_passes` / `ceiling_unknown_section_fails` / `ceiling_viewpoint_id_off_the_list_fails` / `ceiling_three_viewpoints_fails` / `ceiling_verdict_order_fails` / `ceiling_missing_document_fails` / `ceiling_read_doc_not_a_document_fails` / `ceiling_refute_weight_not_a_value_fails` / `ceiling_finding_required_without_evidence_fails` / `ceiling_record_required_without_bundle_fails` / `ceiling_duplicate_document_id_fails`

### `crates/folio/tests/link.rs`（憲法の enums との一致）
`link_retreat_kind_drift_fails`（fixture `tests/fixtures/link/retreat-kind-drift/`）

### `crates/folio/tests/face_index.rs`（`index.yaml` の 4 表の針）
`face_index_unknown_when_a_document_is_missing` / `face_index_unknown_when_a_relation_id_is_outside_the_table` / `face_index_unknown_when_a_third_annex_is_added` / `face_index_unknown_when_a_legend_id_is_outside_the_table` / `face_index_unknown_when_a_record_status_is_outside_the_table` / `face_index_unknown_when_a_note_status_is_outside_the_table` / `face_index_unknown_when_constitution_counts_differ` / `face_index_unknown_when_a_stop_anchor_is_outside_the_face` / `face_index_unknown_when_a_sheet_document_is_outside_the_targets` / `face_index_unknown_when_a_sheet_annex_is_outside_the_annexes` / `face_index_write_matches_the_frozen_fixture` / `face_index_census_on_the_real_sources_counts_and_verbatims`

### `crates/folio/tests/bundle.rs`（`ceiling.yaml bundle` と `bundle.rs` の定数）
`bundle_unknown_when_the_bundle_contents_differ_from_the_floor` / `bundle_unknown_when_a_read_doc_is_not_a_document` / `bundle_write_matches_the_frozen_anchor` / `bundle_on_the_real_source_builds_four_bundles`

### `crates/folio/tests/parts.rs`（parts.json と組み立て時の導出）
`parts_check_is_unknown_when_the_catalog_differs_from_build_time` / `parts_print_matches_the_floor_catalog_byte_for_byte` / `parts_check_fails_on_a_class_outside_the_catalog` / `parts_check_fails_on_a_component_outside_the_catalog` / `parts_check_fails_on_a_component_not_allowed_on_the_face` / `parts_check_fails_on_a_disallowed_inline_style_prop`

### `crates/folio/tests/schema.rs`（生成区間 = 既に解決済みの 2 本）
`schema_check_matches_the_real_file_and_its_frozen_digest` / `schema_check_fails_on_one_byte_drift_inside_the_region` / `schema_write_restores_the_region_and_is_idempotent` / `schema_write_leaves_bytes_outside_the_region_alone` / `schema_markers_are_invisible_to_folio_check` / `schema_check_matches_the_real_design_note_file_and_its_frozen_digest` / `schema_check_fails_on_one_byte_drift_inside_the_design_note_region` / `schema_write_restores_the_design_note_region_and_is_idempotent`

### `crates/folio/tests/adr.rs` / `note.rs`（床の定数と欄の決まりの写し）
`adr_schema_drift_fails`（fixture `tests/fixtures/adr/schema-drift/`）/ `adr_figure_type_outside_the_catalog_fails` / `note_schema_copy_drift_fails` / `note_figure_type_not_in_the_catalog_fails` / `note_fields_row_need_not_in_enum_fails` / `note_status_not_in_enum_fails` / `note_external_schema_with_another_head_is_unknown` / `note_extra_contract_field_without_derived_field_fails`

### `crates/folio/tests/check.rs` / `intake.rs` / `entrance.rs`（最上位の節と値域 = 類 B の針）
`check_unknown_section_fails` / `check_srs_figure_type_outside_the_catalog_fails` / `check_srs_figure_with_an_unknown_field_fails` / `intake_unknown_section_fails` / `intake_recommend_not_in_values_fails` / `intake_over_the_question_limit_fails` / `intake_target_id_not_on_the_shelf_fails` / `intake_target_with_not_an_annex_fails` / `intake_question_yes_not_a_target_fails` / `entrance_unknown_section_fails` / `entrance_stop_doc_not_on_shelf_fails` / `entrance_relation_to_not_on_shelf_fails`

### src の中の unit test（parts.json との突き合わせ＝類 A19〜A22）
`crates/folio/src/face_constitution.rs` の `face_rail_node_limit_matches_the_parts_catalog` / `face_parts_are_all_allowed_on_the_constitution_face`
`crates/folio/src/face_srs.rs` の `face_srs_limits_match_the_parts_catalog` / `face_parts_are_all_allowed_on_the_srs_face`
`crates/folio/src/face_index.rs` の `face_parts_are_all_allowed_on_the_index_face`
`crates/folio/src/face_note.rs` の `face_note_figure_labels_match_the_parts_catalog_type_ids`
`crates/folio/src/link.rs` の `mod tests`（`adr::floor_strs(["enums","retreat_kind"])` の値の凍結・`link.rs:645-654`）

`crates/folio/tests/floor_cases.rs`（754 行・134 case）は床の字面の凍結一覧で、類 A の定数を直接は数えない（**推測**: 違反の文言が変わると落ちるので、生成区間化で文言が変われば書き換えが要る）。

---

## 5. 類 A を生成区間へ移すなら file ごとに何が動くか

「生成区間へ移す」= `adr/schema.yaml` と同じ形（`# folio:schema:begin/end` の間を `folio schema --write` が実装の定数から導出し、人は触らない）にする、の意。

### `ceiling.yaml`（類 A1〜A8・A17・A18 = 10 本）
- 動く欄は 6 か所: `verdicts.values` / `finding.required` / `finding.place.required` / `finding.refute.values` / `record.required` / `bundle.contents`＋`bundle.digest`。この 6 か所は**純粋な一覧の写し**なので、`schema:`（仮）節を 1 つ新設してそこへまとめて生成区間として出せば、`ceiling.rs` の `same_list` / `same_set` / `contains_all` の 6 呼び出し（`ceiling.rs:83/154/164/175/188/200`）と `bundle.rs:200-201` の 2 つが不要になる。
- 難所は ★2 本（`VIEWPOINT_IDS` / `DOCUMENT_IDS`）。`viewpoints` と `documents` は**人が書く中身**（`name` / `reader` / `question` / `reads` / `file` / `note`）を持つ行なので、行ごと生成区間には入れられない。移すなら「id の列だけを生成区間に出し、行は人が書いて id をそこへ解く（類 C の `resolve` と同じ形）」という 2 段の形になる。
- `bundle.contents` は今 3 枚（ceiling.yaml・`ceiling.rs:62`・`bundle.rs:27`）あり、比較の厳しさも違う（包含 vs 順つき等値）。生成区間化の前に**実装側を 1 本へ寄せる**のが先。
- 便 41 で入れた天井の 3 値の規則（`ceiling.rs:31`）は要件書 FR5 とも重なるので、**推測**: 生成区間の正本を要件書側にするか実装側にするかで ADR-11 の論点が 1 つ増える。

### `index.yaml`（類 A12〜A15 = 4 本・すべて ★行の id の固定）
- 4 か所とも「行の id を実装の表が過不足なく固定し、行の中身は人が書く」形＝`ceiling.yaml` の ★と同じ構造だが、**こちらは床ではなく面の生成器が確かめる**（落ちると違反ではなく「まだ分からない」）。生成区間化しても床の終了コードは変わらない。
- `SHELF_DOCS` は id 以外に `place`（css の class）・`face`（面の file 名）・`en`・`sep` を持つ＝**見た目の値**。生成区間へ出すと `index.yaml` に css の class 名が載る。P-2.3（design token は 1 か所）との折り合いを ADR-11 で書く必要がある。
- `ANNEXES` は `(憲法の章番号, 数の単位)` を持ち、`face_index.rs:529-545` が語彙と rules の件数を数えて埋める＝**導出の材料**。生成区間より「`index.yaml` 側に章番号と単位を人が書き、実装が読む（類 C 化）」の方が素直。**推測**。
- `SHELF_LEGEND` だけ片方向（`lookup`）なので、`index.yaml` から凡例の行を消しても今は落ちない。生成区間化すると落ちるようになる＝**振る舞いが変わる**（便を 1 本使う理由になる）。
- `index.yaml` は入口の面の正本なので、生成区間を足すと `INDEX_TOP_LEVEL`（類 B）の一覧も 1 つ増える（`schema` を足す）。

### `intake.yaml`（類 A′ のみ・類 A は 0 本）
- 床が確かめている類 A は無い。動くのは A′-1（`sheet.rs YES/NO` ↔ `answers.values`）と A′-2（`DEFAULT_RECOMMEND` ↔ `answers.default`）の 2 本で、どちらも**今は突き合わせが無い**。
- `answers.values: [はい, いいえ]` を正本のままにするなら `sheet.rs:389-394` が YAML から読む（類 C 化）のが筋。生成区間化（実装が正本）にすると、相談窓口の回答の言葉を持ち主が変えられなくなる＝ADR-6 決定 (4) の趣旨（回答は旗で渡す）と噛み合うか要検討。
- `MAX_QUESTIONS`（類 B）は「要件書 FR1 の規範文の数」が根拠なので、生成区間より**要件書から導出**の方が P-5.1 に沿う。**推測**。
- `intake.yaml` に `schema` 節を足すと `INTAKE_TOP_LEVEL` も 1 つ増える。

### `srs.yaml`（類 A は 0 本・類 B が 3 本）
- `SRS_TOP_LEVEL`（16）・`SRS_FIGURE_REQUIRED`（4）・`SRS_FIGURE_OPTIONAL`（2）はすべて類 B＝`srs.yaml` に写しが無く、**P-6.4 には触れていない**。ADR-11 で動かす必然性は薄い。
- 動かすとしたら「憲法・rules と同じく `srs.yaml` にも `schema.top_level` を置く（類 C 化）」か「生成区間で出す」かの二択。前者は人が書く面が 1 つ増える＝P-6.4 の方向と逆。後者なら `check.rs:39` を `schema.rs:29 TARGETS` に足すだけで済む。
- `face_srs.rs:41 CHAPTERS`（8 章）・`BANDS`（9 組）は**ページの章名と絵記号**であり `srs.yaml` の節の一覧ではない（実測: 章名「ゴール/範囲/機能要件/…」と節名 `goals/scope/requirements/…` は対応するが字面が違い、突き合わせも無い）。生成器の内部の定数として別枠に置くのが正しい。
- 要件書の図の欄（4+2）は `adr/schema.yaml` の生成区間（`figures.entry`）と同形なので、**3 つの文書型の図の欄を 1 本の床の定数に寄せて 1 か所から生成する**のが片付け方。**推測**。

### その他（`constitution.yaml`・`rules.yaml`・`parts.json`・`contracts/schema.toml`）
- **`constitution.yaml`（類 A9・A10・A16 + A′-1 の 10 本）**: ここが本丸。`schema.enums` の 10 本のうち突き合わせを持つのは 2 本だけで、残り 8 本は二重の記述のまま放置されている。生成区間化の向きは**逆**（憲法 → 実装）でなければならない＝`link.rs:157` の字面が「憲法が正・床の定数を直す」と言っており、A-2（条文の改訂）の対象でもあるので、**実装の定数を正本にはできない**。取りうる形は (1) 残り 8 本にも `link.rs` 型の突き合わせを足す（類 A を増やす）か、(2) `face.rs` の表から値域の列を消して憲法から読む（類 C 化・名札の日本語だけ実装に残す）の 2 つ。(2) が P-6.4 に沿うが、`face.rs` の 10 表・`render.rs` の 10 表・`face_adr.rs` / `face_note.rs` / `face_index.rs` の重複表まで届く大きな便になる。
- **`rules.yaml`（A11 + A′-1 の 3 本）**: `schema.enums.kind/status/stage` が `face.rs RULE_KIND/RULE_STATUS/STAGE` と二重。`RULE_STATUS` は順まで違う（YAML `[仮, 凍結, 未定]` / 実装 `[凍結, 仮, 未定]`）。憲法と同じ向き（YAML が正本）で類 C 化するのが筋。`SURFACE = ["R-8"]`（A11）は rules 行の id を実装が名指しており、P-5.2（id で参照）には沿うが、値そのものは実装の定数のまま。
- **`parts.json`（A19〜A22）**: 部品目録は既に「parts.json が正本・実装は `build.rs` の導出物」という形が出来ている（`parts.rs:16`）。残る 4 本（上限 3 + 図の名札 1）だけが手書きの写しで、しかも**歯でしか確かめていない**（`folio check` では落ちない）。`build.rs` の導出に `max_nodes` / `max_per_band` / `type_ids` を足せば 4 本とも消え、歯 3 本（`face_constitution.rs:1097` / `face_srs.rs:1321` / `face_note.rs:997`）も不要になる。**ADR-11 の中で最も費用対効果が高い片付け**。**推測**: 便としては S 相当。
- **`contracts/schema.toml`（器 scribe2 の導出物）**: `note.rs EXTERNAL_*`（6 本・生成区間へ出る）と `face_note.rs EXTERNAL_*`（4 本・出ない）が同じ字面を 2 枚持つ（`face_note.rs:117` の注が自認）。外部 file なので生成区間化の対象外だが、**実装内の 2 枚を 1 枚に寄せる**のは ADR-11 の射程に入る。
- **`findings.rs REFUTE_RULE` ほか（類 B）**: `findings.rs:44` の注が「天井の正本 v0.2 で正本へ移すかは持ち主の裁定の後」と既に書いている＝ADR-11 が拾うべき明示の宿題。v0.2 は既に発効しているので、移すなら `ceiling.yaml` に `refute` の節を足す便が要る。

---

## 6. 件数のまとめ

| 類 | 件数 | 内訳 |
|---|---|---|
| 類 A（一致を確かめている） | **22** | 床 11（A1〜A11）／面の生成器 5（A12〜A16）／命令 2（A17・A18）／歯だけ 4（A19〜A22）。うち ★行の id の固定 = 6 |
| 類 A′（二重だが確かめていない・本調査で追加した区分） | **23** | 憲法/rules の enums 由来 13 ＋ その他 10（`render.rs` の 10 枚を数えると 33） |
| 類 B（実装だけに在る） | **51** | 最上位の節 6 ＋ 欄の集合・閾値・値域 45 |
| 類 C（YAML が正本） | **24** | design-intent 19 ＋ parts.json / css 5 |
| 解決済み（生成区間） | 2 | `adr.rs FLOOR` / `note.rs FLOOR` |

生成器の内部の定数（表示の文言・ページの部品の並び・体裁）＝本調査の対象外として名前だけ:
`face_constitution.rs` の `PARTS`・`BANDS`・`FRAME`／`face_srs.rs` の `PARTS`・`CHAPTERS`・`BANDS`・`FIGURES_CHAPTER`・`OWNER`・`TOOL_ROLE`／`face_adr.rs` の `PARTS`・`CHAPTERS`・`H2`・`BANDS`・`FIGURES_CHAPTER`・`FAVICON`・`BASIS_GROUPS`／`face_note.rs` の `PARTS`・`BANDS`・`FAVICON`・`REFUSES_NONE`・`NEED`・`SHAPE`／`face_index.rs` の `FRAME`・`NONE`・`SHEET_KICKER`・`SHEET_DOCUMENTS`・`SHEET_RECOMMENDED`・`SHEET_APPROVAL`・`SHEET_SOURCE`・`SHEET_ABSENT`・`SHEET_NO_DOCUMENTS`・`SHEET_NO_RECOMMENDED`・`SHEET_NO_APPROVAL`・`SHEET_MADE_BY`／`face.rs` の `NAV`・`Frame`／`sheet.rs` の `HEADER`・`SHEET_ID`・`DRAFT`・`ANSWERED`・`RECOMMENDED`／`serve.rs` の `TEXT`・`TYPES`・`OCTET`／`render.rs` の `CSS`・`NAMES`／`freeze.rs` の見出しの字面（`freeze.rs:311`・`321`）／`sha256.rs` の `K`・`H0`（算法の定数）／`adr.rs` の `ID_PATTERN`・`DATE_FORMAT`・`RULING_PATTERN`・`OWNER`（欄の決まりの一部として生成区間へ出る）／`note.rs` の `ID_PATTERN`・`VERSION_PATTERN`・`ROW_ID_PATTERN`・`PATH_BASE`・`DATE_FORMAT`（同上）。

`#[cfg(test)]` の中だけの定数（検査の材料・対象外）: `check.rs:511 SRS`／`lineage.rs:635 PREV`・`672 CUR_TITLE`・`673 C_POINTED`／`schema.rs:424 LONG`・`425 F`。
