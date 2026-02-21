# 40 Docs and Observability

## Documentation Requirements

- Update docs whenever behavior, config, or public API changes.
- Keep these sources synchronized:
  - `README.md`
  - `docs/`
  - `.env.example`
  - rustdoc on public modules/types

## Logging and Telemetry

- Prefer structured logging fields over free-text-only logs.
- Include canonical machine-friendly fields for market events:
  - `event`
  - identifiers (for example `feed_id`)
  - numeric values (`price`, `confidence`, `ema_price`)
  - timestamp variants (`publish_time_unix`, localized field)
- Keep field naming stable to avoid downstream ingestion breakage.

## Time Semantics

- Preserve Unix timestamp output for storage and cross-system conversion.
- If localized timestamps are emitted, include explicit timezone label/offset.
- When timezone configuration changes, reflect supported options in docs.
