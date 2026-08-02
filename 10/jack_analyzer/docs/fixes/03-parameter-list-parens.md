# Fix 03: パラメータリストの `(` `)` が二重に消費される

## 症状

Fix 02適用後、`cargo run -- ../ArrayTest/Main.jack` が
`Error: Compile(ParameterList)` で失敗する。

## 原因

`compile_subroutine`は`(`と`)`を自分で`expect_symbol`しつつ、その間で
`compile_parameter_list()`を呼んでいた。ところが`compile_parameter_list`も
内部で独自に`(`を`expect_symbol`、`)`を`accept_symbol`で消費しようとしていた。

```
compile_subroutine:
  expect_symbol('(')       <- ここで '(' を消費
  compile_parameter_list() <- 内部でまた '(' を expect_symbol → 失敗
  expect_symbol(')')
```

`ArrayTest`の`main()`は空のパラメータリストなので、`compile_subroutine`が
`(`を消費した時点で次のトークンは`)`になっている。そこで
`compile_parameter_list`が`(`を期待してすぐに失敗していた。

期待されるXML (`Main.xml`) を見ても、`(`・`)`は`<parameterList>`の
**外側**（`subroutineDec`直下）に出力されており、括弧は呼び出し元
(`compile_subroutine`)が責任を持つべき構造になっている。

```xml
<symbol> ( </symbol>
<parameterList>
</parameterList>
<symbol> ) </symbol>
```

## 修正方針

`compile_parameter_list`から`(`・`)`の処理を削除し、パラメータの中身
（`type varName (',' type varName)*`、0個以上）だけを扱うようにした。
ループは「型が読めなくなったら終了」という条件にし、`)`を明示的に
見に行く必要をなくした（`)`は型にもidentifierにもマッチしないため、
自然にループが終わる）。

## Diff

```diff
--- a/src/engine.rs
+++ b/src/engine.rs
@@
-    pub fn compile_parameter_list(&mut self) -> Result<(), JAError> {
-        // (
-        self.expect_symbol('(', &CompileErrKind::ParameterList)?;
-
-        while !self.accept_symbol(')')? {
-            if self.accept_type()? {
-                self.expect_identifier(&CompileErrKind::ParameterList)?;
-                self.accept_symbol(',')?;
-            }
-        }
-
-        Ok(())
-    }
+    // parameterList: ((type varName) (',' type varName)*)?
+    // The surrounding '(' ')' belong to the caller (compile_subroutine), not here.
+    pub fn compile_parameter_list(&mut self) -> Result<(), JAError> {
+        while self.accept_type()? {
+            self.expect_identifier(&CompileErrKind::ParameterList)?;
+            if !self.accept_symbol(',')? {
+                break;
+            }
+        }
+
+        Ok(())
+    }
```

（`compile_subroutine`側の`expect_symbol('(')` / `expect_symbol(')')`は
変更なし。括弧の所有権をどちらか一方に統一するのが目的で、今回は
呼び出し元に寄せた。）

## 検証

- `cargo build` が通ることを確認。
- `cargo run -- ../ArrayTest/Main.jack` の失敗地点が `Compile(ParameterList)`
  から次のバグ（`Compile(SubroutineBody)`）まで前進したことを確認。
