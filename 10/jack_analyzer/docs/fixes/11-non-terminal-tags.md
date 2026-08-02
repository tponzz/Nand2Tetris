# Fix 11: 非終端記号の開始・終了タグが出力されていない

## 症状

Fix 09まででパースは最後まで成功するようになったが、生成された`Out.xml`と
`../ArrayTest/Main.xml`を比較すると、`class`と`subroutineDec`以外の
非終端記号（`parameterList`, `varDec`, `subroutineBody`, `statements`,
`letStatement`, `ifStatement`, `whileStatement`, `doStatement`,
`returnStatement`, `expression`, `term`, `expressionList`）を囲む
開始・終了タグが丸ごと出力されておらず、終端トークンだけが並んでいた。

## 原因

`compile_class`と`compile_subroutine`は`write_tag("class")`のように
自分の非終端記号タグを出力していたが、他の`compile_XXX`関数には最初から
そのコードが存在しなかった（実装が未完成だった）。全て同一原因（該当関数に
`write_tag`呼び出しが無いだけ）なので、1つのバグとして一括対応した。

## 修正方針・注意点

各`compile_XXX`関数の先頭で開始タグ、末尾（`Ok(())`の直前）で終了タグを
書き込むようにした。単純な追加だけでは済まない箇所が2つあった。

### (a) `compile_statements`の投機的な呼び出しとの衝突

`compile_statements`は`compile_let().is_ok() || compile_if().is_ok() || ...`
という形で、どの文型に一致するかを**試しながら**判定する実装になっている。
`accept`/`expect`は「一致しなければ何もしない」という設計だが、
`write_tag`には元々そのようなガードが無い。そのため各関数の先頭に
無条件で開始タグを書くと、例えば`while`文の手前で`compile_let`・
`compile_if`を「試して失敗」した際にも`<letStatement>`・`<ifStatement>`
が出力されてしまい、閉じタグの無い余分なタグが混入した。

対策として、`current`が対象キーワードと一致するかを**タグを書く前に**
消費なしで確認する`peek_keyword`を追加し、`compile_let`/`compile_if`/
`compile_while`/`compile_do`/`compile_return`の先頭で
「一致しなければ即座にErrを返す（何も書かない）」ガードを入れた。

### (b) 空の`expressionList`のタグが出力されない

`Output.println()`のように引数が0個の呼び出しでは、Fix 07で
`compile_expression_list()`の呼び出し自体をスキップする実装にしていたため、
`<expressionList></expressionList>`タグも出力されなくなっていた。
`compile_expression_list`自体が0個の式（次が`)`なら式なし）に対応するように
直し、呼び出し側(`compile_do`, `compile_term`)の特殊分岐
（`accept_symbol('(') && !accept_symbol(')')`のような書き方）を撤去して、
常に`compile_expression_list()`を呼ぶ単純な形に統一した。

## Diff（抜粋・主要部分のみ）

```diff
--- a/src/engine.rs
+++ b/src/engine.rs
@@ 各 compile_parameter_list / compile_subroutine_body / compile_var_dec /
@@ compile_class_var_dec / compile_statements / compile_expression /
@@ compile_term / compile_expression_list
     pub fn compile_XXX(&mut self) -> Result<(), JAError> {
+        self.write_tag("xxx");
         ...
+        self.write_tag("/xxx");
         Ok(())
     }
```

```diff
--- a/src/engine.rs
+++ b/src/engine.rs
@@ 新規追加
+    // Non-consuming lookahead, used by the statement compilers to check which
+    // alternative applies *before* writing an opening tag: compile_statements
+    // tries compile_let/if/while/do/return speculatively, and unlike accept()
+    // write_tag() has no built-in "only if matched" guard.
+    fn peek_keyword(&self, keyword: Keyword) -> bool {
+        self.current == Some(Token::Keyword(keyword))
+    }
```

```diff
--- a/src/engine.rs
+++ b/src/engine.rs
@@ compile_let / compile_if / compile_while / compile_do / compile_return
     pub fn compile_let(&mut self) -> Result<(), JAError> {
+        if !self.peek_keyword(Keyword::Let) {
+            return Err(JAError::Compile(CompileErrKind::Let));
+        }
         let ekind = &CompileErrKind::Let;
         self.write_tag("letStatement");
```
(`If`/`While`/`Do`/`Return`も同様のガードを追加)

```diff
--- a/src/engine.rs
+++ b/src/engine.rs
@@ pub fn compile_expression_list
     pub fn compile_expression_list(&mut self) -> Result<(), JAError> {
         self.write_tag("expressionList");
-
-        // exp
-        self.compile_expression()?;
-
-        // if appear ',', expect additional exp
-        while self.accept_symbol(',')? {
-            self.compile_expression()?;
-        }
+
+        if !matches!(self.current, Some(Token::Symbol(')'))) {
+            // exp
+            self.compile_expression()?;
+
+            // if appear ',', expect additional exp
+            while self.accept_symbol(',')? {
+                self.compile_expression()?;
+            }
+        }

         self.write_tag("/expressionList");
         Ok(())
     }
```

```diff
--- a/src/engine.rs
+++ b/src/engine.rs
@@ compile_do / compile_term の呼び出し側
-            if self.accept_symbol('(')? && !self.accept_symbol(')')? {
+            if self.accept_symbol('(')? {
                 self.compile_expression_list()?;
                 self.expect_symbol(')', ekind)?;
             } else if self.accept_symbol('.')? {
                 self.expect_identifier(ekind)?;
                 self.expect_symbol('(', ekind)?;
-                if !self.accept_symbol(')')? {
-                    self.compile_expression_list()?;
-                    self.expect_symbol(')', ekind)?;
-                }
+                self.compile_expression_list()?;
+                self.expect_symbol(')', ekind)?;
             }
```

## 検証

- `cargo build` が通ることを確認。
- `cargo run -- ../ArrayTest/Main.jack` がエラー無く完走。
- `Out.xml`と`../ArrayTest/Main.xml`を空白を全て除去して比較したところ、
  残る差分は以下の2点のみになった（いずれも別バグ、後続のFixで対応）。
  1. `<` 記号がXMLエスケープ(`&lt;`)されていない。
  2. `stringConstant`の値に引用符`"`がそのまま残っている。
