

pub const FONT_SIZE_BODY: f32 = 11.0;
pub const FONT_SIZE_TITLE: f32 = 13.0;
pub const FONT_SIZE_HEADING: f32 = 18.0;

pub const MULTISAMPLE_STATE: wgpu::MultisampleState = wgpu::MultisampleState {
    count: 4,
    mask: !0,
    alpha_to_coverage_enabled: false
};

pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;