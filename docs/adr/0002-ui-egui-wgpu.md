# ADR-0002: UI con egui y render con wgpu

- **Estado:** Aceptado
- **Fecha:** 2026-10-02

## Contexto

Con Rust elegido (ADR-0001), hay que elegir la capa de UI y de render. El lector de
webtoon exige scroll/zoom muy fluidos y control fino de texturas en GPU.

## Decisión

Usar **egui** para la interfaz y **wgpu** para el render. El **área de lectura** no se
dibuja con widgets `Image` de egui, sino con un **lienzo propio** mediante el callback de
`egui_wgpu` (quads texturizados). egui queda para el "chrome" (menús, biblioteca,
ajustes).

## Alternativas consideradas

- **Iced:** más estructurado (Elm-like), buena opción a largo plazo, pero más ceremonia.
- **Tauri:** Rust + UI web; fácil si vienes de web, pero reintroduce WebView.
- **gpui / Slint:** menos maduros o con otro enfoque.

## Consecuencias

- **Positivas:** muy ligero y rápido; control total del render del lector; wgpu cubre
  Vulkan/Metal/DX12.
- **Negativas:** egui es immediate-mode, así que la **virtualización es obligatoria**
  (ver ADR-0006); el lienzo propio implica más trabajo que usar widgets estándar.
