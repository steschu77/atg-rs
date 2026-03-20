use crate::core::gl_pipeline::GlMeshId;
use crate::core::gl_pipeline_colored::{self, Vertex};
use crate::core::gl_renderer::RenderContext;
use crate::error::{Error, Result};
use crate::v2d::v3::V3;
use std::path::Path;

// ----------------------------------------------------------------------------
const TERRAIN_RESOLUTION: f32 = 0.5;
const TERRAIN_RESOLUTION_INV: f32 = 1.0 / TERRAIN_RESOLUTION;
const TERRAIN_CHUNK_SIZE: usize = 32;

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Terrain {
    chunks_cx: usize,
    chunks_cz: usize,
    width: usize,
    height: usize,
    heightmap: Vec<f32>,
}

// ----------------------------------------------------------------------------
impl Terrain {
    // ------------------------------------------------------------------------
    pub fn new(chunks_cx: usize, chunks_cz: usize) -> Self {
        let width = chunks_cx * TERRAIN_CHUNK_SIZE;
        let height = chunks_cz * TERRAIN_CHUNK_SIZE;

        let mut heightmap: Vec<f32> = vec![0.0; width * height];
        generate_hills(&mut heightmap, width, height);
        //generate_flat(&mut heightmap, width, height);

        Terrain {
            chunks_cx,
            chunks_cz,
            width,
            height,
            heightmap,
        }
    }

    // ------------------------------------------------------------------------
    pub fn from_png(path: &Path) -> Result<Self> {
        let contents = std::fs::read(path)?;
        let (png, _plte, data) = miniz::png_read::png_read(&contents)?;

        if png.color_type != miniz::png_read::PNGColorType::Greyscale {
            return Err(Error::InvalidColorFormat);
        }

        let h_norm: f32 = 1.0 / 5.0; // 5 levels per meter
        let chunks_cx = png.width / TERRAIN_CHUNK_SIZE;
        let chunks_cz = png.height / TERRAIN_CHUNK_SIZE;
        let width = chunks_cx * TERRAIN_CHUNK_SIZE;
        let height = chunks_cz * TERRAIN_CHUNK_SIZE;

        let mut heightmap: Vec<f32> = vec![0.0; width * height];
        for y in 0..png.height {
            let src_offset = y * (png.width + 1) + 1;
            let dst_offset = y * width;
            for x in 0..png.width {
                let height = data[src_offset + x] as f32;
                heightmap[dst_offset + x] = height * h_norm;
            }
        }

        Ok(Terrain {
            chunks_cx,
            chunks_cz,
            width,
            height,
            heightmap,
        })
    }

    // ------------------------------------------------------------------------
    pub fn create_chunk_mesh(
        &self,
        context: &mut RenderContext,
        chunk_x: usize,
        chunk_z: usize,
    ) -> Result<GlMeshId> {
        let resolution: f32 = TERRAIN_RESOLUTION;
        let chunk_size: usize = TERRAIN_CHUNK_SIZE;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let chunk_origin_x = chunk_x * chunk_size;
        let chunk_origin_z = chunk_z * chunk_size;

        // Generate grid of vertices
        for z in 0..=chunk_size {
            for x in 0..=chunk_size {
                let x = x + chunk_origin_x;
                let z = z + chunk_origin_z;
                let world_x = x as f32 * resolution;
                let world_z = z as f32 * resolution;
                let height = self.get_height_at(x, z);
                let normal = self.get_normal_at(x, z);

                vertices.push(Vertex {
                    pos: V3::new([world_x, height, world_z]),
                    n: normal,
                });
            }
        }

        // Generate triangle indices (two triangles per quad)
        for z in 0..chunk_size {
            for x in 0..chunk_size {
                let i0 = z * (chunk_size + 1) + x;
                let i1 = i0 + 1;
                let i2 = i0 + (chunk_size + 1);
                let i3 = i2 + 1;

                let i0 = i0 as u32;
                let i1 = i1 as u32;
                let i2 = i2 as u32;
                let i3 = i3 as u32;

                indices.extend_from_slice(&[i0, i1, i2, i1, i3, i2]);
            }
        }

        context.create_colored_mesh(&vertices, &indices, true)
    }

