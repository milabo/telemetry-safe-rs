# telemetry-safe 開発ポリシー

このリポジトリでは「telemetry に出してよい表現だけを型で許可する」という目的を最優先にします。
便利さや `tracing` との構文互換は重要ですが、この目的を弱める変更は受け入れません。

## 我々が防ぎたい失敗

- PII を含む値が、`Debug` / `Display` / `instrument` のデフォルト挙動経由で外部サービスへ送られること
- 「基本は safe だが、一部の便利機能だけは暗黙に全部出る」という抜け道が残ること
- 利用者が review 運用や注意力に依存しないと安全性を維持できないこと

## 判断原則

- fail-open ではなく fail-closed を選ぶ
- compile-time guarantee を convenience より優先する
- backend 依存の都合を core の安全モデルへ持ち込まない
- blanket impl や passthrough macro は、目的を弱めるなら入れない

## `ToTelemetry` の原則

- `ToTelemetry` は「値をどう見せるか」ではなく「何なら外へ出してよいか」の境界
- `String` / `&str` の blanket 許可はしない
- 既存の `Debug` / `Display` をそのまま safe とみなさない
- 明示的な値オブジェクトや wrapper で安全性を表現する
- 固定文字列を許可する場合でも、暗黙ではなく marker 付き opt-in にする

## `safe_instrument` の原則

`safe_instrument` は `tracing::instrument` の完全互換を目標にしません。
互換性より、「unsafe な出力経路を増やさないこと」を優先します。

第1版の方針:

- 許可:
  - `name`
  - `level`
  - `target`
  - `fields(...)` の中の `%expr`
- 禁止:
  - デフォルト引数記録
  - `fields(...)` の中の `?expr`
  - 裸の field 値
- `err`
- `ret`

`safe_instrument` は常に implicit `skip_all` として振る舞い、
telemetry に出す値は `fields(...)` の `%expr` からだけ明示的に opt-in させる。
ここを緩めると、`instrument` の ambient `Debug` 記録が最も危険な抜け道になる。

`err` / `ret` を禁止する理由:
- error や return value 全体を ambient な `Debug` / `Display` に委ねやすい
- どの field が安全なのかを局所的に判断できない
- 一度許可すると、後から制限を強めるのが breaking になりやすい

`&'static str` を blanket に許可しない理由:
- 文字列リテラルを楽にしたい気持ちは理解できるが、一般の `&str` との境界が利用側に伝わりにくい
- 借用形式であることと safe であることは無関係
- 許可するなら `trusted_literal` のような feature-gated helper で、明示 marker を必須にする

## コメント方針

コード変更時は、「何をしているか」ではなく次を優先してコメントや説明に残します。

- なぜその実装なのか
- どの unsafe path を避けるための制約なのか
- 一見不便でも fail-closed を選んだ理由
- 将来の拡張で緩めてはいけない境界

特に、一見すると回りくどい macro 展開、wrapper、temporary binding、compile_error の導入箇所には、
設計意図が誤解されないよう 1〜3 行の簡潔なコメントを残します。
