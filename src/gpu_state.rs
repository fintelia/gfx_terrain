use std::{borrow::Cow, collections::HashMap};

use crate::cache::{LayerType, MeshType, SingularLayerType};
use vec_map::VecMap;

#[repr(C)]
pub(crate) struct DrawIndirect {
    vertex_count: u32,   // The number of vertices to draw.
    instance_count: u32, // The number of instances to draw.
    base_vertex: u32,    // The Index of the first vertex to draw.
    base_instance: u32,  // The instance ID of the first instance to draw.
}

pub(crate) struct GpuMeshLayer {
    pub indirect: wgpu::Buffer,
    pub storage: wgpu::Buffer,
}

pub(crate) struct GpuState {
    pub noise: wgpu::Texture,
    pub sky: wgpu::Texture,
    pub transmittance: wgpu::Texture,
    pub inscattering: wgpu::Texture,
    pub aerial_perspective: wgpu::Texture,

    pub tile_cache: VecMap<wgpu::Texture>,
    pub mesh_cache: VecMap<GpuMeshLayer>,
    pub texture_cache: VecMap<wgpu::Texture>,

    pub bc4_staging: wgpu::Texture,
    pub bc5_staging: wgpu::Texture,

    pub globals: wgpu::Buffer,
    pub node_buffer: wgpu::Buffer,
}
impl GpuState {
    pub(crate) fn bind_group_for_shader(
        &self,
        device: &wgpu::Device,
        shader: &rshader::ShaderSet,
        buffers: HashMap<Cow<str>, (bool, wgpu::BindingResource)>,
        image_views: HashMap<Cow<str>, wgpu::TextureView>,
        group_name: &str,
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
        let mut layout_descriptor_entries = shader.layout_descriptor().entries.to_vec();

        let mut buffers = buffers;
        let mut image_views = image_views;
        let mut samplers = HashMap::new();
        for (name, layout) in shader.desc_names().iter().zip(layout_descriptor_entries.iter()) {
            let name = &**name.as_ref().unwrap();
            match layout.ty {
                wgpu::BindingType::StorageTexture { .. } | wgpu::BindingType::Texture { .. } => {
                    if !image_views.contains_key(name) {
                        image_views.insert(
                            name.into(),
                            match name {
                                "noise" => &self.noise,
                                "sky" => &self.sky,
                                "transmittance" => &self.transmittance,
                                "inscattering" => &self.inscattering,
                                "aerial_perspective" => &self.aerial_perspective,
                                "displacements" => &self.tile_cache[LayerType::Displacements],
                                "albedo" => &self.tile_cache[LayerType::Albedo],
                                "roughness" => &self.tile_cache[LayerType::Roughness],
                                "normals" => &self.tile_cache[LayerType::Normals],
                                "heightmaps" => &self.tile_cache[LayerType::Heightmaps],
                                "grass_canopy" => {
                                    &self.texture_cache[SingularLayerType::GrassCanopy]
                                }
                                "bc4_staging" => &self.bc4_staging,
                                "bc5_staging" => &self.bc5_staging,
                                _ => unreachable!("unrecognized image: {}", name),
                            }
                            .create_view(
                                &wgpu::TextureViewDescriptor {
                                    label: Some(&format!("view.{}", name)),
                                    ..Default::default()
                                },
                            ),
                        );
                    }
                }
                wgpu::BindingType::Buffer { .. } => {
                    if !buffers.contains_key(name) {
                        let buffer = match name {
                            "grass_indirect" => &self.mesh_cache[MeshType::Grass].indirect,
                            "grass_storage" => &self.mesh_cache[MeshType::Grass].storage,
                            "nodes" => &self.node_buffer,
                            "globals" => &self.globals,
                            _ => unreachable!("unrecognized storage buffer: {}", name),
                        };
                        let resource = wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer,
                            size: None,
                            offset: 0,
                        });
                        buffers.insert(name.into(), (false, resource));
                    }
                }
                wgpu::BindingType::Sampler { .. } => {
                    samplers.insert(
                        name,
                        match name {
                            "nearest" => device.create_sampler(&wgpu::SamplerDescriptor {
                                address_mode_u: wgpu::AddressMode::ClampToEdge,
                                address_mode_v: wgpu::AddressMode::ClampToEdge,
                                address_mode_w: wgpu::AddressMode::ClampToEdge,
                                mag_filter: wgpu::FilterMode::Nearest,
                                min_filter: wgpu::FilterMode::Nearest,
                                mipmap_filter: wgpu::FilterMode::Nearest,
                                label: Some("sampler.nearest"),
                                ..Default::default()
                            }),
                            "linear" => device.create_sampler(&wgpu::SamplerDescriptor {
                                address_mode_u: wgpu::AddressMode::ClampToEdge,
                                address_mode_v: wgpu::AddressMode::ClampToEdge,
                                address_mode_w: wgpu::AddressMode::ClampToEdge,
                                mag_filter: wgpu::FilterMode::Linear,
                                min_filter: wgpu::FilterMode::Linear,
                                mipmap_filter: wgpu::FilterMode::Nearest,
                                label: Some("sampler.linear"),
                                ..Default::default()
                            }),
                            "linear_wrap" => device.create_sampler(&wgpu::SamplerDescriptor {
                                address_mode_u: wgpu::AddressMode::Repeat,
                                address_mode_v: wgpu::AddressMode::Repeat,
                                address_mode_w: wgpu::AddressMode::Repeat,
                                mag_filter: wgpu::FilterMode::Linear,
                                min_filter: wgpu::FilterMode::Linear,
                                mipmap_filter: wgpu::FilterMode::Nearest,
                                label: Some("sampler.linear_wrap"),
                                ..Default::default()
                            }),
                            _ => unreachable!("unrecognized sampler: {}", name),
                        },
                    );
                }
            }
        }

        let mut bindings = Vec::new();
        for (name, layout) in shader.desc_names().iter().zip(layout_descriptor_entries.iter_mut()) {
            let name = &**name.as_ref().unwrap();
            bindings.push(wgpu::BindGroupEntry {
                binding: layout.binding,
                resource: match layout.ty {
                    wgpu::BindingType::Sampler { .. } => {
                        wgpu::BindingResource::Sampler(&samplers[name])
                    }
                    wgpu::BindingType::StorageTexture { .. } => {
                        wgpu::BindingResource::TextureView(&image_views[name])
                    }
                    wgpu::BindingType::Texture { ref mut sample_type, .. } => {
                        match name {
                            "transmittance" | "inscattering" | "heightmaps" | "displacements" => {
                                *sample_type = wgpu::TextureSampleType::Float { filterable: false }
                            }
                            _ => {}
                        }
                        wgpu::BindingResource::TextureView(&image_views[name])
                    }
                    wgpu::BindingType::Buffer { ref mut has_dynamic_offset, .. } => {
                        let (d, ref buf) = buffers[name];
                        *has_dynamic_offset = d;
                        buf.clone()
                    }
                },
            });
        }

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &layout_descriptor_entries,
            label: Some(&format!("layout.{}", group_name)),
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &*bindings,
            label: Some(&format!("bindgroup.{}", group_name)),
        });

        (bind_group, bind_group_layout)
    }
}
