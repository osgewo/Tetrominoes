use std::ops::Deref;

use wgpu::{BindingResource, BindingType, Device, ShaderStages};

pub struct Entry<'a> {
    pub binding: u32,
    pub visibility: ShaderStages,
    pub ty: BindingType,
    pub resource: BindingResource<'a>,
}

pub struct BindGroup {
    inner: wgpu::BindGroup,
    layout: wgpu::BindGroupLayout,
}

impl BindGroup {
    pub fn new(device: &Device, entries: &[Entry]) -> Self {
        let layout_entries = entries
            .iter()
            .map(|e| wgpu::BindGroupLayoutEntry {
                binding: e.binding,
                visibility: e.visibility,
                ty: e.ty,
                count: None,
            })
            .collect::<Vec<_>>();
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &layout_entries,
        });

        let group_entries = entries
            .iter()
            .map(|e| wgpu::BindGroupEntry {
                binding: e.binding,
                resource: e.resource.clone(),
            })
            .collect::<Vec<_>>();
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &group_entries,
        });

        Self {
            inner: group,
            layout,
        }
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }
}

impl Deref for BindGroup {
    type Target = wgpu::BindGroup;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
