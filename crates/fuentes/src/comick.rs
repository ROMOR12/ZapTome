//! Adaptador de la Catalog API de ComicK.
//!
//! No necesita motor de extensiones ni JVM: habla directamente con la API HTTP.

use async_trait::async_trait;
use serde::Deserialize;

use zaptome_dominio::{
    Capitulo, Consulta, ErrorFuente, EstadoObra, FuenteManga, IdCapitulo, IdFuente, IdObra, Obra,
    Pagina, TipoFuente,
};

const BASE_POR_DEFECTO: &str = "https://api.comick.dev";
const BASE_IMAGENES: &str = "https://meo.comick.pictures";
const ID_FUENTE: &str = "comick";
const USER_AGENT: &str = concat!(
    "ZapTome/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/ROMOR12/ZapTome)"
);

/// Fuente basada en la Catalog API de ComicK.
pub struct Comick {
    cliente: reqwest::Client,
    base: String,
}

impl Comick {
    /// Crea el adaptador con la configuración por defecto.
    pub fn nuevo() -> Result<Self, ErrorFuente> {
        let cliente = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| ErrorFuente::Red(e.to_string()))?;
        Ok(Self {
            cliente,
            base: BASE_POR_DEFECTO.into(),
        })
    }

    fn url(&self, ruta: &str) -> String {
        format!("{}{}", self.base, ruta)
    }

    async fn enviar_json<T>(&self, peticion: reqwest::RequestBuilder) -> Result<T, ErrorFuente>
    where
        T: for<'de> Deserialize<'de>,
    {
        let respuesta = peticion
            .send()
            .await
            .map_err(|e| ErrorFuente::Red(e.to_string()))?;
        let estado = respuesta.status();
        if estado == reqwest::StatusCode::NOT_FOUND {
            return Err(ErrorFuente::NoEncontrado);
        }
        if !estado.is_success() {
            return Err(ErrorFuente::RespuestaInvalida(format!(
                "HTTP {}",
                estado.as_u16()
            )));
        }
        respuesta
            .json::<T>()
            .await
            .map_err(|e| ErrorFuente::RespuestaInvalida(e.to_string()))
    }
}

#[async_trait]
impl FuenteManga for Comick {
    fn id(&self) -> IdFuente {
        IdFuente(ID_FUENTE.into())
    }

    fn nombre(&self) -> &str {
        "ComicK"
    }

    fn tipo(&self) -> TipoFuente {
        TipoFuente::Api
    }

    async fn buscar(&self, consulta: &Consulta) -> Result<Vec<Obra>, ErrorFuente> {
        let peticion = self.cliente.get(self.url("/v1.0/search")).query(&[
            ("q", consulta.texto.as_str()),
            ("limit", "20"),
        ]);
        let resultados: Vec<ComicBusqueda> = self.enviar_json(peticion).await?;
        Ok(resultados.iter().map(ComicBusqueda::a_obra).collect())
    }

    async fn detalle(&self, obra: &IdObra) -> Result<Obra, ErrorFuente> {
        let peticion = self
            .cliente
            .get(self.url(&format!("/comic/{}", obra.0)));
        let respuesta: DetalleRespuesta = self.enviar_json(peticion).await?;
        Ok(respuesta.a_obra())
    }

    async fn capitulos(&self, obra: &IdObra) -> Result<Vec<Capitulo>, ErrorFuente> {
        let peticion = self
            .cliente
            .get(self.url(&format!("/comic/{}/chapters", obra.0)))
            .query(&[("lang", "en"), ("limit", "1000")]);
        let respuesta: CapitulosRespuesta = self.enviar_json(peticion).await?;
        Ok(respuesta
            .chapters
            .iter()
            .map(|dto| dto.a_capitulo(obra))
            .collect())
    }

