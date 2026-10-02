//! Adaptadores de fuentes de contenido.
//!
//! Aquí viven las implementaciones concretas del puerto `FuenteManga`:
//! MangaDex, ComicK y, más adelante, el motor de extensiones vía Suwayomi.

pub mod comick;
pub mod mangadex;

pub use comick::Comick;
pub use mangadex::MangaDex;
