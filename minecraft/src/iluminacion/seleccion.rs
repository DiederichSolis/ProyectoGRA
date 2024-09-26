
use crate::{blocks::block::FaceDirections, material::Texture, player::Player, state::State};

use super::{pipeline_manager::PipelineManager, Pipeline};


/// El pipeline que se encarga de resaltar el bloque seleccionado.
/// Esta estructura contiene el pipeline de renderizado y los buffers
/// de vértices e índices necesarios para dibujar el bloque seleccionado.
pub struct HighlightSelectedPipeline {
    /// El pipeline de renderizado de WGPU.
    pub pipeline: wgpu::RenderPipeline,
    /// Buffer que almacena los vértices del bloque seleccionado.
    pub selected_block_vertex_buffer: wgpu::Buffer,
    /// Buffer que almacena los índices del bloque seleccionado.
    pub selected_block_index_buffer: wgpu::Buffer,
    /// Cantidad de índices usados para dibujar el bloque seleccionado.
    pub indices: u32,
}

impl Pipeline for HighlightSelectedPipeline {
    /// Renderiza el bloque seleccionado en la pantalla.
    ///
    /// # Parámetros
    /// - `state`: Estado general del juego que contiene los datos necesarios para el renderizado.
    /// - `encoder`: Codificador de comandos de WGPU.
    /// - `view`: Vista de la textura a la que se dibuja.
    /// - `_player`: Referencia al jugador (no usado en este método).
    /// - `_chunks`: Referencia a los chunks del mundo (no usado en este método).
    fn render(
        &self,
        state: &State,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        _player: &std::sync::RwLockReadGuard<'_, Player>,
        _chunks: &Vec<std::sync::RwLockReadGuard<'_, crate::chunk::Chunk>>,
    ) {
        // Obtiene una referencia al pipeline principal.
        let main_pipeline_ref = state
            .pipeline_manager
            .main_pipeline
            .as_ref()
            .unwrap()
            .borrow();

        // Inicia el proceso de renderizado.
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Configura el pipeline, los buffers y realiza la llamada de dibujo.
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &main_pipeline_ref.bind_group_0, &[]);
        rpass.set_vertex_buffer(0, self.selected_block_vertex_buffer.slice(..));
        rpass.set_index_buffer(
            self.selected_block_index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        rpass.draw_indexed(0..self.indices, 0, 0..1);
    }

    /// Actualiza los datos del pipeline, ajustando los buffers del bloque seleccionado.
    ///
    /// # Parámetros
    /// - `_pipeline_manager`: Referencia al administrador de pipelines (no usado en este método).
    /// - `state`: Estado general del juego.
    ///
    /// # Devuelve
    /// `Ok` si la operación fue exitosa o un error en caso contrario.
    fn update(
        &mut self,
        _pipeline_manager: &PipelineManager,
        state: &State,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let player = state.player.read().unwrap();
        
        // Si el jugador está apuntando a un bloque, calcula las posiciones del bloque seleccionado.
        if let Some(block_ptr) = player.facing_block.as_ref() {
            let mut face_data = FaceDirections::all()
                .iter()
                .find(|f| **f == player.facing_face.unwrap())
                .unwrap()
                .create_face_data(block_ptr.clone(), &vec![]);

            let block = block_ptr.read().unwrap();
            let block_positions = face_data
                .0
                .iter_mut()
                .map(|v| {
                    [
                        v.position[0] + block.absolute_position.x - block.position.x,
                        v.position[1] + block.absolute_position.y - block.position.y,
                        v.position[2] + block.absolute_position.z - block.position.z,
                    ]
                })
                .collect::<Vec<_>>();

            // Actualiza los buffers de vértices e índices.
            state.queue.write_buffer(
                &self.selected_block_vertex_buffer,
                0,
                bytemuck::cast_slice(&block_positions),
            );
            state.queue.write_buffer(
                &self.selected_block_index_buffer,
                0,
                bytemuck::cast_slice(&face_data.1),
            );
            self.indices = 6;
        } else {
            // Si no hay bloque seleccionado, vacía los índices.
            self.indices = 0;
        }
        Ok(())
    }

    /// Inicializa el pipeline para resaltar el bloque seleccionado.
    ///
    /// # Parámetros
    /// - `state`: Estado general del juego.
    /// - `pipeline_manager`: Administrador de pipelines que gestiona los pipelines principales.
    ///
    /// # Devuelve
    /// Una nueva instancia de `HighlightSelectedPipeline`.
    fn init(state: &State, pipeline_manager: &PipelineManager) -> Self {
        // Obtiene el formato de la swapchain y el shader para el pipeline.
        let swapchain_capabilities = state.surface.get_capabilities(&state.adapter);
        let swapchain_format = swapchain_capabilities.formats[0];
        let shader_source = include_str!("../shaders/highlight.wgsl");

        let shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });

        // Crea los buffers de vértices e índices para el bloque seleccionado.
        let selected_block_vertex_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<[[f32; 3]; 4]>() as u64 * 4,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let selected_block_index_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<u32>() as u64 * 6,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Crea el pipeline de renderizado.
        let pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&pipeline_manager
                        .main_pipeline
                        .as_ref()
                        .unwrap()
                        .borrow()
                        .bind_group_0_layout],
                    push_constant_ranges: &[],
                });
        let render_pipeline =
            state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[Self::get_vertex_data_layout()],
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
                        depth_write_enabled: false,
                        depth_compare: wgpu::CompareFunction::Always,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        Self {
            indices: 6,
            pipeline: render_pipeline,
            selected_block_index_buffer,
            selected_block_vertex_buffer,
        }
    }
}

impl HighlightSelectedPipeline {
    /// Obtiene el formato de los datos de los vértices para este pipeline.
    pub fn get_vertex_data_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            }],
        }
    }
}
