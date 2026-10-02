# ADR-0004: Base ligera + componente de extensiones on-demand

- **Estado:** Aceptado
- **Fecha:** 2026-10-02

## Contexto

Ejecutar extensiones exige una **JVM**: un JRE recortado con `jlink` suma **~40–70 MB**,
más el jar de Suwayomi. Es menos que un runtime de Electron, pero sería un peso inútil
para quien solo use las APIs.

## Decisión

**Componente de extensiones on-demand:**

- El **instalador base es ligero** (solo Rust, sin JVM).
- Al activar extensiones por primera vez, la app **descarga** el componente
  (JRE recortado + Suwayomi), lo **verifica** (checksum) y lo instala en el directorio de
  datos del usuario. Solo entonces se usa la JVM.
- Al activar extensiones, el JVM se **arranca en segundo plano** mostrando estado
  "preparando"; no debe bloquear la app.

## Alternativas consideradas

- **Todo incluido en el instalador:** una sola descarga, pero pesada desde el inicio.
- **Dos instaladores (lite/full):** más simple, pero mantiene dos artefactos.
- **Requerir Java del sistema:** cero peso, pero mala experiencia.

## Consecuencias

- **Positivas:** descarga inicial mínima; el usuario paga el peso solo si usa extensiones.
- **Negativas:** lógica de descarga/verificación/actualización del componente; hay que
  soportar varias arquitecturas (win-x64, linux-x64, mac-arm64/x64).
