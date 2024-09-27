pub mod tree;

use std::sync::{Arc, RwLock};

/// Trait `Structure` define una interfaz para estructuras que generan bloques en posiciones
/// absolutas en el mundo. Las estructuras que implementen este trait deben proporcionar
/// una función que devuelva una lista de bloques a partir de una posición inicial dada.
pub trait Structure {
    /// Genera una lista de bloques que representan la estructura en la posición dada.
    /// 
    /// # Argumentos
    /// - `position`: Posición absoluta inicial de la estructura en el mundo, representada
    /// como un `glam::Vec3` (vector tridimensional).
    ///
    /// # Retorno
    /// Devuelve un `Vec` de punteros atómicos referenciados (`Arc`) y protegidos por un bloqueo 
    /// de lectura-escritura (`RwLock`) a los bloques que componen la estructura.
    fn get_blocks(position: glam::Vec3) -> Vec<Arc<RwLock<Block>>>;
}

/// Reexporta el módulo `Tree` desde el submódulo `tree`.
pub use tree::Tree;

use crate::blocks::block::Block; // Reexporta el bloque dentro del módulo de estructuras
