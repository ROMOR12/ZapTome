//! Casos de uso de ZapTome.
//!
//! Orquesta el dominio a través de sus puertos. No conoce implementaciones concretas:
//! recibe los adaptadores por inyección.

use std::sync::Arc;

use thiserror::Error;

use zaptome_dominio::{
    Capitulo, CatalogoFuentes, Consulta, EntradaBiblioteca, ErrorFuente, ErrorPersistencia,
    Fuente, FuenteManga, IdCapitulo, IdFuente, IdObra, Obra, Pagina, Progreso,
    RepositorioBiblioteca, RepositorioProgreso,
};

/// Error de la capa de aplicación.
#[derive(Debug, Error)]
pub enum ErrorAplicacion {
    /// La fuente solicitada no está registrada.
    #[error("fuente no encontrada: {0}")]
    FuenteNoEncontrada(String),
    /// Error devuelto por una fuente.
    #[error(transparent)]
    Fuente(#[from] ErrorFuente),
    /// Error de persistencia.
    #[error(transparent)]
    Persistencia(#[from] ErrorPersistencia),
}

/// Registro de fuentes disponibles.
#[derive(Default)]
pub struct Catalogo {
    fuentes: Vec<Arc<dyn FuenteManga>>,
}

impl Catalogo {
    /// Crea un catálogo vacío.
    pub fn nuevo() -> Self {
        Self::default()
    }

    /// Registra una fuente.
    pub fn agregar(&mut self, fuente: Arc<dyn FuenteManga>) {
        self.fuentes.push(fuente);
    }
}

impl CatalogoFuentes for Catalogo {
    fn fuentes(&self) -> Vec<Fuente> {
        self.fuentes
            .iter()
            .map(|f| Fuente {
                id: f.id(),
                nombre: f.nombre().to_string(),
                tipo: f.tipo(),
            })
            .collect()
    }

    fn obtener(&self, id: &IdFuente) -> Option<Arc<dyn FuenteManga>> {
        self.fuentes.iter().find(|f| &f.id() == id).cloned()
    }
}

/// Servicio con los casos de uso de la aplicación.
pub struct Servicio {
    catalogo: Arc<dyn CatalogoFuentes>,
    biblioteca: Arc<dyn RepositorioBiblioteca>,
    progreso: Arc<dyn RepositorioProgreso>,
}

impl Servicio {
    /// Construye el servicio con sus dependencias.
    pub fn nuevo(
        catalogo: Arc<dyn CatalogoFuentes>,
        biblioteca: Arc<dyn RepositorioBiblioteca>,
        progreso: Arc<dyn RepositorioProgreso>,
    ) -> Self {
        Self {
            catalogo,
            biblioteca,
            progreso,
        }
    }

    /// Fuentes disponibles.
    pub fn fuentes(&self) -> Vec<Fuente> {
        self.catalogo.fuentes()
    }

    fn fuente(&self, id: &IdFuente) -> Result<Arc<dyn FuenteManga>, ErrorAplicacion> {
        self.catalogo
            .obtener(id)
            .ok_or_else(|| ErrorAplicacion::FuenteNoEncontrada(id.0.clone()))
    }

    /// Busca obras en una fuente.
    pub async fn buscar(
        &self,
        fuente: &IdFuente,
        consulta: &Consulta,
    ) -> Result<Vec<Obra>, ErrorAplicacion> {
        Ok(self.fuente(fuente)?.buscar(consulta).await?)
    }

    /// Obtiene el detalle de una obra.
    pub async fn detalle(
        &self,
        fuente: &IdFuente,
        obra: &IdObra,
    ) -> Result<Obra, ErrorAplicacion> {
        Ok(self.fuente(fuente)?.detalle(obra).await?)
    }

    /// Lista los capítulos de una obra.
    pub async fn capitulos(
        &self,
        fuente: &IdFuente,
        obra: &IdObra,
    ) -> Result<Vec<Capitulo>, ErrorAplicacion> {
        Ok(self.fuente(fuente)?.capitulos(obra).await?)
    }

    /// Obtiene las páginas de un capítulo.
    pub async fn paginas(
        &self,
        fuente: &IdFuente,
        capitulo: &IdCapitulo,
    ) -> Result<Vec<Pagina>, ErrorAplicacion> {
        Ok(self.fuente(fuente)?.paginas(capitulo).await?)
    }

    /// Añade una obra a la biblioteca.
    pub async fn agregar_a_biblioteca(&self, obra: &Obra) -> Result<(), ErrorAplicacion> {
        let entrada = EntradaBiblioteca {
            obra: obra.id.clone(),
            fuente: obra.fuente.clone(),
            titulo: obra.titulo.clone(),
            portada: obra.portada.clone(),
            categorias: Vec::new(),
            favorito: false,
        };
        self.biblioteca.guardar(&entrada).await?;
        Ok(())
    }

    /// Lista la biblioteca.
    pub async fn biblioteca(&self) -> Result<Vec<EntradaBiblioteca>, ErrorAplicacion> {
        Ok(self.biblioteca.listar().await?)
    }

    /// Quita una obra de la biblioteca.
    pub async fn quitar_de_biblioteca(
        &self,
        obra: &IdObra,
        fuente: &IdFuente,
    ) -> Result<(), ErrorAplicacion> {
        self.biblioteca.eliminar(obra, fuente).await?;
        Ok(())
    }

    /// Guarda el progreso de lectura.
    pub async fn guardar_progreso(&self, progreso: &Progreso) -> Result<(), ErrorAplicacion> {
        self.progreso.guardar(progreso).await?;
        Ok(())
    }

    /// Recupera el progreso de lectura de una obra.
    pub async fn progreso_de(
        &self,
        obra: &IdObra,
        fuente: &IdFuente,
    ) -> Result<Option<Progreso>, ErrorAplicacion> {
        Ok(self.progreso.obtener(obra, fuente).await?)
    }
}