    // ------------------------------------------------------------------------
    pub fn height_at(&self, x: f32, z: f32) -> f32 {
        // Convert world coordinates to heightmap indices
        let hx = x * TERRAIN_RESOLUTION_INV;
        let hz = z * TERRAIN_RESOLUTION_INV;

        // Bilinear interpolation between 4 neighboring samples
        let x0 = hx.floor() as usize;
        let z0 = hz.floor() as usize;
        let x1 = x0 + 1;
        let z1 = z0 + 1;

        let fx = hx.fract();
        let fz = hz.fract();

        let h00 = self.get_height_at(x0, z0);
        let h10 = self.get_height_at(x1, z0);
        let h01 = self.get_height_at(x0, z1);
        let h11 = self.get_height_at(x1, z1);

        // Bilinear interpolation
        let h0 = h00 * (1.0 - fx) + h10 * fx;
        let h1 = h01 * (1.0 - fx) + h11 * fx;
        h0 * (1.0 - fz) + h1 * fz
    }

    // ------------------------------------------------------------------------
    pub fn normal_at(&self, x: f32, z: f32) -> V3 {
        // Convert world coordinates to heightmap indices
        let hx = x * TERRAIN_RESOLUTION_INV;
        let hz = z * TERRAIN_RESOLUTION_INV;

        // Bilinear interpolation between 4 neighboring samples
        let x0 = hx.floor() as usize;
        let z0 = hz.floor() as usize;
        let x1 = x0 + 1;
        let z1 = z0 + 1;

        let fx = hx.fract();
        let fz = hz.fract();

        let n00 = self.get_normal_at(x0, z0);
        let n10 = self.get_normal_at(x1, z0);
        let n01 = self.get_normal_at(x0, z1);
        let n11 = self.get_normal_at(x1, z1);

        // Bilinear interpolation
        let n0 = n00 * (1.0 - fx) + n10 * fx;
        let n1 = n01 * (1.0 - fx) + n11 * fx;
        (n0 * (1.0 - fz) + n1 * fz).norm()
    }

    // ------------------------------------------------------------------------
    pub fn create_normal_arrow_mesh(
        &self,
        context: &mut RenderContext,
        x: f32,
        z: f32,
        length: f32,
    ) -> Result<GlMeshId> {
        let pos = V3::new([x, self.height_at(x, z), z]);
        let normal = self.normal_at(x, z);
        let verts = gl_pipeline_colored::arrow(pos, pos + length * normal)?;
        context.create_colored_mesh(&verts, &[], true)
    }

    // ------------------------------------------------------------------------
    fn get_height_at(&self, x: usize, z: usize) -> f32 {
        let x = x.min(self.width - 1);
        let z = z.min(self.height - 1);
        self.heightmap[x + z * self.width]
    }

    // ------------------------------------------------------------------------
    fn get_normal_at(&self, x: usize, z: usize) -> V3 {
        let west = if x > 0 {
            self.get_height_at(x - 1, z)
        } else {
            self.get_height_at(x, z)
        };
        let east = if x < self.width - 1 {
            self.get_height_at(x + 1, z)
        } else {
            self.get_height_at(x, z)
        };
        let south = if z > 0 {
            self.get_height_at(x, z - 1)
        } else {
            self.get_height_at(x, z)
        };
        let north = if z < self.height - 1 {
            self.get_height_at(x, z + 1)
        } else {
            self.get_height_at(x, z)
        };

        let n_x = west - east;
        let n_y = 2.0;
        let n_z = south - north;

        let normal = V3::new([n_x, n_y, n_z]);
        normal.norm()
    }
}

// ----------------------------------------------------------------------------
fn generate_flat(_heightmap: &mut [f32], _width: usize, _height: usize) {}

// ----------------------------------------------------------------------------
fn generate_hills(heightmap: &mut [f32], width: usize, height: usize) {
    use std::f32::consts::PI;

    for z in 0..height {
        for x in 0..width {
            let nx = x as f32 / width as f32;
            let nz = z as f32 / height as f32;

            // Multiple sine waves for variation
            let h1 = (nx * PI * 3.0).sin() * 2.0;
            let h2 = (nz * PI * 2.0).sin() * 1.5;
            let h3 = ((nx + nz) * PI * 4.0).sin() * 0.8;

            heightmap[z * width + x] = h1 + h2 + h3;
        }
    }
}
