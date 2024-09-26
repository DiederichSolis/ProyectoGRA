use super::block::{FaceDirections, TexturedBlock};
use crate::world::{RNG_SEED, WATER_HEIGHT_LEVEL};
use rand::{rngs::StdRng, Rng, SeedableRng};

#[derive(Clone, Copy, Debug)]
// Estructura para representar la textura de una cara de un bloque.
// El valor `u32` representa el índice de la textura a utilizar.
pub struct FaceTexture(u32);

#[derive(Clone, Copy, Debug)]
// Estructura que define las configuraciones para un tipo de bloque.
pub struct BlockTypeConfigs {
    pub id: u32,                      // Identificador único del tipo de bloque.
    pub textures: [FaceTexture; 3],   // Arreglo que contiene las texturas para las caras: lateral, superior e inferior.
    pub is_translucent: bool,         // Indica si el bloque es translúcido o no.
}

impl BlockTypeConfigs {
    // Devuelve las configuraciones de un tipo de bloque dado.
    pub fn get(block_type: BlockType) -> BlockTypeConfigs {
        match block_type {
            BlockType::Grass => BlockTypeConfigs {
                id: 0,
                textures: [FaceTexture(6), FaceTexture(7), FaceTexture(8)],
                is_translucent: false,
            },
            BlockType::Dirt => BlockTypeConfigs {
                id: 1,
                textures: [FaceTexture(0), FaceTexture(0), FaceTexture(0)],
                is_translucent: false,
            },
            BlockType::Water => BlockTypeConfigs {
                id: 2,
                textures: [FaceTexture(1), FaceTexture(1), FaceTexture(1)],
                is_translucent: true,
            },
            BlockType::Wood => BlockTypeConfigs {
                id: 3,
                textures: [FaceTexture(4), FaceTexture(5), FaceTexture(5)],
                is_translucent: false,
            },
            BlockType::Leaf => BlockTypeConfigs {
                id: 4,
                textures: [FaceTexture(2), FaceTexture(2), FaceTexture(2)],
                is_translucent: false,
            },
            BlockType::Stone => BlockTypeConfigs {
                id: 5,
                textures: [FaceTexture(3), FaceTexture(3), FaceTexture(3)],
                is_translucent: false,
            },
            BlockType::Sand => BlockTypeConfigs {
                id: 6,
                textures: [FaceTexture(9), FaceTexture(9), FaceTexture(9)],
                is_translucent: false,
            },
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
// Enum que define los distintos tipos de bloques en el juego.
pub enum BlockType {
    Grass,
    Dirt,
    Water,
    Wood,
    Leaf,
    Stone,
    Sand,
}

impl BlockType {
    pub const MAX_ID: u32 = 6;

    // Obtiene la configuración del bloque actual.
    pub fn get_config(&self) -> BlockTypeConfigs {
        BlockTypeConfigs::get(*self)
    }

    // Devuelve el ID correspondiente al tipo de bloque.
    pub fn to_id(&self) -> u32 {
        self.get_config().id
    }

    // Crea un bloque a partir de su ID.
    pub fn from_id(id: u32) -> BlockType {
        match id {
            0 => Self::Grass,
            1 => Self::Dirt,
            2 => Self::Water,
            3 => Self::Wood,
            4 => Self::Leaf,
            5 => Self::Stone,
            6 => Self::Sand,
            _ => panic!("Invalid id"),
        }
    }
}

// Calcula un valor escalar basado en la altura `y` y el umbral `t`.
fn calc_scalar(y: u32, t: Threshold) -> f32 {
    (y as f32 - t[0] as f32) / (t[1] as f32 - t[0] as f32)
}

// Definición de un umbral de altura (límite inferior y superior).
type Threshold = [u32; 2];

// Constantes que definen los umbrales de altura para la piedra y la arena.
const STONE_THRESHOLD: Threshold = [15, 24];
const SAND_THRESHOLD: Threshold = [WATER_HEIGHT_LEVEL as u32, WATER_HEIGHT_LEVEL as u32 + 2];

impl BlockType {
    // Determina el tipo de bloque basado en su posición `(x, y, z)`.
    pub fn from_position(x: u32, y: u32, z: u32) -> BlockType {
        // Inicializa un generador de números aleatorios con una semilla específica.
        let mut rng = StdRng::seed_from_u64(RNG_SEED + (y * x * z) as u64);

        if y <= SAND_THRESHOLD[0] {
            BlockType::Sand
        } else if y <= SAND_THRESHOLD[1] {
            // Calcula probabilidades para decidir si el bloque es arena o tierra.
            let r = rng.gen::<f32>();
            let s = calc_scalar(y, SAND_THRESHOLD);
            if r + s > 1.0 {
                BlockType::Dirt
            } else {
                BlockType::Sand
            }
        } else if y < STONE_THRESHOLD[0] {
            BlockType::Dirt
        } else if y <= STONE_THRESHOLD[1] {
            let r = rng.gen::<f32>();
            let s = calc_scalar(y, STONE_THRESHOLD);
            if r + s >= 1.0 {
                BlockType::Stone
            } else {
                BlockType::Dirt
            }
        } else {
            BlockType::Stone
        }
    }
}

// Constantes relacionadas con las texturas de los bloques.
const TEXTURE_SIZE: u32 = 256;
const BLOCK_PER_ROW: u32 = 8;
// Cada bloque tiene un tamaño de 32px.
const BLOCK_OFFSET: u32 = TEXTURE_SIZE / BLOCK_PER_ROW;
const BLOCK_OFFSET_NORMALIZED: f32 = BLOCK_OFFSET as f32 / TEXTURE_SIZE as f32;

// Obtiene las coordenadas base de la textura según la dirección de la cara.
fn get_base_coords(config: &BlockTypeConfigs, face_dir: FaceDirections) -> glam::Vec2 {
    let face_offset = match face_dir {
        FaceDirections::Top => config.textures[1],
        FaceDirections::Bottom => config.textures[2],
        _ => config.textures[0],
    };
    let y_offset = (face_offset.0 / BLOCK_PER_ROW) as f32;
    let x_offset = (face_offset.0 % BLOCK_PER_ROW) as f32;

    let low_bound = y_offset * BLOCK_OFFSET_NORMALIZED + BLOCK_OFFSET_NORMALIZED;
    let left_bound = x_offset * BLOCK_OFFSET_NORMALIZED;
    glam::vec2(left_bound, low_bound)
}

// Obtiene las coordenadas de la textura para una cara específica del bloque.
fn get_tex_coords(config: &BlockTypeConfigs, face_dir: FaceDirections) -> [[f32; 2]; 4] {
    let bc = get_base_coords(config, face_dir);
    [
        [bc.x, bc.y],
        [bc.x, bc.y - BLOCK_OFFSET_NORMALIZED],
        [
            bc.x + BLOCK_OFFSET_NORMALIZED,
            bc.y - BLOCK_OFFSET_NORMALIZED,
        ],
        [bc.x + BLOCK_OFFSET_NORMALIZED, bc.y],
    ]
}

impl TexturedBlock for BlockType {
    // Implementa la interfaz `TexturedBlock` para obtener las coordenadas de textura de un bloque.
    fn get_texcoords(&self, face_dir: FaceDirections) -> [[f32; 2]; 4] {
        get_tex_coords(&self.get_config(), face_dir)
    }
}
