use crate::*;

pub fn buffer(
    device: &Device,
    buffer: &[impl bytemuck::NoUninit],
    usage: BufferUsages,
) -> Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(buffer),
        usage,
    })
}
pub fn vertex_buffer(device: &Device, buf: &[impl bytemuck::NoUninit]) -> Buffer {
    buffer(
        device,
        buf,
        BufferUsages::VERTEX
            | BufferUsages::COPY_DST
            | BufferUsages::COPY_SRC
            | BufferUsages::STORAGE,
    )
}
pub fn index_buffer(device: &Device, buf: &[impl bytemuck::NoUninit]) -> Buffer {
    buffer(device, buf, BufferUsages::INDEX)
}
pub fn empty_buffer(
    device: &Device,
    size: u64,
    usage: BufferUsages,
    mapped_at_creation: bool,
) -> Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!("Buffer of {size} {usage:?}")),
        usage,
        size,
        mapped_at_creation,
    })
}



pub fn resize_buffer(old_buffer: &wgpu::Buffer, new_size: wgpu::BufferAddress) -> wgpu::Buffer {
    let state = get_wgpu_state();
    let new_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Resized Buffer"),
        size: new_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | old_buffer.usage(),
        mapped_at_creation: false,
    });

    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Buffer Resize Encoder"),
        });

    let copy_size = old_buffer.size().min(new_size);
    encoder.copy_buffer_to_buffer(old_buffer, 0, &new_buffer, 0, copy_size);

    state.queue.submit(Some(encoder.finish()));

    new_buffer
}
#[track_caller]
pub fn write_to_buffer(
    buffer: &wgpu::Buffer,
    offset: wgpu::BufferAddress,
    data: &[impl NoUninit],
) {
    // Ensure the buffer was created with MAP_WRITE usage
    assert!(buffer.usage().contains(wgpu::BufferUsages::MAP_WRITE));

    // Map the buffer to allow CPU access
    let slice = buffer.slice(offset..offset + data.len() as wgpu::BufferAddress);
    slice.map_async(wgpu::MapMode::Write, |r| {
        r.unwrap();
    });
    get_wgpu_state().device.poll(wgpu::MaintainBase::Wait).unwrap();
    // Write data into the mapped range
    let mut view = slice.get_mapped_range_mut();
    view.copy_from_slice(bytemuck::cast_slice(data));

    // Unmap the buffer to make it available for GPU use again
    buffer.unmap();
}

pub fn write_to_buffer_with_staging(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target_buffer: &wgpu::Buffer,
    offset: wgpu::BufferAddress,
    data: &[impl NoUninit],
) {
    let contents: &[u8] = bytemuck::cast_slice(data);
    let staging_buffer = buffer(device, contents, BufferUsages::COPY_SRC);
    let mut encoder = encoder!(device, "Buffer Copy Encoder");

    encoder.copy_buffer_to_buffer(
        &staging_buffer,
        0,
        target_buffer,
        offset,
        contents.len() as u64,
    );

    queue.submit(Some(encoder.finish()));
}


#[macro_export]
macro_rules! create_buffer {
    ($device: expr, $size: ident, $usage: expr, MAPPED, $label: expr) => {{
        $device.create_buffer(&BufferDescriptor {
            size: $size,
            usage: $usage,
            mapped_at_creation: true,
            label: $label,
        })
    }};
    ($size: ident, $usage: expr, MAPPED, $label: expr) => {{
        $crate::create_buffer!(get_state().device,
            $size,
            $usage,
            MAPPED,
            $label,
        )
    }};
    ($size: ident, $usage: expr, MAPPED) => {{
        $crate::create_buffer!(get_state().device,
            size,
            usage,
            MAPPED,
            None,
        )
    }};
    ($device: expr, $size: ident, $usage: expr) => {{
        #[allow(non_snake_case)]
        let STORAGE = BufferUsages::STORAGE;
        $device.create_buffer(&BufferDescriptor {
            size: $size as _,
            usage: $usage,
            mapped_at_creation: false,
            label: None,
        })
    }};
    ($size: ident, $usage: expr) => {{
        $crate::create_buffer!(get_wgpu_state().device,
            $size,
            $usage)
    }};
}

#[macro_export]
macro_rules! create_texture {
    ($width: expr,$height:expr, $usage: expr) => {{
        let state = $crate::get_tate();
        state.device.create_texture(&TextureDescriptor { 
            size: $crate::extent_3d!($width, $height), 
            mip_level_count: 1, 
            sample_count: 1, 
            dimension: TextureDimension::D2, 
            format: TextureFormat::Bgra8Unorm, 
            usage: $usage, 
            view_formats: &[],
            label: None, 
        })
    }}
}

#[macro_export]
macro_rules! create_view {
    ($texture: expr) => {
        $texture.create_view(&TextureViewDescriptor::default())
    };
}

#[macro_export]
macro_rules! extent_3d {
    ($width: expr, $height: expr) => {
        Extent3d {
            width: $width as _,
            height: $height as _,
            depth_or_array_layers: 1,
        }
    };
    ($width: expr, $height: expr, $depth_or_array_layers: expr) => {
        Extent3d {
            width: $width as _,
            height: $height as _,
            depth_or_array_layers: $depth_or_array_layers as _,
        }
    };
}
#[macro_export]
macro_rules! encoder {
    () => {
        $crate::state::get_tate()
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None })
    };
    ($device: expr) => {
        $device.create_command_encoder(&CommandEncoderDescriptor { label: None })
    };
    ($device: expr, $label: expr) => {
        $device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some($label),
        })
    };
    (label=$label: expr) => {
        $crate::state::get_tate()
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some($label),
            })
    };
}

#[macro_export]
macro_rules! command {
    ($closure: expr) => {
        let state = $crate::state::get_wgpu_state();
        let mut encoder: CommandEncoder = $crate::encoder!(state.device);
        let mut closure: Box<dyn FnMut(&mut CommandEncoder)> = Box::new($closure);
        closure(&mut encoder);
        state.queue.submit(Some(encoder.finish()));
        drop(closure)
    };
}

#[macro_export]
macro_rules! compute_pass {
    () => {
        todo!();
        $crate::compute_pass!(encoder!())
    };
    ($encoder: expr) => {
        $encoder.begin_compute_pass(&ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        })
    };
}
#[macro_export]
macro_rules! render_pass {
    ($encoder: ident) => {
        $encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Empty Render Pass"),
            color_attachments: &[],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        })
    };
    ($encoder: ident, $view: ident, $clear_color: expr) => {
        $encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Clear Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: $view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear($clear_color),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        })
    };
    ($encoder: ident, $view: ident) => {
        $encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Clear Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: $view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear($clear_color),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        })
    };
}

pub fn copy_buffer_to_texture(
    encoder: &mut CommandEncoder,
    buffer: &Buffer,
    texture: &Texture,
    texture_extent: Extent3d,
) {
    encoder.copy_buffer_to_texture(
        TexelCopyBufferInfo {
            buffer,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(std::mem::size_of::<f32>() as u32 * texture_extent.width),
                rows_per_image: Some(texture_extent.height),
            },
        },
        TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            aspect: TextureAspect::All,
            origin: Origin3d { x: 0, y: 0, z: 0 },
        },
        texture_extent,
    )
}


pub fn copy_texture_to_texture(
    encoder: &mut CommandEncoder,
    src: &Texture,
    dst: &Texture,
    size: Extent3d,
) {
    encoder.copy_texture_to_texture(src.as_image_copy(), dst.as_image_copy(), size)
}
