use std::{error::Error, sync::RwLockReadGuard};
use self::pipeline_manager::PipelineManager;
use crate::{chunk::Chunk, player::Player, state::State};


/// Cada implementación de `Pipeline` debe proporcionar tres funciones clave:
/// 1. `init`: Inicializa el pipeline, configurándolo con los recursos necesarios.
/// 2. `update`: Actualiza el estado interno del pipeline basado en el estado actual del juego.
/// 3. `render`: Renderiza los datos procesados, utilizando el encoder de comandos de renderizado y la vista de textura.

pub trait Pipeline {
    /// Inicializa el pipeline con el estado de la aplicación y una referencia al `PipelineManager` se encutra en renderGR
    /// .
    ///
    /// # Parámetros
    /// - `state`: El estado de la aplicación, que contiene todos los recursos y configuraciones globales.
    /// - `pipeline_manager`: El gestor de pipelines que coordina todos los pipelines utilizados.
    ///
    /// # Retorno
    /// Retorna una instancia de la implementación del pipeline.
    fn init(state: &State, pipeline_manager: &PipelineManager) -> Self;

    /// Actualiza el pipeline en base al estado actual del juego y otros parámetros.
    ///
    /// # Parámetros
    /// - `pipeline_manager`: El gestor de pipelines, para acceder a otros pipelines si es necesario.
    /// - `state`: El estado de la aplicación, que proporciona información actualizada sobre el juego.
    ///
    /// # Retorno
    /// Retorna un `Result` que es `Ok(())` si la actualización fue exitosa, o un error si algo falló.
    fn update(
        &mut self,
        pipeline_manager: &PipelineManager,
        state: &State,
    ) -> Result<(), Box<dyn Error>>;

    /// Renderiza el contenido del pipeline, utilizando un encoder de comandos y una vista de textura.
    ///
    /// # Parámetros
    /// - `state`: El estado actual de la aplicación.
    /// - `encoder`: El `CommandEncoder` de wgpu, que registra los comandos de renderizado.
    /// - `view`: Un `TextureView` que especifica el objetivo de renderizado.
    /// - `player`: Una referencia al jugador, utilizada para personalizar el renderizado según la posición del jugador.
    /// - `chunks`: Una referencia a los chunks (bloques del mundo), que se utilizarán para renderizar el entorno.
    fn render(
        &self,
        state: &State,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        player: &RwLockReadGuard<'_, Player>,
        chunks: &Vec<RwLockReadGuard<'_, Chunk>>,
    );
}

/// Módulo que implementa el pipeline de resaltado de objetos seleccionados.
mod highlight_selected;

/// Módulo que implementa el pipeline principal para renderizar los objetos sólidos del juego.
mod main;

/// Módulo que contiene el gestor de pipelines (`PipelineManager`), responsable de coordinar
/// los diferentes pipelines de la aplicación.
pub mod pipeline_manager;

/// Módulo que implementa el pipeline para renderizar objetos translúcidos, como agua o vidrio.
mod translucent;

/// Módulo que implementa el pipeline para renderizar la interfaz de usuario (UI).
mod ui;
 