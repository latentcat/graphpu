use crate::constant::{CAST_TEXTURE_FORMAT, DEPTH_FORMAT, TEXTURE_FORMAT};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: Option<wgpu::Sampler>,
}

impl Texture {

    pub fn create_depth_texture(device: &wgpu::Device, size: &wgpu::Extent3d, label: &str, is_multisample: bool) -> Self {
        let size = wgpu::Extent3d { // 2.
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: if is_multisample { 4 } else { 1 },
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor { // 4.
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual), // 5.
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );

        Self { texture, view, sampler: Some(sampler) }
    }

    pub fn create_texture(device: &wgpu::Device, size: &wgpu::Extent3d, sample_count: u32, is_cast: bool) -> Self {
        let size = wgpu::Extent3d { // 2.
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: if !is_cast { TEXTURE_FORMAT} else { CAST_TEXTURE_FORMAT },
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            label: None,
        };

        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self { texture, view, sampler: None }
    }

}