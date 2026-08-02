# Fix 12: `<` `>` `&` 記号がXMLエスケープされていない

## 症状

Fix 11までで非終端タグも揃ったが、`Out.xml`と`../ArrayTest/Main.xml`を
空白を除去して比較すると、`while (i < length)`の`<`部分だけ差分が残る。

```
< <symbol>i</symbol><symbol><</symbol><term>...
---
> <symbol>i</symbol><symbol>&lt;</symbol><term>...
```

## 原因

`accept()`は`Token::Symbol(value)`をそのまま`write_tag_with_value`に渡し、
`char`の`Display`実装（素の文字）でXMLに書き込んでいた。Jackの記号セット
`{}()[].,;+-*/&|<>=~`には`<`, `>`, `&`というXMLのメタ文字が含まれるが、
これらはXML本文中では`&lt;`, `&gt;`, `&amp;`にエスケープする必要がある
（nand2tetrisの仕様でも明示されている）。

## 修正方針

`accept()`内の`Token::Symbol`処理で、`<`/`>`/`&`の場合だけ対応する
エスケープ済み文字列に変換してから書き込むようにした。

## Diff

```diff
--- a/src/engine.rs
+++ b/src/engine.rs
@@ fn accept
         let written = match token {
-            Token::Symbol(value) => self.write_tag_with_value(&value, TokenType::Symbol),
+            Token::Symbol(value) => {
+                // '<', '>', '&' are also Jack symbols, but they are XML
+                // metacharacters and must be escaped in the output.
+                let escaped = match value {
+                    '<' => "&lt;".to_string(),
+                    '>' => "&gt;".to_string(),
+                    '&' => "&amp;".to_string(),
+                    other => other.to_string(),
+                };
+                self.write_tag_with_value(&escaped, TokenType::Symbol)
+            }
```

## 検証

- `cargo build` が通ることを確認。
- `Out.xml`で`<symbol> &lt; </symbol>`が正しく出力されることを確認。
- 空白を全て除去した`Out.xml`と`Main.xml`の差分は、文字列定数の引用符
  （Fix 13で対応）のみになった。
