use bytemuck::{Pod, Zeroable}; // Importa traits necesarios para manipulación de datos binarios.

use super::block_type::BlockType;
use crate::chunk::BlockVec;
use crate::collision::CollisionBox; // Para manejar la detección de colisiones.
use crate::effects::ao::{convert_ao_u8_to_f32, from_vertex_position}; // Utilidades para el cálculo de oclusión ambiental.
use crate::world::CHUNK_SIZE; // Constante global que define el tamaño de un chunk en el mundo.
use glam::Vec3; // Biblioteca para operaciones con vectores 3D.
use std::sync::{Arc, RwLock}; // Para manejo de concurrencia segura.

/// Estructura que representa un bloque en el mundo.
#[derive(Debug)]
pub struct Block {
    pub position: glam::Vec3,           // Posición relativa del bloque dentro del chunk.
    pub absolute_position: glam::Vec3,   // Posición absoluta del bloque en el mundo.
    pub collision_box: CollisionBox,     // Caja de colisión para detección de colisiones.
    pub block_type: BlockType,           // El tipo de bloque, que define texturas, propiedades, etc.
}

/// Coordenadas de los vértices de un cubo (24 valores, 8 vértices).
#[rustfmt::skip]
pub const CUBE_VERTEX: [f32; 24] = [
    -0.5, -0.5, -0.5,   // Vértice 0
    -0.5, 0.5, -0.5,    // Vértice 1
    0.5, 0.5, -0.5,     // Vértice 2
    0.5, -0.5, -0.5,    // Vértice 3
    -0.5, -0.5, 0.5,    // Vértice 4
    -0.5, 0.5, 0.5,     // Vértice 5
    0.5, 0.5, 0.5,      // Vértice 6
    0.5, -0.5, 0.5,     // Vértice 7
];

/// Trait para bloques con texturas. Define cómo obtener las coordenadas de textura según la cara.
pub trait TexturedBlock {
    fn get_texcoords(&self, face_dir: FaceDirections) -> [[f32; 2]; 4]; // Devuelve coordenadas UV de textura.
}

/// Enum que define las direcciones de las caras de un bloque.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FaceDirections {
    Front,   // Cara delantera
    Back,    // Cara trasera
    Left,    // Cara izquierda
    Right,   // Cara derecha
    Top,     // Cara superior
    Bottom,  // Cara inferior
}

impl FaceDirections {
    /// Crea los datos de una cara del bloque, incluyendo los vértices y sus índices correspondientes.
    pub fn create_face_data(
        &self,
        block: Arc<RwLock<Block>>, // El bloque al que pertenece la cara.
        blocks: &Vec<((i32, i32), BlockVec)>, // Vecindario de bloques.
    ) -> (Vec<BlockVertexData>, Vec<u32>) {
        let indices = self.get_indices(); // Obtener los índices de los vértices para esta cara.
        let mut unique_indices: Vec<u32> = Vec::with_capacity(4); // Índices únicos de vértices.
        let mut vertex_data: Vec<BlockVertexData> = Vec::with_capacity(4); // Datos de los vértices.

        // Crear un mapeo de índices únicos.
        let mut indices_map: Vec<u32> = vec![0; 6];
        for ind in indices.iter() {
            if unique_indices.contains(ind) {
                continue; // Si el índice ya está en la lista, lo omitimos.
            } else {
                unique_indices.push(*ind); // Agregamos el índice único.
            }
        }

        // Mapeamos los índices para construir la cara correctamente.
        for (i, indices_map) in indices_map.iter_mut().enumerate() {
            let index_of = unique_indices
                .iter()
                .enumerate()
                .find_map(|(k, ind)| if *ind == indices[i] { Some(k) } else { None })
                .unwrap();
            *indices_map = index_of as u32;
        }

        // Leemos el bloque para acceder a su tipo y posición.
        let block_read = block.read().unwrap();
        let face_texcoords = block_read.block_type.get_texcoords(*self); // Coordenadas UV de la textura de la cara.
        let normals = self.get_normal_vector(); // Normal de la cara para iluminación.

        // Procesamos los vértices de la cara.
        unique_indices.iter().enumerate().for_each(|(i, index)| {
            let vertex_position = glam::vec3(
                CUBE_VERTEX[*index as usize * 3_usize] + block_read.absolute_position.x,
                CUBE_VERTEX[*index as usize * 3 + 1] + block_read.absolute_position.y,
                CUBE_VERTEX[*index as usize * 3 + 2] + block_read.absolute_position.z,
            );

            vertex_data.push(BlockVertexData {
                position: [
                    CUBE_VERTEX[*index as usize * 3_usize] + block_read.position.x,
                    CUBE_VERTEX[*index as usize * 3 + 1] + block_read.position.y,
                    CUBE_VERTEX[*index as usize * 3 + 2] + block_read.position.z,
                ],
                ao: convert_ao_u8_to_f32(from_vertex_position(&vertex_position, blocks)), // Oclusión ambiental.
                normal: normals.into(), // Vector normal.
                tex_coords: face_texcoords[i], // Coordenadas UV.
            })
        });

        (vertex_data, indices_map)
    }
}

