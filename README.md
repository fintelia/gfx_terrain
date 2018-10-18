# Terra [![crates.io](https://img.shields.io/crates/v/terra.svg)](https://crates.io/crates/terra) [![docs.rs](https://docs.rs/terra/badge.svg)](https://docs.rs/terra) [![Travis](https://img.shields.io/travis/fintelia/terra.svg)]()

Terra is a large scale terrain rendering library built on top of
[gfx pre-ll](https://github.com/gfx-rs/gfx).

![Screenshot](/screenshot.png?raw=true)

# Features

### Open World

Terra supports rendering an entire planet with a "playable area" of tens or even hundreds of square
kilometers.

### Level of detail (LOD)

In Terra, terrain is treated as a [heightmap](https://en.wikipedia.org/wiki/Heightmap) along with a
collection of texture maps storing the surface normal, color, etc. However, to ensure smooth frame
rates and avoid noticable "LOD popping", Terra internally uses [a much more sophisticated
representation](https://pdfs.semanticscholar.org/6a75/892f45b72f8765379134e8d2a4ed6a04f1b0.pdf) that provides
continuous level of detail.

### Convenient Coordinate System

The coordinate system is very simple: the x-axis points east, y-axis points up and z-axis points
south. The origin is chosen during map file creation and can be any point on the planet surface
(except for some points too close to the poles)

### Automatic Generation

Terra is also capable of generating terrains based on real world data. Elevation data is sourced
from either the STRM 30m dataset or USGS's National Elevation Dataset and then enhanced using
fractal refinement, while water masks and land cover data are incorporated to generate other terrain
features.

# Status
Terrain:
- [x] CDLOD quadtree implementation
- [x] Frustum culling
- [ ] Oceans
- [ ] Biomes

Foliage:
- [ ] Grass
- [x] Trees

Atmosphere:
- [x] Fog rendering
- [x] Aerial perspective
- [x] Precomputed scattering
- [ ] Clouds

# Data Sources / Credits

During operation, this library downloads and merges datasets from a variety of sources. If you integrate
it into your own project, please be sure to give proper credit to all of the following as applicable.

## Elevation data

* [USGS National Elevation Dataset (NED)](https://lta.cr.usgs.gov/NED)
* [Shuttle Radar Topography Mission (SRTM) 1 Arc-Second Global](https://lta.cr.usgs.gov/SRTM1Arc)
* [ETOPO1 Global Relief Model](https://www.ngdc.noaa.gov/mgg/global)

## Land Cover

* [Global Surface Water](https://landcover.usgs.gov/glc/WaterDescriptionAndDownloads.php)
* [Global Tree Canopy Cover circa 2010](https://landcover.usgs.gov/glc/TreeCoverDescriptionAndDownloads.php)
* [0.5 km MODIS-based Global Land Cover Climatology](https://landcover.usgs.gov/global_climatology.php)

## Other Datasets

* [Blue Marble Next Generation](https://visibleearth.nasa.gov/view.php?id=76487)

## Textures

* [Skybox](https://opengameart.org/content/clouds-skybox-1)
* [Terrain textures pack](https://opengameart.org/content/terrain-textures-pack-from-stunt-rally-23)
