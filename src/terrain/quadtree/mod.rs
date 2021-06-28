use crate::cache::Priority;
use crate::cache::TileCache;
use cgmath::*;
use fnv::FnvHashMap;
use std::convert::TryInto;

pub(crate) mod node;
pub(crate) mod render;

pub(crate) use crate::terrain::quadtree::node::*;
pub(crate) use crate::terrain::quadtree::render::*;

/// The central object in terra. It holds all relevant state and provides functions to update and
/// render the terrain.
pub(crate) struct QuadTree {
    // ocean: Ocean<R>,
    /// List of nodes that will be rendered.
    visible_nodes: Vec<VNode>,
    partially_visible_nodes: Vec<(VNode, u8)>,

    heights_resolution: u32,

    node_states: Vec<NodeState>,

    node_priorities: FnvHashMap<VNode, Priority>,
    last_camera_position: Option<mint::Point3<f64>>,
}

impl std::fmt::Debug for QuadTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QuadTree")
    }
}

#[allow(unused)]
impl QuadTree {
    pub(crate) fn new(heights_resolution: u32) -> Self {
        Self {
            visible_nodes: Vec::new(),
            partially_visible_nodes: Vec::new(),
            node_states: Vec::new(),
            heights_resolution,
            node_priorities: FnvHashMap::default(),
            last_camera_position: None,
        }
    }

    pub(crate) fn create_index_buffers(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let mut make_index_buffer = |resolution: u16| -> Vec<u16> {
            let mut data = Vec::new();

            let width = resolution + 1;
            for y in 0..resolution {
                for x in 0..resolution {
                    for offset in [0, 1, width, 1, width + 1, width].iter() {
                        data.push(offset + (x + y * width));
                    }
                }
            }
            data
        };
        let resolution = self.heights_resolution as u16;
        let full = make_index_buffer(resolution);
        let half = make_index_buffer(resolution / 2);

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: (2 * (full.len() + half.len())).try_into().unwrap(),
            usage: wgpu::BufferUsage::INDEX,
            label: Some("buffer.terrain.index"),
            mapped_at_creation: true,
        });
        let mut buffer_view = buffer.slice(..).get_mapped_range_mut();
        buffer_view[0..(full.len() * 2)].copy_from_slice(bytemuck::cast_slice(&full));
        buffer_view[(full.len() * 2)..].copy_from_slice(bytemuck::cast_slice(&half));
        drop(buffer_view);
        buffer.unmap();
        buffer
    }

    pub fn update_priorities(&mut self, camera: mint::Point3<f64>) {
        if self.last_camera_position == Some(camera) {
            return;
        }
        self.last_camera_position = Some(camera);
        let camera = Vector3::new(camera.x, camera.y, camera.z);

        self.node_priorities.clear();
        VNode::breadth_first(|node| {
            let priority = node.priority(camera);
            self.node_priorities.insert(node, priority);
            priority >= Priority::cutoff() && node.level() < VNode::LEVEL_CELL_2CM
        });
    }

    pub fn update_visibility(&mut self, tile_cache: &TileCache) {
        self.visible_nodes.clear();
        self.partially_visible_nodes.clear();

        // Any node with all needed layers in cache is visible...
        let mut node_visibilities: FnvHashMap<VNode, bool> = FnvHashMap::default();
        VNode::breadth_first(|node| {
            let priority = self.node_priority(node);
            let visible = (node.level() == 0 || priority >= Priority::cutoff());
            node_visibilities.insert(node, visible);
            visible && node.level() < VNode::LEVEL_CELL_2CM
        });
        // let min_missing_level = node_visibilities
        //     .iter()
        //     .filter(|(&n, &v)| v && !tile_cache.contains(n, LayerType::Displacements))
        //     .map(|(n, v)| n.level())
        //     .min();
        // if let Some(min) = min_missing_level {
        //     for (n, v) in node_visibilities.iter_mut() {
        //         if n.level() >= min {
        //             *v = false;
        //         }
        //     }
        // }

        // ...Except if all its children are visible instead.
        VNode::breadth_first(|node| {
            if node.level() < VNode::LEVEL_CELL_2CM && node_visibilities[&node] {
                let mut mask = 0;
                for (i, c) in node.children().iter().enumerate() {
                    if !node_visibilities[c] {
                        mask = mask | (1 << i);
                    }
                }

                if mask == 15 {
                    self.visible_nodes.push(node);
                } else if mask > 0 {
                    self.partially_visible_nodes.push((node, mask));
                }

                mask < 15
            } else if node_visibilities[&node] {
                self.visible_nodes.push(node);
                false
            } else {
                false
            }
        });
    }

    pub fn node_buffer_length(&self) -> usize {
        self.node_states.len()
    }

    pub fn node_priority(&self, node: VNode) -> Priority {
        self.node_priorities.get(&node).cloned().unwrap_or(Priority::none())
    }

    // pub fn get_height(
    //     &self,
    //     mapfile: &MapFile,
    //     tile_cache: &VecMap<TileCache>,
    //     p: Point2<f32>,
    // ) -> Option<f32> {
    //     if self.nodes[0].bounds.min.x > p.x
    //         || self.nodes[0].bounds.max.x < p.x
    //         || self.nodes[0].bounds.min.z > p.y
    //         || self.nodes[0].bounds.max.z < p.y
    //     {
    //         return None;
    //     }

    //     let mut id = NodeId::root();
    //     while self.nodes[id].children.iter().any(|c| c.is_some()) {
    //         for c in self.nodes[id].children.iter() {
    //             if let Some(c) = *c {
    //                 if self.nodes[c].bounds.min.x <= p.x
    //                     && self.nodes[c].bounds.max.x >= p.x
    //                     && self.nodes[c].bounds.min.z <= p.y
    //                     && self.nodes[c].bounds.max.z >= p.y
    //                 {
    //                     id = c;
    //                     break;
    //                 }
    //             }
    //         }
    //     }

    //     let layer = &tile_cache[LayerType::Heights.index()];
    //     let resolution = (layer.resolution() - 1) as f32;
    //     let x = (p.x - self.nodes[id].bounds.min.x) / self.nodes[id].side_length() * resolution;
    //     let y = (p.y - self.nodes[id].bounds.min.z) / self.nodes[id].side_length() * resolution;

    //     let get_texel =
    //         |x, y| layer.get_texel(mapfile, id, x, y).read_f32::<LittleEndian>().unwrap();

    //     let (mut fx, mut fy) = (x.fract(), y.fract());
    //     let (mut ix, mut iy) = (x.floor() as usize, y.floor() as usize);
    //     if ix == layer.resolution() as usize - 1 {
    //         ix = layer.resolution() as usize - 2;
    //         fx = 1.0;
    //     }
    //     if iy == layer.resolution() as usize - 1 {
    //         iy = layer.resolution() as usize - 2;
    //         fy = 1.0;
    //     }

    //     if fx + fy < 1.0 {
    //         Some(
    //             (1.0 - fx - fy) * get_texel(ix, iy)
    //                 + fx * get_texel(ix + 1, iy)
    //                 + fy * get_texel(ix, iy + 1),
    //         )
    //     } else {
    //         Some(
    //             (fx + fy - 1.0) * get_texel(ix + 1, iy + 1)
    //                 + (1.0 - fx) * get_texel(ix, iy + 1)
    //                 + (1.0 - fy) * get_texel(ix + 1, iy),
    //         )
    //     }
    // }
}
