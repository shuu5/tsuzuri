//! 画面の部品: 「?」の注釈（help・便 g-frame）と hover の card（hover・便 g-parts で層と置き場と猶予を動かす）と、
//! 注釈の行 `{fig:名}` を描く手順と流れの図（fig・行 g-help-fig・host でも組む SVG の字）と、
//! 電文の節点 1 つから組む節点の card（nodecard・行 g-card-node・host でも組む）と、
//! 初心者の mode の home の頁の最初の案内（coach・行 g-coach・決めと置き場は host でも組む）と、
//! 帯の印を押すと開く窓の枠（modal・行 g-win-frame・閉じる判定と窓の積みは host でも組む）と、
//! 札と一覧の行を押すと開く吹き出し（pop・行 g-pop・欄の組みと置き場と開閉は host でも組む）と、
//! 吹き出しの run の段の流れと run の歴（runflow・行 g-pop-flow・host でも組む）と、
//! pipeline の札の段ごとの要の 1 行（keyline・行 g-pipe-cards・host でも組む）と、
//! 表示の型で概要を 1 つ選ぶ部品と長い概要の畳み（sumpick・行 g-sum-pick・host でも組む）。

pub mod coach;
pub mod fig;
pub mod help;
pub mod hover;
pub mod keyline;
pub mod modal;
pub mod nodecard;
pub mod pop;
pub mod runflow;
pub mod sumpick;
