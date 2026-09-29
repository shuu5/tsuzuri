---
name: stage-setup
description: 表示先の設定の手引き。持ち主が起こす聞き取りで層 A の名と今の設定を見せ、選んだ PC を全体の既定と project ごとの上書きに置く。
disable-model-invocation: true
---

# 表示先の設定の手引き（stage-setup）

どの project の board をどの PC の画面に出すかを、版管理に載らない表示先の設定（全体の既定と project ごとの上書き）に置く手引きである（要件 FR16・判断の記録 ADR-15 の決定 (2)）。設定の値は層 A（器の host の面の端末の表）に在る名だけである。

## 決まり

- **持ち主が起こす聞き取りである。** この skill は持ち主が起こし、model は起こさない。持ち主が起こす聞き取りなので判断の記録 ADR-14 の決定 (5)（席への直接の問いの見張り）の例外にあたり、この skill の中でだけ表示先をチャットで問う。skill の外で表示先を決めてほしい時は、チャットで問わず board の問いで出す（ADR-15 の決定 (2)）。
- **問いは散文で 1 つずつ出す。** 問いごとに推奨を 1 つ添えて、持ち主の答えを待ってから次へ進む。AskUserQuestion の道具は使わない。
- **設定を読み書きするのは tz stage target の口だけである。** 設定の file と器の host の面を直に読まず、直に書かない。旗は --all と --project だけを使い、撃つ program と設定の path を差し替える旗は使わない。
- **命令は repo の根を cwd にして撃つ。** tz stage target は cwd の repo の git config の scribe2.statedir から器の state dir を読む。tz が PATH に無ければ、repo の根で build した target/debug/tz を同じ引数で撃つ（根からの相対の path で撃ち、絶対の path を書かない）。
- **名は形だけで書く。** 下の命令の <端末の名> と <project> は置き字で、実の名は show の出力から読んで置き換える。show の出力の名を版管理に載る file と commit の字に写さない（行 D-4）。
- **落ちたら止まる。** 命令が rc 1 で落ちたら、断りの行をそのまま持ち主に見せて止まる。file を直に書く・別の旗で撃ち直すといった回り道をしない。

## 1 今の設定を見る

今の設定と層 A の名を並べる。

```
tz stage target show
```

出力の行の読み方。

- `既定 <端末の名>` か `既定 無し`: 全体の既定。
- `上書き <project> <端末の名>`: project ごとの上書き（上書きごとに 1 行）。
- `project <project> <端末の名>（既定）` か `project <project> <端末の名>（上書き）`: その project に効く表示先と出所。名が層 A に無ければ出所の後に `・層 A に無い` が付く。
- `project <project> 無し（席の目と URL に落ちる）`: 既定も上書きも無い project。
- `層 A の名 <名の列>` か `層 A の名 無し`: 設定に置ける名。

## 2 層 A に PC が無いか足りない時

層 A の名が無いか、出したい PC の名が無ければ、次を案内する。

- 層 A を書くのは持ち主か器の席で、tsuzuri の席は書かない。
- PC を足すのは今は器の席の運びである（器の覚え書き s2-07l.727 の書きの口が着地したら画面から足す）。
- 足す行の欄 profile-dir（席専用の Chrome の設定の置き場）は、その PC のホームの下の ~/.cache/tsuzuri-stage にする（ADR-15 の決定 (4)・持ち主の裁定 t3-hub.59.2）。
- ほかの欄は name・ssh・chrome・os・display の欄の名だけを示し、値の見本は出さない。

層 A の名が無いままなら、ここで止まる。

## 3 全部の project の既定を選ぶ

層 A の名を並べて、全部の project に出す PC を問う。推奨は、今の既定が層 A に在ればその名、層 A の名が 1 つならその名とする。

set --all は project ごとの上書きを全部外す。1 の出力に上書きの行が在ったら、撃つ前に外れる上書きを並べて見せ、外してよいかを確かめる。

```
tz stage target set --all <端末の名>
```

## 4 project ごとの上書き

既定と違う PC に出したい project が在るかを問う（推奨は上書きを置かず既定に任せること）。在れば、その project に上書きを置く。

```
tz stage target set --project <project> <端末の名>
```

上書きを外して既定に戻す project が在れば、上書きを外す。

```
tz stage target clear --project <project>
```

## 5 確かめて終える

show をもう 1 度撃ち、project ごとに効く表示先を持ち主に見せて終える。

```
tz stage target show
```

この skill は設定を置くだけで窓を開かない。表示先を替えると、その PC に初めて見せる時の 1 度だけ窓が開く（ADR-15 の決定 (6)）ことを添える。
