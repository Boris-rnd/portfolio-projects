use std::ops::Range;

use lines::Line;
use state::get_wgpu_state;

use super::*;

pub const CHAR_WIDTH: u32 = 64;
pub const CHAR_HEIGHT: u32 = CHAR_WIDTH;
pub const CHAR_RANGE: Range<u8> = 0x20..127;
pub const CHAR_COUNT: u8 = CHAR_RANGE.end - CHAR_RANGE.start;

pub struct FontRenderer {
    pub chars: Vec<Vec<Line>>,
}
impl FontRenderer {
    pub fn new(device: &Device) -> Self {
        let mut chars = Vec::with_capacity(CHAR_COUNT as _);
        let face = ttf_parser::Face::parse(include_bytes!("../../JB Mono.ttf"), 0).unwrap();
        let state = get_wgpu_state();

        let start_x = face.global_bounding_box().x_min as f32;
        let end_x = face.global_bounding_box().x_max as f32;
        let start_y = face.global_bounding_box().y_min as f32;
        let end_y = face.global_bounding_box().y_max as f32;
        let char_w = face.global_bounding_box().width() as f32;
        let char_h = face.global_bounding_box().height() as f32;
        for chr in CHAR_RANGE {
            let mut builder = Builder::new();
            let code = chr as char;
            if let Some(id) = face.glyph_index(code) {
                face.tables().glyf.unwrap().outline(id, &mut builder);

                chars.push(
                    builder
                        .lines
                        .iter()
                        .enumerate()
                        .map(|(i, line)| {
                            let sx = ((line.start.x) / (char_w * 2.));
                            let ex = ((line.end.x) / (char_w * 2.));
                            let sy = (line.start.y) / (char_h * 2.) + 0.0;
                            let ey = (line.end.y) / (char_h * 2.) + 0.0;
                            // if sx<0. || ex <0. || sx>=1. || ex >=1. {panic!()}
                            (lines::Line {
                                start: vec2f(sx, sy),
                                end: vec2f(ex, ey),
                                // start: vec2f((pt.0-start_x as f32) / char_w as f32*CHAR_WIDTH as f32, (pt.1-start_y as f32) / char_h as f32*CHAR_HEIGHT as f32),
                                // end: vec2f((pt2.0-start_x as f32) / char_w as f32*CHAR_WIDTH as f32, (pt2.1-start_y as f32) / char_h as f32*CHAR_HEIGHT as f32),
                            })
                        })
                        .collect::<Vec<_>>(),
                );
            } else {
                chars.push(vec![]);
            }
        }

        Self { chars }
    }
    pub fn add_text(&self, pos: Vec2f, text: &str, font_size: f32) {
        let lines_pipe = &mut get_state().pipelines.lines_pipeline;
        let mut lines = Vec::with_capacity(text.len() * self.chars[0].len()); // self.chars[0].len() is kind of average amount of lines
        let mut pos = normalize_pos(pos);
        for chr in text.chars() {
            let idx = chr as u8 as u32 - CHAR_RANGE.start as u32;
            lines.append(
                &mut self.chars[idx as usize]
                    .iter()
                    .cloned()
                    .map(|mut line| {
                        line.start.x *= font_size / 64.;
                        line.start.y *= font_size / 64.;
                        line.end.x *= font_size / 64.;
                        line.end.y *= font_size / 64.;
                        // line.start.x += 0.5;
                        line.start.y -= 0.25 * font_size / 64.;
                        // line.end.x += 0.5;
                        line.end.y -= 0.25 * font_size / 64.;
                        line.start.x += pos.x;
                        line.end.x += pos.x;
                        line.start.y += pos.y;
                        line.end.y += pos.y;
                        // line.start = normalize_pos(line.start);
                        // line.end = normalize_pos(line.end);
                        (line)
                    })
                    .collect::<Vec<_>>(),
            );
            pos.x += (10. + font_size) / get_wgpu_state().screen_size().width as f32;
        }
        lines_pipe.append_lines(&lines);
    }

    // pub(crate) fn load_fonts() -> (Vec<Vec<u8>>, LinesPipeline) {
    //     let state = get_wgpu_state();

