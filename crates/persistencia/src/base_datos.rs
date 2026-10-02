//! Base de datos local en SQLite.

use std::path::Path;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rusqlite::{params, Connection, OptionalExtension};

use zaptome_dominio::{
    EntradaBiblioteca, ErrorPersistencia, IdCapitulo, IdFuente, IdObra, Progreso,
    RepositorioAjustes, RepositorioBiblioteca, RepositorioProgreso,
};

/// Esquema de la base de datos. Se aplica al abrir y es idempotente.
const ESQUEMA: &str = r#"
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS biblioteca (
    obra       TEXT    NOT NULL,
    fuente     TEXT    NOT NULL,
    titulo     TEXT    NOT NULL,
    sinopsis   TEXT,
    portada    TEXT,
    categorias TEXT    NOT NULL DEFAULT '[]',
    favorito   INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (obra, fuente)
);

CREATE TABLE IF NOT EXISTS progreso (
    obra        TEXT    NOT NULL,
    fuente      TEXT    NOT NULL,
    capitulo    TEXT    NOT NULL,
    pagina      INTEGER NOT NULL DEFAULT 0,
    leido       INTEGER NOT NULL DEFAULT 0,
    actualizado INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (obra, fuente)
);

CREATE TABLE IF NOT EXISTS ajustes (
    clave TEXT PRIMARY KEY,
    valor TEXT NOT NULL
);
"#;

/// Base de datos SQLite local.
pub struct BaseDeDatos {
    conexion: Arc<Mutex<Connection>>,
}

impl BaseDeDatos {
    /// Abre o crea la base de datos en la ruta indicada y aplica el esquema.
    pub fn abrir(ruta: impl AsRef<Path>) -> Result<Self, ErrorPersistencia> {
        let conexion = Connection::open(ruta).map_err(map_err)?;
        Self::desde_conexion(conexion)
    }

    /// Crea una base de datos en memoria. Útil para pruebas.
    pub fn en_memoria() -> Result<Self, ErrorPersistencia> {
        let conexion = Connection::open_in_memory().map_err(map_err)?;
        Self::desde_conexion(conexion)
    }

    fn desde_conexion(conexion: Connection) -> Result<Self, ErrorPersistencia> {
        conexion.execute_batch(ESQUEMA).map_err(map_err)?;
        // Migración: añade la columna a bases creadas antes de que existiera. Si ya está,
        // el error se ignora.
        let _ = conexion.execute("ALTER TABLE biblioteca ADD COLUMN sinopsis TEXT", []);
        Ok(Self {
            conexion: Arc::new(Mutex::new(conexion)),
        })
    }

    fn bloqueo(&self) -> Result<std::sync::MutexGuard<'_, Connection>, ErrorPersistencia> {
        self.conexion
            .lock()
            .map_err(|_| ErrorPersistencia::BaseDeDatos("bloqueo envenenado".into()))
    }
}

fn map_err(error: rusqlite::Error) -> ErrorPersistencia {
    ErrorPersistencia::BaseDeDatos(error.to_string())
}

fn decodificar_categorias(json: &str) -> Vec<String> {
    serde_json::from_str(json).unwrap_or_default()
}

/// Lee una fila de la biblioteca.
fn leer_entrada(fila: &rusqlite::Row<'_>) -> rusqlite::Result<EntradaBiblioteca> {
    let categorias: String = fila.get(5)?;
    Ok(EntradaBiblioteca {
        obra: IdObra(fila.get(0)?),
        fuente: IdFuente(fila.get(1)?),
        titulo: fila.get(2)?,
        sinopsis: fila.get(3)?,
        portada: fila.get(4)?,
        categorias: decodificar_categorias(&categorias),
        favorito: fila.get::<_, i64>(6)? != 0,
    })
}

