use state::get_wgpu_state;

use super::*;

pub struct FontRenderer {
    pub fonts_atlas: Texture,
    pub fonts_atlas_id: u32,
}
impl FontRenderer {
    pub fn new(device: &Device, rect_texture: &mut RectangleTexturePipeline) -> Self {
        let mut lines = LinesPipeline::new();
        let face = ttf_parser::Face::parse(include_bytes!("../../JB Mono.ttf"), 0).unwrap();
        for chr in 40..127u8 {
            let mut builder = Builder(vec![]);
            let code = chr as char;
            let id = face.glyph_index(code).unwrap();
            face.tables().glyf.unwrap().outline(id, &mut builder);
            let mut pts = vec![];
            for pt in builder.0 {
                pts.push(pt)
            }
            lines.append_lines(&pts.iter().skip(1).enumerate().map(|(i,pt)| {
                let pt2 = pts[i];
                lines::Line::new(
                    vec2f(200. + pt.0 / 10., 200. + pt.1 / 10.),
                    vec2f(200. + pt2.0 / 10., 200. + pt2.1 / 10.),
                )
            }).collect::<Vec<_>>());
        }

        let chars = vec![0u8];
        let char_width = 1;
        let char_height = 1;
        let fonts_atlas = device.create_texture(&TextureDescriptor {
            label: Some("Font atlas"),
            size: extent_3d!(chars.len() * char_width, char_height),
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            usage: TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let atlas_view = &fonts_atlas.create_view(&TextureViewDescriptor::default());
        command!(|encoder: &mut CommandEncoder| {
            let mut pass = render_pass!(encoder, atlas_view, Color::BLACK);
            pass.set_pipeline(&lines.render);
            pass.set_vertex_buffer(0, lines.instances.0.slice(..));
            pass.draw(0..lines.instances.1, 0..lines.instances.1);
        });
        
        Self {
            fonts_atlas,
            fonts_atlas_id: 0,
        }
    }
    pub fn add_text(&mut self, pos: Vec2f, text: &str) {
        if self.fonts_atlas_id == 0 {
            let raw_texture = State::read_texture(&self.fonts_atlas, extent_3d!(self.fonts_atlas.width(), self.fonts_atlas.height()));
            self.fonts_atlas_id = get_state().pipelines.rect_texture_pipeline.add_texture(raw_texture);
            dbg!(self.fonts_atlas_id);
        }
        get_state().pipelines.rect_texture_pipeline.push_rect(RawRectTexture { rect: RawRect { x: pos.x, y: pos.y, w: 10., h: 10. }, texture_id: self.fonts_atlas_id })
        
    }
    
    pub(crate) fn load_fonts() -> Vec<Vec<u8>> {
        let state = get_wgpu_state();

        let mut lines = LinesPipeline::new();
        let face = ttf_parser::Face::parse(include_bytes!("../../JB Mono.ttf"), 0).unwrap();
        for chr in 39..127u8 {
            let mut builder = Builder(vec![]);
            let code = chr as char;
            let id = face.glyph_index(code).unwrap();
            face.tables().glyf.unwrap().outline(id, &mut builder);
            let mut pts = vec![];
            for pt in builder.0 {
                pts.push(pt)
            }
            lines.append_lines(&pts.iter().skip(1).enumerate().map(|(i,pt)| {
                let pt2 = pts[i];
                lines::Line::new(
                    vec2f(200. + pt.0 / 10., 200. + pt.1 / 10.),
                    vec2f(200. + pt2.0 / 10., 200. + pt2.1 / 10.),
                )
            }).collect::<Vec<_>>());
        }

        let char_width = 64;
        let char_height = 64;
        let fonts_atlas = state.device.create_texture(&TextureDescriptor {
            label: Some("Font atlas"),
            size: extent_3d!((127-39) * char_width, char_height),
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            usage: TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let atlas_view = &fonts_atlas.create_view(&TextureViewDescriptor::default());
        command!(|encoder: &mut CommandEncoder| {
            let mut pass = render_pass!(encoder, atlas_view, Color {r:0.5,g:0.,b:0.,a:0.});
            pass.set_pipeline(&lines.render);
            pass.set_vertex_buffer(0, lines.instances.0.slice(..));
            pass.draw(0..lines.instances.1, 0..lines.instances.1);
        });
        let raw = State::read_full_texture(&fonts_atlas);
        raw.chunks_exact(char_width*char_height*4).map(|ch| ch.to_vec()).collect()
    }
}

struct Builder(Vec<(f32, f32)>);
impl ttf_parser::OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.push((x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.push((x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        // Here you might want to approximate the quadratic curve with more points
        self.0.push((x1, y1));
        self.0.push((x, y));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        // Similarly, for cubic curves, you might want to approximate or just store control points
        self.0.push((x1, y1));
        self.0.push((x2, y2));
        self.0.push((x, y));
    }

    fn close(&mut self) {
        // Optionally handle closing the path if needed
    }
}



// impl FontRenderer {
//     pub fn new(device: &Device, lines: &mut LinesPipeline) -> Self {
//         let mut lines = LinesPipeline::new();
//         let face = ttf_parser::Face::parse(include_bytes!("../../JB Mono.ttf"), 0).unwrap();
//         for chr in 40..127u8 {
//             let mut builder = Builder(vec![]);
//             let code = chr as char;
//             let id = face.glyph_index(code).unwrap();
//             face.tables().glyf.unwrap().outline(id, &mut builder);
//             let mut pts = vec![];
//             for pt in builder.0 {
//                 pts.push(pt)
//             }
//             lines.append_lines(&pts.iter().skip(1).enumerate().map(|(i,pt)| {
//                 let pt2 = pts[i];
//                 lines::Line::new(
//                     vec2f(200. + pt.0 / 10., 200. + pt.1 / 10.),
//                     vec2f(200. + pt2.0 / 10., 200. + pt2.1 / 10.),
//                 )
//             }).collect::<Vec<_>>());
//         }

//         let chars = vec![0u8];
//         let char_width = 1;
//         let char_height = 1;
//         let fonts_atlas = device.create_texture(&TextureDescriptor {
//             label: Some("Font atlas"),
//             size: extent_3d!(chars.len() * char_width, char_height),
//             mip_level_count: 1,
//             sample_count: 1,
//             dimension: TextureDimension::D2,
//             format: TextureFormat::Bgra8UnormSrgb,
//             usage: TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,
//             view_formats: &[],
//         });
//         let atlas_view = &fonts_atlas.create_view(&TextureViewDescriptor::default());
//         command!(|encoder: &mut CommandEncoder| {
//             let mut pass = render_pass!(encoder, atlas_view, Color::BLACK);
//             pass.set_pipeline(&lines.render);
//             pass.set_vertex_buffer(0, lines.instances.0.slice(..));
//             pass.draw(0..lines.instances.1, 0..lines.instances.1);
//         });
        
//         Self {
//             fonts_atlas,
//             fonts_atlas_id: 0,
//         }
//     }
//     pub fn add_text(&mut self, pos: Vec2f, text: &str) {
//         if self.fonts_atlas_id == 0 {
//             let raw_texture = State::read_texture(&self.fonts_atlas, extent_3d!(self.fonts_atlas.width(), self.fonts_atlas.height()));
//             self.fonts_atlas_id = get_state().pipelines.rect_texture_pipeline.add_texture(raw_texture);
//             dbg!(self.fonts_atlas_id);
//         }
//         get_state().pipelines.rect_texture_pipeline.push_rect(RawRectTexture { rect: RawRect { x: pos.x, y: pos.y, w: 10., h: 10. }, texture_id: self.fonts_atlas_id })
        
//     }
// }