/// Estructura que representa los datos de un vértice para un bloque.
#[repr(C)]
#[derive(Pod, Copy, Clone, Zeroable)]
pub struct BlockVertexData {
    pub position: [f32; 3],  // Posición del vértice.
    pub normal: [f32; 3],    // Vector normal para iluminación.
    pub tex_coords: [f32; 2], // Coordenadas de textura UV.
    pub ao: f32,             // Oclusión ambiental.
}

impl Block {
    /// Crea un nuevo bloque, dada su posición y tipo de bloque.
    pub fn new(position: Vec3, chunk: (i32, i32), block_type: BlockType) -> Block {
        let absolute_position = glam::vec3(
            (chunk.0 * CHUNK_SIZE as i32 + position.x as i32) as f32,
            position.y,
            (chunk.1 * CHUNK_SIZE as i32 + position.z as i32) as f32,
        );
        let collision_box = CollisionBox::from_block_position(
            absolute_position.x,
            absolute_position.y,
            absolute_position.z,
        );
        Block {
            collision_box,
            position,
            block_type,
            absolute_position,
        }
    }

    /// Devuelve las coordenadas de los chunks vecinos, si el bloque está en los bordes de su chunk.
    pub fn get_neighbour_chunks_coords(&self) -> Vec<(i32, i32)> {
        let chunk = self.get_chunk_coords();
        let mut neighbour_chunks = vec![];

        if self.position.x == 15.0 {
            neighbour_chunks.push((chunk.0 + 1, chunk.1));
        }
        if self.position.x == 0.0 {
            neighbour_chunks.push((chunk.0 - 1, chunk.1));
        }
        if self.position.z == 15.0 {
            neighbour_chunks.push((chunk.0, chunk.1 + 1));
        }
        if self.position.z == 0.0 {
            neighbour_chunks.push((chunk.0, chunk.1 - 1));
        }
        neighbour_chunks
    }

    /// Verifica si el bloque está en el borde del chunk.
    pub fn is_on_chunk_border(&self) -> bool {
        self.position.x == 0.0
            || self.position.x == 15.0
            || self.position.z == 0.0
            || self.position.z == 15.0
    }

    /// Obtiene las coordenadas del chunk actual en el que se encuentra el bloque.
    pub fn get_chunk_coords(&self) -> (i32, i32) {
        (
            (f32::floor(self.absolute_position.x / CHUNK_SIZE as f32)) as i32,
            (f32::floor(self.absolute_position.z / CHUNK_SIZE as f32)) as i32,
        )
    }

    /// Define el layout de los datos de vértices para la GPU, necesario para la representación gráfica.
    pub fn get_vertex_data_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BlockVertexData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3, // Formato para la posición.
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3, // Formato para el vector normal.
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2, // Formato para las coordenadas UV.
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32, // Formato para la oclusión ambiental.
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                },
            ],
        }
    }
}

impl FaceDirections {
    /// Devuelve todas las direcciones de las caras de un cubo.
    pub fn all() -> [FaceDirections; 6] {
        [
            FaceDirections::Back,
            FaceDirections::Bottom,
            FaceDirections::Top,
            FaceDirections::Front,
            FaceDirections::Left,
            FaceDirections::Right,
        ]
    }

    /// Devuelve la dirección opuesta de una cara.
    pub fn opposite(&self) -> FaceDirections {
        match self {
            FaceDirections::Back => FaceDirections::Front,
            FaceDirections::Bottom => FaceDirections::Top,
            FaceDirections::Top => FaceDirections::Bottom,
            FaceDirections::Front => FaceDirections::Back,
            FaceDirections::Left => FaceDirections::Right,
            FaceDirections::Right => FaceDirections::Left,
        }
    }

    /// Devuelve el vector normal asociado a una cara del cubo.
    pub fn get_normal_vector(&self) -> glam::Vec3 {
        match self {
            FaceDirections::Back => glam::vec3(0.0, 0.0, 1.0),
            FaceDirections::Bottom => glam::vec3(0.0, -1.0, 0.0),
            FaceDirections::Top => glam::vec3(0.0, 1.0, 0.0),
            FaceDirections::Front => glam::vec3(0.0, 0.0, -1.0),
            FaceDirections::Left => glam::vec3(-1.0, 0.0, 0.0),
            FaceDirections::Right => glam::vec3(1.0, 0.0, 0.0),
        }
    }

    /// Devuelve los índices de los vértices que forman una cara específica del cubo.
    pub fn get_indices(&self) -> [u32; 6] {
        match self {
            FaceDirections::Back => [7, 6, 5, 7, 5, 4],
            FaceDirections::Front => [0, 1, 2, 0, 2, 3],
            FaceDirections::Left => [4, 5, 1, 4, 1, 0],
            FaceDirections::Right => [3, 2, 6, 3, 6, 7],
            FaceDirections::Top => [1, 5, 6, 1, 6, 2],
            FaceDirections::Bottom => [4, 0, 3, 4, 3, 7],
        }
    }
}
