//! Interfaz de usuario construida con egui y un tema Material 3.
//!
//! El área de lectura se dibuja en modo webtoon con virtualización: solo se descargan y
//! se suben a GPU las páginas visibles y las cercanas.

mod iconos;
mod lector_view;
mod tema;

use std::collections::HashMap;
use std::sync::Arc;

use eframe::egui::{
    self, Align, Align2, Color32, Context, FontId, Frame, Layout, Margin, Rect, RichText, Rounding,
    Sense, Vec2,
};
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, Semaphore};

use zaptome_aplicacion::Servicio;
use zaptome_dominio::{
    Capitulo, Consulta, EntradaBiblioteca, EstadoObra, Fuente, IdFuente, IdObra, Obra, Pagina,
};
use zaptome_lector::ImagenDecodificada;

use lector_view::Lector;
use tema::Paleta;

const MAX_DESCARGAS_SIMULTANEAS: usize = 4;
const ANCHO_PORTADA: u32 = 320;
const TAM_PORTADA: Vec2 = Vec2::new(104.0, 150.0);

/// Estado de una portada en la caché.
enum EstadoPortada {
    Cargando,
    Lista(egui::TextureHandle),
    Fallida,
}

/// Mensajes que llegan desde las tareas en segundo plano.
enum Mensaje {
    Busqueda(Result<Vec<Obra>, String>),
    Biblioteca(Result<Vec<EntradaBiblioteca>, String>),
    Capitulos {
        obra: Obra,
        capitulos: Result<Vec<Capitulo>, String>,
    },
    Paginas(Result<(IdFuente, Obra, Capitulo, Vec<Pagina>), String>),
    Imagen {
        indice: usize,
        resultado: Result<ImagenDecodificada, String>,
    },
    Portada {
        url: String,
        resultado: Result<ImagenDecodificada, String>,
    },
    Quitado(Result<(), String>),
}

#[derive(PartialEq, Clone, Copy)]
enum Pantalla {
    Biblioteca,
    Buscar,
    Capitulos,
    Lector,
}

/// Aplicación de ZapTome.
pub struct AppZapTome {
    servicio: Arc<Servicio>,
    runtime: Runtime,
    cliente: reqwest::Client,
    tx: mpsc::UnboundedSender<Mensaje>,
    rx: mpsc::UnboundedReceiver<Mensaje>,
    semaforo: Arc<Semaphore>,

    pantalla: Pantalla,
    oscuro: bool,
    iniciado: bool,
    ocupado: bool,
    error: Option<String>,

    fuentes: Vec<Fuente>,
    fuente_sel: usize,
    consulta: String,
    resultados: Vec<Obra>,
    biblioteca: Vec<EntradaBiblioteca>,
    capitulos_obra: Option<(Obra, Vec<Capitulo>)>,
    lector: Option<Lector>,
    portadas: HashMap<String, EstadoPortada>,
}

impl AppZapTome {
    /// Construye la aplicación.
    pub fn nuevo(servicio: Servicio, runtime: Runtime) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let fuentes = servicio.fuentes();
        let cliente = reqwest::Client::builder()
            .user_agent(concat!("ZapTome/", env!("CARGO_PKG_VERSION")))
            .build()
            .unwrap_or_default();

