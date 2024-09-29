use crate::persistence::{Loadable, Saveable};
use crate::player::Player;
use crate::utils::math_utils::Plane;
use crate::world::{ChunkMap, RNG_SEED, WATER_HEIGHT_LEVEL};
use crate::{
    blocks::{
        block::{Block, BlockVertexData, FaceDirections},
        block_type::BlockType,
    },
    structures::Structure,
    world::{NoiseData, CHUNK_SIZE, MAX_TREES_PER_CHUNK, NOISE_CHUNK_PER_ROW, NOISE_SIZE},
};

use glam::Vec3;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::any::Any;
use std::error::Error;
use std::sync::{Arc, RwLock};
use wgpu::util::DeviceExt;

/// Tipo que representa un vector de bloques que está envuelto en estructuras
/// de sincronización (Arc y RwLock) para manejar concurrencia segura.
pub type BlockVec = Arc<RwLock<Vec<Vec<Option<Arc<RwLock<Block>>>>>>>;

/// Estructura que representa un Chunk, que contiene bloques y otros recursos
/// relacionados a la generación y visualización de estos en un entorno 3D.
#[derive(Debug)]
pub struct Chunk {
    pub x: i32, // Coordenada x del chunk.
    pub y: i32, // Coordenada y del chunk.
    pub blocks: BlockVec, // Vector de bloques del chunk.
    pub indices: u32, // Índices para el renderizado de los bloques.
    pub water_indices: u32, // Índices para el renderizado del agua.
    pub device: Arc<wgpu::Device>, // Dispositivo GPU utilizado para el renderizado.
    pub queue: Arc<wgpu::Queue>, // Cola de comandos GPU.
    pub noise_data: Arc<NoiseData>, // Datos de ruido usados para la generación procedural.
    pub chunk_bind_group: wgpu::BindGroup, // Grupo de bindings del chunk en GPU.
    pub chunk_position_buffer: wgpu::Buffer, // Buffer de posición del chunk.
    pub chunk_index_buffer: Option<wgpu::Buffer>, // Buffer de índices del chunk.
    pub chunk_vertex_buffer: Option<wgpu::Buffer>, // Buffer de vértices del chunk.
    pub chunk_water_vertex_buffer: Option<wgpu::Buffer>, // Buffer de vértices de agua.
    pub chunk_water_index_buffer: Option<wgpu::Buffer>, // Buffer de índices de agua.
    pub outside_blocks: Vec<Arc<RwLock<Block>>>, // Bloques externos al chunk.
    pub visible: bool, // Determina si el chunk es visible.
    pub modified: bool, // Indica si el chunk ha sido modificado y necesita ser guardado.
}

impl Chunk {
    /// Agrega un bloque al chunk en una posición específica.
    /// Si `modify_status` es verdadero, marca el chunk como modificado.
    pub fn add_block(&mut self, block: Arc<RwLock<Block>>, modify_status: bool) {
        let block_borrow = block.read().unwrap();
        let block_position = block_borrow.position;
        std::mem::drop(block_borrow);
        let mut blocks_borrow = self.blocks.write().unwrap();

        // Obtiene la lista de bloques en la posición (x, z).
        let y_blocks = blocks_borrow
            .get_mut(((block_position.x * CHUNK_SIZE as f32) + block_position.z) as usize)
            .expect("No se puede agregar un bloque fuera de los límites");

        // Si la posición y es mayor que el tamaño actual de la lista de bloques, la ajusta.
        if block_position.y as usize >= y_blocks.len() {
            y_blocks.resize(block_position.y as usize + 1, None);
        }

        y_blocks[block_position.y as usize] = Some(block);
        if modify_status {
            self.modified = true;
        }
    }