#[async_trait]
impl RepositorioBiblioteca for BaseDeDatos {
    async fn guardar(&self, entrada: &EntradaBiblioteca) -> Result<(), ErrorPersistencia> {
        let categorias = serde_json::to_string(&entrada.categorias)
            .map_err(|e| ErrorPersistencia::BaseDeDatos(e.to_string()))?;
        let conexion = self.bloqueo()?;
        conexion
            .execute(
                "INSERT INTO biblioteca (obra, fuente, titulo, sinopsis, portada, categorias, favorito)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(obra, fuente) DO UPDATE SET
                     titulo     = excluded.titulo,
                     sinopsis   = excluded.sinopsis,
                     portada    = excluded.portada,
                     categorias = excluded.categorias,
                     favorito   = excluded.favorito",
                params![
                    entrada.obra.0,
                    entrada.fuente.0,
                    entrada.titulo,
                    entrada.sinopsis,
                    entrada.portada,
                    categorias,
                    entrada.favorito as i64,
                ],
            )
            .map_err(map_err)?;
        Ok(())
    }

    async fn eliminar(&self, obra: &IdObra, fuente: &IdFuente) -> Result<(), ErrorPersistencia> {
        let conexion = self.bloqueo()?;
        conexion
            .execute(
                "DELETE FROM biblioteca WHERE obra = ?1 AND fuente = ?2",
                params![obra.0, fuente.0],
            )
            .map_err(map_err)?;
        Ok(())
    }

    async fn listar(&self) -> Result<Vec<EntradaBiblioteca>, ErrorPersistencia> {
        let conexion = self.bloqueo()?;
        let mut sentencia = conexion
            .prepare(
                "SELECT obra, fuente, titulo, sinopsis, portada, categorias, favorito
                 FROM biblioteca ORDER BY titulo COLLATE NOCASE",
            )
            .map_err(map_err)?;
        let filas = sentencia.query_map([], leer_entrada).map_err(map_err)?;

        let mut resultado = Vec::new();
        for fila in filas {
            resultado.push(fila.map_err(map_err)?);
        }
        Ok(resultado)
    }

    async fn obtener(
        &self,
        obra: &IdObra,
        fuente: &IdFuente,
    ) -> Result<Option<EntradaBiblioteca>, ErrorPersistencia> {
        let conexion = self.bloqueo()?;
        conexion
            .query_row(
                "SELECT obra, fuente, titulo, sinopsis, portada, categorias, favorito
                 FROM biblioteca WHERE obra = ?1 AND fuente = ?2",
                params![obra.0, fuente.0],
                leer_entrada,
            )
            .optional()
            .map_err(map_err)
    }
}

#[async_trait]
impl RepositorioProgreso for BaseDeDatos {
    async fn guardar(&self, progreso: &Progreso) -> Result<(), ErrorPersistencia> {
        let conexion = self.bloqueo()?;
        conexion
            .execute(
                "INSERT INTO progreso (obra, fuente, capitulo, pagina, leido, actualizado)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(obra, fuente) DO UPDATE SET
                     capitulo    = excluded.capitulo,
                     pagina      = excluded.pagina,
                     leido       = excluded.leido,
                     actualizado = excluded.actualizado",
                params![
                    progreso.obra.0,
                    progreso.fuente.0,
                    progreso.capitulo.0,
                    progreso.pagina as i64,
                    progreso.leido as i64,
                    progreso.actualizado,
                ],
            )
            .map_err(map_err)?;
        Ok(())
    }

