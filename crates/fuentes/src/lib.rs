//! Adaptadores de fuentes de contenido.
//!
//! Aquí viven las implementaciones concretas del puerto `FuenteManga`: MangaDex y, más
//! adelante, el motor de extensiones vía Suwayomi.

pub mod mangadex;

pub use mangadex::MangaDex;
