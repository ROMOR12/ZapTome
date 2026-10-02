//! Adaptador de la API pública de MangaDex.
//!
//! No necesita motor de extensiones ni JVM: habla directamente con la API HTTP.

use std::collections::HashMap;

use async_trait::async_trait;
use serde::Deserialize;

use zaptome_dominio::{
    Capitulo, Consulta, ErrorFuente, EstadoObra, FuenteManga, IdCapitulo, IdFuente, IdObra, Obra,
    Orden, Pagina, TipoFuente,
};

const BASE_POR_DEFECTO: &str = "https://api.mangadex.org";
const BASE_PORTADAS: &str = "https://uploads.mangadex.org/covers";
const ID_FUENTE: &str = "mangadex";
const USER_AGENT: &str = concat!(
    "ZapTome/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/ROMOR12/ZapTome)"
);

type Localizado = HashMap<String, String>;

/// Fuente basada en la API pública de MangaDex.
pub struct MangaDex {
    cliente: reqwest::Client,
    base: String,
}

impl MangaDex {
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
impl FuenteManga for MangaDex {
    fn id(&self) -> IdFuente {
        IdFuente(ID_FUENTE.into())
    }

    fn nombre(&self) -> &str {
        "MangaDex"
    }

    fn tipo(&self) -> TipoFuente {
        TipoFuente::Api
    }

    async fn buscar(&self, consulta: &Consulta) -> Result<Vec<Obra>, ErrorFuente> {
        let mut parametros: Vec<(String, String)> = vec![
            ("limit".into(), "20".into()),
            ("includes[]".into(), "cover_art".into()),
            ("includes[]".into(), "author".into()),
        ];

        let texto = consulta.texto.trim();
        if !texto.is_empty() {
            parametros.push(("title".into(), texto.to_string()));
        }

        let filtros = &consulta.filtros;
        if let Some(estado) = estado_a_mangadex(filtros.estado) {
            parametros.push(("status[]".into(), estado.into()));
        }
        if let Some(idioma) = &filtros.idioma_lectura {
            parametros.push(("availableTranslatedLanguage[]".into(), idioma.clone()));
        }
        if let Some(idioma) = &filtros.idioma_original {
            parametros.push(("originalLanguage[]".into(), idioma.clone()));
        }
        if let Some(demografia) = &filtros.demografia {
            parametros.push(("publicationDemographic[]".into(), demografia.clone()));
        }
        if !filtros.incluir_adulto {
            parametros.push(("contentRating[]".into(), "safe".into()));
            parametros.push(("contentRating[]".into(), "suggestive".into()));
        }

        match filtros.orden {
            Orden::Relevancia => {
                // Sin texto, ordenamos por popularidad para que siempre haya resultados.
                if texto.is_empty() {
                    parametros.push(("order[followedCount]".into(), "desc".into()));
                }
            }
            Orden::Popularidad => {
                parametros.push(("order[followedCount]".into(), "desc".into()));
            }
            Orden::Recientes => {
                parametros.push(("order[latestUploadedChapter]".into(), "desc".into()));
            }
            Orden::Titulo => {
                parametros.push(("order[title]".into(), "asc".into()));
            }
        }

        let peticion = self.cliente.get(self.url("/manga")).query(&parametros);
        let respuesta: RespuestaLista<MangaDto> = self.enviar_json(peticion).await?;
        Ok(respuesta.data.iter().map(MangaDto::a_obra).collect())
    }

    async fn detalle(&self, obra: &IdObra) -> Result<Obra, ErrorFuente> {
        let peticion = self
            .cliente
            .get(self.url(&format!("/manga/{}", obra.0)))
            .query(&[("includes[]", "cover_art"), ("includes[]", "author")]);
        let respuesta: RespuestaUnica<MangaDto> = self.enviar_json(peticion).await?;
        Ok(respuesta.data.a_obra())
    }

    async fn capitulos(&self, obra: &IdObra) -> Result<Vec<Capitulo>, ErrorFuente> {
        let peticion = self
            .cliente
            .get(self.url(&format!("/manga/{}/feed", obra.0)))
            .query(&[
                ("limit", "500"),
                ("translatedLanguage[]", "es"),
                ("translatedLanguage[]", "en"),
                ("order[chapter]", "asc"),
                ("includeExternalUrl", "0"),
            ]);
        let respuesta: RespuestaLista<CapituloDto> = self.enviar_json(peticion).await?;
        Ok(respuesta
            .data
            .iter()
            .filter_map(|dto| dto.a_capitulo(obra))
            .collect())
    }

