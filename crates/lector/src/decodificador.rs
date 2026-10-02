//! Decodificación de imágenes a RGBA8.

use std::io::Cursor;

use image::ImageReader;

use crate::maqueta::Dimensiones;
use crate::ErrorLector;

/// Imagen decodificada a RGBA8, lista para subir a la GPU.
pub struct ImagenDecodificada {
    pub ancho: u32,
    pub alto: u32,
    pub pixeles: Vec<u8>,
}

impl ImagenDecodificada {
    /// Bytes que ocuparía como textura RGBA8.
    pub fn bytes(&self) -> usize {
        self.pixeles.len()
    }
}

/// Decodifica bytes de imagen a RGBA8.
pub fn decodificar(bytes: &[u8]) -> Result<ImagenDecodificada, ErrorLector> {
    let imagen =
        image::load_from_memory(bytes).map_err(|e| ErrorLector::Decodificacion(e.to_string()))?;
    let rgba = imagen.to_rgba8();
    let (ancho, alto) = rgba.dimensions();
    Ok(ImagenDecodificada {
        ancho,
        alto,
        pixeles: rgba.into_raw(),
    })
}

/// Decodifica la imagen y la reduce para usarla como miniatura.
pub fn decodificar_escalado(
    bytes: &[u8],
    ancho_max: u32,
) -> Result<ImagenDecodificada, ErrorLector> {
    let imagen =
        image::load_from_memory(bytes).map_err(|e| ErrorLector::Decodificacion(e.to_string()))?;

    let imagen = if ancho_max > 0 && imagen.width() > ancho_max {
        let escala = ancho_max as f64 / imagen.width() as f64;
        let alto = ((imagen.height() as f64) * escala).round().max(1.0) as u32;
        imagen.resize_exact(ancho_max, alto, image::imageops::FilterType::Triangle)
    } else {
        imagen
    };

    let rgba = imagen.to_rgba8();
    let (ancho, alto) = rgba.dimensions();
    Ok(ImagenDecodificada {
        ancho,
        alto,
        pixeles: rgba.into_raw(),
    })
}

/// Lee las dimensiones desde la cabecera, sin decodificar toda la imagen.
pub fn dimensiones_desde_cabecera(bytes: &[u8]) -> Result<Dimensiones, ErrorLector> {
    let lector = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| ErrorLector::Decodificacion(e.to_string()))?;
    let (ancho, alto) = lector
        .into_dimensions()
        .map_err(|e| ErrorLector::Decodificacion(e.to_string()))?;
    Ok(Dimensiones { ancho, alto })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_de_prueba(ancho: u32, alto: u32) -> Vec<u8> {
        let imagen = image::RgbaImage::from_pixel(ancho, alto, image::Rgba([200, 100, 50, 255]));
        let mut bytes = Vec::new();
        imagen
            .write_to(
                &mut Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .unwrap();
        bytes
    }

    #[test]
    fn decodifica_a_rgba() {
        let bytes = png_de_prueba(2, 3);
        let imagen = decodificar(&bytes).unwrap();
        assert_eq!(imagen.ancho, 2);
        assert_eq!(imagen.alto, 3);
        assert_eq!(imagen.bytes(), 2 * 3 * 4);
    }

    #[test]
    fn lee_dimensiones_de_la_cabecera() {
        let bytes = png_de_prueba(4, 7);
        let dimensiones = dimensiones_desde_cabecera(&bytes).unwrap();
        assert_eq!(dimensiones.ancho, 4);
        assert_eq!(dimensiones.alto, 7);
    }
}
