# workspace 構成

```text
telemetry-safe/
├─ Cargo.toml
├─ README.md
├─ LICENSE-APACHE
├─ LICENSE-MIT
├─ crates/
│  ├─ telemetry-safe-core/
│  │  ├─ Cargo.toml
│  │  └─ src/
│  │     └─ lib.rs
│  ├─ telemetry-safe-derive/
│  │  ├─ Cargo.toml
│  │  └─ src/
│  │     └─ lib.rs
│  ├─ telemetry-safe/
│  │  ├─ Cargo.toml
│  │  └─ src/
│  │     └─ lib.rs
│  └─ telemetry-safe-tracing/
│     ├─ Cargo.toml
│     └─ src/
│        └─ lib.rs
└─ examples/
   ├─ basic.rs
   └─ tracing.rs
```

---

# 各 crate の役割

## 1. `telemetry-safe-core`

最小の中核です。

中に置くもの:

* `ToTelemetry`
* `TelemetryDisplay`
* `TelemetryDebug`
* `telemetry(...)`
* `telemetry_debug(...)`
* 必要なら `ToTelemetryExt`

ここは **proc-macro 依存なし**、**tracing 依存なし** にします。

### 責務

* 安全な telemetry 表現のモデル
* trait と adapter の定義

---

## 2. `telemetry-safe-derive`

proc-macro 専用です。

中に置くもの:

* `#[derive(ToTelemetry)]`
* `#[telemetry(skip)]`
* `#[telemetry("{}")]`
* 将来 `#[telemetry(with = path)]`

### 責務

* struct / enum への `ToTelemetry` 実装自動生成

---

## 3. `telemetry-safe`

利用者向けの facade crate です。

中に置くもの:

* `pub use telemetry_safe_core::*;`
* `pub use telemetry_safe_derive::ToTelemetry;`

### 責務

* ユーザーが基本これだけ依存すればよい入口

これを作っておくと利用側がかなりきれいです。

```toml id="d7vkrb"
[dependencies]
telemetry-safe = "0.1"
```

---

## 4. `telemetry-safe-tracing`

`tracing` 連携専用です。

中に置くもの:

* `#[safe_instrument]`
* 将来必要なら `safe_event!` 系

依存:

* `telemetry-safe`
* `tracing`

### 責務

* `tracing` 専用 integration
* `%rhs` を `telemetry(&rhs)` に書き換える
* `?rhs` を禁止する

---

# ルート `Cargo.toml`

こんな感じです。

```toml id="uvz4es"
[workspace]
members = [
    "crates/telemetry-safe-core",
    "crates/telemetry-safe-derive",
    "crates/telemetry-safe",
    "crates/telemetry-safe-tracing",
]
resolver = "2"

[workspace.package]
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/your-org/telemetry-safe"
homepage = "https://github.com/your-org/telemetry-safe"
documentation = "https://docs.rs/telemetry-safe"
rust-version = "1.75"

[workspace.dependencies]
syn = { version = "2", features = ["full", "extra-traits"] }
quote = "1"
proc-macro2 = "1"
tracing = "0.1"
```

---

# 各 `Cargo.toml` の例

## `telemetry-safe-core/Cargo.toml`

```toml id="fo2j9m"
[package]
name = "telemetry-safe-core"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true
homepage.workspace = true
documentation.workspace = true
rust-version.workspace = true

[dependencies]
```

---

## `telemetry-safe-derive/Cargo.toml`

```toml id="ay2zld"
[package]
name = "telemetry-safe-derive"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true
homepage.workspace = true
documentation.workspace = true
rust-version.workspace = true

[lib]
proc-macro = true

[dependencies]
syn.workspace = true
quote.workspace = true
proc-macro2.workspace = true
telemetry-safe-core = { path = "../telemetry-safe-core" }
```

`derive` から `core` への依存は、生成コード内の path を安定させるためにあってよいです。

---

## `telemetry-safe/Cargo.toml`

```toml id="cloeq5"
[package]
name = "telemetry-safe"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true
homepage.workspace = true
documentation.workspace = true
rust-version.workspace = true

[dependencies]
telemetry-safe-core = { path = "../telemetry-safe-core" }
telemetry-safe-derive = { path = "../telemetry-safe-derive" }
```

---

## `telemetry-safe-tracing/Cargo.toml`