        Self {
            servicio: Arc::new(servicio),
            runtime,
            cliente,
            tx,
            rx,
            semaforo: Arc::new(Semaphore::new(MAX_DESCARGAS_SIMULTANEAS)),
            pantalla: Pantalla::Biblioteca,
            oscuro: true,
            iniciado: false,
            ocupado: false,
            error: None,
            fuentes,
            fuente_sel: 0,
            consulta: String::new(),
            resultados: Vec::new(),
            biblioteca: Vec::new(),
            capitulos_obra: None,
            lector: None,
            portadas: HashMap::new(),
        }
    }

    fn tarea<F>(&self, ctx: &Context, futuro: F)
    where
        F: std::future::Future<Output = Mensaje> + Send + 'static,
    {
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        self.runtime.spawn(async move {
            let mensaje = futuro.await;
            let _ = tx.send(mensaje);
            ctx.request_repaint();
        });
    }

    fn textura_portada(&self, url: &str) -> Option<egui::TextureId> {
        match self.portadas.get(url) {
            Some(EstadoPortada::Lista(textura)) => Some(textura.id()),
            _ => None,
        }
    }

    fn solicitar_portada(&mut self, ctx: &Context, url: &str) {
        if self.portadas.contains_key(url) {
            return;
        }
        self.portadas
            .insert(url.to_string(), EstadoPortada::Cargando);

        let cliente = self.cliente.clone();
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        let semaforo = self.semaforo.clone();
        let url_tarea = url.to_string();

        self.runtime.spawn(async move {
            let resultado = match semaforo.acquire().await {
                Ok(_permiso) => match zaptome_lector::descargar(&cliente, &url_tarea).await {
                    Ok(bytes) => zaptome_lector::decodificar_escalado(&bytes, ANCHO_PORTADA)
                        .map_err(|e| e.to_string()),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            };
            let _ = tx.send(Mensaje::Portada {
                url: url_tarea,
                resultado,
            });
            ctx.request_repaint();
        });
    }

    fn iniciar_busqueda(&mut self, ctx: &Context) {
        let Some(fuente) = self.fuentes.get(self.fuente_sel) else {
            return;
        };
        let id = fuente.id.clone();
        let consulta = Consulta {
            texto: self.consulta.clone(),
            pagina: 0,
        };
        let servicio = self.servicio.clone();
        self.ocupado = true;
        self.error = None;

        self.tarea(ctx, async move {
            match servicio.buscar(&id, &consulta).await {
                Ok(obras) => Mensaje::Busqueda(Ok(obras)),
                Err(e) => Mensaje::Busqueda(Err(e.to_string())),
            }
        });
    }

    fn cargar_biblioteca(&mut self, ctx: &Context) {
        let servicio = self.servicio.clone();
        self.tarea(ctx, async move {
            match servicio.biblioteca().await {
                Ok(entradas) => Mensaje::Biblioteca(Ok(entradas)),
                Err(e) => Mensaje::Biblioteca(Err(e.to_string())),
            }
        });
    }

    fn quitar_de_biblioteca(&mut self, ctx: &Context, obra: IdObra, fuente: IdFuente) {
        let servicio = self.servicio.clone();
        self.tarea(ctx, async move {
            match servicio.quitar_de_biblioteca(&obra, &fuente).await {
                Ok(()) => Mensaje::Quitado(Ok(())),
                Err(e) => Mensaje::Quitado(Err(e.to_string())),
            }
        });
    }

    fn cargar_capitulos(&mut self, ctx: &Context, obra: Obra) {
        let servicio = self.servicio.clone();
        self.ocupado = true;
        self.error = None;
        self.tarea(ctx, async move {
            let _ = servicio.agregar_a_biblioteca(&obra).await;
            match servicio.capitulos(&obra.fuente, &obra.id).await {
                Ok(capitulos) => Mensaje::Capitulos {
                    obra,
                    capitulos: Ok(capitulos),
                },
                Err(e) => Mensaje::Capitulos {
                    obra,
                    capitulos: Err(e.to_string()),
                },
            }
        });
    }

    fn abrir_capitulo(&mut self, ctx: &Context, obra: Obra, capitulo: Capitulo) {
        let servicio = self.servicio.clone();
        let fuente = obra.fuente.clone();
        self.ocupado = true;
        self.error = None;
        self.tarea(ctx, async move {
            match servicio.paginas(&fuente, &capitulo.id).await {
                Ok(paginas) => Mensaje::Paginas(Ok((fuente, obra, capitulo, paginas))),
                Err(e) => Mensaje::Paginas(Err(e.to_string())),
            }
        });
    }

    fn solicitar_imagen(&mut self, ctx: &Context, indice: usize) {
        let url = match &mut self.lector {
            Some(lector) => {
                if lector.ya_solicitada(indice) {
                    return;
                }
                let Some(url) = lector.url_pagina(indice) else {
                    return;
                };
                lector.marcar_solicitada(indice);
                url
            }
            None => return,
        };

        let cliente = self.cliente.clone();
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        let semaforo = self.semaforo.clone();

        self.runtime.spawn(async move {
            let resultado = match semaforo.acquire().await {
                Ok(_permiso) => match zaptome_lector::descargar(&cliente, &url).await {
                    Ok(bytes) => zaptome_lector::decodificar(&bytes).map_err(|e| e.to_string()),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            };
            let _ = tx.send(Mensaje::Imagen { indice, resultado });
            ctx.request_repaint();
        });
    }

    fn procesar_mensajes(&mut self, ctx: &Context) {
        while let Ok(mensaje) = self.rx.try_recv() {
            match mensaje {
                Mensaje::Busqueda(Ok(obras)) => {
                    self.resultados = obras;
                    self.ocupado = false;
                }
                Mensaje::Busqueda(Err(e)) => {
                    self.error = Some(e);
                    self.ocupado = false;
                }
                Mensaje::Biblioteca(Ok(entradas)) => {
                    self.biblioteca = entradas;
                    self.ocupado = false;
                }
                Mensaje::Biblioteca(Err(e)) => {
                    self.error = Some(e);
                    self.ocupado = false;
                }
                Mensaje::Capitulos { obra, capitulos } => {
                    self.ocupado = false;
                    match capitulos {
                        Ok(capitulos) => {
                            self.capitulos_obra = Some((obra, capitulos));
                            self.pantalla = Pantalla::Capitulos;
                        }
                        Err(e) => self.error = Some(e),
                    }
                }
                Mensaje::Paginas(Ok((fuente, obra, capitulo, paginas))) => {
                    self.ocupado = false;
                    self.lector = Some(Lector::nuevo(fuente, obra, capitulo, paginas));
                    self.pantalla = Pantalla::Lector;
                }
                Mensaje::Paginas(Err(e)) => {
                    self.ocupado = false;
                    self.error = Some(e);
                }
                Mensaje::Imagen { indice, resultado } => match resultado {
                    Ok(imagen) => {
                        if let Some(lector) = &mut self.lector {
                            lector.registrar_imagen(ctx, indice, imagen);
                        }
                    }
                    Err(e) => {
                        if let Some(lector) = &mut self.lector {
                            lector.olvidar_solicitud(indice);
                            lector.error = Some(e);
                        }
                    }
                },
                Mensaje::Portada { url, resultado } => match resultado {
                    Ok(imagen) => {
                        let color = egui::ColorImage::from_rgba_unmultiplied(
                            [imagen.ancho as usize, imagen.alto as usize],
                            &imagen.pixeles,
                        );
                        let textura = ctx.load_texture(
                            format!("portada:{url}"),
                            color,
                            egui::TextureOptions::LINEAR,
                        );
                        self.portadas.insert(url, EstadoPortada::Lista(textura));
                    }
                    Err(_) => {
                        self.portadas.insert(url, EstadoPortada::Fallida);
                    }
                },
                Mensaje::Quitado(Ok(())) => {
                    self.cargar_biblioteca(ctx);
                }
                Mensaje::Quitado(Err(e)) => {
                    self.error = Some(e);
                }
            }
        }
    }

    fn ui_biblioteca(&mut self, ui: &mut egui::Ui, ctx: &Context, p: &Paleta) {
        ui.label(tema::titulo("Biblioteca", 22.0));
        ui.add_space(4.0);

        if self.biblioteca.is_empty() {
            ui.label(
                RichText::new("Aún no tienes nada guardado. Busca una obra y ábrela.")
                    .color(p.on_surface_variant),
            );
            return;
        }

        let mut abrir: Option<Obra> = None;
        let mut borrar: Option<(IdObra, IdFuente)> = None;
        let mut pedir_portadas: Vec<String> = Vec::new();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for entrada in &self.biblioteca {
                tema::tarjeta(ui, p, |ui| {
                    ui.horizontal_top(|ui| {
                        if let Some(url) = &entrada.portada {
                            portada_widget(ui, p, self.textura_portada(url));
                            if !self.portadas.contains_key(url) {
                                pedir_portadas.push(url.clone());
                            }
                        }
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(tema::titulo(&entrada.titulo, 17.0));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if tema::boton_icono(
                                        ui,
                                        iconos::PAPELERA,
                                        20.0,
                                        p.on_surface_variant,
                                    )
                                    .clicked()
                                    {
                                        borrar =
                                            Some((entrada.obra.clone(), entrada.fuente.clone()));
                                    }
                                });
                            });
                            if let Some(sinopsis) = &entrada.sinopsis {
                                let recorte: String = sinopsis.chars().take(140).collect();
                                ui.label(
                                    RichText::new(format!("{recorte}…"))
                                        .size(13.0)
                                        .color(p.on_surface_variant),
                                );
                            }
                            ui.add_space(8.0);
                            if tema::boton_relleno(ui, "Capítulos", p).clicked() {
                                abrir = Some(obra_desde_entrada(entrada));
                            }
                        });
                    });
                });
                ui.add_space(8.0);
            }
        });

        for url in pedir_portadas {
            self.solicitar_portada(ctx, &url);
        }
        if let Some((obra, fuente)) = borrar {
            self.quitar_de_biblioteca(ctx, obra, fuente);
        }
        if let Some(obra) = abrir {
            self.cargar_capitulos(ctx, obra);
        }
    }

    fn ui_buscar(&mut self, ui: &mut egui::Ui, ctx: &Context, p: &Paleta) {
        ui.horizontal(|ui| {
            ui.label(tema::titulo("Buscar", 22.0));
            ui.add_space(10.0);

            egui::ComboBox::from_id_salt("fuente")
                .selected_text(
                    self.fuentes
                        .get(self.fuente_sel)
                        .map(|f| f.nombre.clone())
                        .unwrap_or_default(),
                )
                .show_ui(ui, |ui| {
                    for (i, fuente) in self.fuentes.iter().enumerate() {
                        ui.selectable_value(&mut self.fuente_sel, i, &fuente.nombre);
                    }
                });

            let campo = ui.add(
                egui::TextEdit::singleline(&mut self.consulta)
                    .hint_text("Buscar manga o manhwa…")
                    .desired_width(340.0),
            );
            let enter = campo.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if tema::boton_relleno(ui, "Buscar", p).clicked() || enter {
                self.iniciar_busqueda(ctx);
            }
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(6.0);

        let mut abrir: Option<Obra> = None;
        let mut pedir_portadas: Vec<String> = Vec::new();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for obra in &self.resultados {
                tema::tarjeta(ui, p, |ui| {
                    ui.horizontal_top(|ui| {
                        if let Some(url) = &obra.portada {
                            portada_widget(ui, p, self.textura_portada(url));
                            if !self.portadas.contains_key(url) {
                                pedir_portadas.push(url.clone());
                            }
                        }
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(tema::titulo(&obra.titulo, 17.0));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if tema::boton_relleno(ui, "Abrir", p).clicked() {
                                        abrir = Some(obra.clone());
                                    }
                                });
                            });
                            if let Some(sinopsis) = &obra.sinopsis {
                                let recorte: String = sinopsis.chars().take(140).collect();
                                ui.label(
                                    RichText::new(format!("{recorte}…"))
                                        .size(13.0)
                                        .color(p.on_surface_variant),
                                );
                            }
                        });
                    });
                });
                ui.add_space(8.0);
            }
        });

        for url in pedir_portadas {
            self.solicitar_portada(ctx, &url);
        }
        if let Some(obra) = abrir {
            self.cargar_capitulos(ctx, obra);
        }
    }

    fn ui_capitulos(&mut self, ui: &mut egui::Ui, ctx: &Context, p: &Paleta) {
        let Some((obra, capitulos)) = self.capitulos_obra.clone() else {
            return;
        };

        ui.horizontal(|ui| {
            if tema::boton_icono(ui, iconos::ATRAS, 22.0, p.on_surface_variant).clicked() {
                self.pantalla = Pantalla::Biblioteca;
            }
            ui.label(tema::titulo(&obra.titulo, 22.0));
        });
        ui.label(
            RichText::new(format!("{} capítulos", capitulos.len())).color(p.on_surface_variant),
        );
        ui.add_space(8.0);

        let mut elegido: Option<Capitulo> = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for capitulo in &capitulos {
                tema::tarjeta(ui, p, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&capitulo.titulo).size(15.0));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if tema::boton_tonal(ui, "Leer", p).clicked() {
                                elegido = Some(capitulo.clone());
                            }
                        });
                    });
                });
                ui.add_space(6.0);
            }
        });

        if let Some(capitulo) = elegido {
            self.abrir_capitulo(ctx, obra, capitulo);
        }
    }

    fn ui_lector(&mut self, ui: &mut egui::Ui, ctx: &Context, p: &Paleta) {
        let mut pedir = Vec::new();
        let mut volver = false;

        if let Some(lector) = &mut self.lector {
            ui.horizontal(|ui| {
                if tema::boton_icono(ui, iconos::ATRAS, 22.0, p.on_surface_variant).clicked() {
                    volver = true;
                }
                ui.add_space(4.0);
                ui.label(tema::titulo(&lector.obra.titulo, 18.0));
                ui.label(RichText::new(&lector.capitulo.titulo).color(p.on_surface_variant));
                ui.label(RichText::new(format!("· {}", lector.fuente.0)).color(p.outline));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{} páginas", lector.paginas.len()))
                            .color(p.on_surface_variant),
                    );
                    if let Some(error) = &lector.error {
                        ui.colored_label(p.error, error);
                    }
                });
            });

            ui.add_space(8.0);
            let ancho = ui.available_width();
            lector.asegurar_maqueta(ancho);
            lector.dibujar(ui, &mut pedir, p);
        }

        if volver {
            self.lector = None;
            self.pantalla = Pantalla::Biblioteca;
            self.cargar_biblioteca(ctx);
        }

        for indice in pedir {
            self.solicitar_imagen(ctx, indice);
        }
    }
}

