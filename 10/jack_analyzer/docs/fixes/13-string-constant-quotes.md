# Fix 13: 文字列定数の値に引用符が残っている

## 症状

Fix 12までで残っていた最後の差分。

```
< <stringConstant> "HOW MANY NUMBERS? " </stringConstant>
---
> <stringConstant> HOW MANY NUMBERS?  </stringConstant>
```

## 原因

`Tokenizer::string_val()`は`eq_token_type(&TokenType::StringConst)`が返す
トークン全体（前後の`"`を含む）をそのまま返していた。nand2tetris仕様の
`stringVal()`は**引用符を含まない中身**を返すことになっている。

## 修正方針

`string_val()`で前後1文字（`"` `"`）を取り除いてから返すようにした。
文字列定数と判定された時点で先頭・末尾が`"`であることは
`token_type()`側で保証されているため、単純なスライスで安全に取り除ける。

## Diff

```diff
--- a/src/tokenizer.rs
+++ b/src/tokenizer.rs
@@
     pub fn string_val(&self) -> Option<&str> {
-        self.eq_token_type(&TokenType::StringConst)
+        // strip the surrounding quotes; the value itself never contains one
+        let with_quotes = self.eq_token_type(&TokenType::StringConst)?;
+        Some(&with_quotes[1..with_quotes.len() - 1])
     }
```

## 検証

- `cargo build` が通ることを確認。
- `cargo run -- ../ArrayTest/Main.jack` を実行し、生成された`Out.xml`と
  `../ArrayTest/Main.xml`を、空白・改行コード(`\r\n`)を全て取り除いた上で
  比較したところ、**完全に一致**することを確認（`md5sum`が同一）。

```
$ tr -d ' \t\n\r' < Out.xml | md5sum
82243de7ff12f9d514a171cded1bb7e6
$ tr -d ' \t\n\r' < ../ArrayTest/Main.xml | md5sum
82243de7ff12f9d514a171cded1bb7e6
```

これで`ArrayTest`のパース・XML出力は仕様通りになった。