    async fn paginas(&self, capitulo: &IdCapitulo) -> Result<Vec<Pagina>, ErrorFuente> {
        let peticion = self
            .cliente
            .get(self.url(&format!("/at-home/server/{}", capitulo.0)));
        let respuesta: AtHomeDto = self.enviar_json(peticion).await?;
        Ok(urls_de_paginas(&respuesta))
    }
}

// --- DTOs de la API ---

#[derive(Debug, Deserialize)]
struct RespuestaLista<T> {
    data: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct RespuestaUnica<T> {
    data: T,
}

#[derive(Debug, Deserialize)]
struct MangaDto {
    id: String,
    attributes: MangaAtributos,
    #[serde(default)]
    relationships: Vec<Relacion>,
}

#[derive(Debug, Deserialize)]
struct MangaAtributos {
    title: Localizado,
    #[serde(default)]
    description: Localizado,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    tags: Vec<Etiqueta>,
}

#[derive(Debug, Deserialize)]
struct Etiqueta {
    attributes: EtiquetaAtributos,
}

#[derive(Debug, Deserialize)]
struct EtiquetaAtributos {
    name: Localizado,
}

#[derive(Debug, Deserialize)]
struct Relacion {
    #[serde(rename = "type")]
    tipo: String,
    #[serde(default)]
    attributes: Option<RelacionAtributos>,
}

#[derive(Debug, Deserialize)]
struct RelacionAtributos {
    #[serde(rename = "fileName", default)]
    file_name: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CapituloDto {
    id: String,
    attributes: CapituloAtributos,
}

#[derive(Debug, Deserialize)]
struct CapituloAtributos {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    chapter: Option<String>,
    #[serde(rename = "translatedLanguage", default)]
    translated_language: Option<String>,
    #[serde(rename = "publishAt", default)]
    publish_at: Option<String>,
    #[serde(rename = "externalUrl", default)]
    external_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AtHomeDto {
    #[serde(rename = "baseUrl")]
    base_url: String,
    chapter: AtHomeCapitulo,
}

#[derive(Debug, Deserialize)]
struct AtHomeCapitulo {
    hash: String,
    data: Vec<String>,
}

// --- Mapeo de la API al dominio ---

fn texto_localizado(mapa: &Localizado) -> String {
    mapa.get("es")
        .or_else(|| mapa.get("en"))
        .or_else(|| mapa.values().next())
        .cloned()
        .unwrap_or_default()
}

fn estado_a_mangadex(estado: Option<EstadoObra>) -> Option<&'static str> {
    match estado {
        Some(EstadoObra::EnCurso) => Some("ongoing"),
        Some(EstadoObra::Finalizada) => Some("completed"),
        Some(EstadoObra::Pausada) => Some("hiatus"),
        Some(EstadoObra::Cancelada) => Some("cancelled"),
        Some(EstadoObra::Desconocido) | None => None,
    }
}

fn estado_desde(valor: Option<&str>) -> EstadoObra {
    match valor {
        Some("ongoing") => EstadoObra::EnCurso,
        Some("completed") => EstadoObra::Finalizada,
        Some("hiatus") => EstadoObra::Pausada,
        Some("cancelled") => EstadoObra::Cancelada,
        _ => EstadoObra::Desconocido,
    }
}

fn urls_de_paginas(at_home: &AtHomeDto) -> Vec<Pagina> {
    at_home
        .chapter
        .data
        .iter()
        .enumerate()
        .map(|(indice, archivo)| Pagina {
            indice,
            url: format!(
                "{}/data/{}/{}",
                at_home.base_url, at_home.chapter.hash, archivo
            ),
            ancho: None,
            alto: None,
        })
        .collect()
}

impl MangaDto {
    fn a_obra(&self) -> Obra {
        let mut autores = Vec::new();
        let mut portada = None;

        for relacion in &self.relationships {
            match relacion.tipo.as_str() {
                "author" | "artist" => {
                    if let Some(attrs) = &relacion.attributes {
                        if let Some(nombre) = &attrs.name {
                            autores.push(nombre.clone());
                        }
                    }
                }
                "cover_art" => {
                    if let Some(attrs) = &relacion.attributes {
                        if let Some(archivo) = &attrs.file_name {
                            // Se pide la miniatura para no descargar la portada completa.
                            portada =
                                Some(format!("{}/{}/{}.256.jpg", BASE_PORTADAS, self.id, archivo));
                        }
                    }
                }
                _ => {}
            }
        }

        let sinopsis = texto_localizado(&self.attributes.description);

        Obra {
            id: IdObra(self.id.clone()),
            fuente: IdFuente(ID_FUENTE.into()),
            titulo: texto_localizado(&self.attributes.title),
            sinopsis: if sinopsis.is_empty() {
                None
            } else {
                Some(sinopsis)
            },
            portada,
            autores,
            etiquetas: self
                .attributes
                .tags
                .iter()
                .map(|t| texto_localizado(&t.attributes.name))
                .collect(),
            estado: estado_desde(self.attributes.status.as_deref()),
        }
    }
}

impl CapituloDto {
    fn a_capitulo(&self, obra: &IdObra) -> Option<Capitulo> {
        if self.attributes.external_url.is_some() {
            return None;
        }

        let titulo = self
            .attributes
            .title
            .clone()
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| match &self.attributes.chapter {
                Some(numero) => format!("Capítulo {numero}"),
                None => "Capítulo".to_string(),
            });

