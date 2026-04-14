use super::*;
use image::RgbaImage;
use state::get_wgpu_state;
use wgpu::*;
use winit::dpi::{PhysicalSize, Size};

pub struct TextureAtlas {
    pub(crate) texture: Texture,
    pub(crate) atlas_size: PhysicalSize<usize>,
    pub(crate) images: Vec<GenericRect<usize>>,
}

impl TextureAtlas {
    pub fn new(device: &Device, atlas_size: PhysicalSize<usize>) -> Self {
        Self {
            texture: device.create_texture(&TextureDescriptor {
                size: extent_3d!(atlas_size.width, atlas_size.height),
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                usage: TextureUsages::COPY_SRC
                    | TextureUsages::COPY_DST
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
                label: None,
                mip_level_count: 1,
                sample_count: 1,
            }),
            atlas_size,
            images: Vec::new(),
        }
    }
    pub fn add_image(&mut self, img: RgbaImage) -> Option<RawRect> {
        let (iw, ih) = img.dimensions();
        let pos = self.try_find_empty_space(iw as _, ih as _)?;
        self.images.push(GenericRect {
            x: pos.x,
            y: pos.y,
            w: iw as _,
            h: ih as _,
        });
        self.write_img(img, pos);
        let size = self.texture.size();
        Some(RawRect {
            x: pos.x as f32 / size.width as f32,
            y: pos.y as f32 / size.height as f32,
            w: iw as f32 / size.width as f32,
            h: ih as f32 / size.height as f32,
        })
    }
    pub fn try_find_empty_space(
        &self,
        required_width: usize,
        required_height: usize,
    ) -> Option<GenericVec2<usize>> {
        let last_img = self.images.last().unwrap_or(&GenericRect::<usize>::EMPTY);
        let end = last_img.end();
        if end.x >= self.atlas_size.width || end.y >= self.atlas_size.height {
            return None;
        }
        Some(end)
    }
    fn write_img(&mut self, img: RgbaImage, origin: GenericVec2<usize>) {
        let state = get_wgpu_state();
        let img_texture = state.device.create_texture_with_data(
            &state.queue,
            &TextureDescriptor {
                label: None,
                size: extent_3d!(img.width(), img.height()),
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                usage: TextureUsages::COPY_SRC,
                view_formats: &[],
            },
            util::TextureDataOrder::LayerMajor,
            &img,
        );
        let view = self.texture.create_view(&TextureViewDescriptor::default());
        command!(|encoder| encoder.copy_texture_to_texture(
            TexelCopyTextureInfo {
                texture: &img_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All
            },
            TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d {
                    x: origin.x as _,
                    y: origin.y as _,
                    z: 0
                },
                aspect: TextureAspect::All
            },
            img_texture.size()
        ));
    }
    fn ix(&self, pos: GenericVec2<usize>) -> usize {
        self.atlas_size.width * pos.y + pos.x
    }
    fn pos(&self, ix: usize) -> GenericVec2<usize> {
        GenericVec2::new(ix / self.atlas_size.width, ix % self.atlas_size.width)
    }
}