impl eframe::App for AppZapTome {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        tema::aplicar(ctx, self.oscuro);
        let p = tema::paleta(self.oscuro);

        if !self.iniciado {
            self.iniciado = true;
            self.cargar_biblioteca(ctx);
        }

        self.procesar_mensajes(ctx);

        egui::TopBottomPanel::top("barra")
            .exact_height(64.0)
            .frame(
                Frame::none()
                    .fill(p.surface)
                    .inner_margin(Margin::symmetric(16.0, 10.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(tema::titulo("ZapTome", 24.0).color(p.primary));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let glyph = if self.oscuro {
                            iconos::CLARO
                        } else {
                            iconos::OSCURO
                        };
                        if tema::boton_icono(ui, glyph, 22.0, p.on_surface_variant).clicked() {
                            self.oscuro = !self.oscuro;
                        }
                        if self.ocupado {
                            ui.spinner();
                        }
                        if let Some(error) = &self.error {
                            ui.colored_label(p.error, error);
                        }
                    });
                });
            });

        if self.pantalla != Pantalla::Lector {
            egui::SidePanel::left("nav")
                .exact_width(96.0)
                .resizable(false)
                .frame(
                    Frame::none()
                        .fill(p.surface)
                        .inner_margin(Margin::symmetric(10.0, 12.0)),
                )
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(4.0);
                        if boton_nav(
                            ui,
                            &p,
                            self.pantalla == Pantalla::Biblioteca,
                            iconos::BIBLIOTECA,
                            "Biblioteca",
                        ) {
                            self.pantalla = Pantalla::Biblioteca;
                            self.cargar_biblioteca(ctx);
                        }
                        ui.add_space(6.0);
                        if boton_nav(
                            ui,
                            &p,
                            self.pantalla == Pantalla::Buscar,
                            iconos::BUSCAR,
                            "Buscar",
                        ) {
                            self.pantalla = Pantalla::Buscar;
                        }
                    });
                });
        }

        let relleno = if self.pantalla == Pantalla::Lector {
            p.surface_container_low
        } else {
            p.surface
        };

        egui::CentralPanel::default()
            .frame(Frame::none().fill(relleno).inner_margin(Margin::same(16.0)))
            .show(ctx, |ui| match self.pantalla {
                Pantalla::Biblioteca => self.ui_biblioteca(ui, ctx, &p),
                Pantalla::Buscar => self.ui_buscar(ui, ctx, &p),
                Pantalla::Capitulos => self.ui_capitulos(ui, ctx, &p),
                Pantalla::Lector => self.ui_lector(ui, ctx, &p),
            });
    }
}

