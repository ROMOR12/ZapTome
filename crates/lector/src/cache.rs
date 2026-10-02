//! Presupuesto de memoria para texturas, con evicción LRU.
//!
//! Mantiene un conjunto acotado de entradas (texturas decodificadas) y desaloja las
//! usadas hace más tiempo cuando se supera el presupuesto.

use std::collections::HashMap;
use std::hash::Hash;

struct Entrada {
    bytes: usize,
    ultimo_uso: u64,
}

/// Gestor de presupuesto de VRAM con evicción LRU.
pub struct CachePresupuesto<K> {
    presupuesto: usize,
    usado: usize,
    reloj: u64,
    entradas: HashMap<K, Entrada>,
}

impl<K: Eq + Hash + Clone> CachePresupuesto<K> {
    /// Crea un gestor con el presupuesto indicado en bytes.
    pub fn nuevo(presupuesto: usize) -> Self {
        Self {
            presupuesto,
            usado: 0,
            reloj: 0,
            entradas: HashMap::new(),
        }
    }

    /// Presupuesto máximo en bytes.
    pub fn presupuesto(&self) -> usize {
        self.presupuesto
    }

    /// Bytes usados actualmente.
    pub fn usado(&self) -> usize {
        self.usado
    }

    /// Número de entradas guardadas.
    pub fn len(&self) -> usize {
        self.entradas.len()
    }

    /// Indica si no hay entradas.
    pub fn is_empty(&self) -> bool {
        self.entradas.is_empty()
    }

    /// Indica si la clave está presente.
    pub fn contiene(&self, clave: &K) -> bool {
        self.entradas.contains_key(clave)
    }

    /// Registra o actualiza una entrada y devuelve las claves desalojadas.
    ///
    /// Se conserva al menos una entrada aunque supere el presupuesto por sí sola.
    pub fn registrar(&mut self, clave: K, bytes: usize) -> Vec<K> {
        self.reloj += 1;

        if let Some(entrada) = self.entradas.get_mut(&clave) {
            self.usado = self.usado.saturating_sub(entrada.bytes);
            entrada.bytes = bytes;
            entrada.ultimo_uso = self.reloj;
            self.usado += bytes;
        } else {
            self.entradas.insert(
                clave,
                Entrada {
                    bytes,
                    ultimo_uso: self.reloj,
                },
            );
            self.usado += bytes;
        }

        self.desalojar()
    }

    /// Marca una entrada como usada recientemente.
    pub fn tocar(&mut self, clave: &K) {
        if let Some(entrada) = self.entradas.get_mut(clave) {
            self.reloj += 1;
            entrada.ultimo_uso = self.reloj;
        }
    }

    /// Elimina una entrada.
    pub fn eliminar(&mut self, clave: &K) {
        if let Some(entrada) = self.entradas.remove(clave) {
            self.usado = self.usado.saturating_sub(entrada.bytes);
        }
    }

    fn desalojar(&mut self) -> Vec<K> {
        let mut desalojadas = Vec::new();

        while self.usado > self.presupuesto && self.entradas.len() > 1 {
            let clave_lru = self
                .entradas
                .iter()
                .min_by_key(|(_, entrada)| entrada.ultimo_uso)
                .map(|(clave, _)| clave.clone());

            match clave_lru {
                Some(clave) => {
                    if let Some(entrada) = self.entradas.remove(&clave) {
                        self.usado = self.usado.saturating_sub(entrada.bytes);
                    }
                    desalojadas.push(clave);
                }
                None => break,
            }
        }

        desalojadas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desaloja_la_mas_antigua_al_superar_el_presupuesto() {
        let mut cache = CachePresupuesto::<String>::nuevo(100);
        assert!(cache.registrar("a".into(), 60).is_empty());
        let desalojadas = cache.registrar("b".into(), 60);
        assert_eq!(desalojadas, vec!["a".to_string()]);
        assert_eq!(cache.usado(), 60);
        assert!(cache.contiene(&"b".into()));
        assert!(!cache.contiene(&"a".into()));
    }

    #[test]
    fn tocar_protege_de_la_eviccion() {
        let mut cache = CachePresupuesto::<String>::nuevo(100);
        cache.registrar("a".into(), 60);
        cache.registrar("b".into(), 30);
        cache.tocar(&"a".into());
        // Al añadir "c" (30) se supera el presupuesto; debe caer "b", no "a".
        let desalojadas = cache.registrar("c".into(), 30);
        assert_eq!(desalojadas, vec!["b".to_string()]);
        assert!(cache.contiene(&"a".into()));
    }

    #[test]
    fn conserva_al_menos_una_entrada_grande() {
        let mut cache = CachePresupuesto::<String>::nuevo(10);
        cache.registrar("a".into(), 100);
        assert!(cache.contiene(&"a".into()));
        assert_eq!(cache.len(), 1);
    }
}
