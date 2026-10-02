# Architecture Decision Records (ADRs)

Registro de las decisiones de arquitectura de ZapTome y el porqué de cada una.

## ¿Qué es un ADR?

Un documento corto que captura **una** decisión relevante: el contexto, la decisión
tomada y sus consecuencias. Sirve para no repetir discusiones y entender el historial.

## Formato (MADR simplificado)

Cada ADR incluye:

- **Estado:** Propuesto / Aceptado / Reemplazado / Descartado.
- **Contexto:** el problema y las restricciones.
- **Decisión:** qué se decide.
- **Alternativas consideradas:** qué se evaluó.
- **Consecuencias:** lo bueno y lo malo.

## Índice

| ADR | Título | Estado |
|-----|--------|--------|
| [0001](0001-app-nativa-rust.md) | Aplicación nativa de PC en Rust | Aceptado |
| [0002](0002-ui-egui-wgpu.md) | UI con egui y render con wgpu | Aceptado |
| [0003](0003-fuentes-rust-y-suwayomi.md) | Fuentes API en Rust + extensiones vía Suwayomi local | Aceptado |
| [0004](0004-empaquetado-componente-on-demand.md) | Base ligera + componente de extensiones on-demand | Aceptado |
| [0005](0005-persistencia-y-sync.md) | Persistencia en SQLite y sync por archivo | Aceptado |
| [0006](0006-lector-virtualizacion-tiling.md) | Lector con virtualización, tiling y presupuesto de VRAM | Aceptado |
