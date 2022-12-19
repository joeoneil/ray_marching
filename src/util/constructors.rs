use wgpu::{
    Adapter, BindGroup, BindGroupLayout, BindingResource, BufferSize, Device, Instance, Queue,
    Surface,
};

pub async fn request_adapter(instance: &Instance, surface: &Surface) -> Adapter {
    instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap()
}

pub async fn request_device(adapter: &Adapter) -> (Device, Queue) {
    adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::BUFFER_BINDING_ARRAY
                    | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY,
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None,
        )
        .await
        .unwrap()
}

pub fn create_bind_group_layout(
    device: &Device,
    binding: u32,
    ty: wgpu::BufferBindingType,
    has_dynamic_offset: bool,
    min_binding_size: Option<BufferSize>,
    label: &str,
) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty,
                has_dynamic_offset,
                min_binding_size,
            },
            count: None,
        }],
        label: Some(label),
    })
}

pub fn create_bind_group(
    device: &Device,
    layout: &BindGroupLayout,
    binding: u32,
    resource: BindingResource,
    label: &str,
) -> BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        entries: &[wgpu::BindGroupEntry { binding, resource }],
        label: Some(label),
    })
}
