# ADR-0005: Persistencia en SQLite y sincronización por archivo

- **Estado:** Aceptado
- **Fecha:** 2026-10-02

## Contexto

Sin servidor central (Camino C), cada instalación es dueña de sus datos: biblioteca,
progreso de lectura, ajustes y descargas. Hay que decidir dónde viven y cómo se mueven
entre dispositivos.

## Decisión

- **Persistencia local en SQLite** (`rusqlite` o `sqlx`) para biblioteca, progreso y ajustes.
- **Descargas en disco** (archivos), con caché de imágenes propia.
- **Sincronización: backup/restauración por archivo** (manual), por ahora.

## Alternativas consideradas

- **IndexedDB / ficheros JSON:** IndexedDB es de navegador (no aplica); JSON no escala ni
  da consultas.
- **Sync automático por servidor:** descartado explícitamente por el usuario (sin VPS).
- **Carpeta en nube / P2P en LAN:** posible a futuro, no ahora.

## Consecuencias

- **Positivas:** consultas potentes y portables; offline real; sin infraestructura.
- **Negativas:** la sincronización entre dispositivos es **manual**; el formato de backup
  debe versionarse para no romper compatibilidad.
