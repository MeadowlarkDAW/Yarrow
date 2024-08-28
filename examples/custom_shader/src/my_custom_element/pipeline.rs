//! A simple custom pipeline that just renders a triangle with a solid color.

use bytemuck::{Pod, Zeroable};
use yarrow::math::{PhysicalSizeI32, Point, ScaleFactor, Size};
use yarrow::vg::{
    buffer::Buffer,
    color::{PackedSrgb, RGBA8},
    pipeline::{CustomPipeline, CustomPipelinePrimitive, DefaultConstantUniforms},
};

const INITIAL_VERTICES: usize = 8;

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct MyCustomPrimitive {
    pub color: PackedSrgb,
    pub position: [f32; 2],
    pub size: [f32; 2],
    // Note, custom primitives are automatically wrapped inside of an `Rc` pointer
    // so that they are cheap to clone and diff, even if they contain heap-allocated
    // data (like a `Vec` of vertices).
    //
    // However, if your custom primitive is particuarly large or if it contains
    // heap-allocated data, then consider wrapping that data inside of another
    // `Rc<RefCell<T>>` here so that you don't have to clone/reconstruct the entire
    // contents to create a new updated primitive in your elements `render()`
    // method.
}

impl MyCustomPrimitive {
    pub fn new(color: RGBA8, position: Point, size: Size) -> Self {
        Self {
            color: color.into(),
            position: position.into(),
            size: size.into(),
        }
    }
}

pub struct MyCustomPrimitivePipeline {
    pipeline: wgpu::RenderPipeline,

    constants_buffer: wgpu::Buffer,
    constants_bind_group: wgpu::BindGroup,

    vertex_buffer: Buffer<MyCustomPrimitive>,
}

impl MyCustomPrimitivePipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        multisample: wgpu::MultisampleState,
    ) -> Self {
        // A default shader uniform struct containing the scale factor and a scaling vector
        // used to convert from screen space to clip space.
        let (constants_layout, constants_buffer, constants_bind_group) =
            DefaultConstantUniforms::layout_buffer_and_bind_group(device);

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("my custom primitive pipeline layout"),
            push_constant_ranges: &[],
            bind_group_layouts: &[&constants_layout],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("my custom primitive shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("my custom primitive pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<MyCustomPrimitive>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array!(
                        // Color
                        0 => Float32x4,
                        // Position
                        1 => Float32x2,
                        // Size
                        2 => Float32x2,
                    ),
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Cw,
                ..Default::default()
            },
            depth_stencil: None,
            multisample,
            multiview: None,
            cache: None,
        });

        let vertex_buffer = Buffer::new(
            device,
            "my custom primitive vertex buffer",
            INITIAL_VERTICES,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        Self {
            pipeline,
            constants_buffer,
            constants_bind_group,
            vertex_buffer,
        }
    }
}

impl CustomPipeline for MyCustomPrimitivePipeline {
    /// Prepare to render the given list of primitives
    ///
    /// Note, if the screen size, scale factor, and list of primitives have not
    /// changed since the last preparation, then Yarrow will automatically
    /// skip calling this method.
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_size: PhysicalSizeI32,
        scale_factor: ScaleFactor,
        primitives: &[CustomPipelinePrimitive],
    ) -> Result<(), Box<dyn std::error::Error>> {
        DefaultConstantUniforms::prepare_buffer(
            &self.constants_buffer,
            screen_size,
            scale_factor,
            queue,
        );

        let vertices: Vec<MyCustomPrimitive> = primitives
            .iter()
            .map(|p| {
                let mut primitive = p
                    .primitive
                    .downcast_ref::<MyCustomPrimitive>()
                    .copied()
                    .unwrap();

                // Offset the primitive by the requested amount
                primitive.position[0] += p.offset.x;
                primitive.position[1] += p.offset.y;

                primitive
            })
            .collect();

        self.vertex_buffer
            .expand_to_fit_new_size(device, primitives.len());
        self.vertex_buffer.write(queue, 0, &vertices);

        Ok(())
    }

    /// Render a primitive
    ///
    /// The `primitive_index` is the index into the slice of primitives that
    /// was previously passed into `CustomPipeline::prepare`.
    fn render_primitive<'pass>(
        &'pass self,
        primitive_index: usize,
        render_pass: &mut wgpu::RenderPass<'pass>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.constants_bind_group, &[]);

        render_pass.set_vertex_buffer(
            0,
            self.vertex_buffer
                .slice(primitive_index..primitive_index + 1),
        );
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}