ここは `safe_instrument` が attribute proc-macro なら、実質 proc-macro crate になります。
なので最初は **別 proc-macro crate にする** のが自然です。

おすすめは名前をこう分けることです。

* 公開名: `telemetry-safe-tracing`
* 内部実装: `telemetry-safe-tracing-macros`

ただ、公開パッケージを増やしたくなければ `telemetry-safe-tracing` 自体を proc-macro crate にしてもよいです。

最小構成なら:

```toml id="gzyjyr"
[package]
name = "telemetry-safe-tracing"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true
homepage.workspace = true
documentation = "https://docs.rs/telemetry-safe-tracing"
rust-version.workspace = true

[lib]
proc-macro = true

[dependencies]
syn.workspace = true
quote.workspace = true
proc-macro2.workspace = true
tracing.workspace = true
telemetry-safe = { path = "../telemetry-safe" }
```

---

# さらに整った公開構成

OSSとしては、実は次の **5 crate 構成**が一番きれいです。

```text
crates/
  telemetry-safe-core/
  telemetry-safe-derive/
  telemetry-safe/
  telemetry-safe-tracing-core/
  telemetry-safe-tracing/
```

## 追加する `telemetry-safe-tracing-core`

ここに

* `TelemetryAsDisplay`
* tracing向け helper
* proc-macro 以外の共通コード

を置く。

## `telemetry-safe-tracing`

ここは proc-macro だけ

* `#[safe_instrument]`

### この分離の利点

Rust の proc-macro crate は制約が強いので、ロジックを普通の crate に逃がせます。
長期的にはこっちのほうが保守しやすいです。

---

# 私のおすすめ最終形

## 公開パッケージ

* `telemetry-safe`
* `telemetry-safe-tracing`

## workspace 内部

* `telemetry-safe-core`
* `telemetry-safe-derive`
* `telemetry-safe`
* `telemetry-safe-tracing-core`
* `telemetry-safe-tracing`

これが一番バランス良いです。

---

# `lib.rs` のイメージ

## `telemetry-safe/src/lib.rs`

```rust id="n6s8wg"
pub use telemetry_safe_core::{
    telemetry,
    telemetry_debug,
    TelemetryDebug,
    TelemetryDisplay,
    ToTelemetry,
};

pub use telemetry_safe_derive::ToTelemetry;
```

利用者はこれで:

```rust id="riuwc8"
use telemetry_safe::ToTelemetry;
```

だけで済みます。

---

## `telemetry-safe-tracing/src/lib.rs`

もし proc-macro crate なら、ここは macro export のみです。

```rust id="r2gk4s"
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn safe_instrument(attr: TokenStream, item: TokenStream) -> TokenStream {
    telemetry_safe_tracing_core::expand_safe_instrument(attr, item)
}
```

---

# examples の分け方

## `examples/basic.rs`

* `ToTelemetry` trait のみ使う
* derive のみ使う
* tracing なし

## `examples/tracing.rs`

* `safe_instrument`
* `%rhs`
* `?rhs` が禁止される例

これで core と integration の境界が伝わりやすいです。

---

# docs.rs / README 的にも良い見せ方

README 冒頭はこう整理すると伝わりやすいです。

## telemetry-safe

> 型に基づいて、安全な telemetry 表現を定義する Rust ライブラリ

## telemetry-safe-tracing

> `telemetry-safe` を `tracing` の `#[instrument]` と統合するアダプタ

この2段構えがすごく自然です。

---

# 命名補足

crate 名は次で十分です。

* `telemetry-safe`
* `telemetry-safe-core`
* `telemetry-safe-derive`
* `telemetry-safe-tracing`
* `telemetry-safe-tracing-core`

かなり素直でよいです。

---

# 最小スタート案

最初は 4 crate でも十分です。

```text
crates/
  telemetry-safe-core/
  telemetry-safe-derive/
  telemetry-safe/
  telemetry-safe-tracing/
```

実装が育って、

* proc-macro が太ってきた
* tracing側ロジックを単体テストしたい

となったら `telemetry-safe-tracing-core` を切り出せばよいです。

---

# いま着手するならおすすめ順

1. `telemetry-safe-core`
2. `telemetry-safe-derive`
3. `telemetry-safe`
4. `telemetry-safe-tracing`

この順なら無理がないです。

必要なら次に、**各 crate の `src/lib.rs` のひな形**まで書けます。
