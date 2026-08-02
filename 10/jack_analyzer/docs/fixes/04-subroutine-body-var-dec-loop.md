# Fix 04: サブルーチン本体内の `var` 宣言が1回しか呼ばれない

## 症状

Fix 03適用後も `cargo run -- ../ArrayTest/Main.jack` は
`Error: Compile(SubroutineBody)` のまま失敗する。

## 原因

`compile_subroutine_body`は以下のようになっていた。

```rust
// var dec
// 0..*     <- コメントには「0回以上」と書かれている
self.compile_var_dec()?;
```

コメント通り`varDec`は0回以上出現し得るが、実装は`compile_var_dec()`を
1回だけ呼んでいた。`ArrayTest`の`main()`は

```
var Array a;
var int length;
var int i, sum;
```

と`var`宣言が3つ連続するため、1つ目の`var Array a;`を処理した直後に
`compile_statements()`へ進んでしまい、2つ目の`var int length;`が
「文」として解釈されようとして構文が崩れる。

## 修正方針

`current`が`Keyword::Var`である間ループするように変更した
（Fix 02でクラス変数宣言に対して行ったのと同じパターン）。

## Diff

```diff
--- a/src/engine.rs
+++ b/src/engine.rs
@@ pub fn compile_subroutine_body
         // var dec
         // 0..*
-        self.compile_var_dec()?;
+        while matches!(self.current, Some(Token::Keyword(Keyword::Var))) {
+            self.compile_var_dec()?;
+        }
```

## 検証

- `cargo build` が通ることを確認。
- 診断用の`eprintln`を`expect`に一時的に仕込んで実行したところ、
  3つの`var`宣言が正しく全て処理され、最初の文
  `let length = Keyboard.readInt("HOW MANY NUMBERS? ");`の
  `Keyboard.readInt(`まで到達したことを確認（このFix自体は成功）。
- ただしその直後、文字列リテラル`"HOW MANY NUMBERS? "`を読もうとした
  タイミングで`current=None`になり、依然として`Compile(SubroutineBody)`で
  失敗する。これはFix 04とは別の原因（トークナイザが文字列リテラルを
  1トークンとして扱えないバグ）によるもので、次のFix 05で対応する。
  診断ログは確認後に削除済み。
