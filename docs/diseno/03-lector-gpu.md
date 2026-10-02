# Diseño 03 — Lector: virtualización, tiling y GPU

## Objetivo

Scroll y zoom fluidos (60–120 fps) en webtoons de 60–100 imágenes de alta resolución,
sin agotar la VRAM.

## Modelo de datos del lector

Al abrir un capítulo:

1. Se obtiene la lista de `Pagina` con **URL y dimensiones**.
2. Las dimensiones se leen de las **cabeceras** (JPEG SOF / PNG IHDR) o del propio
   proveedor, **sin decodificar** la imagen completa.
3. Se calcula el **offset acumulado** de cada página → altura total del scroll.

## Pipeline

```
Scroll/zoom (input)
   │
   ▼
Calcular rango visible (offset + viewport, ±2–3 páginas de margen)
   │
   ▼
Virtualización: solo se procesan/instancian las páginas visibles
   │
   ▼
Tiling: cada página alta se divide en tiras (p. ej. 1200×512)
   │
   ▼
Cola priorizada de tiles (por cercanía + dirección)
   │
   ▼
Pool de hilos: decodifica (image / zune-jpeg)  ──►  subida a GPU
   │                                   (Queue::write_texture / StagingBelt)
   ▼
Gestor de texturas (presupuesto VRAM + evicción LRU)
   │
   ▼
Render: callback egui_wgpu dibuja quads texturizados (no widgets Image)
```

## Reglas

- **Presupuesto de VRAM** fijo (a ajustar con medición, p. ej. 256–512 MB). Al superarlo,
  evicción **LRU** de los tiles más lejanos.
- **Prioridad** de tiles por cercanía al viewport; al invertir la dirección del scroll, se
  **cancelan** los prefetch obsoletos.
- **Prefetch** según dirección y velocidad del scroll.
- **Mipmaps / downscale** cuando el zoom muestra la página reducida.
- **Modos:** webtoon (tira continua vertical) y página (una/doble página).
- egui repinta **bajo demanda**; mientras no hay input no se recalcula.

## Rendimiento esperado

- No se crea una textura por página completa: se crean y destruyen **tiles**.
- El área de lectura se pinta en **un solo callback** con N quads, no con N widgets.

## Métricas a medir

FPS, VRAM usada, tiles en vuelo, latencia de decodificación, tiempo de layout por frame.
