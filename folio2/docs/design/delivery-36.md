# 設計: 便 36 — 要件書の面の用語集（章 08）を憲法の面の用語集（章 07）と同じ形にする（FR4 / NFR2・ADR-5 決定 (1)(3)）

- 要件: FR4（3 枚を 1 つの生成器で出す・見た目が割れない）/ NFR2（部品目録に無い class は 0・部品目録の faces の一致）
- 条: P-2.1（生成器は 1 つ・語の行の生成を面ごとに持たない）/ P-2.4（部品目録は閉じた一覧）/ P-6.3（語彙の正本は vocabulary.yaml・2 面はそこから導出）/ N-1.2（退役は可逆な移動）
- 判断の記録: ADR-5 決定 (1)（見た目は見本に合わせる・文章と数は正本から）・決定 (3)（手書きの生成器）・撤退条件 ②（見た目だけの直しの数え・要件書の面は便 35 に続く 2 本目 = 上限 R-7 の 2 に達する・3 本目でテンプレートの案を問う）。便 35（f2-648.50）の後に直列で置く。
- 裁定: 持ち主 2026-09-19「各ページの語彙の項目だがconstitutionとSRSでスタイルが異なる。これに関してはconstitution側に統一したほうが良いのではと思う」（f2-648 notes・要件書 v1.5 の発効の逐語）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 ak が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

憲法の面の章 07（用語集）は部品 glossary-term-table で、語彙の正本（design-intent/vocabulary.yaml・terms 47 行）の各語を 1 行ずつ「語（term）+ 英語（en）+ 定義（def）+ 注（note・在れば）+ 目次へ」の形で出す。要件書の面の章 08（用語集）は部品 glossary-links で、同じ語を「憲法の面へのリンク + ? で開く短い説明（short）+ 憲法 §7 の定義へ」の形で出す（便 15・見本 v4 の形）。持ち主の裁定で、要件書の面の章 08 を憲法の面の章 07 と同じ形にする。語彙の正本は変えない（P-6.3・2 面とも vocabulary.yaml から導出）。

planner の実測（2026-09-19・main 51263f3）: 憲法の面の語の行は crates/folio/src/face_constitution.rs の関数 glossary_chapter（行 977〜1009・正規化 1,150・余地 350）が出し、要件書の面の章 08 は crates/folio/src/face_srs_rtm.rs の関数 glossary_chapter（行 101〜122・正規化 124・余地 1,376）が出す。要件書の面の章 08 の h2「この文書に出てくる専門語 → 説明は憲法 §7 で」は crates/folio/src/face_srs.rs の関数 chapter_h2（行 575・正規化 1,383・余地 117）の固定の字で、lead は要件書の正本の glossary_pointer（要件書 v1.5 で「読者が引く語の正本。要件書 §8 は憲法 §7 と同じ形で語と定義を出す・ここには写さない」に改めた・便の前提・planner の PR）。生成した要件書の面（配信先の srs.html）で憲法の面の語彙へのリンク（constitution.html#g-…）は 80 = 章 08 の 40 語 × 2 で、章 08 の外には無い。部品目録 design-intent/preview/parts.json の glossary-term-table の faces は [constitution]・glossary-links の faces は [srs]。凍結目録 tests/fixtures/floor/parts-catalog.json は同じ値。要件書の面の部品の表 PARTS（face_srs.rs 行 20〜38）は 17 種で glossary-links を含み、憲法の面の PARTS（face_constitution.rs）は 14 種で glossary-term-table を含む。folio.css の glossary-links の規則（class hint・hint-btn・hint-body）は見本の写しで、他の面も使う hint の規則と共有なので触らない。

(a) 共有の語の行（crates/folio/src/face.rs・正規化 955・余地 545）。関数 glossary_rows（pub(crate)・引数は o・語彙の正本の X）を足し、憲法の面の glossary_chapter が出す語の行（div.grow〔id g-語の id〕・div.gword〔term + span.en〕・div〔p.gdef の def + note が在れば p.gdef + a.back「目次へ」〕）と同じ字面を、terms の正本の順で出す。en が null の語は span.en を出さない（今の憲法の面と同じ）。

(b) 憲法の面（crates/folio/src/face_constitution.rs）。glossary_chapter は帯と div.chapbody と部品 glossary-term-table の div を出し、語の行は (a) を呼ぶ形に縮める。出力は byte 不変（凍結 tests/fixtures/face/expected.html は変えない・歯で見る）。

(c) 要件書の面（crates/folio/src/face_srs_rtm.rs と crates/folio/src/face_srs.rs）。章 08 の部品を glossary-links から glossary-term-table に替え、語の行は (a) を呼ぶ（憲法の面の章 07 と語の行の字面が同じ・目次へのリンクは同じ面の #toc）。h2 は憲法の面と同じ「本文に出てくる専門語のやさしい説明」（chapter_h2 の行 575 の字を替える・face_srs.rs の変更はこの 1 行と PARTS の 1 要素）。lead は glossary_pointer のまま（正本の値）。PARTS は 17 種のまま（glossary-links → glossary-term-table）。

