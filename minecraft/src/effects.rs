pub mod ao {
    use crate::blocks::block_type::BlockType; // Importa el tipo de bloque desde el módulo de bloques.
    use crate::chunk::BlockVec; // Importa el tipo BlockVec desde el módulo de chunk.
    use crate::utils::{ChunkFromPosition, RelativeFromAbsolute}; // Importa funciones de utilidad para conversión de posiciones.
    use crate::world::CHUNK_SIZE; // Importa el tamaño del chunk desde el módulo de world.

    // Función que calcula la oclusión ambiental (ambient occlusion) para un vértice.
    // Para mundos similares a Minecraft. Consulta: https://0fps.net/2013/07/03/ambient-occlusion-for-minecraft-like-worlds/
    pub(crate) fn calc_vertex_ao(side1: bool, side2: bool, up: bool) -> u8 {
        // Si ambos lados están presentes, devuelve 0 (máxima oclusión).
        if side1 && side2 {
            return 0;
        }
        // Calcula la oclusión ambiental basada en los lados y devuelve un valor de 0 a 3.
        3 - (side1 as u8 + side2 as u8 + up as u8)
    }

    // Convierte la posición de un vértice en coordenadas absolutas a un valor de oclusión ambiental.
    pub(crate) fn from_vertex_position(
        vertex_position: &glam::Vec3, // Posición del vértice.
        blocks_positions: &Vec<((i32, i32), BlockVec)>, // Vec vector que contiene las posiciones de los bloques.
    ) -> u8 {
        // Define posiciones para los lados y la esquina del vértice.
        let side1_position = *vertex_position + glam::vec3(1.0, 1.0, 0.0); // Lado 1.
        let side2_position = *vertex_position + glam::vec3(0.0, 1.0, 1.0); // Lado 2.
        let corner_position = *vertex_position + glam::vec3(1.0, 1.0, 1.0); // Esquina.

        // Obtiene los chunks correspondientes a las posiciones de los lados y la esquina.
        let side1_chunk = side1_position.get_chunk_from_position_absolute();
        let side1_position = side1_position.relative_from_absolute();

        let side2_chunk = side2_position.get_chunk_from_position_absolute();
        let side2_position = side2_position.relative_from_absolute();

        let corner_chunk = corner_position.get_chunk_from_position_absolute();
        let corner_position = corner_position.relative_from_absolute();

        // Inicializa las variables que indican si hay bloques en los lados y la esquina.
        let mut has_side1 = false;
        let mut has_side2 = false;
        let mut has_corner = false;

        // Verifica la presencia de bloques en las posiciones de los lados y la esquina.
        for (position, chunk, val) in [
            (side1_position, side1_chunk, &mut has_side1),
            (side2_position, side2_chunk, &mut has_side2),
            (corner_position, corner_chunk, &mut has_corner),
        ] {
            if let Some(blocks) = blocks_positions.iter().find_map(|c| {
                if c.0 == chunk { // Si el chunk coincide.
                    Some(c.1.clone()) // Retorna los bloques del chunk.
                } else {
                    None // En caso contrario, retorna None.
                }
            }) {
                let blocks = blocks.read().unwrap(); // Bloques leídos en modo seguro.
                let ycol = &blocks[((position.x * CHUNK_SIZE as f32) + position.z) as usize]; // Obtiene la columna de bloques correspondiente.
                if let Some(place) = ycol.get(position.y as usize) { // Verifica la posición del bloque.
                    if let Some(block) = place { // Si existe un bloque en esa posición.
                        if block.read().unwrap().block_type != BlockType::Water { // Si no es un bloque de agua.
                            *val = true // Establece el valor correspondiente a verdadero.
                        }
                    }
                }
            }
        }
        // Calcula y devuelve el valor de oclusión ambiental para el vértice.
        calc_vertex_ao(has_side1, has_side2, has_corner)
    }

    // Convierte el valor de oclusión ambiental de un entero a un valor flotante entre 0 y 1.
    // ao -> 1 (máximo)
    // ao -> 0 (mínimo)
    pub(crate) fn convert_ao_u8_to_f32(ao: u8) -> f32 {
        1.0 - (ao as f32 / 3.0) // Devuelve 1.0 menos el valor normalizado de oclusión.
    }
}
