//! Puertos del dominio.
//!
//! Son *traits* que definen **qué** necesita el dominio, no **cómo** se obtiene. Los
//! adaptadores de otros crates los implementan (MangaDex, ComicK, SQLite, etc.).

use std::sync::Arc;

use async_trait::async_trait;

use crate::error::{ErrorFuente, ErrorPersistencia};
use crate::modelo::{
    Capitulo, Consulta, EntradaBiblioteca, Fuente, IdCapitulo, IdFuente, IdObra, Obra, Pagina,
    Progreso, TipoFuente,
};

/// Puerto de una fuente de contenido.
#[async_trait]
pub trait FuenteManga: Send + Sync {
    /// Identificador único de la fuente.
    fn id(&self) -> IdFuente;
    /// Nombre legible de la fuente.
    fn nombre(&self) -> &str;
    /// Cómo se obtiene el contenido.
    fn tipo(&self) -> TipoFuente;

    /// Busca obras por texto.
    async fn buscar(&self, consulta: &Consulta) -> Result<Vec<Obra>, ErrorFuente>;
    /// Obtiene el detalle de una obra.
    async fn detalle(&self, obra: &IdObra) -> Result<Obra, ErrorFuente>;
    /// Lista los capítulos de una obra.
    async fn capitulos(&self, obra: &IdObra) -> Result<Vec<Capitulo>, ErrorFuente>;
    /// Obtiene las páginas de un capítulo.
    async fn paginas(&self, capitulo: &IdCapitulo) -> Result<Vec<Pagina>, ErrorFuente>;
}

/// Registro de las fuentes disponibles.
pub trait CatalogoFuentes: Send + Sync {
    /// Lista las fuentes registradas.
    fn fuentes(&self) -> Vec<Fuente>;
    /// Devuelve una fuente concreta, si existe.
    fn obtener(&self, id: &IdFuente) -> Option<Arc<dyn FuenteManga>>;
}

/// Repositorio de la biblioteca del usuario.
#[async_trait]
pub trait RepositorioBiblioteca: Send + Sync {
    async fn guardar(&self, entrada: &EntradaBiblioteca) -> Result<(), ErrorPersistencia>;
    async fn eliminar(&self, obra: &IdObra, fuente: &IdFuente) -> Result<(), ErrorPersistencia>;
    async fn listar(&self) -> Result<Vec<EntradaBiblioteca>, ErrorPersistencia>;
    async fn obtener(
        &self,
        obra: &IdObra,
        fuente: &IdFuente,
    ) -> Result<Option<EntradaBiblioteca>, ErrorPersistencia>;
}

/// Repositorio del progreso de lectura.
#[async_trait]
pub trait RepositorioProgreso: Send + Sync {
    async fn guardar(&self, progreso: &Progreso) -> Result<(), ErrorPersistencia>;
    async fn obtener(
        &self,
        obra: &IdObra,
        fuente: &IdFuente,
    ) -> Result<Option<Progreso>, ErrorPersistencia>;
    async fn listar(&self) -> Result<Vec<Progreso>, ErrorPersistencia>;
}

/// Repositorio de ajustes clave-valor.
#[async_trait]
pub trait RepositorioAjustes: Send + Sync {
    async fn obtener(&self, clave: &str) -> Result<Option<String>, ErrorPersistencia>;
    async fn guardar(&self, clave: &str, valor: &str) -> Result<(), ErrorPersistencia>;
}
