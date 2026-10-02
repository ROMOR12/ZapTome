//! Errores del dominio, uno por frontera.
//!
//! Cada adaptador tiene su propio error para no acoplar capas: las fuentes no devuelven
//! errores de base de datos y viceversa.

use thiserror::Error;

/// Error al obtener datos de una fuente de contenido.
#[derive(Debug, Error)]
pub enum ErrorFuente {
    /// La fuente no está disponible o está desactivada.
    #[error("la fuente no está disponible: {0}")]
    NoDisponible(String),
    /// El recurso solicitado no existe en la fuente.
    #[error("no se encontró el recurso solicitado")]
    NoEncontrado,
    /// Fallo de red o de conexión.
    #[error("error de red: {0}")]
    Red(String),
    /// La respuesta de la fuente no tiene el formato esperado.
    #[error("respuesta no válida de la fuente: {0}")]
    RespuestaInvalida(String),
}

/// Error de persistencia local.
#[derive(Debug, Error)]
pub enum ErrorPersistencia {
    /// Fallo al acceder a la base de datos.
    #[error("error de base de datos: {0}")]
    BaseDeDatos(String),
    /// No se encontró el registro solicitado.
    #[error("no se encontró el registro solicitado")]
    NoEncontrado,
}
