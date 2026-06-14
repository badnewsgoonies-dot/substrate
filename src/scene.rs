//! Scene as a coordinate.
//!
//! A whole textured world — its grid plus which procedural material
//! clads each tile — serializes to a tiny text description that both the
//! addressing substrate and this raycaster agree on. The raycaster
//! `parse`s it and renders; the substrate side can address/store the
//! same bytes by content hash. This is where "art from math" meets
//! "the world is a coordinate": the *description* is a few hundred bytes;
//! the *frame* is rendered from it.
//!
//! Format (line-oriented, v1):
//!   v1
//!   grid <W>x<H>
//!   <H rows of tile digits>
//!   mat <tile_id> <surface> <seed> <r>,<g>,<b>
//!   ...
//! surface is one of: wood marble tile plaster brick

use crate::space::{Tile, TileGrid};
use crate::texture::{Material, Surface};

pub struct Scene {
    pub grid: TileGrid,
    /// material per tile id (index 0..=7); tile 0 is FLOOR (unused for walls)
    pub materials: [Material; 8],
}

fn surface_from(s: &str) -> Option<Surface> {
    Some(match s {
        "wood" => Surface::Wood,
        "marble" => Surface::Marble,
        "tile" => Surface::Tile,
        "plaster" => Surface::Plaster,
        "brick" => Surface::Brick,
        _ => return None,
    })
}

fn surface_name(s: Surface) -> &'static str {
    match s {
        Surface::Wood => "wood",
        Surface::Marble => "marble",
        Surface::Tile => "tile",
        Surface::Plaster => "plaster",
        Surface::Brick => "brick",
    }
}

impl Scene {
    pub fn parse(text: &str) -> Result<Scene, String> {
        let mut lines = text.lines().map(|l| l.trim()).filter(|l| !l.is_empty());
        let ver = lines.next().ok_or("empty scene")?;
        if ver != "v1" {
            return Err(format!("unknown scene version {ver:?}"));
        }
        let g = lines.next().ok_or("missing grid line")?;
        let dims = g.strip_prefix("grid ").ok_or("expected `grid WxH`")?;
        let (w, h) = dims.split_once('x').ok_or("grid dims must be WxH")?;
        let w: usize = w.parse().map_err(|_| "bad grid width")?;
        let h: usize = h.parse().map_err(|_| "bad grid height")?;

        let mut rows: Vec<String> = Vec::with_capacity(h);
        for _ in 0..h {
            let r = lines.next().ok_or("scene ended before all grid rows")?;
            if r.len() != w {
                return Err(format!("grid row width {} != {}", r.len(), w));
            }
            rows.push(r.to_string());
        }
        let row_refs: Vec<&str> = rows.iter().map(|s| s.as_str()).collect();
        let grid = TileGrid::from_ascii(&row_refs);

        // default materials, overridden by `mat` lines
        let mut materials = [Material { surface: Surface::Plaster, seed: 1, tint: (170, 170, 170) }; 8];
        for line in lines {
            let rest = match line.strip_prefix("mat ") {
                Some(r) => r,
                None => continue,
            };
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() != 4 {
                return Err(format!("mat line needs 4 fields, got {}: {line:?}", parts.len()));
            }
            let id: usize = parts[0].parse().map_err(|_| "bad tile id")?;
            if id >= 8 {
                return Err(format!("tile id {id} out of range 0..8"));
            }
            let surface = surface_from(parts[1]).ok_or_else(|| format!("bad surface {:?}", parts[1]))?;
            let seed: u32 = parts[2].parse().map_err(|_| "bad seed")?;
            let rgb: Vec<u8> = parts[3].split(',')
                .map(|c| c.parse::<u8>().map_err(|_| "bad rgb"))
                .collect::<Result<_, _>>()?;
            if rgb.len() != 3 {
                return Err("tint must be r,g,b".into());
            }
            materials[id] = Material { surface, seed, tint: (rgb[0], rgb[1], rgb[2]) };
        }
        Ok(Scene { grid, materials })
    }

    /// Material lookup for `render_frame_with`.
    pub fn material(&self, tile: Tile) -> Material {
        self.materials[(tile.0 as usize) & 7]
    }

    /// Re-serialize (round-trips through `parse`). Lets the substrate side
    /// and this side agree on identical bytes.
    pub fn to_text(&self) -> String {
        let mut out = String::from("v1\n");
        out.push_str(&format!("grid {}x{}\n", self.grid.width, self.grid.height));
        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                out.push(match self.grid.at(x as i32, y as i32) {
                    Tile::FLOOR => '.',
                    Tile::BEDROOM => '1',
                    Tile::KITCHEN => '2',
                    Tile::DOORFRAME => '3',
                    Tile::BATHROOM => '4',
                    _ => '0',
                });
            }
            out.push('\n');
        }
        for (id, m) in self.materials.iter().enumerate() {
            if id == 0 { continue; }
            out.push_str(&format!("mat {} {} {} {},{},{}\n",
                id, surface_name(m.surface), m.seed, m.tint.0, m.tint.1, m.tint.2));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCENE: &str = "v1\n\
        grid 6x4\n\
        111111\n\
        1....1\n\
        1.22.1\n\
        111111\n\
        mat 1 wood 7 196,150,96\n\
        mat 2 marble 11 210,210,200\n";

    #[test]
    fn parses_grid_and_materials() {
        let s = Scene::parse(SCENE).unwrap();
        assert_eq!(s.grid.width, 6);
        assert_eq!(s.grid.height, 4);
        assert_eq!(s.material(Tile::BEDROOM).surface, Surface::Wood);
        assert_eq!(s.material(Tile::KITCHEN).surface, Surface::Marble);
        assert_eq!(s.material(Tile::BEDROOM).seed, 7);
    }

    #[test]
    fn round_trips_through_text() {
        let s = Scene::parse(SCENE).unwrap();
        let again = Scene::parse(&s.to_text()).unwrap();
        // grid identical
        for y in 0..4 {
            for x in 0..6 {
                assert_eq!(s.grid.at(x, y), again.grid.at(x, y));
            }
        }
        // materials identical
        assert_eq!(s.material(Tile::KITCHEN).surface, again.material(Tile::KITCHEN).surface);
        assert_eq!(s.material(Tile::KITCHEN).seed, again.material(Tile::KITCHEN).seed);
    }

    #[test]
    fn bad_version_refuses() {
        assert!(Scene::parse("v2\ngrid 1x1\n1\n").is_err());
    }

    #[test]
    fn wrong_row_width_refuses() {
        assert!(Scene::parse("v1\ngrid 4x1\n11\n").is_err());
    }
}