    /// Elimina un bloque en una posición específica.
    pub fn remove_block(&mut self, block_r_position: &Vec3) {
        let mut blocks_borrow = self.blocks.write().unwrap();
        let y_blocks = blocks_borrow
            .get_mut(((block_r_position.x * CHUNK_SIZE as f32) + block_r_position.z) as usize)
            .expect("No se puede eliminar un bloque fuera de los límites");
        y_blocks[block_r_position.y as usize] = None;
        self.modified = true;
    }

    /// Devuelve el tipo de bloque en una posición dada, si existe.
    pub fn block_type_at(&self, position: &glam::Vec3) -> Option<BlockType> {
        let block = self.get_block_at_relative(position)?;
        let block_type = block.read().unwrap().block_type;
        Some(block_type)
    }

    /// Verifica si existe un bloque en una posición dada.
    pub fn exists_block_at(&self, position: &glam::Vec3) -> bool {
        if let Some(y_blocks) = self
            .blocks
            .read()
            .unwrap()
            .get(((position.x as u32 * CHUNK_SIZE) + position.z as u32) as usize)
        {
            if let Some(block_opt) = y_blocks.get(position.y as usize) {
                if block_opt.is_some() {
                    return true;
                }
            }
        }
        false
    }

    /// Obtiene un bloque en una posición relativa, si existe.
    pub fn get_block_at_relative(&self, position: &glam::Vec3) -> Option<Arc<RwLock<Block>>> {
        if let Some(y_blocks) = self
            .blocks
            .read()
            .unwrap()
            .get(((position.x * CHUNK_SIZE as f32) + position.z) as usize)
        {
            if let Some(block) = y_blocks.get(position.y as usize)? {
                return Some(Arc::clone(block));
            }
        }
        None
    }

    /// Verifica si una posición está fuera de los límites del chunk.
    pub fn is_outside_chunk(position: &glam::Vec3) -> bool {
        position.x < 0.0
            || position.x >= CHUNK_SIZE as f32
            || position.z < 0.0 || position.z >= CHUNK_SIZE as f32
    }

    /// Verifica si una posición está fuera de los límites del mundo (en términos de altura).
    pub fn is_outside_bounds(position: &glam::Vec3) -> bool {
        position.y < 0.0
    }

    /*
    Devuelve una tupla que contiene:
    0: índices de vértices, 1: índices de vértices de agua,
    2: buffer de vértices, 3: buffer de índices,
    4: buffer de vértices de agua, 5: buffer de índices de agua.
    */
    pub fn build_mesh(
        &self,
        other_chunks: ChunkMap,
    ) -> (
        u32,
        u32,
        wgpu::Buffer,
        wgpu::Buffer,
        wgpu::Buffer,
        wgpu::Buffer,
    ) {
        // Generación de buffers y datos para bloques de agua y bloques regulares
        // ...
    }

    /// Devuelve la descripción del grupo de bindings para el chunk en GPU.
    pub fn get_bind_group_layout() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("chunk_bind_group"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        }
    }
}

/// Obtiene el valor de altura para un bloque en un chunk específico, usando datos de ruido.
/// 
/// # Parámetros
/// - `chunk_x`: La coordenada X del chunk.
/// - `chunk_y`: La coordenada Y del chunk.
/// - `x`: La coordenada X dentro del chunk.
/// - `z`: La coordenada Z dentro del chunk.
/// - `noise_data`: Datos de ruido que se utilizan para calcular la altura.
///
/// # Retorna
/// La altura del bloque como un valor `u32`.
pub fn get_height_value(
    chunk_x: i32,
    chunk_y: i32,
    x: u32,
    z: u32,
    noise_data: Arc<NoiseData>,
) -> u32 {
    let mut x = (chunk_x * CHUNK_SIZE as i32) + x as i32 % NOISE_SIZE as i32;
    let mut z = (chunk_y * CHUNK_SIZE as i32) + z as i32 % NOISE_SIZE as i32;

    // Ajusta las coordenadas X si son negativas
    if x < 0 {
        x = NOISE_SIZE as i32 + (x % (NOISE_CHUNK_PER_ROW * CHUNK_SIZE) as i32);
    }
    // Ajusta las coordenadas Z si son negativas
    if z < 0 {
        z = NOISE_SIZE as i32 + (z % (NOISE_CHUNK_PER_ROW * CHUNK_SIZE) as i32);
    }
    // Obtiene el valor de ruido y calcula la altura
    if let Some(v) = noise_data.get((z * (NOISE_SIZE as i32) + x) as usize) {
        let y_top = (v + 1.0) * 0.5;
        (f32::powf(100.0, y_top) - 1.0) as u32
    } else {
        0
    }
}

