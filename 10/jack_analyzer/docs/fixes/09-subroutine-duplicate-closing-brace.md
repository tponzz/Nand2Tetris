# Fix 09: `compile_subroutine` が本体解析後に余分な `}` を要求している

## 症状

Fix 08適用後、`cargo run -- ../ArrayTest/Main.jack` は
`Compile(SubroutineBody)`ではなく`Compile(Class)`で失敗するようになった
（サブルーチン本体自体の解析は最後まで成功するようになった）。

## 原因

`compile_subroutine_body`は自身の末尾で既に本体の閉じ`}`を
`expect_symbol('}', &CompileErrKind::SubroutineBody)`で消費している。
ところが呼び出し元の`compile_subroutine`は、`compile_subroutine_body()`の
呼び出しの直後に**もう一度**`expect_symbol('}', &CompileErrKind::Subroutine)`
を実行していた。

`ArrayTest`の`Main`クラスはサブルーチンが`main`の1つだけなので、
サブルーチン本体の閉じ`}`の次に来るトークンは**クラス自体の閉じ`}`**
（ファイル最後の`}`）である。`compile_subroutine`の余分な`}`要求が
このクラスの閉じ`}`を誤って消費してしまい、`compile_class`側の最後の
`expect_symbol('}', &CompileErrKind::Class)`がトークン切れ(`None`)に
対して実行され、失敗していた。

## 修正方針

`compile_subroutine_body`が既に閉じ`}`を消費済みであることを踏まえ、
`compile_subroutine`側の重複した`expect_symbol('}')`を削除した。

## Diff

```diff
--- a/src/engine.rs
+++ b/src/engine.rs
@@ pub fn compile_subroutine
         // class method declarations
+        // (compile_subroutine_body already consumes the body's closing '}')
         self.compile_subroutine_body()?;
-
-        // }
-        self.expect_symbol('}', &CompileErrKind::Subroutine)?;
```

## 検証

- `cargo build` が通ることを確認。
- `cargo run -- ../ArrayTest/Main.jack` が**エラーを出さずに完走**する
  ようになったことを確認。これで`ArrayTest`に含まれる構文はすべて
  正しくパースできるようになった。
- ただし生成された`Out.xml`と期待される`../ArrayTest/Main.xml`を
  `diff`すると差分が残っている。パース自体は成功しているが、非終端記号
  （`parameterList`, `varDec`, `statements`, `letStatement`,
  `expression`, `term` など）を囲む開始・終了タグの出力が丸ごと
  抜けていること、および`stringConstant`/`integerConstant`のタグ名が
  誤っている（`stringconst`/`intconst`になっている）ことが原因。
  これは次のFixで対応する。
