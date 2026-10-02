# ADR-0001: Aplicación nativa de PC en Rust

- **Estado:** Aceptado
- **Fecha:** 2026-10-02

## Contexto

ZapTome quiere diferenciarse siendo una app de PC **nativa y ultraveloz**. Los clientes
existentes usan tecnologías pesadas:

- **Suwayomi-JUI**: Kotlin + JetBrains Compose (JVM).
- **Sorayomi**: Flutter (Dart VM).
- **Suwayomi WebUI**: React en navegador.

Además, el motor de extensiones (Suwayomi) es Kotlin/JVM, y las apps se distribuyen sin
servidor central.

## Decisión

Desarrollar ZapTome como **aplicación nativa de PC en Rust**, sin WebView ni navegador.

## Alternativas consideradas

- **Electron / Tauri (web UI):** rápido de desarrollar, pero mete WebView/Chromium y
  pierde la ventaja de velocidad.
- **Kotlin + Compose Desktop:** cómodo y comparte runtime con Suwayomi, pero es
  exactamente lo que ya hace Suwayomi-JUI → no diferencia.
- **Flutter:** lo que usa Sorayomi → no diferencia.
- **C++:** mismo rendimiento que Rust, pero más riesgo de memoria y menor productividad.

## Consecuencias

- **Positivas:** binario ligero, arranque instantáneo, bajo consumo; ecosistema moderno
  (Cargo) y seguridad de memoria.
- **Negativas:** curva de aprendizaje de Rust; sin reutilización del stack web previo;
  Android queda como trabajo aparte y futuro.
