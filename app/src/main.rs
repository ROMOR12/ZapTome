//! Binario de ZapTome.
//!
//! Ensambla todas las piezas: dominio, aplicación, fuentes, persistencia, lector e
//! interfaz. Aquí se compone todo y se arranca la aplicación.

use std::path::PathBuf;
use std::sync::Arc;

use zaptome_aplicacion::{Catalogo, Servicio};
use zaptome_dominio::{FuenteManga, RepositorioBiblioteca, RepositorioProgreso};
use zaptome_fuentes::MangaDex;
use zaptome_persistencia::BaseDeDatos;
use zaptome_ui::ejecutar;

fn main() {
    if let Err(error) = arrancar() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn arrancar() -> Result<(), String> {
    let ruta = ruta_base_datos()?;
    let base = Arc::new(BaseDeDatos::abrir(&ruta).map_err(|e| e.to_string())?);

    let mut catalogo = Catalogo::nuevo();
    let mangadex = MangaDex::nuevo().map_err(|e| e.to_string())?;
    catalogo.agregar(Arc::new(mangadex) as Arc<dyn FuenteManga>);

    let biblioteca: Arc<dyn RepositorioBiblioteca> = base.clone();
    let progreso: Arc<dyn RepositorioProgreso> = base.clone();

    let servicio = Servicio::nuevo(Arc::new(catalogo), biblioteca, progreso);
    ejecutar(servicio)
}

fn ruta_base_datos() -> Result<PathBuf, String> {
    let base = if cfg!(windows) {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .map_err(|_| "no se encontró APPDATA".to_string())?
    } else {
        let hogar = std::env::var("HOME").map_err(|_| "no se encontró HOME".to_string())?;
        PathBuf::from(hogar).join(".local").join("share")
    };

    let directorio = base.join("ZapTome");
    std::fs::create_dir_all(&directorio).map_err(|e| e.to_string())?;
    Ok(directorio.join("zaptome.db"))
}