(d) 部品目録（design-intent/preview/parts.json）と凍結目録（tests/fixtures/floor/parts-catalog.json）。glossary-term-table の faces を [constitution, srs] にする。glossary-links は退役 = 部品目録から外す（版管理の履歴に残るので可逆・N-1.2・parts.json の note に便 36 で退役した旨を 1 行足す）。組み立ての script（crates/folio/build.rs）が部品目録から生成する Component の一覧は自動で追従する（Component::GlossaryLinks が消えるので、参照は (c) で全部替える）。凍結目録は parts --print の出力で再凍結する（差分は glossary-term-table の faces と glossary-links の行だけ）。crates/folio/src/parts.rs（正規化 548・余地 952）の unit の歯 parts_catalog_names_are_derived_from_the_catalog（行 528・cfg(test)）は Component::ALL の数を 30 と数えるので、退役で 29 になる 1 行を替える（unit は done に数えない・共通の検証 cargo nextest --workspace で見る）。

(e) 凍結 fixture（tests/fixtures/face/expected-srs.html）。(c) の形で再凍結する。差分は章 08 の h2 と chapbody（glossary-links の span.hint の列 → glossary-term-table の div.grow の列）と foot の部品の一覧だけ（他の章は byte 不変・章 09 と承認欄は不変）。tests/fixtures/face/srs.yaml と vocabulary.yaml は変えない。

(f) 歯。crates/folio/tests/face.rs（関数名は face を含める・--test face の scope・正規化 1,152・余地 348）: 憲法の面の既存の歯（部品 14 種・凍結 expected.html との一致）は不変で緑（(b) の byte 不変を見る）。要件書の面の census（行 672〜676）の用語の一覧の期待を「data-component=glossary-term-table の中の div.grow の数 = terms の数 ∧ a.back の数 = terms の数 ∧ id g-<語の id> が terms の順」に替え、ALLOWED（行 686〜706）の glossary-links を glossary-term-table に替える。足す歯: 実の正本の憲法の面と要件書の面で、語の行の区間（glossary-term-table の div の中身）が byte で同じ。写しの面の章 08 の h2 が「本文に出てくる専門語のやさしい説明」∧ glossary-links の字が面に無い。crates/folio/tests/parts.rs（関数名は parts を含める・--test parts の scope）: 生成した面での parts --check 合格（既存の歯・不変）∧ parts --print が再凍結した凍結目録と一致（既存の歯・再凍結に追従）。crates/folio/tests/site.rs（関数名は site を含める・--test site の scope）: 本文は変えず、再凍結した expected-srs.html に追従して緑。

(g) 便 35 までの形との接続: 新規 file は無い。crates/folio/src/face.rs は (a)（+25・余地 545）。crates/folio/src/face_constitution.rs は (b) で縮む（-15・中身を変える既存 file として載せる・余地 350）。crates/folio/src/face_srs_rtm.rs は (c)（-5・余地 1,376）。crates/folio/src/face_srs.rs は (c) の 2 行（余地 117）。crates/folio/src/parts.rs は (d) の unit の 1 行（余地 952）。design-intent/preview/parts.json と tests/fixtures/floor/parts-catalog.json は (d)。tests/fixtures/face/expected-srs.html は (e)。crates/folio/tests/face.rs は (f)（+30・余地 348）。crates/folio/tests/parts.rs と crates/folio/tests/site.rs は本文不変（verify の scope）。入口の面・判断の記録の面・設計ノートの面は触らず、凍結 expected-index.html・expected-adr.html・expected-note.html・expected-site-adr-2.html は不変。readable.html（render）と inject の導出物は本便では変わらない（要件書 v1.5 の再生成は便の前提の PR で済ませてある）。

## 2. 範囲

- 入れる: 語の行の共有・要件書の面の章 08 の形・部品目録の faces と glossary-links の退役・凍結 fixture 2 本の再凍結・歯。
- 入れない: 語彙の正本の変更・憲法の面の字面の変更・folio.css の変更・他の章と他の面。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| rows | 語の行 | face.rs の glossary_rows を 2 面で共有 |
| chapter | 章 08 | 要件書の面を glossary-term-table に |
| catalog | 目録 | faces の変更と glossary-links の退役 |
| freeze | 再凍結 | expected-srs.html と parts-catalog.json |

## 4. 検査（歯）

§1 (f) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ak"
title = "要件書の面の用語集（章 08）を憲法の面の用語集（章 07）と同じ形（glossary-term-table・語 + 英語 + 定義 + 目次へ）にし、語の行の生成を face.rs に 1 つ持って 2 面で共有する（憲法の面は byte 不変・glossary-links は退役）"
req = ["FR4", "NFR2"]
section = "1"
write-set = ["crates/folio/src/face.rs", "crates/folio/src/face_constitution.rs", "crates/folio/src/face_srs.rs", "crates/folio/src/face_srs_rtm.rs", "crates/folio/src/parts.rs", "design-intent/preview/parts.json", "tests/fixtures/floor/parts-catalog.json", "tests/fixtures/face/expected-srs.html", "crates/folio/tests/face.rs", "crates/folio/tests/parts.rs", "crates/folio/tests/site.rs"]
verify = ["cargo nextest run -p folio --test face face", "cargo nextest run -p folio --test parts parts", "cargo nextest run -p folio --test site site", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "face の歯（憲法の面は byte 不変・要件書の面の章 08 が glossary-term-table で語の行が 2 面で byte 同一・h2 の字・glossary-links の字が無い）が緑、parts の歯（生成した面で parts --check 合格・parts --print が再凍結した凍結目録と一致）が緑、site の歯が再凍結に追従して本文不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
