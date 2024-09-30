use crate::player::Camera;
use bytemuck::{Pod, Zeroable};

/// Estructura `Uniforms` utilizada para almacenar matrices de vista y proyección en shaders.
/// 
/// Esta estructura está alineada a C (`#[repr(C)]`), lo que asegura que sea compatible con estructuras
/// de datos de bajo nivel, como las usadas en la GPU. Es utilizada para enviar matrices de vista y proyección 
/// a los shaders en aplicaciones gráficas.
/// 
/// # Campos:
/// - `view`: Una matriz de vista de 4x4 representada como un arreglo de 16 valores `f32`.
/// - `projection`: Una matriz de proyección de 4x4 representada como un arreglo de 16 valores `f32`.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct Uniforms {
    pub view: [f32; 16],
    pub projection: [f32; 16],
}

/// Implementación de `From<&Camera>` para la conversión de una cámara a la estructura `Uniforms`.
/// 
/// # Descripción:
/// Convierte una referencia a una instancia de `Camera` en una estructura `Uniforms`, extrayendo las matrices 
/// de vista y proyección de la cámara y copiándolas en la estructura `Uniforms`.
/// 
/// # Parámetros:
/// - `camera`: Referencia a una instancia de `Camera` de la que se extraen las matrices.
/// 
/// # Retorna:
/// Un objeto de tipo `Uniforms` con la matriz de vista y la matriz de proyección de la cámara.
impl From<&Camera> for Uniforms {
    fn from(camera: &Camera) -> Self {
        Self {
            view: *camera.build_view_matrix().as_ref(),
            projection: *camera.build_projection_matrix().as_ref(),
        }
    }
}