/// Dibuja una portada con relación 2:3, o un marcador si aún no está.
fn portada_widget(ui: &mut egui::Ui, p: &Paleta, textura: Option<egui::TextureId>) {
    let (rect, _) = ui.allocate_exact_size(TAM_PORTADA, Sense::hover());
    match textura {
        Some(id) => {
            ui.painter().image(
                id,
                rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        None => {
            ui.painter()
                .rect_filled(rect, Rounding::same(10.0), p.surface_container_high);
        }
    }
}

/// Botón del riel de navegación, con indicador animado.
fn boton_nav(ui: &mut egui::Ui, p: &Paleta, activo: bool, icono: &str, texto: &str) -> bool {
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::new(72.0, 60.0), Sense::click());

    let t = ui
        .ctx()
        .animate_bool_with_time(ui.id().with(("nav", texto)), activo, 0.22);
    if t > 0.01 {
        ui.painter().rect_filled(
            rect,
            Rounding::same(16.0),
            p.secondary_container.gamma_multiply(t),
        );
    }

    let color = if activo {
        p.on_secondary_container
    } else {
        p.on_surface_variant
    };

    let pintor = ui.painter();
    pintor.text(
        rect.center() - Vec2::new(0.0, 11.0),
        Align2::CENTER_CENTER,
        icono,
        FontId::proportional(22.0),
        color,
    );
    pintor.text(
        rect.center() + Vec2::new(0.0, 13.0),
        Align2::CENTER_CENTER,
        texto,
        FontId::proportional(12.0),
        color,
    );

    respuesta.clicked()
}

fn obra_desde_entrada(entrada: &EntradaBiblioteca) -> Obra {
    Obra {
        id: entrada.obra.clone(),
        fuente: entrada.fuente.clone(),
        titulo: entrada.titulo.clone(),
        sinopsis: None,
        portada: entrada.portada.clone(),
        autores: Vec::new(),
        etiquetas: Vec::new(),
        estado: EstadoObra::Desconocido,
    }
}

/// Arranca la interfaz de ZapTome.
pub fn ejecutar(servicio: Servicio) -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;

    let opciones = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 820.0])
            .with_min_inner_size([820.0, 600.0])
            .with_title("ZapTome"),
        ..Default::default()
    };

    eframe::run_native(
        "ZapTome",
        opciones,
        Box::new(move |cc| {
            tema::instalar_fuentes(&cc.egui_ctx);
            Ok(Box::new(AppZapTome::nuevo(servicio, runtime)))
        }),
    )
    .map_err(|e| e.to_string())
}
