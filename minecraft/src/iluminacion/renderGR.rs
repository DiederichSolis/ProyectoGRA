use std::cell::RefCell;
use wgpu::{CommandEncoder, TextureView};
use crate::state::State;
use super::{
    highlight_selected::HighlightSelectedPipeline, main::MainPipeline,
    translucent::TranslucentPipeline, ui::UIPipeline, Pipeline,
};


pub struct PipelineManager {
    /// Pipeline principal que renderiza los objetos sólidos.
    pub main_pipeline: Option<RefCell<MainPipeline>>,
    /// Pipeline que renderiza objetos translúcidos como agua o vidrio.
    pub translucent_pipeline: Option<RefCell<TranslucentPipeline>>,
    /// Pipeline que resalta objetos seleccionados, proporcionando un efecto visual.
    pub highlight_selected_pipeline: Option<RefCell<HighlightSelectedPipeline>>,
    /// Pipeline que maneja la renderización de la interfaz de usuario.
    pub ui_pipeline: Option<RefCell<UIPipeline>>,
}

impl PipelineManager {
    /// Renderiza el contenido de cada pipeline. 
    /// Actualmente está incompleta, por eso usa `todo!()` como un marcador para implementar.
    ///
    /// # Parámetros
    /// - `_encoder`: Un `CommandEncoder` de wgpu que se utiliza para registrar los comandos de renderizado.
    /// - `_view`: Un `TextureView` que define el objetivo de renderizado.
    /// - `_main_pipeline`: El pipeline principal responsable de renderizar objetos sólidos.
    pub fn render(
        &self,
        _encoder: &mut CommandEncoder,
        _view: &TextureView,
        _main_pipeline: &MainPipeline,
    ) {
        todo!();
    }

    /// Inicializa el `PipelineManager` creando e instanciando todos los pipelines 
    /// (principal, translúcido, resaltado y UI).
    ///
    /// # Parámetros
    /// - `state`: Referencia al estado de la aplicación, que contiene la configuración global necesaria.
    ///
    /// # Retorno
    /// Retorna una instancia completamente inicializada de `PipelineManager`.
    pub fn init(state: &State) -> PipelineManager {
        let mut pipeline = PipelineManager {
            highlight_selected_pipeline: None,
            main_pipeline: None,
            translucent_pipeline: None,
            ui_pipeline: None,
        };
        // Inicializa los pipelines con el estado y asigna a los campos correspondientes.
        pipeline.main_pipeline = Some(RefCell::new(MainPipeline::init(state, &pipeline)));
        pipeline.translucent_pipeline =
            Some(RefCell::new(TranslucentPipeline::init(state, &pipeline)));
        pipeline.highlight_selected_pipeline = Some(RefCell::new(HighlightSelectedPipeline::init(
            state, &pipeline,
        )));
        pipeline.ui_pipeline = Some(RefCell::new(UIPipeline::init(state, &pipeline)));
        pipeline
    }

    /// Actualiza todos los pipelines en base al estado actual de la aplicación.
    ///
    /// # Parámetros
    /// - `state`: Estado de la aplicación con la información actualizada para la renderización.
    ///
    /// # Retorno
    /// Retorna un `Result` con `Ok` si todo se actualizó correctamente, o un error en caso contrario.
    pub fn update(&self, state: &State) -> Result<(), Box<dyn std::error::Error>> {
        // Actualiza el pipeline principal.
        self.main_pipeline
            .as_ref()
            .unwrap()
            .borrow_mut()
            .update(self, state)?;
        // Actualiza el pipeline de objetos translúcidos.
        self.translucent_pipeline
            .as_ref()
            .unwrap()
            .borrow_mut()
            .update(self, state)?;
        // Actualiza el pipeline de resaltado de objetos seleccionados.
        self.highlight_selected_pipeline
            .as_ref()
            .unwrap()
            .borrow_mut()
            .update(self, state)?;
        // Actualiza el pipeline de UI.
        self.ui_pipeline
            .as_ref()
            .unwrap()
            .borrow_mut()
            .update(self, state)?;

        Ok(())
    }
}
