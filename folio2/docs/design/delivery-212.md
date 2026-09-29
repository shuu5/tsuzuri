# 設計: 便 212 — yaml-rust2 の既定の機能を外し、外の部品 encoding_rs と、それだけが引いていた 8 本を無くす（S）

- 要件: FR5（検査結果を必ず返す）・FR7（組み立てて、見せる）。本便はどちらの振る舞いも変えない（床の答えと `folio build` の出力が byte で同じことを done で数える）。規範文・確かめ方・受入基準は変えない。
- 条: A-3.1（外部ライブラリを増やす・減らすとき、予算〔規則の表の行 R-6〕と使ってよい条件〔ライセンス〕に照らし、持ち主の確認を得る）・要件書の制約 CON2（確認の義務は A-3.1 が持つ）・P-12.1 / P-12.2（承認は行 R-8 の対話面を通ったものだけ・逐語と日付を台帳の notes に）。行 R-6 の値は未定のまま変えない。
- 出所: tsuzuri の設計席の求め（2026-09-29・持ち込みの前の 3 本目の直し F3）。tsuzuri の持ち主が 2026-09-29T11:32Z に「folio の外の部品 encoding_rs（ライセンスに BSD-3-Clause が要る）を持ち込みの前に外す」を選んだ。**この知らせは folio2 の承認ではない**（P-12.1）。
- **受付の前提（順）**: 外の部品を減らすことへの folio2 の持ち主の承認（A-3.1・行 R-8 の対話面）を、逐語と日付で台帳の notes に記帳した**後で**便を受け付ける。
  - 承認: 逐語「推奨で良い」・日付 2026-09-29 21:33 JST・台帳の notes の所 = f2-648 notes【2026-09-29 21:33 JST 持ち主の承認（対話面 R-8）】（席の問いの推奨は「はい」・台帳 f2-648.275.15）
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `hg` が指す §1 だけ。write-set 3 本（manifest 1・解いた依存の一覧 1・新しい歯の file 1）・新しい dir は無い・src は無い。
- 門: **0（通す）**。本流の binary で `folio ceiling --gate --dir design-intent --write-set crates/folio/Cargo.toml Cargo.lock +crates/folio/tests/deps.rs` の答えは「通す（設計文書の正本を書き換えない便）」。
- 前の便: **base = 本流 0c910db**（起草の時点）。受付の前に本流は b99daea（便 211 の契約 6b4ba70 と着地 b99daea・serve.rs と tests/serve.rs）へ進んだが、write-set の 3 本・workspace の Cargo.toml・Cargo.lock は 0c910db と b99daea で byte で同じ＝数え直しは要らない（検証役が b99daea に差分を当てて衝突 0・組み立ての後の Cargo.lock は見本と byte で同じ・verify 3 行 緑を確かめた）。数は base と見本の写しの実測（参考値・行 D-13）。組み直す手順は控え `~/.local/share/folio2/handoff-2026-09-28/f3-scripts/`（chain.sh・run.sh・mut.py・count.sh・tree.sh）。便 211（行 hf）の契約の取り込みとは file が重ならない（§1 (g)）。
- 見本: origin の枝 `impl/d212`（**8b24827**・親 0c910db・1 commit）。`git diff 0c910db 8b24827` が便の全体の差分（3 file）。作業者は**受付の時点の本流にこの差分を当てる**（write-set の file を見本の file の中身へ置き換えない）。`Cargo.lock` は見本の file で置き換えず、manifest を直した後に cargo に解き直させる（受付までに本流の `Cargo.lock` が動いていれば、その変化を消さないため）。

## 1. 設計

### (a) いま起きていること（参考値・base の実測）

