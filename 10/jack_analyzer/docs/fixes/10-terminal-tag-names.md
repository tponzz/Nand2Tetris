# Fix 10: 整数定数・文字列定数のXMLタグ名が間違っている

## 症状

`Compile`エラーは出なくなったが、生成された`Out.xml`と期待される
`../ArrayTest/Main.xml`を比較すると、整数定数・文字列定数のタグ名が
一致しない。

```
< <stringconst> "HOW MANY NUMBERS? " </stringconst>
---
> <stringConstant> HOW MANY NUMBERS?  </stringConstant>
```

## 原因

`TokenType`の`Display`実装が、nand2tetrisの仕様で定められたタグ名
（`integerConstant`, `stringConstant`）ではなく、`intconst`,
`stringconst`という独自の文字列を出力していた。

```rust
TokenType::IntConst => write!(f, "intconst"),
TokenType::StringConst => write!(f, "stringconst"),
```

（`keyword`, `symbol`, `identifier`は元々仕様通りの名前だったため、
この2つだけが漏れていた。）

## 修正方針

`Display`実装を仕様通りのタグ名に修正した。

## Diff

```diff
--- a/src/tokenizer.rs
+++ b/src/tokenizer.rs
@@ impl Display for TokenType
-            TokenType::IntConst => write!(f, "intconst"),
-            TokenType::StringConst => write!(f, "stringconst"),
+            TokenType::IntConst => write!(f, "integerConstant"),
+            TokenType::StringConst => write!(f, "stringConstant"),
```

## 検証

- `cargo build` が通ることを確認。
- `Out.xml`で`<integerConstant>`/`<stringConstant>`タグが出力される
  ようになったことを確認。

## 補足（別バグとして確認済み・未修正）

タグ名は直ったが、`<stringConstant>`の中身に引用符`"`がそのまま残っている
（`"HOW MANY NUMBERS? "` vs 期待値`HOW MANY NUMBERS?  `）。これは
トークナイザが文字列リテラルの値を返す際に前後の`"`を取り除いていない
ことが原因で、次の非終端記号タグの修正とは別の独立したバグ。追って対応する。
