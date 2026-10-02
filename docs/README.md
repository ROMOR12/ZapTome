# Documentación de ZapTome

Documentación de diseño del proyecto. **Fase actual: 0 (diseño, sin código).**

## Índice

- [`adr/`](adr/README.md) — Architecture Decision Records (decisiones y su porqué).
- [`diseno/`](diseno/) — Documentos de diseño detallado.

## Documentos de diseño

- [`diseno/01-arquitectura.md`](diseno/01-arquitectura.md) — Arquitectura hexagonal y workspace de crates.
- [`diseno/02-dominio.md`](diseno/02-dominio.md) — Modelo de dominio y puertos (traits).
- [`diseno/03-lector-gpu.md`](diseno/03-lector-gpu.md) — Lector: virtualización, tiling y GPU.
- [`diseno/04-suwayomi.md`](diseno/04-suwayomi.md) — Contrato y ciclo de vida del sidecar Suwayomi.
- [`diseno/05-roadmap.md`](diseno/05-roadmap.md) — Fases y criterios de aceptación.

## Convenciones

- Documentación en **español**.
- Los ADRs siguen el formato **MADR** (ver `adr/README.md`).
- Los fragmentos de tipos Rust en los documentos son **esbozos de diseño**, no código final.