/// Crea datos de bloques para un chunk específico utilizando datos de ruido.
///
/// # Parámetros
/// - `chunk_x`: La coordenada X del chunk.
/// - `chunk_y`: La coordenada Y del chunk.
/// - `noise_data`: Datos de ruido que se utilizan para generar los bloques.
///
/// # Retorna
/// Un vector de bloques (`BlockVec`) para el chunk.
pub fn create_blocks_data(chunk_x: i32, chunk_y: i32, noise_data: Arc<NoiseData>) -> BlockVec {
    let size = (CHUNK_SIZE * CHUNK_SIZE) as usize;
    let blocks: BlockVec = Arc::new(RwLock::new(vec![
        Vec::with_capacity(
            WATER_HEIGHT_LEVEL as usize
        );
        size
    ]));

    // Genera los bloques para el chunk
    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            let y_top = Chunk::get_height_value(chunk_x, chunk_y, x, z, noise_data.clone());

            let curr = &mut blocks.write().unwrap()[((x * CHUNK_SIZE) + z) as usize];

            // Crea los bloques hasta la altura y_top
            for y in 0..=y_top {
                let block_type = match BlockType::from_position(x, y, z) {
                    BlockType::Dirt if y == y_top => BlockType::Grass,
                    b => b,
                };

                let block = Arc::new(RwLock::new(Block::new(
                    glam::vec3(x as f32, y as f32, z as f32),
                    (chunk_x, chunk_y),
                    block_type,
                )));

                curr.push(Some(block.clone()));
            }
            // Rellena con bloques de agua vacíos
            for y in curr.len()..=(WATER_HEIGHT_LEVEL as usize) {
                if curr.get(y).is_none() {
                    let block = Arc::new(RwLock::new(Block::new(
                        glam::vec3(x as f32, y as f32, z as f32),
                        (chunk_x, chunk_y),
                        BlockType::Water,
                    )));
                    curr.push(Some(block));
                }
            }
        }
    }

    blocks
}

/// Coloca árboles en el chunk utilizando un generador de números aleatorios.
///
/// # Nota
/// - Se debe usar ruido blanco y verificar que el árbol no se coloque sobre agua.
pub fn place_trees(&mut self) {
    let mut rng = StdRng::seed_from_u64((self.x * 10 * self.y) as u64 + RNG_SEED);
    let number_of_trees = rng.gen::<f32>();
    let mut number_of_trees = f32::floor(number_of_trees * MAX_TREES_PER_CHUNK as f32) as u32;

    // Intenta colocar hasta 100 árboles
    for _ in 0..100 {
        if number_of_trees == 0 {
            break;
        }
        let mut tree_blocks = vec![];
        {
            let x = f32::floor(rng.gen::<f32>() * CHUNK_SIZE as f32) as usize;
            let z = f32::floor(rng.gen::<f32>() * CHUNK_SIZE as f32) as usize;

            let blocks_read = self.blocks.read().unwrap();
            let block_column = blocks_read
                .get((x * CHUNK_SIZE as usize) + z)
                .expect("TODO: fix this case");
            let highest_block = block_column
                .last()
                .expect("TODO: Fix this case -h")
                .as_ref()
                .unwrap()
                .read()
                .unwrap();
            // Verifica que el bloque más alto no sea agua ni hoja
            if highest_block.block_type == BlockType::Water
                || highest_block.block_type == BlockType::Leaf
            {
                continue;
            }
            let highest_block_position = highest_block.absolute_position;

            tree_blocks.append(&mut crate::structures::Tree::get_blocks(
                highest_block_position,
            ));
            number_of_trees -= 1;
        }
        // Agrega los bloques del árbol al chunk o a la lista de bloques externos
        for block in tree_blocks.iter() {
            let block_brw = block.read().unwrap();
            let block_chunk = block_brw.get_chunk_coords();
            if block_chunk == (self.x, self.y) {
                self.add_block(block.clone(), false);
            } else {
                self.outside_blocks.push(block.clone())
            }
        }
    }
}

