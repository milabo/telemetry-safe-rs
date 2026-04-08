# ドラフト設計

## コアコンセプト

「テレメトリに出してよい表現」を trait として定義し、それを通らない値は出力できないようにする。

---

## 1. Trait 設計

```rust
pub trait ToTelemetry {
    fn fmt_telemetry(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}
```

### ポイント

* `Display` に似たインターフェース
* allocation を避けるため `fmt` ベース
* 明示的に telemetry 用であることを示す

---

## 2. Adapter

### Display adapter

```rust
pub struct TelemetryDisplay<'a, T: ?Sized>(&'a T);

impl<T: ToTelemetry + ?Sized> std::fmt::Display for TelemetryDisplay<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt_telemetry(f)
    }
}

pub fn telemetry<T: ToTelemetry + ?Sized>(value: &T) -> TelemetryDisplay<'_, T> {
    TelemetryDisplay(value)
}
```

### Debug adapter（derive 用）

```rust
pub struct TelemetryDebug<'a, T: ?Sized>(&'a T);

impl<T: ToTelemetry + ?Sized> std::fmt::Debug for TelemetryDebug<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt_telemetry(f)
    }
}
```

---

## 3. Derive Macro

```rust
#[derive(ToTelemetry)]
pub struct Example {
    a: A,
    b: B,
    #[telemetry("{}")]
    c: chrono::DateTime<Utc>,
}
```

### 挙動

* 各フィールドに対して再帰的に `ToTelemetry` を要求
* 未実装の場合はコンパイルエラー
* `#[telemetry("{}")]

  * Display を使用
* `#[telemetry(skip)]`

  * フィールドを出力しない

### 展開イメージ

```rust
impl ToTelemetry for Example {
    fn fmt_telemetry(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut ds = f.debug_struct("Example");
        ds.field("a", &TelemetryDebug(&self.a));
        ds.field("b", &TelemetryDebug(&self.b));
        ds.field("c", &format_args!("{}", self.c));
        ds.finish()
    }
}
```

---

## 4. Safety Policy

### 禁止事項

* `Debug` による直接出力
* 生の `String` / `&str` の直接出力
* trait 未実装型の出力

### 推奨

* 値オブジェクトに `ToTelemetry` を実装
* 明示的に安全な表現を定義

---

## 5. tracing 連携（別 crate）

```rust
#[safe_instrument(fields(
    user.id = %user_id,
    user.email = %email,
))]
```

### 内部変換

```rust
%telemetry(&user_id)
%telemetry(&email)
```

### 制約

* `%` のみ許可
* `?` は禁止

---

## 6. 拡張性

### 将来的な拡張

* ハッシュ化サポート
* ポリシーベース制御（環境ごとに変える）
* OpenTelemetry semantic conventions 連携

---

## 7. 最小構成

### crate: telemetry-safe

* `ToTelemetry`
* adapters
* derive macro

### crate: tracing-safe-instrument（別）

* safe wrapper macro

---

## まとめ

* 型で安全性を保証
* trait 未実装はコンパイルエラー
* derive で ergonomics を確保
* tracing とは分離
