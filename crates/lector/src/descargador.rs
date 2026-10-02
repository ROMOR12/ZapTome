//! Descarga de imágenes desde una URL.

use crate::ErrorLector;

/// Descarga el contenido de una URL y lo devuelve como bytes.
pub async fn descargar(cliente: &reqwest::Client, url: &str) -> Result<Vec<u8>, ErrorLector> {
    let respuesta = cliente
        .get(url)
        .send()
        .await
        .map_err(|e| ErrorLector::Descarga(e.to_string()))?;

    let estado = respuesta.status();
    if !estado.is_success() {
        return Err(ErrorLector::Descarga(format!("HTTP {}", estado.as_u16())));
    }

    let bytes = respuesta
        .bytes()
        .await
        .map_err(|e| ErrorLector::Descarga(e.to_string()))?;
    Ok(bytes.to_vec())
}