1. `crates/folio/Cargo.toml` は yaml-rust2 を 2 か所で `yaml-rust2 = "0.10"` と書く（10 行の `[dependencies]` と 13 行の `[build-dependencies]`・組み立ての script `build.rs` も部品目録と憲法を読むのに使う）。この書き方は yaml-rust2 の既定の機能（default）を入れ、既定の機能は `encoding` 1 つで、`encoding` は外の部品 encoding_rs を引く（yaml-rust2 0.10.4 の Cargo.toml の `[features]`）。
2. 解いた依存の一覧 `Cargo.lock` の package は 36 本で、folio を除く外の部品は **35 本**（`cargo tree --workspace --target all -e normal,build,dev` の重複を除いた名の数も 35）。encoding_rs を引くのは yaml-rust2 だけで（`cargo tree -i encoding_rs`）、encoding_rs だけが引く部品が 8 本在る: cfg-if・core_detect・multiversion・multiversion-macros・multiversion_no_op・rustversion・scopeguard・simdutf8（multiversion-macros は multiversion から）。
3. 使ってよい条件（ライセンス）: 35 本のうち MIT か Apache-2.0 の選びだけでは満たせない条件を持つのは 3 本（`cargo metadata` の resolve の node の license の欄）: encoding_rs（`(Apache-2.0 OR MIT) AND BSD-3-Clause`・本便で消える）・foldhash（`Zlib`・yaml-rust2 が必ず引く hashlink → hashbrown から・残る）・unicode-ident（`(MIT OR Apache-2.0) AND Unicode-3.0`・proc-macro2 → quote / syn → clap_derive の道で組み立ての時だけ使い folio の binary には入らない・本便の前後で変わらず残る）。
4. folio が使う yaml-rust2 の口は、src が `parser::{Event, MarkedEventReceiver, Parser}`・`scanner::{Marker, TScalarStyle}`、build.rs が `Yaml`・`YamlLoader`・`yaml::Hash`、tests が `YamlLoader::load_from_str`・`YamlEmitter` ほか（読みと書きの口）だけ。字の符号を見分けて読む口（`YamlDecoder`・`YAMLDecodingTrap`）は 0 か所（`git grep`）。
5. **振る舞いが同じ根拠（yaml-rust2 0.10.4 の source）**: 機能 `encoding` が切り替えるのは `src/yaml.rs` の `mod encoding`（`YamlDecoder`・`YAMLDecodingTrap`・`YAMLDecodingTrapFn` とその再公開）と、その機能が在るときだけ組む歯だけ。folio が使う読み（scanner・parser・`YamlLoader`）と書き（`YamlEmitter`）には機能の有無で組み方を変える印（cfg の feature の条件）が 1 つも無く、読みは既定の機能の有無に依らず UTF-8 の字だけを受ける。見本の写しで床 4 本の答えと `folio build` の出力（40 file の sha）が base と同じだった（(h)）。

### (b) 直す先（変えないもの）

1. `crates/folio/Cargo.toml` の yaml-rust2 の 2 行を、2 か所とも `yaml-rust2 = { version = "0.10", default-features = false }` にする（同じ 1 行の形・機能を名指さない）。
2. `Cargo.lock` を cargo に解き直させる（`cargo build` が manifest に合わせて使わない package を消す）。差分は**消すだけ**: package 9 本（encoding_rs と (a) 2. の 8 本）と、yaml-rust2 の依存の列の `encoding_rs` の 1 行。ほかの package の版は変えない（`cargo update` は撃たない）。外の部品は **35 → 26 本**（Cargo.lock の package の数・cargo tree の名の数とも・参考値）。foldhash（Zlib）と unicode-ident（Unicode-3.0 を AND で持つ）は残る。解いた機能の差は、yaml-rust2 の `default`・`encoding` が消えることのほかに、syn の機能 3 つ（extra-traits・visit・visit-mut・入れていたのは消える multiversion-macros だけ）が減ること（`cargo tree -e features`）。syn は clap_derive の組み立ての時の依存で、機能は API を足すだけ＝clap_derive が生む code は変わらない（`folio build` の出力の byte の比べで確かめる・(h)）。
3. 変えないもの: src・build.rs・clap の行・workspace の Cargo.toml・ほかの歯。行 R-6 の値（未定）。

### (c) 歯（f212_・base で 0 件）

新しい file `crates/folio/tests/deps.rs`（binary を撃たない・file を読むだけ）に 2 本。

1. **f212_manifest_turns_off_yaml_rust2_default_features**: `crates/folio/Cargo.toml` を行ごとに読み、`[dependencies]` と `[build-dependencies]` の節のそれぞれで `yaml-rust2 =` で始まる行がちょうど 1 つ在り、その値（空白を除いた字）が `default-features=false` を含み、機能 `"encoding"` を名指さないこと。行の形は (b) 1. の 1 行の形（`[dependencies.yaml-rust2]` の表の形・鍵を引用する形 `"yaml-rust2" =`・節の見出しの後ろに注を付ける形 `[dependencies] # …` は数えない＝同じ意味でも歯が落ちる。作業者は (b) 1. の字のとおり、鍵を引用せず、節の見出しに注を付けずに書く）。
2. **f212_lockfile_resolves_no_encoding_rs**: `Cargo.lock` の package ごとの名と依存の名の列を読み、一覧が読めていること（folio と yaml-rust2 が在り、yaml-rust2 の依存に hashlink が在る・読めない一覧を合格にしない・P-4.1）を先に確かめてから、yaml-rust2 の依存に encoding_rs が無く、package にも encoding_rs が無いこと。
3. **RED**: base（0c910db）に歯の file だけを当てると 2 本とも落ちる（1 本目 =「[dependencies] の yaml-rust2 は既定の機能を外すはず: "0.10"」・2 本目 =「yaml-rust2 の依存に encoding_rs が残る: ["arraydeque", "encoding_rs", "hashlink"]」）。見本 8b24827 では 2 本とも緑。
4. 歯は外の部品の本数（26）も、消えた 8 本の名も固定しない（(d) 3.）。