        Some(Capitulo {
            id: IdCapitulo(self.id.clone()),
            obra: obra.clone(),
            fuente: IdFuente(ID_FUENTE.into()),
            titulo,
            numero: self
                .attributes
                .chapter
                .as_ref()
                .and_then(|c| c.parse::<f32>().ok()),
            idioma: self.attributes.translated_language.clone(),
            fecha: self.attributes.publish_at.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapea_un_manga() {
        let json = r#"{
            "id": "abc-123",
            "attributes": {
                "title": { "en": "Ejemplo", "es": "Ejemplo ES" },
                "description": { "en": "Sinopsis" },
                "status": "ongoing",
                "tags": [
                    { "attributes": { "name": { "en": "Action" } } }
                ]
            },
            "relationships": [
                { "type": "author", "attributes": { "name": "Autor Uno" } },
                { "type": "cover_art", "attributes": { "fileName": "cover.jpg" } }
            ]
        }"#;
        let dto: MangaDto = serde_json::from_str(json).unwrap();
        let obra = dto.a_obra();

        assert_eq!(obra.id.0, "abc-123");
        assert_eq!(obra.titulo, "Ejemplo ES");
        assert_eq!(obra.estado, EstadoObra::EnCurso);
        assert_eq!(obra.autores, vec!["Autor Uno".to_string()]);
        assert_eq!(
            obra.portada.as_deref(),
            Some("https://uploads.mangadex.org/covers/abc-123/cover.jpg.256.jpg")
        );
        assert_eq!(obra.etiquetas, vec!["Action".to_string()]);
    }

    #[test]
    fn descarta_capitulos_externos() {
        let interno = r#"{"id":"cap-1","attributes":{"title":"Uno","chapter":"1","translatedLanguage":"es"}}"#;
        let externo = r#"{"id":"cap-2","attributes":{"chapter":"2","externalUrl":"https://otro.test"}}"#;

        let dto: CapituloDto = serde_json::from_str(interno).unwrap();
        assert!(dto.a_capitulo(&IdObra("obra".into())).is_some());

        let dto: CapituloDto = serde_json::from_str(externo).unwrap();
        assert!(dto.a_capitulo(&IdObra("obra".into())).is_none());
    }

    #[test]
    fn construye_urls_de_paginas() {
        let json = r#"{
            "baseUrl": "https://uploads.mangadex.org",
            "chapter": {
                "hash": "hash1",
                "data": ["1-a.png", "2-b.png"]
            }
        }"#;
        let dto: AtHomeDto = serde_json::from_str(json).unwrap();
        let paginas = urls_de_paginas(&dto);

        assert_eq!(paginas.len(), 2);
        assert_eq!(paginas[0].indice, 0);
        assert_eq!(
            paginas[1].url,
            "https://uploads.mangadex.org/data/hash1/2-b.png"
        );
    }
}
