use std::sync::{Arc, RwLock};

use crate::{
    blocks::{block::Block, block_type::BlockType},
    utils::{ChunkFromPosition, RelativeFromAbsolute},
};

use super::Structure;

/// Representa una estructura de tipo árbol en el juego.
/// Esta estructura genera tanto el tronco como las hojas del árbol en posiciones relativas.
pub struct Tree;

impl Structure for Tree {
    /// Devuelve una lista de bloques (`Block`) que componen la estructura del árbol.
    /// El árbol está formado por bloques de tronco (`Wood`) y hojas (`Leaf`), 
    /// cuyas posiciones se calculan en función de la posición inicial proporcionada.
    ///
    /// # Argumentos
    /// * `position` - La posición base del árbol en el mundo, representada por un vector 3D (`Vec3`).
    ///
    /// # Retorna
    /// Un `Vec<Arc<RwLock<Block>>>` que contiene todos los bloques que forman el árbol.
    fn get_blocks(position: glam::Vec3) -> Vec<Arc<RwLock<Block>>> {
        // Definimos las posiciones de los bloques de tronco en relación con la posición base
        let trunk_pos = [
            position + glam::vec3(0.0, 1.0, 0.0), // Tronco en la base + 1 en Y
            position + glam::vec3(0.0, 2.0, 0.0), // Tronco en la base + 2 en Y
            position + glam::vec3(0.0, 3.0, 0.0), // Tronco en la base + 3 en Y
        ];

        // Definimos las posiciones de los bloques de hojas en relación con la posición base
        #[rustfmt::skip]
        let leafs_pos = [
            // Bloques de hojas alrededor de la parte superior del tronco
            position + glam::vec3(0.0, 3.0, 1.0),
            position + glam::vec3(0.0, 4.0, 1.0),
            position + glam::vec3(1.0, 3.0, 1.0),
            position + glam::vec3(1.0, 4.0, 1.0),
            position + glam::vec3(-1.0, 3.0, 1.0),
            position + glam::vec3(-1.0, 4.0, 1.0),

            position + glam::vec3(0.0, 3.0, -1.0),
            position + glam::vec3(0.0, 4.0, -1.0),
            position + glam::vec3(1.0, 3.0, -1.0),
            position + glam::vec3(1.0, 4.0, -1.0),
            position + glam::vec3(-1.0, 3.0, -1.0),
            position + glam::vec3(-1.0, 4.0, -1.0),

            position + glam::vec3(1.0, 3.0, 0.0),
            position + glam::vec3(1.0, 4.0, 0.0),
            position + glam::vec3(-1.0, 3.0, 0.0),
            position + glam::vec3(-1.0, 4.0, 0.0),

            // Hojas en la parte superior del árbol
            position + glam::vec3(0.0, 5.0, 0.0),
        ];

        // Mapeamos las posiciones del tronco a bloques de tipo `Wood` y obtenemos su chunk correspondiente
        let blocks = trunk_pos.iter().map(|p| {
            Arc::new(RwLock::new(Block::new(
                p.relative_from_absolute(),                  // Calcula la posición relativa
                p.get_chunk_from_position_absolute(),        // Calcula el chunk correspondiente
                BlockType::Wood,                             // Tipo de bloque: tronco
            )))
        });

        // Mapeamos las posiciones de las hojas a bloques de tipo `Leaf` y obtenemos su chunk correspondiente
        let leafs_iter = leafs_pos.iter().map(|p| {
            Arc::new(RwLock::new(Block::new(
                p.relative_from_absolute(),                  // Calcula la posición relativa
                p.get_chunk_from_position_absolute(),        // Calcula el chunk correspondiente
                BlockType::Leaf,                             // Tipo de bloque: hoja
            )))
        });
        
        // Combinamos las iteraciones de bloques de tronco y hojas en un solo vector
        blocks.chain(leafs_iter).collect::<Vec<_>>()
    }
}