### (d) 採らなかった形

1. **歯の中から `cargo metadata` / `cargo tree` を撃つ**: 解いた機能を直に見られるが、歯の中から cargo を入れ子に撃つと、検査の cargo と組み立て置き場の鍵を取り合い、全 target を解くと部品の取得（network）に依る。`Cargo.lock` は有効になった任意の依存だけを依存の列に載せる（見本の差分で確かめた）ので、file を読む歯で足りる。解いた一覧が manifest と揃っていることは verify の `--locked` が数える（(f) 4.）。
2. **外の部品の本数を歯で固定する**: 行 R-6（予算）の値が未定のまま歯に数を置くと、散文でも表でもない所に規則を持ち込む（N-2.1・P-5.1）。消えた 8 本も、ほかの部品が正当に引き直しうるので固定しない。
3. **既存の歯の file に置く**: 外の部品を数える歯の file は無い。新しい file 1 本（新しい dir は無い）にした。

### (e) 既存の歯のうち落ちるもの・突然変異

1. 既存の歯で落ちるものは無い。見本 8b24827 の workspace の nextest は 1184 / 1184（参考値・host で同時 1 本・base 0c910db の本数 1182 より f212_ の 2 本多い）・clippy 0 警告。base の workspace の nextest は host の負荷を減らすため撃っていない（base の verify の 3 行は撃った・(f) 4.）。
2. 突然変異 9 通り（控えの mut.py・見本の木で 1 つずつ当てて f212_ の 2 本を撃つ）: 期待と違う 0。

| 変異 | 期待 |
| --- | --- |
| M1 `[dependencies]` の 1 か所だけ既定の機能に戻す | 2 本とも落ちる（cargo が encoding_rs を解き戻す） |
| M2 `[build-dependencies]` の 1 か所だけ既定の機能に戻す | 2 本とも落ちる |
| M3 2 か所とも既定の機能を外したまま `features = ["encoding"]` を名指す | 2 本とも落ちる |
| M4 2 か所とも `default-features = true` と書く | 2 本とも落ちる |
| M5 `[dev-dependencies]` に既定の機能の yaml-rust2 を足す | 2 本目が落ちる |
| M6 folio の `[dependencies]` に encoding_rs を直に足す | 2 本目が落ちる |
| M7 `Cargo.lock` だけ base に戻し `--locked` で撃つ | cargo が撃つ前に断る（verify の 1 行目が落ちる） |
| M8 `Cargo.lock` だけ base に戻し `--locked` 無しで撃つ | 緑（cargo が撃つ前に解き直して消す・(d) 1. と (f) 4. の理由） |
| M9 `Cargo.lock` だけ base に戻し、組んだ歯の binary を直に撃つ | 2 本目が落ちる |

### (f) 大きさ・余地・verify と done

1. **write-set 3 本**: `crates/folio/Cargo.toml`・`Cargo.lock`（縮む・印 `-`）・`crates/folio/tests/deps.rs`（新規・印 `+`）。差分は +105 −75・8,194 byte（参考値）。
2. **余地**: write-set に `crates/folio/src/` の file が無い＝余地を数える file は 0 本。
3. **size は S**（src の変更なし・manifest 4 行と歯 103 行）。
4. **verify は 3 行**で、done の塊と 1 対 1: `--locked --test deps f212_`（本便の歯 2 本・`--locked` で `Cargo.lock` が manifest と揃っていないと cargo が撃つ前に断る）・`--locked --test floor_cases`（凍結 fixture の 134 の組を YAML で読み書きして folio check に掛ける歯の file・12 本・YAML の読み書きが同じことの確かめ）・clippy。見本では 3 行とも rc 0（2・12 本と 0 警告）。base では歯の file tests/deps.rs が無いので f212_ の行は cargo が歯の file を見つけられず終了コード 101、ほかは緑（12 本と 0 警告）（控えの verify.sh・logs/verify-*.log）。

