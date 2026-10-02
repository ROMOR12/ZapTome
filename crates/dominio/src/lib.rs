//! Dominio de ZapTome: entidades, reglas de negocio y puertos.
//!
//! Este crate no conoce la interfaz, ni la red, ni la base de datos, ni el motor de
//! extensiones. Todo acceso al exterior se define aquí como *traits* (puertos) que
//! implementan los adaptadores en otros crates.

pub mod error;
pub mod modelo;
pub mod puertos;

pub use error::{ErrorFuente, ErrorPersistencia};
pub use modelo::{
    Capitulo, Consulta, EntradaBiblioteca, EstadoObra, Fuente, IdCapitulo, IdFuente, IdObra,
    Obra, Pagina, Progreso, TipoFuente,
};
pub use puertos::{
    CatalogoFuentes, FuenteManga, RepositorioAjustes, RepositorioBiblioteca, RepositorioProgreso,
};
