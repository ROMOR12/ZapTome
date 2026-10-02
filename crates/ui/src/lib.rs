//! Interfaz de usuario construida con egui.
//!
//! El área de lectura se dibuja en modo webtoon con virtualización: solo se descargan y
//! se suben a GPU las páginas visibles y las cercanas.

mod lector_view;

use std::sync::Arc;

use eframe::egui::{self, Context, RichText};
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, Semaphore};

use zaptome_aplicacion::Servicio;
use zaptome_dominio::{
    Capitulo, Consulta, EntradaBiblioteca, EstadoObra, Fuente, IdFuente, Obra, Pagina,
};
use zaptome_lector::ImagenDecodificada;

use lector_view::Lector;

const MAX_DESCARGAS_SIMULTANEAS: usize = 4;

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
                    Ok(bytes) => {
                        zaptome_lector::decodificar(&bytes).map_err(|e| e.to_string())
                    }
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
            }
        }
    }

    fn ui_biblioteca(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        ui.heading("Biblioteca");
        ui.separator();

        if self.biblioteca.is_empty() {
            ui.label("La biblioteca está vacía. Busca una obra y ábrela.");
            return;
        }

        let mut abrir: Option<Obra> = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for entrada in &self.biblioteca {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&entrada.titulo).strong());
                    if ui.button("Capítulos").clicked() {
                        abrir = Some(obra_desde_entrada(entrada));
                    }
                });
                ui.separator();
            }
        });

        if let Some(obra) = abrir {
            self.cargar_capitulos(ctx, obra);
        }
    }

    fn ui_buscar(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        ui.heading("Buscar");
        ui.separator();

        ui.horizontal(|ui| {
            egui::ComboBox::from_label("Fuente")
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

            let campo = ui.text_edit_singleline(&mut self.consulta);
            let enter = campo.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if ui.button("Buscar").clicked() || enter {
                self.iniciar_busqueda(ctx);
            }
        });

        ui.separator();

        let mut abrir: Option<Obra> = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for obra in &self.resultados {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&obra.titulo).strong());
                    if ui.button("Abrir").clicked() {
                        abrir = Some(obra.clone());
                    }
                });
                if let Some(sinopsis) = &obra.sinopsis {
                    ui.label(RichText::new(sinopsis).weak());
                }
                ui.separator();
            }
        });

        if let Some(obra) = abrir {
            self.cargar_capitulos(ctx, obra);
        }
    }

    fn ui_capitulos(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        let Some((obra, capitulos)) = self.capitulos_obra.clone() else {
            return;
        };

        ui.heading(&obra.titulo);
        ui.label(format!("{} capítulos", capitulos.len()));
        ui.separator();

        let mut elegido: Option<Capitulo> = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for capitulo in &capitulos {
                if ui.button(&capitulo.titulo).clicked() {
                    elegido = Some(capitulo.clone());
                }
            }
        });

        if let Some(capitulo) = elegido {
            self.abrir_capitulo(ctx, obra, capitulo);
        }
    }

    fn ui_lector(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        let mut pedir = Vec::new();

        if let Some(lector) = &mut self.lector {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{} — {}", lector.obra.titulo, lector.capitulo.titulo))
                        .strong(),
                );
                ui.label(format!("{} páginas", lector.paginas.len()));
                ui.label(format!("Fuente: {}", lector.fuente.0));
                if let Some(error) = &lector.error {
                    ui.colored_label(egui::Color32::RED, error);
                }
            });

            let ancho = ui.available_width();
            lector.asegurar_maqueta(ancho);
            lector.dibujar(ui, &mut pedir);
        }

        for indice in pedir {
            self.solicitar_imagen(ctx, indice);
        }
    }
}

impl eframe::App for AppZapTome {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if !self.iniciado {
            self.iniciado = true;
            self.cargar_biblioteca(ctx);
        }

        self.procesar_mensajes(ctx);

        egui::TopBottomPanel::top("barra").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.pantalla == Pantalla::Biblioteca, "Biblioteca")
                    .clicked()
                {
                    self.pantalla = Pantalla::Biblioteca;
                    self.cargar_biblioteca(ctx);
                }
                if ui
                    .selectable_label(self.pantalla == Pantalla::Buscar, "Buscar")
                    .clicked()
                {
                    self.pantalla = Pantalla::Buscar;
                }
                if self.pantalla == Pantalla::Lector && ui.button("← Volver").clicked() {
                    self.lector = None;
                    self.pantalla = Pantalla::Biblioteca;
                    self.cargar_biblioteca(ctx);
                }
                if self.ocupado {
                    ui.spinner();
                }
                if let Some(error) = &self.error {
                    ui.colored_label(egui::Color32::RED, error);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.pantalla {
            Pantalla::Biblioteca => self.ui_biblioteca(ui, ctx),
            Pantalla::Buscar => self.ui_buscar(ui, ctx),
            Pantalla::Capitulos => self.ui_capitulos(ui, ctx),
            Pantalla::Lector => self.ui_lector(ui, ctx),
        });
    }
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
            .with_inner_size([1100.0, 800.0])
            .with_title("ZapTome"),
        ..Default::default()
    };

    eframe::run_native(
        "ZapTome",
        opciones,
        Box::new(move |_cc| Ok(Box::new(AppZapTome::nuevo(servicio, runtime)))),
    )
    .map_err(|e| e.to_string())
}
