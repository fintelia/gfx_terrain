#![feature(ord_max_min)]
#![feature(unboxed_closures)]

extern crate bincode;
extern crate byteorder;
extern crate cgmath;
extern crate curl;
#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate image;
#[macro_use]
extern crate lazy_static;
extern crate lru_cache;
extern crate memmap;
extern crate notify;
extern crate num;
extern crate pbr;
extern crate rand;
#[macro_use]
extern crate rshader;
extern crate rustfft;
extern crate safe_transmute;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate shader_version;
extern crate vecmath;
extern crate vec_map;
extern crate zip;

mod utils;
mod runtime_texture;

pub mod cache;
pub mod ocean;
pub mod sky;
pub mod terrain;

pub use sky::Skybox;
pub use terrain::dem::{DigitalElevationModelParams, DemSource};
pub use terrain::file::TerrainFileParams;
pub use terrain::material::MaterialSet;
pub use terrain::quadtree::QuadTree;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
