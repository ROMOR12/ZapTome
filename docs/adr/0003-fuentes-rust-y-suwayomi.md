# ADR-0003: Fuentes API en Rust + extensiones vía Suwayomi local

- **Estado:** Aceptado
- **Fecha:** 2026-10-02

## Contexto

Las extensiones de Tachiyomi/Mihon son **APK con bytecode Android (dex)**. No se pueden
ejecutar fuera de ART (Android) o de una **JVM** (Suwayomi convierte APK→JAR con
`AndroidCompat`). Rust no puede ejecutarlas directamente.

A la vez, existen fuentes con API HTTP estable (MangaDex, ComicK) que no necesitan motor.

## Decisión

Dos tipos de fuente, tras un mismo puerto `FuenteManga`:

1. **APIs oficiales (MangaDex, ComicK):** implementadas **nativamente en Rust** (`reqwest`
   + `serde`). No usan JVM → arranque y consultas inmediatos.
2. **Extensiones de Tachiyomi:** se ejecutan en un **Suwayomi local** (sidecar JVM) que
   solo se usa si el usuario activa extensiones.

## Alternativas consideradas

- **Solo APIs (sin extensiones):** más simple, pero pierde el catálogo de extensiones.
- **Reimplementar extensiones en Rust:** inviable (son dex).
- **GraalVM native-image para el motor:** inviable por el classloading dinámico de las
  extensiones.

## Consecuencias

- **Positivas:** el caso normal (APIs) no necesita JVM; se conserva el catálogo de
  extensiones sin servidor externo.
- **Negativas:** dos caminos de datos que mantener; dependencia de un proyecto externo
  (Suwayomi) para las extensiones.