    async fn obtener(
        &self,
        obra: &IdObra,
        fuente: &IdFuente,
    ) -> Result<Option<Progreso>, ErrorPersistencia> {
        let conexion = self.bloqueo()?;
        conexion
            .query_row(
                "SELECT obra, fuente, capitulo, pagina, leido, actualizado
                 FROM progreso WHERE obra = ?1 AND fuente = ?2",
                params![obra.0, fuente.0],
                |fila| {
                    Ok(Progreso {
                        obra: IdObra(fila.get(0)?),
                        fuente: IdFuente(fila.get(1)?),
                        capitulo: IdCapitulo(fila.get(2)?),
                        pagina: fila.get::<_, i64>(3)? as usize,
                        leido: fila.get::<_, i64>(4)? != 0,
                        actualizado: fila.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(map_err)
    }

    async fn listar(&self) -> Result<Vec<Progreso>, ErrorPersistencia> {
        let conexion = self.bloqueo()?;
        let mut sentencia = conexion
            .prepare(
                "SELECT obra, fuente, capitulo, pagina, leido, actualizado
                 FROM progreso ORDER BY actualizado DESC",
            )
            .map_err(map_err)?;
        let filas = sentencia
            .query_map([], |fila| {
                Ok(Progreso {
                    obra: IdObra(fila.get(0)?),
                    fuente: IdFuente(fila.get(1)?),
                    capitulo: IdCapitulo(fila.get(2)?),
                    pagina: fila.get::<_, i64>(3)? as usize,
                    leido: fila.get::<_, i64>(4)? != 0,
                    actualizado: fila.get(5)?,
                })
            })
            .map_err(map_err)?;

        let mut resultado = Vec::new();
        for fila in filas {
            resultado.push(fila.map_err(map_err)?);
        }
        Ok(resultado)
    }
}

#[async_trait]
impl RepositorioAjustes for BaseDeDatos {
    async fn obtener(&self, clave: &str) -> Result<Option<String>, ErrorPersistencia> {
        let conexion = self.bloqueo()?;
        conexion
            .query_row(
                "SELECT valor FROM ajustes WHERE clave = ?1",
                params![clave],
                |fila| fila.get(0),
            )
            .optional()
            .map_err(map_err)
    }

    async fn guardar(&self, clave: &str, valor: &str) -> Result<(), ErrorPersistencia> {
        let conexion = self.bloqueo()?;
        conexion
            .execute(
                "INSERT INTO ajustes (clave, valor) VALUES (?1, ?2)
                 ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
                params![clave, valor],
            )
            .map_err(map_err)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entrada() -> EntradaBiblioteca {
        EntradaBiblioteca {
            obra: IdObra("obra-1".into()),
            fuente: IdFuente("mangadex".into()),
            titulo: "Ejemplo".into(),
            sinopsis: Some("Una sinopsis".into()),
            portada: Some("https://example.test/portada.jpg".into()),
            categorias: vec!["favoritos".into()],
            favorito: true,
        }
    }

    #[tokio::test]
    async fn biblioteca_guarda_y_recupera() {
        let bd = BaseDeDatos::en_memoria().unwrap();
        RepositorioBiblioteca::guardar(&bd, &entrada()).await.unwrap();

        let recuperada = RepositorioBiblioteca::obtener(
            &bd,
            &IdObra("obra-1".into()),
            &IdFuente("mangadex".into()),
        )
        .await
        .unwrap()
        .expect("debe existir");
        assert_eq!(recuperada.titulo, "Ejemplo");
        assert_eq!(recuperada.sinopsis.as_deref(), Some("Una sinopsis"));
        assert!(recuperada.favorito);
        assert_eq!(recuperada.categorias, vec!["favoritos".to_string()]);
        assert_eq!(RepositorioBiblioteca::listar(&bd).await.unwrap().len(), 1);

        RepositorioBiblioteca::eliminar(
            &bd,
            &IdObra("obra-1".into()),
            &IdFuente("mangadex".into()),
        )
        .await
        .unwrap();
        assert!(RepositorioBiblioteca::listar(&bd).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn progreso_se_actualiza() {
        let bd = BaseDeDatos::en_memoria().unwrap();
        let mut progreso = Progreso {
            obra: IdObra("obra-1".into()),
            fuente: IdFuente("mangadex".into()),
            capitulo: IdCapitulo("cap-1".into()),
            pagina: 3,
            leido: false,
            actualizado: 1,
        };
        RepositorioProgreso::guardar(&bd, &progreso).await.unwrap();

        progreso.pagina = 10;
        progreso.leido = true;
        progreso.actualizado = 2;
        RepositorioProgreso::guardar(&bd, &progreso).await.unwrap();

        let recuperado = RepositorioProgreso::obtener(
            &bd,
            &IdObra("obra-1".into()),
            &IdFuente("mangadex".into()),
        )
        .await
        .unwrap()
        .expect("debe existir");
        assert_eq!(recuperado.pagina, 10);
        assert!(recuperado.leido);
    }

    #[tokio::test]
    async fn ajustes_clave_valor() {
        let bd = BaseDeDatos::en_memoria().unwrap();
        assert_eq!(RepositorioAjustes::obtener(&bd, "tema").await.unwrap(), None);
        RepositorioAjustes::guardar(&bd, "tema", "oscuro")
            .await
            .unwrap();
        assert_eq!(
            RepositorioAjustes::obtener(&bd, "tema").await.unwrap(),
            Some("oscuro".into())
        );
    }
}
