//
// ASCII renderer
//

use std::num::NonZeroU32;

use bytemuck::cast_slice;
use bytemuck_derive::{Pod, Zeroable};
use thiserror::Error;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, BufferBindingType, BufferUsage,
    Color, ColorTargetState, ColorWrite, CommandEncoderDescriptor, Device, DeviceDescriptor,
    Extent3d, Features, FragmentState, FrontFace, ImageCopyTexture, ImageDataLayout, Instance,
    Limits, LoadOp, MultisampleState, Operations, Origin3d, PipelineLayoutDescriptor, PolygonMode,
    PowerPreference, PresentMode, PrimitiveState, PrimitiveTopology, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, RequestDeviceError, ShaderFlags, ShaderModuleDescriptor, ShaderSource,
    ShaderStage, Surface, SwapChain, SwapChainDescriptor, SwapChainError, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsage, TextureViewDescriptor,
    TextureViewDimension, VertexState,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::FontData;

//
// Rendering system errors that are passed into Results
//

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Appropriate graphics device was not found")]
    AdapterNotFound,

    #[error(transparent)]
    BadDevice(#[from] RequestDeviceError),

    #[error("Could not find a texture format compatible with the swap chain")]
    BadSwapChainFormat,
}

pub type RenderResult<T> = Result<T, RenderError>;

//
// Rendering state and interface
//

pub struct RenderState {
    surface: Surface,
    device: Device,
    queue: Queue,
    swapchain_desc: SwapChainDescriptor,
    swapchain: SwapChain,
    render_pipeline: RenderPipeline,

    fg_texture: Texture,
    bg_texture: Texture,
    chars_texture: Texture,
    font_texture: Texture,
    texture_bind_group_layout: BindGroupLayout,
    texture_bind_group: BindGroup,

    uniform_bind_group: BindGroup,

    font_char_size: (u32, u32),
    size: (u32, u32),
}

