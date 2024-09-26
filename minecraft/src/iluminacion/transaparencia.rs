use std::sync::RwLockReadGuard;
use super::pipeline_manager::PipelineManager;
use super::Pipeline;
use crate::blocks::block::Block;
use crate::chunk::Chunk;
use crate::material::Texture;
use crate::player::Player;
use crate::state::State;

/// Representa la clase `Water` (Agua), que contiene funcionalidades para obtener el diseño de los vértices de agua.
pub struct Water;

impl Water {
    /// Devuelve la disposición de los datos de vértices en un diseño de búfer de vértices.
    /// Utiliza la misma disposición que los bloques (`Block`).
    pub fn get_vertex_data_layout() -> wgpu::VertexBufferLayout<'static> {
        Block::get_vertex_data_layout()
    }
}

/// Representa un pipeline translúcido que maneja el renderizado de objetos translúcidos como el agua.
pub struct TranslucentPipeline {
    pub pipeline: wgpu::RenderPipeline,
}

impl Pipeline for TranslucentPipeline {
    /// Actualiza el pipeline. Este método está vacío porque no se necesita actualización en este caso específico.
    fn update(
        &mut self,
        _pipeline_manager: &PipelineManager,
        _state: &State,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    /// Inicializa el pipeline translúcido para manejar el renderizado del agua.
    /// Aquí se establece el formato de la cadena de intercambio y se cargan los shaders de agua.
    fn init(state: &State, pipeline_manager: &PipelineManager) -> Self {
        let swapchain_capabilities = state.surface.get_capabilities(&state.adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        // Carga el shader de agua desde el archivo WGSL.
        let shader_source = include_str!("../shaders/water_shader.wgsl");

        let shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });

        // Configuración de los layouts del pipeline.
        let pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[
                        &pipeline_manager
                            .main_pipeline
                            .as_ref()
                            .unwrap()
                            .borrow()
                            .bind_group_0_layout,
                        &state.world.chunk_data_layout,
                        &state
                            .player
                            .read()
                            .unwrap()
                            .camera
                            .position_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        // Creación del pipeline de renderizado.
        let render_pipeline =
            state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[Water::get_vertex_data_layout()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: swapchain_format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        cull_mode: Some(wgpu::Face::Front),
                        ..Default::default()
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: Texture::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        // Devuelve el pipeline translúcido creado.
        Self {
            pipeline: render_pipeline,
        }
    }

    /// Renderiza el agua utilizando el pipeline translúcido.
    /// Asocia buffers y grupos de enlaces para el renderizado y dibuja los chunks de agua visibles.
    fn render(
        &self,
        state: &State,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        player: &RwLockReadGuard<'_, Player>,
        chunks: &Vec<RwLockReadGuard<'_, Chunk>>,
    ) {
        let main_pipeline_ref = state
            .pipeline_manager
            .main_pipeline
            .as_ref()
            .unwrap()
            .borrow();
        
        // Comienza el pase de renderizado.
        let mut water_rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &main_pipeline_ref.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Configura el pipeline para el pase de renderizado.
        water_rpass.set_pipeline(&self.pipeline);
        water_rpass.set_bind_group(0, &main_pipeline_ref.bind_group_0, &[]);
        water_rpass.set_bind_group(2, &player.camera.position_bind_group, &[]);

        // Renderiza los chunks de agua visibles.
        for chunk in chunks.iter() {
            if chunk.visible {
                water_rpass.set_bind_group(1, &chunk.chunk_bind_group, &[]);
                water_rpass.set_vertex_buffer(
                    0,
                    chunk
                        .chunk_water_vertex_buffer
                        .as_ref()
                        .expect("Vertex buffer no iniciado")
                        .slice(..),
                );
                water_rpass.set_index_buffer(
                    chunk
                        .chunk_water_index_buffer
                        .as_ref()
                        .expect("Index buffer no iniciado")
                        .slice(..),
                    wgpu::IndexFormat::Uint32,
                );
                water_rpass.draw_indexed(0..chunk.water_indices, 0, 0..1);
            }
        }
    }
}

impl TranslucentPipeline {}