### (g) 受付・並行の便・運ばないもの

1. **受付の順**: §0 の受付の前提のとおり、持ち主の承認の記帳の後。
2. **並行の枝との重なり**（2026-09-29 20:4x の origin の枝・読むだけ）: 本流に取り込まれていない枝で、write-set の 3 本（と workspace の Cargo.toml）を書く枝は無い。便 211（行 hf・serve.rs と tests/serve.rs）とも重ならない。便 211 の歯（追跡される file の tailnet の住所の字）に、本便の 3 本は当たらない。受付の時点で本流が 0c910db と違い、`Cargo.lock` が変わっていれば、cargo に解き直させて (b) 2. の「消すだけ」を数え直す。
3. 本便の後に yaml-rust2 の既定の機能を戻す・encoding_rs を足す変更は、workspace の nextest で落ちる（A-3.1 の確認を経た変更なら、同じ便で歯を直す）。
4. 運ばないもの: 行 R-6 の値・使ってよい条件の一覧（どちらも持ち主の裁定の要る行）・foldhash（Zlib）を外すこと（yaml-rust2 の必須の依存）・ほかの部品の版の更新。

### (h) 床・面・天井が変わらないこと

設計文書の正本も生成器も触らない。見本で床 4 本（check・inject --check・schema --check・derive --check）は base と同じく rc 0（check は合格・違反 0・まだ分からない 0）、`folio build` の出力は base と file の数（40）も sha も同じ（参考値・控えの run.sh と chain.sh）。門は「通す（設計文書の正本を書き換えない便）」。

## 2. 範囲

- 入れる: §1 (b) の 1 と 2（manifest の 2 行と、解き直した `Cargo.lock`）と、f212_ の歯 2 本。
- 入れない: src・build.rs・ほかの部品の版・規則の表の行・新しい dir・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| U1 | 既定の機能を外す manifest | yaml-rust2 の 2 か所を `default-features = false` の 1 行にする |
| U2 | 解き直した依存の一覧 | `Cargo.lock` から encoding_rs と 8 本を消す |
| U3 | 外の部品の歯 | tests/deps.rs の f212_ の 2 本と、manifest と `Cargo.lock` を読む 2 つの下請けの関数 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate を**減らす**（encoding_rs と 8 本）。A-3.1 の持ち主の確認が受付の前提（§0）。足す crate は無い。新しい dir も無い。
- host に要る命令: cargo（解き直しは消すだけで、部品の取得は要らない）。
- 着地の後に席が見ること: 本流の target/debug/folio を組み直す。tsuzuri の席へ、folio の外の部品が 26 本になり encoding_rs が無くなったことを知らせる。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "hg"
title = "folio の外の部品を減らす（条 A-3.1・持ち主の承認の記帳が受付の前提）。crates/folio/Cargo.toml の yaml-rust2 を [dependencies] と [build-dependencies] の 2 か所とも版 0.10 と default-features = false を持つ 1 行の形にし、Cargo.lock を cargo に解き直させて、既定の機能 encoding だけが引いていた encoding_rs と、それだけが引いていた 8 本（cfg-if・core_detect・multiversion・multiversion-macros・multiversion_no_op・rustversion・scopeguard・simdutf8）を消す（消すだけ・ほかの版は変えない・外の部品 35 から 26 本）。folio が使う読み書きの口（YamlLoader・parser・scanner・YamlEmitter）は機能 encoding に依らないので振る舞いは同じ。新しい tests/deps.rs に f212_ の歯 2 本（2 か所の既定の機能が外れていること・Cargo.lock に encoding_rs が無いこと）を足す"
req = ["FR5", "FR7"]
section = "1"
write-set = ["crates/folio/Cargo.toml", "-Cargo.lock", "+crates/folio/tests/deps.rs"]
verify = ["cargo nextest run -p folio --locked --test deps f212_", "cargo nextest run -p folio --locked --test floor_cases", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/deps.rs の f212_ の歯 2 本（crates/folio/Cargo.toml の yaml-rust2 の 2 か所が既定の機能を外していること・Cargo.lock に encoding_rs が無いこと）が --locked で緑（Cargo.lock が manifest と揃っている）、凍結 fixture の 134 の組を回す歯の file（tests/floor_cases.rs の全部）が --locked で緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、Cargo.lock の差分は消すだけ（package 9 本と yaml-rust2 の依存の encoding_rs の 1 行）で、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main と byte で同じ"
<!-- contracts:end -->
