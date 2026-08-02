# Fix 07: `do`文が終端の `;` を消費しない／引数0個の呼び出しに対応していない

## 症状

Fix 06適用後、`cargo run -- ../ArrayTest/Main.jack` は依然
`Compile(SubroutineBody)`で失敗する。診断ログでは、3つ目の`do`文
（`do Output.println();`）の付近で`current`が`Symbol(';')`のまま止まる、
あるいは`current=Some(Keyword(Do))`のまま複数回失敗が続いていた。

## 原因

`compile_do`には2つの問題があった。

1. **終端の`;`を消費していない**。`doStatement: 'do' subroutineCall ';'`
   のはずだが、`subroutineCall`部分を解析した後、`;`を`expect_symbol`する
   コードが存在しなかった。これにより`do`文の後に必ず`;`が残ってしまい、
   次の文解析（let/if/while/do/return のどれにも`;`はマッチしない）が
   そこで止まる。

2. **`Foo.bar()`のように引数が0個の`.`呼び出しに対応していない**。
   識別子直接呼び出し(`foo()`)の分岐は
   `accept_symbol('(') && !accept_symbol(')')`という書き方で、空の`()`が
   来た場合に`)`まで正しく消費できていたが、`.`分岐は
   ```rust
   self.expect_symbol('(', ekind)?;
   self.compile_expression_list()?;   // 無条件に呼ぶ
   self.expect_symbol(')', ekind)?;
   ```
   となっており、`compile_expression_list`は必ず1つ以上の式を要求する
   実装だったため、`Output.println()`のように引数が無い場合
   `compile_term`が`)`を見て失敗し、`do`文全体がエラーになっていた。

## 修正方針

- `.`分岐でも、識別子直接呼び出しの分岐と同様に「`)`が直後に来るなら
  空の引数リストとして扱う」ようにし、引数がある場合だけ
  `compile_expression_list`を呼ぶようにした。
- `subroutineCall`の解析が終わった後、無条件に`;`を`expect_symbol`する
  ようにした。

## Diff

```diff
--- a/src/engine.rs
+++ b/src/engine.rs
@@ pub fn compile_do
         // subroutineCall
         if self.accept_identifier()? {
             if self.accept_symbol('(')? && !self.accept_symbol(')')? {
                 self.compile_expression_list()?;
                 self.expect_symbol(')', ekind)?;
             } else if self.accept_symbol('.')? {
                 self.expect_identifier(ekind)?;
                 self.expect_symbol('(', ekind)?;
-                self.compile_expression_list()?;
-                self.expect_symbol(')', ekind)?;
+                if !self.accept_symbol(')')? {
+                    self.compile_expression_list()?;
+                    self.expect_symbol(')', ekind)?;
+                }
             }
         }

+        // ;
+        self.expect_symbol(';', ekind)?;
+
         Ok(())
     }
```

## 検証

- `cargo build` が通ることを確認。
- 診断ログで3つの`do`文
  (`Output.printString(...)`, `Output.printInt(...)`, `Output.println()`)
  が全て正しく解析され、以前は`Symbol(';')`や`Keyword(Do)`で止まっていた
  箇所を通過することを確認。
- その後、新たなバグ（`return;`（値なしreturn）に対応していない、Fix 08）
  により`cargo run -- ../ArrayTest/Main.jack`は依然
  `Compile(SubroutineBody)`で失敗する。
