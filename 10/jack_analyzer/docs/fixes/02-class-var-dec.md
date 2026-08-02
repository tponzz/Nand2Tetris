# Fix 02: クラス変数宣言(classVarDec)が0回・複数回に対応していない

## 症状

Fix 01適用後、`cargo run -- ../ArrayTest/Main.jack` が
`Error: Compile(VarDec)` で失敗する。

## 原因

Jack文法では:

- `classVarDec`: `('static' | 'field') type varName (',' varName)* ';'`
  （クラス直下、0回以上）
- `varDec`: `'var' type varName (',' varName)* ';'`
  （サブルーチン本体内、0回以上）

の2つは「先頭キーワードが異なるだけで型・変数名部分は同じ」という関係だが、
実装では`compile_var_dec`という1つの関数だけが存在し、先頭キーワードを
常に`Keyword::Var`固定で`expect_keyword`していた。

`compile_class`はこの`compile_var_dec()`を**1回だけ・必須**で呼んでいた
(`self.compile_var_dec()?;`)。`ArrayTest`の`Main`クラスはクラス変数を
1つも持たないため、`{`の直後に来る`function`キーワードに対して
`expect_keyword(Var, ...)`が実行され、必ず失敗していた。

すなわちこのバグは2つの問題が重なっている。

1. クラス直下では本来`static`/`field`を見るべきなのに`var`を見ていた
   （キーワード不一致）。
2. クラス変数宣言は0個以上あり得るのに、1回限りの必須呼び出しになっていた
   （出現回数の誤り）。

## 修正方針

型・変数名部分（`type varName (',' varName)* ';'`）を
`compile_var_dec_tail`として共通化し、先頭キーワードの扱いだけを
`compile_var_dec`（`var`用、サブルーチン内）と`compile_class_var_dec`
（`static`/`field`用、クラス直下）に分離した。

`compile_class`側は、次のトークンが`static`または`field`である間ループして
`compile_class_var_dec()`を呼ぶよう変更し、0回以上のケースに対応した。

## Diff

```diff
--- a/src/engine.rs
+++ b/src/engine.rs
@@ pub fn compile_class
-        // class var declarations
-        self.compile_var_dec()?;
+        // class var declarations (0 or more)
+        while matches!(
+            self.current,
+            Some(Token::Keyword(Keyword::Static)) | Some(Token::Keyword(Keyword::Field))
+        ) {
+            self.compile_class_var_dec()?;
+        }
```

```diff
--- a/src/engine.rs
+++ b/src/engine.rs
@@
-    pub fn compile_var_dec(&mut self) -> Result<(), JAError> {
-        // var
-        self.expect_keyword(Keyword::Var, &CompileErrKind::VarDec)?;
-
-        // type : int|char|boolean|className(identifier)
-        self.expect_type(&CompileErrKind::VarDec)?;
-
-        // varName(identifier)
-        self.expect_identifier(&CompileErrKind::VarDec)?;
-
-        // if symbol ',', other varName(identifier)
-        while !self.accept_symbol(';')? {
-            if self.accept_identifier()? {
-                continue;
-            }
-
-            self.expect_symbol(',', &CompileErrKind::VarDec)?;
-        }
-
-        Ok(())
-    }
+    // type varName (',' varName)* ';' -- shared by varDec and classVarDec,
+    // which differ only in their leading keyword.
+    fn compile_var_dec_tail(&mut self) -> Result<(), JAError> {
+        // type : int|char|boolean|className(identifier)
+        self.expect_type(&CompileErrKind::VarDec)?;
+
+        // varName(identifier)
+        self.expect_identifier(&CompileErrKind::VarDec)?;
+
+        // if symbol ',', other varName(identifier)
+        while !self.accept_symbol(';')? {
+            if self.accept_identifier()? {
+                continue;
+            }
+
+            self.expect_symbol(',', &CompileErrKind::VarDec)?;
+        }
+
+        Ok(())
+    }
+
+    // varDec: 'var' type varName (',' varName)* ';' -- used inside a subroutine body
+    pub fn compile_var_dec(&mut self) -> Result<(), JAError> {
+        self.expect_keyword(Keyword::Var, &CompileErrKind::VarDec)?;
+        self.compile_var_dec_tail()
+    }
+
+    // classVarDec: ('static'|'field') type varName (',' varName)* ';' -- used at class scope
+    pub fn compile_class_var_dec(&mut self) -> Result<(), JAError> {
+        self.expect(
+            |tok| {
+                [
+                    Token::Keyword(Keyword::Static),
+                    Token::Keyword(Keyword::Field),
+                ]
+                .contains(tok)
+            },
+            &CompileErrKind::VarDec,
+        )?;
+        self.compile_var_dec_tail()
+    }
```

## 検証

- `cargo build` が通ることを確認。
- `cargo run -- ../ArrayTest/Main.jack` の失敗地点が `Compile(VarDec)` から
  次のバグ（パラメータリストの括弧二重消費、Fix 03）まで前進したことを確認。

## 補足（今回のスコープ外）

`compile_var_dec`（サブルーチン内`var`宣言）自体も、`compile_subroutine_body`
から1回だけ・必須で呼ばれており、`var`宣言が複数行連続するケース
（`ArrayTest`はまさにこれに該当）に対応できていない。これは別バグとして
後続のFixで扱う。
