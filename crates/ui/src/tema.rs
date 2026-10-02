//! Tema Material 3 para la interfaz.
//!
//! Define la paleta por roles de color, instala las fuentes (Roboto + iconos), aplica el
//! estilo a egui y ofrece componentes básicos con el aspecto de Material 3.

use eframe::egui::{
    self, Color32, Context, FontData, FontDefinitions, FontFamily, Frame, Margin, Response,
    RichText, Rounding, Stroke, Ui, Vec2,
};

/// Paleta de color según los roles de Material 3.
#[derive(Clone, Copy)]
pub struct Paleta {
    pub primary: Color32,
    pub on_primary: Color32,
    pub secondary_container: Color32,
    pub on_secondary_container: Color32,
    pub surface: Color32,
    pub surface_container_low: Color32,
    pub surface_container: Color32,
    pub surface_container_high: Color32,
    pub on_surface: Color32,
    pub on_surface_variant: Color32,
    pub outline: Color32,
    pub outline_variant: Color32,
    pub error: Color32,
}

/// Paleta base oscura de Material 3.
pub fn oscura() -> Paleta {
    Paleta {
        primary: Color32::from_rgb(0xD0, 0xBC, 0xFF),
        on_primary: Color32::from_rgb(0x38, 0x1E, 0x72),
        secondary_container: Color32::from_rgb(0x4A, 0x44, 0x5A),
        on_secondary_container: Color32::from_rgb(0xE8, 0xDE, 0xF9),
        surface: Color32::from_rgb(0x14, 0x12, 0x18),
        surface_container_low: Color32::from_rgb(0x1D, 0x1B, 0x21),
        surface_container: Color32::from_rgb(0x21, 0x1F, 0x26),
        surface_container_high: Color32::from_rgb(0x2B, 0x29, 0x30),
        on_surface: Color32::from_rgb(0xE6, 0xE0, 0xE9),
        on_surface_variant: Color32::from_rgb(0xCA, 0xC4, 0xD0),
        outline: Color32::from_rgb(0x93, 0x8F, 0x99),
        outline_variant: Color32::from_rgb(0x49, 0x45, 0x4F),
        error: Color32::from_rgb(0xF2, 0xB8, 0xB5),
    }
}

/// Paleta base clara de Material 3.
pub fn clara() -> Paleta {
    Paleta {
        primary: Color32::from_rgb(0x67, 0x50, 0xA4),
        on_primary: Color32::from_rgb(0xFF, 0xFF, 0xFF),
        secondary_container: Color32::from_rgb(0xE8, 0xDE, 0xF9),
        on_secondary_container: Color32::from_rgb(0x1D, 0x19, 0x24),
        surface: Color32::from_rgb(0xFE, 0xF7, 0xFF),
        surface_container_low: Color32::from_rgb(0xF7, 0xF2, 0xFA),
        surface_container: Color32::from_rgb(0xF3, 0xED, 0xF7),
        surface_container_high: Color32::from_rgb(0xEC, 0xE6, 0xF0),
        on_surface: Color32::from_rgb(0x1D, 0x1B, 0x20),
        on_surface_variant: Color32::from_rgb(0x49, 0x45, 0x4F),
        outline: Color32::from_rgb(0x79, 0x74, 0x7E),
        outline_variant: Color32::from_rgb(0xC9, 0xC5, 0xD0),
        error: Color32::from_rgb(0xB3, 0x26, 0x1E),
    }
}

/// Devuelve la paleta correspondiente al modo.
pub fn paleta(oscuro: bool) -> Paleta {
    if oscuro {
        oscura()
    } else {
        clara()
    }
}

/// Instala las fuentes: Roboto para el texto, Roboto Bold para los títulos y Material
/// Icons para los iconos. Se llama una sola vez, al arrancar.
pub fn instalar_fuentes(ctx: &Context) {
    let mut fuentes = FontDefinitions::default();

    fuentes.font_data.insert(
        "roboto".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/Roboto-Regular.ttf")).into(),
    );
    fuentes.font_data.insert(
        "roboto-bold".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/Roboto-Bold.ttf")).into(),
    );
    fuentes.font_data.insert(
        "iconos".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/MaterialIcons-Regular.ttf")).into(),
    );

    let proporcional = fuentes
        .families
        .entry(FontFamily::Proportional)
        .or_default();
    proporcional.insert(0, "roboto".to_owned());
    proporcional.push("iconos".to_owned());

    fuentes.families.insert(
        FontFamily::Name("negrita".into()),
        vec!["roboto-bold".to_owned()],
    );

    ctx.set_fonts(fuentes);
}

