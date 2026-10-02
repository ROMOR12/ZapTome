//! Motor de lectura.
//!
//! Se encarga de la maqueta del capítulo (modo webtoon), del troceado en teselas, del
//! presupuesto de memoria de texturas y de la decodificación de imágenes. No conoce las
//! fuentes concretas: recibe páginas ya resueltas.

pub mod cache;
pub mod decodificador;
pub mod descargador;
pub mod maqueta;

pub use cache::CachePresupuesto;
pub use decodificador::{decodificar, dimensiones_desde_cabecera, ImagenDecodificada};
pub use descargador::descargar;
pub use maqueta::{Dimensiones, Maqueta, PaginaColocada, Tesela};

use thiserror::Error;

/// Error del motor de lectura.
#[derive(Debug, Error)]
pub enum ErrorLector {
    /// No se pudo descargar la imagen.
    #[error("no se pudo descargar la imagen: {0}")]
    Descarga(String),
    /// No se pudo decodificar la imagen.
    #[error("no se pudo decodificar la imagen: {0}")]
    Decodificacion(String),
}
