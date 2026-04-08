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
use telemetry_safe::prelude::*;

#[derive(ToTelemetry)]
struct UserId(u64);

#[derive(ToTelemetry)]
struct LoginAttempt {
    user_id: UserId,
    outcome: &'static str,
    #[telemetry(skip)]
    email: String,
}

let attempt = LoginAttempt {
    user_id: UserId(42),
    outcome: "accepted",
    email: "user@example.com".to_owned(),
};

assert_eq!(
    telemetry(&attempt).to_string(),
    "LoginAttempt { user_id: UserId(42), outcome: accepted }",
);
```

`telemetry(&value)` は `Display` として扱えます。`tracing` なら `%telemetry(&value)` のような導線を想定しています。

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

## 設計メモ

- `fmt` ベースにしているのは、高頻度の telemetry 経路で余計な allocation を増やさないためです
- `Debug` をそのまま許可しないのは、既存実装に PII が含まれていても型上は区別できないためです
- `String` を blanket に許可しないのは、「安全な文字列」と「まだ判断していない文字列」を区別したいためです
- まずは本体 crate を backend 非依存に保ち、`tracing` 連携は別の薄い crate に分ける想定です

## あなたのプロダクトでの試し方

最初は次の進め方が安全です。

1. 既存の telemetry 対象から `UserId` や `OrderId` のような識別子型を切り出す
2. それらに `ToTelemetry` を実装、もしくは `derive` を付ける
3. PII を含む `String` / `EmailAddress` / `Name` などは `#[telemetry(skip)]` にする
4. `format!` や `tracing` の `%value` に直接渡していた箇所を `telemetry(&value)` に置き換える

この順序にすると、どの型が「まだ安全性未定義なのか」がコンパイルエラーで見えるようになります。