impl RenderState {
    pub async fn new(window: &Window, font: &FontData) -> RenderResult<Self> {
        let inner_size = window.inner_size();

        // An instance represents access to the WGPU API.  Here we decide which
        // back-end to use (Vulkan, DX12, Metal etc), but we let WGPU decide by
        // stating PRIMARY.
        let instance = Instance::new(wgpu::BackendBit::PRIMARY);

        // This can be unsafe since we know the window has a valid window
        // handle, otherwise we wouldn't get here.  The surface is an interface
        // to the OS window that will host the rendering.
        let surface = unsafe { instance.create_surface(window) };

        // The adapter represents a physical graphics/compute device.  We need a
        // device that can handle the surface we will be rendering to.
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .ok_or(RenderError::AdapterNotFound)?;

        // Now we create the device and queue from the adapter.  A device is a
        // logical software construct around the physical device.  It serves as
        // the interface for creating many resources.  A queue is used to
        // deliver commands to the GPU to carry out actions, such as writing to
        // texture buffers.
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("Render device"),
                    features: Features::empty(),
                    limits: Limits::default(),
                },
                None,
            )
            .await?;

        // We create the swap chain descriptor that provides the configuration
        // for creating the swap chain.  However, we keep it around because we
        // need to recreate the swap chain every time the window resizes.
        let swapchain_desc = SwapChainDescriptor {
            usage: TextureUsage::RENDER_ATTACHMENT,
            format: adapter
                .get_swap_chain_preferred_format(&surface)
                .ok_or(RenderError::BadSwapChainFormat)?,
            width: inner_size.width,
            height: inner_size.height,
            present_mode: PresentMode::Fifo,
        };

        // Now we create the swap chain that will target a particular surface.
        let swapchain = device.create_swap_chain(&surface, &swapchain_desc);

        // Set up the textures we will use to render the ASCII graphics.  There are four:
        //
        // * Foreground colours.  Each pixel represents the ink colour of a character on the screen.
        // * Background colours.  Each pixel represents the paper colour of a character on the screen.
        // * ASCII characters.  Each red channel of a pixel represents the ASCII code.
        // * Font texture.  A 16x16 character grid of the font texture.
        let size = (
            inner_size.width / font.width,
            inner_size.height / font.height,
        );
        let fg_texture = Texture::new(&device, size);
        let bg_texture = Texture::new(&device, size);
        let chars_texture = Texture::new(&device, size);
        let mut font_texture = Texture::new(&device, (16 * font.width, 16 * font.height));

        // Load the font data into the font texture
        font_texture.storage.copy_from_slice(font.data.as_slice());
        font_texture.update(&queue);

        // Now we load the shader in that contains both the vertex and fragment
        // shaders as a single WGSL file.
        let shader_src = include_str!("shader.wgsl");
        let shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("ASCII engine shader"),
            flags: ShaderFlags::all(),
            source: ShaderSource::Wgsl(shader_src.into()),
        });

        // Next we will create a bind group.  This describes a set of resources
        // (namely our textures) and how they can be accessed by a shader.
        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });
        let texture_bind_group = Self::create_texture_bind_group(
            &device,
            &texture_bind_group_layout,
            &fg_texture,
            &bg_texture,
            &chars_texture,
            &font_texture,
        );

        // Next is to create the uniform buffer based on RenderInfo struct.
        let uniforms = RenderInfo {
            font_width: font.width,
            font_height: font.height,
            _padding: [0; 2],
        };
        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: cast_slice(&[uniforms]),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        });
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Uniforms bin group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Uniforms bind group"),
            layout: &uniform_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // The render pipeline layout allows us to connect bind groups to the
        // pipeline that we're currenly constructing.
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Given the layout to bind resources, the shaders, we create the
        // pipeline which brings all of those things together.  It also includes
        // the primitive formats (lists, strips etc), culling, front-face
        // determination, drawing mode (wire frame or filled) and some other
        // information related to depth stencils and multisampling.
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "main",
                targets: &[ColorTargetState {
                    format: swapchain_desc.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrite::ALL,
                }],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        Ok(RenderState {
            surface,
            device,
            queue,
            swapchain_desc,
            swapchain,
            render_pipeline,

            fg_texture,
            bg_texture,
            chars_texture,
            font_texture,
            texture_bind_group_layout,
            texture_bind_group,

            uniform_bind_group,

            font_char_size: (font.width, font.height),
            size,
        })
    }

    fn create_texture_bind_group(
        device: &Device,
        texture_bind_group_layout: &BindGroupLayout,
        fore_image: &Texture,
        back_image: &Texture,
        text_image: &Texture,
        font_image: &Texture,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        &fore_image
                            .texture
                            .create_view(&TextureViewDescriptor::default()),
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &back_image
                            .texture
                            .create_view(&TextureViewDescriptor::default()),
                    ),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(
                        &text_image
                            .texture
                            .create_view(&TextureViewDescriptor::default()),
                    ),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(
                        &font_image
                            .texture
                            .create_view(&TextureViewDescriptor::default()),
                    ),
                },
            ],
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.swapchain_desc.width = new_size.width;
        self.swapchain_desc.height = new_size.height;
        self.swapchain = self
            .device
            .create_swap_chain(&self.surface, &self.swapchain_desc);

        let chars_size = (
            new_size.width / self.font_char_size.0,
            new_size.height / self.font_char_size.1,
        );

        if chars_size != self.size {
            self.size = chars_size;
            self.fg_texture = Texture::new(&self.device, self.size);
            self.bg_texture = Texture::new(&self.device, self.size);
            self.chars_texture = Texture::new(&self.device, self.size);

            self.texture_bind_group = Self::create_texture_bind_group(
                &self.device,
                &self.texture_bind_group_layout,
                &self.fg_texture,
                &self.bg_texture,
                &self.chars_texture,
                &self.font_texture,
            );
        }
    }

    pub fn render(&mut self) -> Result<(), SwapChainError> {
        // Update the textures
        self.fg_texture.update(&self.queue);
        self.bg_texture.update(&self.queue);
        self.chars_texture.update(&self.queue);

        // First, we fetch the current frame from the swap chain that we will
        // render to.  The frame will have the view that covers the whole
        // window.  We will use this later for the render pass.
        let frame = self.swapchain.get_current_frame()?.output;

        // Now we construct an encoder that acts like a factory for commands to
        // be sent to the device.
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        {
            // A render pass describes the attachments that will be referenced during rendering.
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Main render pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            render_pass.draw(0..4, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    pub fn images(&mut self) -> (&mut Vec<u32>, &mut Vec<u32>, &mut Vec<u32>) {
        (
            &mut self.fg_texture.storage,
            &mut self.bg_texture.storage,
            &mut self.chars_texture.storage,
        )
    }

    pub fn chars_size(&self) -> (u32, u32) {
        self.size
    }
}

//
// Texture management
//

pub(crate) struct Texture {
    pub(crate) size: (u32, u32),
    pub(crate) storage: Vec<u32>,
    texture: wgpu::Texture,
}

impl Texture {
    fn new(device: &Device, size: (u32, u32)) -> Self {
        let vec_size = (size.0 * size.1) as usize;
        let storage = vec![0; vec_size];

        let texture_size = Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        });

        Texture {
            size,
            texture,
            storage,
        }
    }

    fn update(&mut self, queue: &Queue) {
        let (width, height) = self.size;
        queue.write_texture(
            ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            cast_slice(&self.storage),
            ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * width),
                rows_per_image: NonZeroU32::new(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct RenderInfo {
    font_width: u32,  // Width of the font characters
    font_height: u32, // Height of the font characters
    _padding: [u32; 2],
}
