use wgpu::*;

pub struct ComputeBindGroup<'a> {
    pub bg: wgpu::BindGroup,
    pub bindings: Vec<BindingResource<'a>>,
}
impl std::ops::Deref for ComputeBindGroup<'_> {
    type Target = wgpu::BindGroup;

    fn deref(&self) -> &Self::Target {
        &self.bg
    }
}
impl std::ops::DerefMut for ComputeBindGroup<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bg
    }
}

pub type SeqBinding<'a> = ((ShaderStages, BindingType, Option<std::num::NonZero<u32>>), BindingResource<'a>);

pub fn sequential_bg<'a>(device: &Device, resources: impl ExactSizeIterator<Item = SeqBinding<'a>>) -> BindGroup {
    let mut entries = Vec::with_capacity(resources.len());
    let mut bg_entries = Vec::with_capacity(resources.len());
    for (res, bg_res) in resources {
        entries.push(res);
        bg_entries.push(bg_res);
    }
    let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor { 
        label: None, 
        entries: &entries.into_iter().enumerate().map(|(binding,(visibility, ty, count))| BindGroupLayoutEntry { binding: binding as _, 
            visibility, ty, count 
        }).collect::<Vec<_>>()
    });
    device.create_bind_group(&BindGroupDescriptor { 
        label: None, 
        layout: &layout,
        entries: &bg_entries.into_iter().enumerate().map(|(binding,r)| BindGroupEntry {
            binding: binding as _,
            resource: r,
        }).collect::<Vec<_>>()
    })
}

pub struct ComputePipelineBuilder<'a> {
    pub pipe: ComputePipelineDescriptor<'a>,
}
impl<'a> ComputePipelineBuilder<'a> {
    pub fn new(module: &'a ShaderModule) -> Self {
        Self {
            pipe: ComputePipelineDescriptor {
                label: None,
                layout: None,
                module,
                entry_point: "main",
                compilation_options: PipelineCompilationOptions::default(),
                cache: None,
            }
        }
    }
    pub fn build(&self, device: &wgpu::Device) -> wgpu::ComputePipeline {
        device.create_compute_pipeline(&self.pipe)
    }
}

pub struct ComputePipelineBundleBuilder {

}

pub struct ComputePipelineBundle {
    pub bindgroup: BindGroup,
    pub pipeline: wgpu::ComputePipeline,
    pub workgroups: (u32, u32, u32),
}
impl ComputePipelineBundle {
    pub fn new(bindgroup: BindGroup, pipeline: wgpu::ComputePipeline, workgroups: (u32, u32, u32)) -> Self {
        Self {
            bindgroup,
            pipeline,
            workgroups,
        }
    }
    pub fn draw(&self, encoder: &mut CommandEncoder) {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &self.bindgroup, &[]);
        // cpass.set_push_constants(offset, data);
        cpass.dispatch_workgroups(self.workgroups.0,self.workgroups.1,self.workgroups.2);
    }
    // pub fn read_back(&self, )
}


pub mod exec_compute;