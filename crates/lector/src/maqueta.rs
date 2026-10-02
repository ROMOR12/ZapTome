//! Maqueta del capítulo en modo webtoon: colocación, virtualización y troceado.

/// Dimensiones de una imagen en píxeles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dimensiones {
    pub ancho: u32,
    pub alto: u32,
}

/// Una página colocada en el scroll vertical continuo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaginaColocada {
    /// Índice de la página en el capítulo.
    pub indice: usize,
    /// Altura que ocupa en pantalla, ya escalada al ancho del viewport.
    pub alto_mostrado: f32,
    /// Posición vertical donde empieza.
    pub offset_y: f32,
}

/// Una tesela (tira horizontal) de una página alta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tesela {
    /// Página a la que pertenece.
    pub indice_pagina: usize,
    /// Orden de la tesela dentro de la página.
    pub indice_tesela: usize,
    /// Coordenada Y dentro de la imagen original.
    pub y: u32,
    /// Altura de la tesela.
    pub alto: u32,
}

/// Maqueta de un capítulo en modo webtoon.
pub struct Maqueta {
    paginas: Vec<PaginaColocada>,
    alto_total: f32,
}

impl Maqueta {
    /// Construye la maqueta para un ancho de viewport y una lista de dimensiones.
    pub fn webtoon(ancho_viewport: f32, dimensiones: &[Dimensiones]) -> Self {
        let mut paginas = Vec::with_capacity(dimensiones.len());
        let mut y = 0.0;

        for (indice, dim) in dimensiones.iter().enumerate() {
            let escala = if dim.ancho > 0 {
                ancho_viewport / dim.ancho as f32
            } else {
                1.0
            };
            let alto_mostrado = dim.alto as f32 * escala;
            paginas.push(PaginaColocada {
                indice,
                alto_mostrado,
                offset_y: y,
            });
            y += alto_mostrado;
        }

        Self {
            paginas,
            alto_total: y,
        }
    }

    /// Altura total del scroll.
    pub fn alto_total(&self) -> f32 {
        self.alto_total
    }

    /// Páginas colocadas.
    pub fn paginas(&self) -> &[PaginaColocada] {
        &self.paginas
    }

    /// Índices de las páginas visibles en la ventana dada, ampliados por `margen`.
    pub fn visibles(&self, scroll_y: f32, alto_viewport: f32, margen: usize) -> Vec<usize> {
        if self.paginas.is_empty() {
            return Vec::new();
        }

        let arriba = scroll_y;
        let abajo = scroll_y + alto_viewport;

        let mut primero = None;
        let mut ultimo = None;
        for (i, pagina) in self.paginas.iter().enumerate() {
            let inicio = pagina.offset_y;
            let fin = pagina.offset_y + pagina.alto_mostrado;
            if fin >= arriba && inicio <= abajo {
                if primero.is_none() {
                    primero = Some(i);
                }
                ultimo = Some(i);
            }
        }

        match (primero, ultimo) {
            (Some(a), Some(b)) => {
                let desde = a.saturating_sub(margen);
                let hasta = (b + margen).min(self.paginas.len() - 1);
                (desde..=hasta).collect()
            }
            _ => Vec::new(),
        }
    }
}

/// Divide una página alta en teselas de, como mucho, `alto_tesela` píxeles.
pub fn teselar(indice_pagina: usize, dimensiones: Dimensiones, alto_tesela: u32) -> Vec<Tesela> {
    if dimensiones.alto == 0 || alto_tesela == 0 {
        return Vec::new();
    }

    let mut teselas = Vec::new();
    let mut y = 0;
    let mut indice = 0;
    while y < dimensiones.alto {
        let alto = alto_tesela.min(dimensiones.alto - y);
        teselas.push(Tesela {
            indice_pagina,
            indice_tesela: indice,
            y,
            alto,
        });
        y += alto;
        indice += 1;
    }
    teselas
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dim(ancho: u32, alto: u32) -> Dimensiones {
        Dimensiones { ancho, alto }
    }

    #[test]
    fn coloca_paginas_y_calcula_alto_total() {
        let maqueta = Maqueta::webtoon(1000.0, &[dim(1000, 2000), dim(1000, 3000)]);
        assert_eq!(maqueta.alto_total(), 5000.0);
        assert_eq!(maqueta.paginas()[1].offset_y, 2000.0);
    }

    #[test]
    fn escala_al_ancho_del_viewport() {
        let maqueta = Maqueta::webtoon(500.0, &[dim(1000, 2000)]);
        assert_eq!(maqueta.alto_total(), 1000.0);
    }

    #[test]
    fn virtualiza_solo_lo_visible_con_margen() {
        let maqueta = Maqueta::webtoon(1000.0, &[dim(1000, 1000); 10]);
        // Ventana de 1000 px en la posición 5000: cae justo en las páginas 4, 5 y 6.
        assert_eq!(maqueta.visibles(5000.0, 1000.0, 0), vec![4, 5, 6]);
        // Con un margen de una página se amplía a las vecinas.
        assert_eq!(maqueta.visibles(5000.0, 1000.0, 1), vec![3, 4, 5, 6, 7]);
    }

    #[test]
    fn teselado_cubre_toda_la_pagina() {
        let teselas = teselar(0, dim(1200, 1300), 512);
        assert_eq!(teselas.len(), 3);
        assert_eq!(teselas[0].alto, 512);
        assert_eq!(teselas[2].y, 1024);
        assert_eq!(teselas[2].alto, 276);
    }
}