    async fn paginas(&self, capitulo: &IdCapitulo) -> Result<Vec<Pagina>, ErrorFuente> {
        let peticion = self
            .cliente
            .get(self.url(&format!("/chapter/{}", capitulo.0)));
        let respuesta: CapituloDetalleRespuesta = self.enviar_json(peticion).await?;
        Ok(respuesta
            .chapter
            .md_images
            .iter()
            .enumerate()
            .map(|(indice, imagen)| Pagina {
                indice,
                url: format!("{}/{}", BASE_IMAGENES, imagen.b2key),
                ancho: imagen.w,
                alto: imagen.h,
            })
            .collect())
    }
}

// --- DTOs de la API ---

#[derive(Debug, Deserialize)]
struct ComicBusqueda {
    hid: String,
    title: String,
    #[serde(default)]
    desc: Option<String>,
    #[serde(default)]
    status: Option<i64>,
    #[serde(default)]
    md_covers: Vec<Portada>,
}

#[derive(Debug, Deserialize)]
struct Portada {
    b2key: String,
}

#[derive(Debug, Deserialize)]
struct DetalleRespuesta {
    comic: ComicDetalle,
    #[serde(default)]
    authors: Vec<Persona>,
    #[serde(default)]
    artists: Vec<Persona>,
}

#[derive(Debug, Deserialize)]
struct ComicDetalle {
    hid: String,
    title: String,
    #[serde(default)]
    desc: Option<String>,
    #[serde(default)]
    status: Option<i64>,
    #[serde(default)]
    md_covers: Vec<Portada>,
    #[serde(default)]
    md_comic_md_genres: Vec<GeneroWrapper>,
}

#[derive(Debug, Deserialize)]
struct GeneroWrapper {
    md_genres: Genero,
}

#[derive(Debug, Deserialize)]
struct Genero {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Persona {
    name: String,
}

#[derive(Debug, Deserialize)]
struct CapitulosRespuesta {
    #[serde(default)]
    chapters: Vec<CapituloComick>,
}

#[derive(Debug, Deserialize)]
struct CapituloComick {
    hid: String,
    #[serde(default)]
    chap: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    lang: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    publish_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CapituloDetalleRespuesta {
    chapter: CapituloDetalle,
}

#[derive(Debug, Deserialize)]
struct CapituloDetalle {
    #[serde(default)]
    md_images: Vec<Imagen>,
}

#[derive(Debug, Deserialize)]
struct Imagen {
    b2key: String,
    #[serde(default)]
    w: Option<u32>,
    #[serde(default)]
    h: Option<u32>,
}

// --- Mapeo de la API al dominio ---

fn estado_desde(codigo: Option<i64>) -> EstadoObra {
    match codigo {
        Some(1) => EstadoObra::EnCurso,
        Some(2) => EstadoObra::Finalizada,
        Some(3) => EstadoObra::Pausada,
        Some(4) => EstadoObra::Cancelada,
        _ => EstadoObra::Desconocido,
    }
}

fn url_portada(portadas: &[Portada]) -> Option<String> {
    portadas
        .first()
        .map(|p| format!("{}/{}", BASE_IMAGENES, p.b2key))
}

fn limpiar_descripcion(desc: Option<String>) -> Option<String> {
    desc.filter(|d| !d.trim().is_empty())
}

impl ComicBusqueda {
    fn a_obra(&self) -> Obra {
        Obra {
            id: IdObra(self.hid.clone()),
            fuente: IdFuente(ID_FUENTE.into()),
            titulo: self.title.clone(),
            sinopsis: limpiar_descripcion(self.desc.clone()),
            portada: url_portada(&self.md_covers),
            autores: Vec::new(),
            etiquetas: Vec::new(),
            estado: estado_desde(self.status),
        }
    }
}

impl DetalleRespuesta {
    fn a_obra(&self) -> Obra {
        let mut autores: Vec<String> = self
            .authors
            .iter()
            .chain(self.artists.iter())
            .map(|p| p.name.clone())
            .collect();
        autores.dedup();

        Obra {
            id: IdObra(self.comic.hid.clone()),
            fuente: IdFuente(ID_FUENTE.into()),
            titulo: self.comic.title.clone(),
            sinopsis: limpiar_descripcion(self.comic.desc.clone()),
            portada: url_portada(&self.comic.md_covers),
            autores,
            etiquetas: self
                .comic
                .md_comic_md_genres
                .iter()
                .map(|g| g.md_genres.name.clone())
                .collect(),
            estado: estado_desde(self.comic.status),
        }
    }
}

impl CapituloComick {
    fn a_capitulo(&self, obra: &IdObra) -> Capitulo {
        let titulo = self
            .title
            .clone()
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| match &self.chap {
                Some(numero) => format!("Capítulo {numero}"),
                None => "Capítulo".to_string(),
            });

