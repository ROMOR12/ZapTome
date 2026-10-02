# Diseño 01 — Arquitectura y workspace de crates

## Principio rector

Arquitectura **hexagonal (puertos y adaptadores)**. La regla de dependencia es
**hacia dentro**: el dominio no conoce la UI, ni HTTP, ni SQLite, ni el motor de
extensiones. Todo acceso al exterior entra por **traits (puertos)**.

```
        ┌────────────────────── zaptome-ui (egui) ──────────────────────┐
        │                                                              │
        └───────────────▲───────────────────────────▲──────────────────┘
                        │                           │
              ┌─────────┴─────────┐        ┌────────┴─────────┐
              │  zaptome-lector   │        │ zaptome-aplicacion│
              │  (render/GPU)     │        │  (casos de uso)   │
              └─────────▲─────────┘        └────────▲─────────┘
                        │                           │
        ┌───────────────┴───────────────┐  ┌────────┴───────────────┐
        │        zaptome-dominio        │  │  puertos (traits)      │
        │  entidades + reglas + Puertos │◄─┤  definidos por dominio │
        └───────────────▲───────────────┘  └────────▲───────────────┘
                        │                           │
        ┌───────────────┴───────────────┐  ┌────────┴───────────────┐
        │      zaptome-fuentes          │  │   zaptome-persistencia │
        │ MangaDex · ComicK · Suwayomi  │  │        (SQLite)        │
        └───────────────────────────────┘  └────────────────────────┘
                        │
              ┌─────────┴──────────┐
              │  suwayomi (JVM)    │  ← sidecar on-demand
              └────────────────────┘
```

## Crates del workspace

| Crate | Responsabilidad | Depende de |
|-------|-----------------|------------|
| `zaptome-dominio` | Entidades, value objects, reglas y **traits de puertos**. Sin I/O. | (solo std / error) |
| `zaptome-aplicacion` | Casos de uso; orquesta puertos. | dominio |
| `zaptome-fuentes` | Adaptadores de fuentes: MangaDex, ComicK, Suwayomi. | dominio |
| `zaptome-persistencia` | Adaptador SQLite. | dominio |
| `zaptome-lector` | Motor de lectura: virtualización, texturas, render wgpu. | dominio |
| `zaptome-ui` | Interfaz egui. | aplicacion, lector |
| `zaptome-app` | Binario: composición, arranque y ciclo de vida. | todos |

## Reglas de dependencia (verificables)

- `dominio` **no** depende de ningún otro crate del workspace.
- `fuentes` y `persistencia` implementan traits del `dominio`; el `dominio` no los conoce.
- La `ui` **no** habla con `fuentes`/`persistencia` directamente: usa `aplicacion`.
- El `lector` no conoce fuentes concretas; recibe imágenes/páginas ya resueltas.

## Runtime asíncrono

- Se propone **Tokio** para red y operaciones de I/O.
- Los puertos que requieren I/O pueden ser `async` (candidato: `async-trait`), a decidir
  y fijar en la Fase 1 con un ADR propio si hace falta.
