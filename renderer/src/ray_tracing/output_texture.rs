pub struct OutputTexture {
    texture_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
}

impl OutputTexture {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

    pub(super) fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        extra_usages: wgpu::TextureUsages,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let texture_view = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Ray Tracing Output Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: Self::FORMAT,
                usage: wgpu::TextureUsages::STORAGE_BINDING | extra_usages,
                view_formats: &[],
            })
            .create_view(&Default::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ray Tracing Output Texture Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            }],
        });

        Self {
            texture_view,
            bind_group,
        }
    }

    pub fn texture_view(&self) -> &wgpu::TextureView {
        &self.texture_view
    }

    pub fn size(&self) -> wgpu::Extent3d {
        self.texture_view.texture().size()
    }

    pub(super) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub(super) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ray Tracing Output Texture Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: Self::FORMAT,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }],
        })
    }
}