    //     let face = ttf_parser::Face::parse(include_bytes!("../../JB Mono.ttf"), 0).unwrap();
    //     let start_x = face.global_bounding_box().x_min;
    //     let end_x = face.global_bounding_box().x_max;
    //     let start_y = face.global_bounding_box().y_min;
    //     let end_y = face.global_bounding_box().y_max;
    //     let char_w = face.global_bounding_box().width();
    //     let char_h = face.global_bounding_box().height();
    //     for chr in CHAR_RANGE {
    //         let mut builder = Builder(vec![]);
    //         let code = chr as char;
    //         let id = face.glyph_index(code).unwrap();
    //         face.tables().glyf.unwrap().outline(id, &mut builder);
    //         let mut pts = vec![];
    //         for pt in builder.0 {
    //             pts.push(pt)
    //         }
    //         lines.append_lines(&pts.iter().skip(1).enumerate().map(|(i,pt)| {
    //             let pt2 = pts[i];
    //             if pt2 == *pt {return Line::default()}
    //             // dbg!(
    //             //     vec2f((pt.0-start_x as f32) / char_w as f32*CHAR_WIDTH as f32, (pt.1-start_y as f32) / char_h as f32*CHAR_HEIGHT as f32),
    //             //     vec2f((pt2.0-start_x as f32) / char_w as f32*CHAR_WIDTH as f32, (pt2.1-start_y as f32) / char_h as f32*CHAR_HEIGHT as f32),
    //             // );
    //             (lines::Line {
    //                 start: vec2f(((pt.0 as f32) / char_w as f32-1.0+i as f32)/CHAR_COUNT as f32, (pt.1) / char_h as f32+1.0),
    //                 end: vec2f(((pt2.0 as f32) / char_w as f32-1.0+i as f32)/CHAR_COUNT as f32, (pt2.1) / char_h as f32+1.0),
    //                 // start: vec2f((pt.0-start_x as f32) / char_w as f32*CHAR_WIDTH as f32, (pt.1-start_y as f32) / char_h as f32*CHAR_HEIGHT as f32),
    //                 // end: vec2f((pt2.0-start_x as f32) / char_w as f32*CHAR_WIDTH as f32, (pt2.1-start_y as f32) / char_h as f32*CHAR_HEIGHT as f32),
    //             }
    //             )
    //         }).collect::<Vec<_>>());
    //     }

    //     let fonts_atlas = state.device.create_texture(&TextureDescriptor {
    //         label: Some("Font atlas"),
    //         size: extent_3d!(CHAR_WIDTH*(CHAR_RANGE.end-CHAR_RANGE.start) as u32, CHAR_HEIGHT),
    //         mip_level_count: 1,
    //         sample_count: 1,
    //         dimension: TextureDimension::D2,
    //         format: TextureFormat::Bgra8UnormSrgb,
    //         usage: TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
    //         view_formats: &[],
    //     });
    //     let atlas_view = &fonts_atlas.create_view(&TextureViewDescriptor::default());
    //     command!(|encoder: &mut CommandEncoder| {
    //         let mut pass = render_pass!(encoder, atlas_view, Color {r:0.0,g:0.,b:0.,a:0.});
    //         lines.draw(&mut pass);
    //         drop(pass);
    //     });
    //     let raw = State::read_full_texture(&fonts_atlas);
    //     dbg!(raw.iter().filter(|x| **x==255).collect::<Vec<_>>().len());
    //     assert_ne!(raw.iter().filter(|x| **x==255).collect::<Vec<_>>().len(), 0);
    //     (raw.chunks_exact(CHAR_WIDTH as usize*CHAR_HEIGHT as usize*4).map(|ch| ch.to_vec()).collect(), lines)
    // }
}

struct Builder {
    lines: Vec<Line>,
    cursor_pos: Vec2f,
}
impl Builder {
    fn new() -> Self {
        Self {
            lines: vec![],
            cursor_pos: Vec2f::ZERO,
        }
    }
}
impl ttf_parser::OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.cursor_pos = vec2f(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.lines.push(Line {
            start: self.cursor_pos,
            end: vec2f(x, y),
        });
        self.cursor_pos = vec2f(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        // todo! approximate the quadratic curve with more points
        self.lines.push(Line {
            start: self.cursor_pos,
            end: vec2f(x1, y1),
        });
        self.lines.push(Line {
            start: vec2f(x1, y1),
            end: vec2f(x2, y2),
        });
        self.cursor_pos = vec2f(x2, y2);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        self.lines.push(Line {
            start: self.cursor_pos,
            end: vec2f(x1, y1),
        });
        self.lines.push(Line {
            start: vec2f(x1, y1),
            end: vec2f(x2, y2),
        });
        self.lines.push(Line {
            start: vec2f(x2, y2),
            end: vec2f(x3, y3),
        });
        self.cursor_pos = vec2f(x3, y3);
    }

    fn close(&mut self) {
        // Optionally handle closing the path if needed
    }
}



// pub struct TextHandle<'a> {
//     /// The range of lines that are owned by this text in the lines pipeline
//     lines_region: Range<usize>,
//     lines_pipeline: &'a mut LinesPipeline,
// }
// impl TextHandle<'_> {
//     pub fn 
// }