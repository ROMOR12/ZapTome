# ZapTome

Un lector de manga y manhwa para PC. Nativo, ligero y pensado para ir rápido.

## Qué es

ZapTome es una aplicación de escritorio para leer manga y manhwa. Abre al instante y
moverse por un webtoon de cien páginas se siente suave. No es una web metida en una
ventana, sino una aplicación hecha directamente para el escritorio que usa la GPU para
dibujar las páginas.

Está pensada para Windows, Linux y macOS. Android llegará más adelante.

## Qué hace

- Lee manga y manhwa desde varias fuentes, como MangaDex y ComicK.
- Soporta las extensiones de Tachiyomi con un motor local que solo se activa si se usa.
- Guarda la biblioteca y el progreso en el propio ordenador.
- Permite leer sin conexión los capítulos que se descarguen.

## Cómo está hecho

Está escrita en Rust, con interfaz egui y render por GPU con wgpu. Las fuentes que tienen
API se consultan directamente desde la aplicación y no necesitan nada más. Para las
extensiones de Tachiyomi se levanta un motor local basado en Suwayomi, que se descarga
aparte la primera vez que se activa y solo entonces usa Java.

Todo se queda en el equipo. No hay servidor, ni cuentas, ni servicios de terceros por medio.

## Estado

En fase de diseño. Todavía no hay código, pero la documentación está bastante avanzada.

## Documentación

La documentación del proyecto está en la carpeta [docs](docs/README.md). Ahí se encuentran
los ADRs y los documentos de diseño.

## Aviso legal

ZapTome no aloja ni distribuye contenido. Es un lector que accede a fuentes externas, y el
uso que se haga de cada una es responsabilidad del usuario.

## Licencia

Pendiente de decidir.
