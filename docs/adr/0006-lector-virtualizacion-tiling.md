# ADR-0006: Lector con virtualización, tiling y presupuesto de VRAM

- **Estado:** Aceptado
- **Fecha:** 2026-10-02

## Contexto

Los webtoons tienen capítulos de **60–100 imágenes verticales** de ~1200×3000 px. Una
textura RGBA de ese tamaño ocupa **~14 MB en VRAM**; 60 serían **~860 MB**. No cabe, y el
immediate-mode de egui no debe recolocar/tesselar todo cada fotograma.

## Decisión

Diseñar el lector con estas reglas (detalle en [`../diseno/03-lector-gpu.md`](../diseno/03-lector-gpu.md)):

- **Virtualización estricta:** solo lo visible + **2–3 páginas de margen**.
- **Tiling:** partir páginas altas en **tiras** (p. ej. 1200×512).
- **Presupuesto de VRAM + evicción LRU:** mantener pocas texturas decodificadas.
- **Decodificación en hilos**; subida a GPU con `wgpu::Queue::write_texture` / `StagingBelt`.
- **Lienzo propio wgpu** para el área de lectura (no widgets `Image`).
- **Dimensiones por cabeceras** (JPEG SOF / PNG IHDR) para el layout del scroll.
- **Mipmaps / downscale** al alejar el zoom.

## Alternativas consideradas

- **Widgets `Image` de egui con `egui_extras`:** cómodo, pero no escala a cientos de
  imágenes de alta resolución.

## Consecuencias

- **Positivas:** scroll/zoom fluidos a 60–120 fps incluso en webtoon largo.
- **Negativas:** implementación del lector más compleja (gestión de texturas, tiling,
  prefetch); hay que medir y ajustar el presupuesto de VRAM.