/// Verifica si el chunk es visible desde la posición del jugador.
///
/// # Parámetros
/// - `player`: Una referencia al jugador que se está utilizando para calcular la visibilidad.
///
/// # Retorna
/// `true` si el chunk es visible, de lo contrario `false`.
pub fn is_visible(&self, player: Arc<RwLock<Player>>) -> bool {
    let player = player.read().unwrap();
    let forward = player.camera.get_forward_dir();
    let right = player.camera.get_right_dir();
    let halfvside = player.camera.zfar / f32::tan(player.camera.fovy / 2.0);
    let halfhside = halfvside * player.camera.aspect_ratio;
    let front_mult_far = player.camera.zfar * forward;

    let chunk_points = [
        (
            (self.x as f32) * CHUNK_SIZE as f32,
            (self.y as f32) * CHUNK_SIZE as f32,
        ),
        (
            (self.x as f32 + 1.0) * CHUNK_SIZE as f32,
            (self.y as f32) * CHUNK_SIZE as f32,
        ),
        (
            (self.x as f32) * CHUNK_SIZE as f32,
            (self.y as f32 + 1.0) * CHUNK_SIZE as f32,
        ),
        (
            (self.x as f32 + 1.0) * CHUNK_SIZE as f32,
            (self.y as f32 + 1.0) * CHUNK_SIZE as f32,
        ),
    ];

    // Define los planos de la cámara
    let near_plane = Plane {
        point: player.camera.eye + player.camera.znear * forward,
        normal: forward,
    };
    let far_plane = Plane {
        point: player.camera.eye + front_mult_far,
        normal: -forward,
    };
    let right_plane = Plane {
        point: player.camera.eye,
        normal: glam::vec3(0.0, 1.0, 0.0)
            .cross(player.camera.eye - (front_mult_far + right * halfhside))
            .normalize(),
    };
    let left_plane = Plane {
        point: player.camera.eye,
        normal: (player.camera.eye - (front_mult_far - right * halfhside))
            .cross(glam::vec3(0.0, 1.0, 0.0))
            .normalize(),
    };

    // Retorna verdadero si al menos un borde de un chunk es visible dentro del frustum
    [far_plane, near_plane, left_plane, right_plane]
        .iter()
        .all(|p| {
            chunk_points.iter().any(|chunk_point| {
                p.signed_plane_dist(glam::vec3(chunk_point.0, 0.0, chunk_point.1)) >= 0.0
            })
        })
}

/// Crea un nuevo chunk con los datos necesarios para su funcionamiento.
///
/// # Parámetros
/// - `x`: La coordenada X del chunk.
/// - `y`: La coordenada Y del chunk.
/// - `noise_data`: Datos de ruido utilizados para generar la topografía del chunk.
/// - `player`: Referencia al jugador, si se necesita.
///
/// # Retorna
/// Un nuevo objeto `Chunk`.
pub fn new(x: i32, y: i32, noise_data: Arc<NoiseData>, player: Arc<RwLock<Player>>) -> Self {
    let mut chunk = Chunk {
        blocks: Arc::new(RwLock::new(vec![])),
        x,
        y,
        outside_blocks: vec![],
        player: Some(player),
    };

    let blocks_data = create_blocks_data(x, y, noise_data);
    chunk.blocks = blocks_data;
    chunk.place_trees();

    chunk
}
