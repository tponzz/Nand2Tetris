# Fix 01: 最初のトークンが一度も読み込まれない (priming漏れ)

## 症状

`ArrayTest/Main.jack` を実行すると、パースが1文字も進まないうちに
`Error: Compile(Class)` で即座に失敗する。

## 原因

`CompilationEngine`は`current: Option<Token>`をトークンの先読み(lookahead)
キャッシュとして使う設計になっている。`accept`/`expect`は「`current`が条件に
一致すればそれを消費し、次のトークンを`advance()`で読み込む」という処理を
行うが、これは**現在のトークンを消費した後に次を先読みする**という前提に
立っている。

つまり `current` に最初のトークンが入るきっかけ（=最初の“消費”イベント）が
存在しない。`new()`で`current: None`のまま構築され、誰も最初の一回だけ
`advance()`を呼んでいなかったため、`compile_class`最初の
`expect_keyword(Class, ...)`が`current == None`に対して判定され、必ず失敗する。

これはlookahead方式の構造上避けられない「初期化(priming)」の必要性であり、
`accept`/`expect`側の実装ミスではない（Peekableイテレータの最初の`peek()`が
必要なのと同じ理由）。

## 修正方針

一度、`compile_class`の先頭に直接`self.advance()?`を書く案を検討したが、
以下の理由でコンストラクタ(`CompilationEngine::new()`)側に置く方針に変更した。

- `compile_class`は「classの文法」だけに集中すべきで、「エンジンの初期化」
  という別の責務を混ぜたくない。
- `new()`が返した時点で「`current`は常に最初のトークンを保持している」という
  不変条件をコンストラクタ自身が保証すれば、以降どのメソッドも
  「currentは必ず有効」と信じてよくなり、呼び出し側の priming 漏れの心配が
  なくなる。

この変更に伴い、`new()`の戻り値を`Result<Self, io::Error>`から
`Result<Self, JAError>`に統一した（`advance()`は`JAError`を返すため、
priming呼び出しを`new()`内に置くには型を揃える必要がある。ちょうど
`JAError::Io`がio::Errorのラップ用に既に存在していたため、そこに寄せた）。

## Diff

```diff
--- a/src/engine.rs
+++ b/src/engine.rs
@@
-use std::{
-    fmt::Display,
-    fs::File,
-    io::{self, Write},
-};
+use std::{fmt::Display, fs::File, io::Write};

 use crate::{
     CompileErrKind, JAError,
     tokenizer::{Keyword, Token, TokenType, Tokenizer},
 };

 pub struct CompilationEngine {
     current: Option<Token>,
     tokenizer: Tokenizer,
     xml_out: File,
 }

 impl CompilationEngine {
-    pub fn new(path_in: &str, path_out: &str) -> Result<Self, io::Error> {
-        let tokenizer = Tokenizer::new(path_in)?;
-        let xml_out = File::create(path_out)?;
-
-        Ok(Self {
-            tokenizer,
-            xml_out,
-            current: None,
-        })
-    }
+    pub fn new(path_in: &str, path_out: &str) -> Result<Self, JAError> {
+        let tokenizer = Tokenizer::new(path_in).map_err(|_| JAError::Io)?;
+        let xml_out = File::create(path_out).map_err(|_| JAError::Io)?;
+
+        let mut engine = Self {
+            tokenizer,
+            xml_out,
+            current: None,
+        };
+
+        // prime the first lookahead token so current is always valid once new() returns
+        engine.advance()?;
+
+        Ok(engine)
+    }
```

```diff
--- a/src/lib.rs
+++ b/src/lib.rs
@@
     match CompilationEngine::new(&cli.source, sink) {
         Ok(mut engine) => engine.compile_class(),
         Err(e) => {
-            eprintln!("Failed to open files: {}", e);
+            eprintln!("Failed to open files: {:?}", e);
             exit(1)
         }
     }
```

(`JAError`は`Display`を実装していないため`{}` → `{:?}`に変更。)

## 検証

- `cargo build` が通ることを確認。
- `cargo test` は既存の1件（`test_token_type_for_int_const_invalid`、
  整数定数の範囲外チェック漏れ）を除き全てパス。このテストは
  `tokenizer.rs`側の別バグであり、今回の変更とは無関係（今回`tokenizer.rs`は
  未変更）。
- `cargo run -- ../ArrayTest/Main.jack` の失敗地点が `Compile(Class)` から
  次のバグ（クラス変数宣言の必須呼び出し、Fix 02）まで前進したことを確認。
