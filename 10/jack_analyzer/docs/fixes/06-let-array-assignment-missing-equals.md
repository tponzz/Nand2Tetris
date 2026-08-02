# Fix 06: 配列代入 `let arr[i] = expr;` で `=` が消費されていない

## 症状

Fix 05適用後、`cargo run -- ../ArrayTest/Main.jack` は依然
`Compile(SubroutineBody)`で失敗する。診断ログを仕込むと、2つ目の
`while`ループ本体（`let a[i] = Keyboard.readInt(...)`を含む箇所）の後、
`current`が`Symbol('=')`のまま止まっていた。

## 原因

`compile_let`は次のようになっていた。

```rust
// if `[ exp ]` otherwise '='
if self.accept_symbol('[')? {
    self.compile_expression()?;
    self.expect_symbol(']', ekind)?;
} else {
    self.expect_symbol('=', ekind)?;
}
```

Jack文法は `letStatement: 'let' varName ('[' expression ']')? '=' expression ';'`
であり、`=`は**配列添字の有無に関わらず必ず出現する**。しかし実装は
「`[...]`がある場合」と「`=`がある場合」を**互いに排他的な分岐**として
書いてしまっており、`let a[i] = ...`のように配列添字がある場合には`=`を
一切消費しないまま右辺の式の解析に入っていた。

`=`は式(`compile_expression`)の先頭として有効なトークンではないため、
`compile_term`が失敗し、`compile_let`全体がエラーで戻る。その時点で
`let`・identifier・`[`・expression・`]`はすでに消費済みという中途半端な
状態になり、`current`が`=`に固定されたまま次の文解析に進んでしまう。

## 修正方針

`[ expression ]`は「あれば処理する」独立したオプション部分とし、その後で
無条件に`=`を`expect_symbol`するように分離した。

## Diff

```diff
--- a/src/engine.rs
+++ b/src/engine.rs
@@ pub fn compile_let
-        // if `[ exp ]` otherwise '='
-        if self.accept_symbol('[')? {
-            self.compile_expression()?;
-            self.expect_symbol(']', ekind)?;
-        } else {
-            self.expect_symbol('=', ekind)?;
-        }
-
-        // exp
-        self.compile_expression()?;
+        // optional `[ exp ]` for array assignment
+        if self.accept_symbol('[')? {
+            self.compile_expression()?;
+            self.expect_symbol(']', ekind)?;
+        }
+
+        // '=' always follows, whether or not '[ exp ]' was present
+        self.expect_symbol('=', ekind)?;
+
+        // exp
+        self.compile_expression()?;
```

## 検証

- `cargo build` が通ることを確認。
- 診断ログで`let a[i] = Keyboard.readInt(...)`が正しく解析され、以前は
  `Symbol('=')`で止まっていた箇所を通過することを確認。
- その後、新たなバグ（`do`文の終端`;`が消費されていない、Fix 07）により
  `cargo run -- ../ArrayTest/Main.jack`は依然`Compile(SubroutineBody)`で
  失敗する。
