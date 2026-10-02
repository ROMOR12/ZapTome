//! Entidades y objetos de valor del dominio.

use serde::{Deserialize, Serialize};

/// Identificador de una fuente de contenido.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdFuente(pub String);

/// Identificador de una obra.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdObra(pub String);

/// Identificador de un capítulo.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdCapitulo(pub String);

/// Tipo de fuente según cómo se obtiene el contenido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TipoFuente {
    /// API HTTP directa, como MangaDex o ComicK.
    Api,
    /// Extensión de Tachiyomi ejecutada por el motor local.
    Extension,
}

/// Una fuente de contenido registrada en la aplicación.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fuente {
    pub id: IdFuente,
    pub nombre: String,
    pub tipo: TipoFuente,
}

/// Estado de publicación de una obra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EstadoObra {
    EnCurso,
    Finalizada,
    Pausada,
    Cancelada,
    Desconocido,
}

/// Un manga o manhwa.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obra {
    pub id: IdObra,
    pub fuente: IdFuente,
    pub titulo: String,
    pub sinopsis: Option<String>,
    pub portada: Option<String>,
    pub autores: Vec<String>,
    pub etiquetas: Vec<String>,
    pub estado: EstadoObra,
}

/// Un capítulo de una obra.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capitulo {
    pub id: IdCapitulo,
    pub obra: IdObra,
    pub fuente: IdFuente,
    pub titulo: String,
    pub numero: Option<f32>,
    pub idioma: Option<String>,
    pub fecha: Option<String>,
}

/// Una página de un capítulo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagina {
    pub indice: usize,
    pub url: String,
    pub ancho: Option<u32>,
    pub alto: Option<u32>,
}

/// Parámetros de una búsqueda.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Consulta {
    pub texto: String,
    pub pagina: u32,
}

/// Entrada de la biblioteca del usuario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntradaBiblioteca {
    pub obra: IdObra,
    pub fuente: IdFuente,
    pub titulo: String,
    pub sinopsis: Option<String>,
    pub portada: Option<String>,
    pub categorias: Vec<String>,
    pub favorito: bool,
}

/// Progreso de lectura de una obra.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progreso {
    pub obra: IdObra,
    pub fuente: IdFuente,
    pub capitulo: IdCapitulo,
    pub pagina: usize,
    pub leido: bool,
    /// Marca de tiempo Unix en segundos.
    pub actualizado: i64,
}