        Capitulo {
            id: IdCapitulo(self.hid.clone()),
            obra: obra.clone(),
            fuente: IdFuente(ID_FUENTE.into()),
            titulo,
            numero: self.chap.as_ref().and_then(|c| c.parse::<f32>().ok()),
            idioma: self.lang.clone(),
            fecha: self.publish_at.clone().or_else(|| self.created_at.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapea_resultado_de_busqueda() {
        let json = r#"[{
            "hid": "abc",
            "title": "Solo Leveling",
            "desc": "Sinopsis",
            "status": 2,
            "md_covers": [{ "b2key": "cover.jpg" }]
        }]"#;
        let resultados: Vec<ComicBusqueda> = serde_json::from_str(json).unwrap();
        let obra = resultados[0].a_obra();

        assert_eq!(obra.id.0, "abc");
        assert_eq!(obra.titulo, "Solo Leveling");
        assert_eq!(obra.estado, EstadoObra::Finalizada);
        assert_eq!(
            obra.portada.as_deref(),
            Some("https://meo.comick.pictures/cover.jpg")
        );
    }

    #[test]
    fn mapea_detalle_con_autores_y_generos() {
        let json = r#"{
            "comic": {
                "hid": "abc",
                "title": "Solo Leveling",
                "desc": "Sinopsis",
                "status": 2,
                "md_covers": [{ "b2key": "cover.jpg" }],
                "md_comic_md_genres": [
                    { "md_genres": { "name": "Action" } },
                    { "md_genres": { "name": "Fantasy" } }
                ]
            },
            "authors": [{ "name": "Chugong" }],
            "artists": [{ "name": "Dubu" }]
        }"#;
        let detalle: DetalleRespuesta = serde_json::from_str(json).unwrap();
        let obra = detalle.a_obra();

        assert_eq!(obra.autores, vec!["Chugong".to_string(), "Dubu".to_string()]);
        assert_eq!(
            obra.etiquetas,
            vec!["Action".to_string(), "Fantasy".to_string()]
        );
    }

    #[test]
    fn mapea_capitulos_e_imagenes() {
        let json = r#"{
            "chapters": [
                { "hid": "cap1", "chap": "1", "title": "Inicio", "lang": "en" }
            ]
        }"#;
        let respuesta: CapitulosRespuesta = serde_json::from_str(json).unwrap();
        let capitulo = respuesta.chapters[0].a_capitulo(&IdObra("abc".into()));
        assert_eq!(capitulo.id.0, "cap1");
        assert_eq!(capitulo.numero, Some(1.0));

        let json = r#"{
            "chapter": {
                "md_images": [
                    { "b2key": "1.png", "w": 800, "h": 1200 },
                    { "b2key": "2.png" }
                ]
            }
        }"#;
        let detalle: CapituloDetalleRespuesta = serde_json::from_str(json).unwrap();
        assert_eq!(detalle.chapter.md_images.len(), 2);
        assert_eq!(detalle.chapter.md_images[0].w, Some(800));
    }
}
