# telemetry-safe

`telemetry-safe` は「テレメトリに出してよい表現」だけを型で許可するための Rust ライブラリです。

`tracing` の `#[instrument]` や通常の logging では、引数やフィールドをうっかりそのまま出してしまうと PII が漏れます。  
この crate では `ToTelemetry` を実装した型だけが出力経路に乗れるため、レビュー運用ではなくコンパイルエラーで事故を防ぎます。

## 目的

- opt-out ではなく opt-in にする
- PII を誤って出力するコードを compile-time に落とす
- `tracing` 専用に閉じず、logging / metrics / error reporting にも流用できる形にする

## 使い方

```rust
use telemetry_safe::{telemetry, ToTelemetry};
use std::fmt::{self, Formatter};

#[derive(ToTelemetry)]
struct UserId(u64);

#[derive(ToTelemetry)]
struct LoginAttempt {
    user_id: UserId,
    outcome: OutcomeLabel,
    #[telemetry(skip)]
    email: String,
}

struct OutcomeLabel(&'static str);

impl ToTelemetry for OutcomeLabel {
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

let attempt = LoginAttempt {
    user_id: UserId(42),
    outcome: OutcomeLabel("accepted"),
    email: "user@example.com".to_owned(),
};

assert_eq!(
    telemetry(&attempt).to_string(),
    "LoginAttempt { user_id: UserId(42), outcome: accepted }",
);
```

`telemetry(&value)` は `Display` として扱えます。`tracing` なら `%telemetry(&value)` のような導線を想定しています。
なお derive macro は `telemetry_safe::ToTelemetry` から、helper 関数群は `telemetry_safe::prelude::*` から取るのが基本です。

## コンパイルエラーになる例

`String` や `&str` をドメイン型の中にそのまま混ぜると、derive は `ToTelemetry` 実装を要求します。  
つまり「安全かどうか未判断の生文字列」をそのまま telemetry に流す実装は通りません。

```compile_fail
use telemetry_safe::{ToTelemetry, telemetry};

#[derive(ToTelemetry)]
struct UnsafePayload {
    raw: String,
}

fn main() {
    let payload = UnsafePayload {
        raw: "secret".to_owned(),
    };

    let _ = telemetry(&payload).to_string();
}
```

## 現時点の API

- `ToTelemetry`
  - 安全に外部へ出してよい表現を定義する trait
- `telemetry(&value)`
  - `Display` adapter
- `telemetry_debug(&value)`
  - `Debug` adapter
- `#[derive(ToTelemetry)]`
  - struct / enum に対する derive
- `#[telemetry(skip)]`
  - フィールドを出力対象から外す
- `#[telemetry("{}")]`
  - その型の `Display` 出力を明示的に採用する

## Safety Policy

`telemetry-safe` は「便利に telemetry を出すための薄い wrapper」ではありません。  
目的は、PII が混入した値を review 運用ではなく型と macro で構造的に外へ出せなくすることです。

この目的のため、意図的にいくつかの convenience を制限しています。

- `String` / `&str` を blanket では許可しない
  - 文字列は安全な識別子なのか、未検査のユーザー入力なのかを型で区別したいためです
- 固定文字列を許可したい場合でも、暗黙には通さない
  - feature-gated な `trusted_literal` のような明示 marker がない限り、文字列は safe とみなしません
- ambient な `Debug` / `Display` をそのまま信用しない
  - 既存実装に PII が混ざっていても、trait 名だけでは安全性が分からないためです
- backend ごとの便利機能より、fail-closed を優先する
  - 一部だけ緩い escape hatch を作ると、「safe」という名前に反して最も危険な経路になりやすいためです

特に `telemetry-safe-tracing` の `#[safe_instrument]` では、この方針を厳密に守ります。

- 関数引数のデフォルト記録は常に無効化する
- `fields(...)` では `%expr` のような明示 opt-in だけを許可する
- `?expr` は許可しない
- `ret` / `err` は ambient `tracing` semantics ではなく、`ToTelemetry` 前提の safe semantics としてのみ許可する
- `&'static str` も暗黙には許可しない

`err` / `ret` は便利ですが、`tracing` 標準の挙動をそのまま通すと関数全体の error / return value を
ambient な `Debug` / `Display` に委ねることになり、PII 混入リスクが高くなります。
そのため `safe_instrument(err)` / `safe_instrument(ret)` は、内部で `ToTelemetry` を要求する別 semantics として実装します。

また、`tracing::instrument` のデフォルト挙動は関数引数を `Debug` で記録するため、
`safe_instrument` は常に implicit `skip_all` として振る舞います。
telemetry に出したい値は、`fields(...)` の `%expr` で明示的に opt-in してください。

```rust,ignore
use std::fmt::{self, Formatter};
use telemetry_safe::ToTelemetry;
use telemetry_safe_tracing::safe_instrument;

struct DomainError;

impl ToTelemetry for DomainError {
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("denied")
    }
}

#[safe_instrument(err)]
fn do_work() -> Result<(), DomainError> {
    Err(DomainError)
}
```

この例では `DomainError: ToTelemetry` が必要で、`String` や未検査型のままではコンパイルエラーになります。

もしプロダクト側の判断で固定文字列だけを許可したい場合は、
`telemetry-safe-tracing` の `trusted-literal` feature を有効にして
`%trusted_literal("signup")` のような明示 marker を使います。
それでも一般の `&str` は通らず、`&'static str` に限定されます。

## workspace 構成

- `crates/telemetry-safe-core`
  - trait と adapter だけを置く最小コア
- `crates/telemetry-safe-derive`
  - `#[derive(ToTelemetry)]` と field attribute
- `crates/telemetry-safe`
  - 通常の利用者が依存する facade crate
- `crates/telemetry-safe-tracing`
  - `tracing` integration の公開入口
- `crates/telemetry-safe-tracing-macros`
  - `safe_instrument` など attribute macro の実装置き場

`tracing` 連携を別 crate に分けているのは、core の安全モデルを backend 非依存のまま保ちつつ、
proc-macro や `tracing` の都合で API 全体が引っ張られないようにするためです。

## 設計メモ

- `fmt` ベースにしているのは、高頻度の telemetry 経路で余計な allocation を増やさないためです
- `Debug` をそのまま許可しないのは、既存実装に PII が含まれていても型上は区別できないためです
- `String` を blanket に許可しないのは、「安全な文字列」と「まだ判断していない文字列」を区別したいためです
- `&'static str` も blanket に許可しないのは、借用か所有かではなく、安全判断の有無を境界にしたいためです
- まずは本体 crate を backend 非依存に保ち、`tracing` 連携は別の薄い crate に分ける想定です

## あなたのプロダクトでの試し方

最初は次の進め方が安全です。

1. 既存の telemetry 対象から `UserId` や `OrderId` のような識別子型を切り出す
2. それらに `ToTelemetry` を実装、もしくは `derive` を付ける
3. PII を含む `String` / `EmailAddress` / `Name` などは `#[telemetry(skip)]` にする
4. `format!` や `tracing` の `%value` に直接渡していた箇所を `telemetry(&value)` に置き換える

この順序にすると、どの型が「まだ安全性未定義なのか」がコンパイルエラーで見えるようになります。
