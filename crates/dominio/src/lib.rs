//! Dominio de ZapTome: entidades, reglas de negocio y puertos.
//!
//! Este crate no conoce la interfaz, ni la red, ni la base de datos, ni el motor de
//! extensiones. Todo acceso al exterior se define aquí como *traits* (puertos) que
//! implementan los adaptadores en otros crates.
