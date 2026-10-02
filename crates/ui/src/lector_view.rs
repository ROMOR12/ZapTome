//! Vista del lector: virtualización, texturas y presupuesto de VRAM.

use std::collections::{HashMap, HashSet};

use eframe::egui;

use zaptome_dominio::{Capitulo, IdFuente, Obra, Pagina};
use zaptome_lector::{CachePresupuesto, Dimensiones, ImagenDecodificada, Maqueta};

use crate::tema::Paleta;

const ANCHO_PLACEHOLDER: u32 = 1000;
const ALTO_PLACEHOLDER: u32 = 1400;
const PRESUPUESTO_VRAM: usize = 256 * 1024 * 1024;
const MARGEN_PAGINAS: usize = 2;

/// Estado del lector para un capítulo abierto.
pub struct Lector {
    pub fuente: IdFuente,
    pub obra: Obra,
    pub capitulo: Capitulo,
    pub paginas: Vec<Pagina>,
    pub error: Option<String>,

    dimensiones: Vec<Option<Dimensiones>>,
    texturas: HashMap<usize, egui::TextureHandle>,
    solicitadas: HashSet<usize>,
    cache: CachePresupuesto<usize>,
    maqueta: Maqueta,
    ancho_maqueta: f32,
    sucia: bool,
}

impl Lector {
    /// Crea el lector para un capítulo con sus páginas.
    pub fn nuevo(fuente: IdFuente, obra: Obra, capitulo: Capitulo, paginas: Vec<Pagina>) -> Self {
        let dimensiones = paginas
            .iter()
            .map(|p| match (p.ancho, p.alto) {
                (Some(ancho), Some(alto)) if ancho > 0 && alto > 0 => Some(Dimensiones { ancho, alto }),
                _ => None,
            })
            .collect();

        let mut lector = Self {
            fuente,
            obra,
            capitulo,
            paginas,
            error: None,
            dimensiones,
            texturas: HashMap::new(),
            solicitadas: HashSet::new(),
            cache: CachePresupuesto::nuevo(PRESUPUESTO_VRAM),
            maqueta: Maqueta::webtoon(1.0, &[]),
            ancho_maqueta: 0.0,
            sucia: true,
        };
        lector.reconstruir_maqueta(ANCHO_PLACEHOLDER as f32);
        lector
    }

    fn dimensiones_efectivas(&self) -> Vec<Dimensiones> {
        self.dimensiones
            .iter()
            .map(|d| {
                d.unwrap_or(Dimensiones {
                    ancho: ANCHO_PLACEHOLDER,
                    alto: ALTO_PLACEHOLDER,
                })
            })
            .collect()
    }

    fn reconstruir_maqueta(&mut self, ancho: f32) {
        let dimensiones = self.dimensiones_efectivas();
        self.maqueta = Maqueta::webtoon(ancho, &dimensiones);
        self.ancho_maqueta = ancho;
        self.sucia = false;
    }

    /// Rehace la maqueta si cambió el ancho o llegaron dimensiones nuevas.
    pub fn asegurar_maqueta(&mut self, ancho: f32) {
        if self.sucia || (ancho - self.ancho_maqueta).abs() > 0.5 {
            self.reconstruir_maqueta(ancho);
        }
    }

    /// Indica si una página ya está cargada o en curso.
    pub fn ya_solicitada(&self, indice: usize) -> bool {
        self.solicitadas.contains(&indice) || self.texturas.contains_key(&indice)
    }

    /// Marca una página como solicitada.
    pub fn marcar_solicitada(&mut self, indice: usize) {
        self.solicitadas.insert(indice);
    }

    /// Permite volver a pedir una página que falló.
    pub fn olvidar_solicitud(&mut self, indice: usize) {
        self.solicitadas.remove(&indice);
    }

    /// URL de una página, si existe.
    pub fn url_pagina(&self, indice: usize) -> Option<String> {
        self.paginas.get(indice).map(|p| p.url.clone())
    }

    /// Registra una imagen decodificada como textura, con evicción por presupuesto.
    pub fn registrar_imagen(
        &mut self,
        ctx: &egui::Context,
        indice: usize,
        imagen: ImagenDecodificada,
    ) {
        let bytes = imagen.bytes();
        let color = egui::ColorImage::from_rgba_unmultiplied(
            [imagen.ancho as usize, imagen.alto as usize],
            &imagen.pixeles,
        );
        let textura = ctx.load_texture(
            format!("pagina-{indice}"),
            color,
            egui::TextureOptions::LINEAR,
        );

        self.texturas.insert(indice, textura);
        if let Some(slot) = self.dimensiones.get_mut(indice) {
            *slot = Some(Dimensiones {
                ancho: imagen.ancho,
                alto: imagen.alto,
            });
        }
        self.solicitadas.remove(&indice);
        self.sucia = true;

        let desalojadas = self.cache.registrar(indice, bytes);
        for clave in desalojadas {
            self.texturas.remove(&clave);
        }
    }

    /// Dibuja el capítulo en modo webtoon y devuelve las páginas que hay que pedir.
    pub fn dibujar(&self, ui: &mut egui::Ui, pedir: &mut Vec<usize>, p: &Paleta) {
        let ancho = self.ancho_maqueta.max(1.0);
        let alto_total = self.maqueta.alto_total().max(1.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show_viewport(ui, |ui, viewport| {
                let (_id, rect) = ui.allocate_space(egui::vec2(ancho, alto_total));
                let painter = ui.painter();

                let visibles = self
                    .maqueta
                    .visibles(viewport.min.y, viewport.height(), MARGEN_PAGINAS);

                for indice in visibles {
                    let Some(pagina) = self.maqueta.paginas().get(indice) else {
                        continue;
                    };
                    let rect_pagina = egui::Rect::from_min_size(
                        egui::pos2(rect.left(), rect.top() + pagina.offset_y),
                        egui::vec2(ancho, pagina.alto_mostrado),
                    );

                    match self.texturas.get(&indice) {
                        Some(textura) => {
                            painter.image(
                                textura.id(),
                                rect_pagina,
                                egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                ),
                                egui::Color32::WHITE,
                            );
                        }
                        None => {
                            painter.rect_filled(rect_pagina, 0.0, p.surface_container_high);
                            painter.text(
                                rect_pagina.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("Página {}", indice + 1),
                                egui::FontId::proportional(16.0),
                                p.on_surface_variant,
                            );
                            if !self.solicitadas.contains(&indice) {
                                pedir.push(indice);
                            }
                        }
                    }
                }
            });
    }
}
