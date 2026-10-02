# Diseño 05 — Roadmap y criterios de aceptación

## Fase 0 — Diseño (actual)

- **Entregable:** ADRs + documentos de diseño. **Sin código.**
- **Hecho cuando:** la arquitectura está documentada y revisada.

## Fase 1 — Lector nativo ultraveloz (sin JVM)

- Shell Rust + egui + wgpu.
- Fuentes **MangaDex** y **ComicK** en Rust.
- Lector GPU con **virtualización + tiling + presupuesto de VRAM**.
- Biblioteca y progreso en **SQLite**.
- **Aceptación:** abrir un webtoon de 60+ páginas y hacer scroll/zoom fluido sin agotar
  memoria; arranque sin JVM.

## Fase 2 — Offline y caché

- Descargas de capítulos a disco y lectura offline.
- Caché de imágenes (originales + decodificadas).
- **Aceptación:** leer un capítulo descargado **sin conexión**.

## Fase 3 — Extensiones on-demand (Suwayomi)

- Descarga/verificación del componente JRE + Suwayomi.
- Sidecar en localhost con arranque en segundo plano.
- Adaptador `FuenteManga` sobre GraphQL de Suwayomi.
- **Aceptación:** instalar y usar una extensión de Tachiyomi sin que la app base tenga JVM.

## Fase 4 — Lectura avanzada

- Modo webtoon y modo página/doble página, atajos de teclado, prefetch afinado.
- **Aceptación:** experiencia fluida a 60–120 fps en webtoon largo y en doble página.

## Fase 5 — Backup / sync por archivo

- Exportar/importar backup (biblioteca, progreso, ajustes) versionado.
- **Aceptación:** restaurar en otra instalación y recuperar biblioteca y progreso.

## Fase 6 — Android nativo (futuro)

- Enfoque nativo Android (fuera del alcance actual).