/// Texto en negrita para títulos.
pub fn titulo(texto: impl Into<String>, tam: f32) -> RichText {
    RichText::new(texto)
        .size(tam)
        .family(FontFamily::Name("negrita".into()))
}

/// Aplica el estilo Material 3 a egui.
pub fn aplicar(ctx: &Context, oscuro: bool) {
    let p = paleta(oscuro);
    let mut visuals = if oscuro {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    visuals.panel_fill = p.surface;
    visuals.window_fill = p.surface_container;
    visuals.extreme_bg_color = p.surface_container_low;
    visuals.faint_bg_color = p.surface_container;
    visuals.override_text_color = Some(p.on_surface);
    visuals.hyperlink_color = p.primary;
    visuals.selection.bg_fill = p.primary;
    visuals.selection.stroke = Stroke::new(1.0_f32, p.on_primary);
    visuals.window_stroke = Stroke::new(1.0_f32, p.outline_variant);

    let radio = Rounding::same(10.0);
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.rounding = radio;
    }

    visuals.widgets.noninteractive.bg_fill = p.surface_container;
    visuals.widgets.inactive.weak_bg_fill = p.surface_container_high;
    visuals.widgets.inactive.bg_fill = p.surface_container_high;
    visuals.widgets.hovered.weak_bg_fill = p.surface_container_high;
    visuals.widgets.hovered.bg_fill = p.surface_container_high;
    visuals.widgets.active.weak_bg_fill = p.secondary_container;
    visuals.widgets.active.bg_fill = p.secondary_container;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, p.outline);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, p.outline);

    ctx.set_visuals(visuals);

    let mut estilo = (*ctx.style()).clone();
    estilo.spacing.item_spacing = Vec2::new(10.0, 10.0);
    estilo.spacing.button_padding = Vec2::new(16.0, 9.0);
    estilo.spacing.interact_size.y = 40.0;
    // Permite ventanas desplegables altas.
    estilo.spacing.combo_height = 600.0;
    // Transiciones suaves al pasar el ratón y al pulsar.
    estilo.animation_time = 0.22;
    ctx.set_style(estilo);
}

/// Botón relleno, para la acción principal.
pub fn boton_relleno(ui: &mut Ui, texto: &str, p: &Paleta) -> Response {
    ui.add(
        egui::Button::new(RichText::new(texto).color(p.on_primary))
            .fill(p.primary)
            .rounding(Rounding::same(20.0)),
    )
}

/// Botón tonal, para acciones secundarias.
pub fn boton_tonal(ui: &mut Ui, texto: &str, p: &Paleta) -> Response {
    ui.add(
        egui::Button::new(RichText::new(texto).color(p.on_secondary_container))
            .fill(p.secondary_container)
            .rounding(Rounding::same(20.0)),
    )
}

/// Botón de icono, redondo y sin relleno.
pub fn boton_icono(ui: &mut Ui, glyph: &str, tam: f32, color: Color32) -> Response {
    let lado = tam + 18.0;
    let (rect, respuesta) = ui.allocate_exact_size(Vec2::splat(lado), egui::Sense::click());

    if respuesta.hovered() {
        ui.painter()
            .rect_filled(rect, Rounding::same(lado / 2.0), color.gamma_multiply(0.12));
    }
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        egui::FontId::proportional(tam),
        color,
    );

    respuesta
}

/// Tarjeta contenedora con superficie y borde suaves.
pub fn tarjeta<R>(ui: &mut Ui, p: &Paleta, contenido: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::none()
        .fill(p.surface_container)
        .rounding(Rounding::same(12.0))
        .inner_margin(Margin::same(14.0))
        .stroke(Stroke::new(1.0_f32, p.outline_variant))
        .show(ui, contenido)
        .inner
}
