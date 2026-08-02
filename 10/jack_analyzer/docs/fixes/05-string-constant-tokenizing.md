# Fix 05: トークナイザが文字列リテラルを1トークンとして扱えない

## 症状

Fix 04適用後も `Compile(SubroutineBody)` で失敗する。診断用ログを仕込んで
調べると、`Keyboard.readInt("HOW MANY NUMBERS? ");`の`(`の直後で
突然`current=None`（＝トークン切れ）になっていた。まだファイルの途中
なのにパースが「終了」したと誤認していた。

## 原因

`Tokenizer::advance()`は、記号(symbol)でない先頭文字を見つけたら
「次の空白文字 or 記号文字」までを1トークンとして切り出す実装だった。

```rust
let end = self.source[win.start..]
    .find(|byte: char| byte.is_ascii_whitespace() || Self::symbols().contains(byte));
```

これは識別子・キーワード・整数値には正しいが、文字列リテラル
`"HOW MANY NUMBERS? "`のように**内部に空白を含む**トークンには対応できない。
先頭の`"`に気づかず、最初の空白（`"HOW`の直後）で区切ってしまい、
`"HOW`という中途半端な断片が1トークンになる。

この断片は`token_type()`のどの分類にも一致しない
（文字列定数判定は「先頭と末尾が`"`」を要求するが、末尾は`W`なので不一致）。
`token_type()`は`None`を返し、`Token::from`も`None`を返す。

さらに`CompilationEngine::advance()`は「トークン切れ(EOF)」と
「未分類トークン」を区別せず、どちらの場合も`current = None`にしてしまう
設計だったため、実際にはファイルの途中であるにもかかわらず
「もう読むトークンがない」と誤認していた。

## 修正方針

`Tokenizer::advance()`に文字列リテラル専用の分岐を追加した。先頭文字が
`"`の場合は、空白や記号を区切りとして探すのではなく、**次の`"`（閉じ引用符）
まで**を丸ごと1トークンとして切り出すようにした。これにより内部の空白は
区切りとして扱われなくなる。

（「EOF」と「未分類トークン」を区別すべきという設計上の問題は別途あるが、
今回はまず文字列リテラルが正しく1トークンになるようにする、という
最小の修正にとどめた。）

## Diff

```diff
--- a/src/tokenizer.rs
+++ b/src/tokenizer.rs
@@ pub fn advance
         win.start = win.end;

+        // string constant: consume up to the matching closing quote as one
+        // token, ignoring any whitespace/symbol characters inside it
+        if self.source.as_bytes().get(win.start) == Some(&b'"') {
+            win.end = match self.source[win.start + 1..].find('"') {
+                Some(pos) => win.start + 1 + pos + 1,
+                None => self.source.len(),
+            };
+            self.token = win.clone();
+            return Ok(());
+        }
+
         // return if head is a symbol
```

## 検証

- `cargo build` が通ることを確認。
- 診断ログで、`"HOW MANY NUMBERS? "`が正しく1つの`StringConst`トークンとして
  読めるようになり、以前は`None`になっていた箇所を通過することを確認。
- ただしその後、新たなバグ（`let a[i] = ...`で`=`が消費されていない、Fix 06）
  により`cargo run -- ../ArrayTest/Main.jack`は依然`Compile(SubroutineBody)`
  で失敗する。
