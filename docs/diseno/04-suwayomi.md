# Diseño 04 — Contrato y ciclo de vida de Suwayomi

## Rol

Suwayomi es el **motor de extensiones** (Kotlin/JVM, MPL-2.0). ZapTome lo usa **solo**
para las fuentes de tipo extensión. **No** gestiona la biblioteca, el progreso ni las
descargas de ZapTome: eso es del dominio propio (SQLite). ZapTome consume su API y mapea
los datos a su propio modelo.

## Ciclo de vida (on-demand)

1. **Descarga del componente** (solo la primera vez que se activan extensiones):
   JRE recortado + jar de Suwayomi desde una fuente conocida.
2. **Verificación** (checksum) y **extracción** en el directorio de datos del usuario.
3. **Arranque del sidecar:** proceso JVM escuchando en **127.0.0.1** y **puerto libre
   dinámico** (nunca expuesto en red).
4. **Health check** hasta que responda; mientras, la UI muestra "preparando extensiones".
5. **Uso** durante la sesión.
6. **Cierre** del proceso al salir de la app.

## Comunicación

- **GraphQL** en `/api/graphql` (REST `/api/v1` está deprecado).
- Operaciones previstas: listar fuentes, instalar extensiones, buscar, detalle, capítulos,
  páginas.

## Mapeo (Suwayomi → dominio ZapTome)

| Suwayomi | ZapTome |
|----------|---------|
| Source | `Fuente` |
| Manga | `Obra` |
| Chapter | `Capitulo` |
| Page | `Pagina` |

## Seguridad y robustez

- Bind **solo a localhost** y puerto aleatorio.
- Si el sidecar cae, la app sigue funcionando con las fuentes API y muestra el error.
- Versionado del componente para poder actualizarlo de forma independiente.

## Fase

Se implementa en la **Fase 3**. Las Fases 1–2 no necesitan JVM.
