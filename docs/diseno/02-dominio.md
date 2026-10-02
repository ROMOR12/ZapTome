# Diseño 02 — Modelo de dominio y puertos

> Los tipos son **esbozos de diseño**, no código final.

## Entidades

| Entidad | Descripción |
|---------|-------------|
| `Obra` | Un manga/manhwa (título, sinopsis, portada, autores, estado, etiquetas). |
| `Capitulo` | Un capítulo de una obra (número, título, fecha, idioma). |
| `Pagina` | Una página/imagen de un capítulo (URL, dimensiones, índice). |
| `Fuente` | Origen de datos (MangaDex, ComicK, o una extensión de Suwayomi). |
| `EntradaBiblioteca` | Relación entre el usuario y una obra (categorías, favorito, seguimiento). |
| `Progreso` | Estado de lectura de un capítulo (página actual, leído/no leído, fecha). |
| `Descarga` | Estado de una descarga local de capítulo. |

## Value objects

`IdObra`, `IdCapitulo`, `IdFuente`, `Titulo`, `Idioma`, `RutaLocal`.
Se modelan como tipos nuevos (newtypes) para no mezclar identificadores.

## Puertos (traits)

```rust
// Fuente de contenido: la implementa cada adaptador (MangaDex, ComicK, Suwayomi).
pub trait FuenteManga {
    fn id(&self) -> IdFuente;
    fn nombre(&self) -> &str;

    async fn buscar(&self, consulta: &Consulta) -> Result<Vec<Obra>, ErrorFuente>;
    async fn detalle(&self, obra: &IdObra) -> Result<Obra, ErrorFuente>;
    async fn capitulos(&self, obra: &IdObra) -> Result<Vec<Capitulo>, ErrorFuente>;
    async fn paginas(&self, capitulo: &IdCapitulo) -> Result<Vec<Pagina>, ErrorFuente>;
}
```

```rust
pub trait RepositorioBiblioteca { /* guardar/leer EntradaBiblioteca */ }
pub trait RepositorioProgreso  { /* guardar/leer Progreso */ }
pub trait RepositorioAjustes   { /* ajustes */ }
pub trait Descargador          { /* encolar y consultar descargas */ }
pub trait CatalogoFuentes      { /* listar fuentes disponibles */ }
```

## Errores

- Un error por frontera (`ErrorFuente`, `ErrorPersistencia`, …) con `thiserror`.
- En la capa de aplicación se unifican si procede. **Nada de `unwrap()` en producción.**

## Reglas de negocio (ejemplos)

- Un capítulo descargado se lee desde disco aunque haya red.
- El progreso se guarda por obra y capítulo.
- Una fuente no disponible no debe romper la biblioteca local.
