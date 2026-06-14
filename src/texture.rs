//! Procedural surface math — the "art from a seed" layer.
//!
//! Every wall/floor surface gets its color from a function evaluated at
//! the ray-hit coordinate, not from a stored image. A `Material` is a
//! few bytes (kind + seed + tint); `sample` turns (u, v) into a color.
//! This is the substrate principle applied to art: tiny recipe, rich
//! surface, computed per pixel at render time.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface { Wood, Marble, Tile, Plaster, Brick }

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub surface: Surface,
    pub seed: u32,
    pub tint: (u8, u8, u8),
}

#[inline]
fn hash2(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374761393)
        ^ (y as u32).wrapping_mul(668265263)
        ^ seed.wrapping_mul(2246822519);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^= h >> 16;
    (h as f32) / (u32::MAX as f32)
}

#[inline]
fn vnoise(u: f32, v: f32, seed: u32) -> f32 {
    let (xi, yi) = (u.floor() as i32, v.floor() as i32);
    let (fx, fy) = (u - u.floor(), v - v.floor());
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let a = hash2(xi, yi, seed) * (1.0 - sx) + hash2(xi + 1, yi, seed) * sx;
    let b = hash2(xi, yi + 1, seed) * (1.0 - sx) + hash2(xi + 1, yi + 1, seed) * sx;
    a * (1.0 - sy) + b * sy
}

fn fbm(u: f32, v: f32, seed: u32) -> f32 {
    let mut s = 0.0;
    let mut amp = 0.5;
    let mut f = 1.0;
    for o in 0..5 {
        s += amp * vnoise(u * f, v * f, seed.wrapping_add(o));
        amp *= 0.5;
        f *= 2.0;
    }
    s
}

fn mix(c: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    (
        (c.0 as f32 * t).clamp(0.0, 255.0) as u8,
        (c.1 as f32 * t).clamp(0.0, 255.0) as u8,
        (c.2 as f32 * t).clamp(0.0, 255.0) as u8,
    )
}

/// Sample a material at surface coordinate (u, v) in world units.
pub fn sample(m: &Material, u: f32, v: f32) -> (u8, u8, u8) {
    match m.surface {
        Surface::Wood => {
            let warp = fbm(u * 0.5, v * 0.4, m.seed) * 3.0;
            let rings = ((u * 1.6 + warp).sin() * 0.5 + 0.5).powf(0.55);
            mix(m.tint, 0.55 + rings * 0.6)
        }
        Surface::Marble => {
            let turb = fbm(u, v, m.seed) * 4.0;
            let veins = ((u * 2.0 + turb) * std::f32::consts::PI).sin().abs();
            mix(m.tint, 1.0 - veins * 0.75)
        }
        Surface::Tile => {
            let (gu, gv) = ((u * 3.0).fract(), (v * 3.0).fract());
            if gu < 0.07 || gv < 0.07 || gu > 0.93 || gv > 0.93 {
                mix(m.tint, 0.30) // grout line
            } else {
                let c = (((u * 3.0).floor() as i32 + (v * 3.0).floor() as i32) % 2) as f32;
                mix(m.tint, 0.80 + c * 0.30)
            }
        }
        Surface::Plaster => {
            let n = fbm(u * 2.0, v * 2.0, m.seed);
            mix(m.tint, 0.85 + n * 0.25)
        }
        Surface::Brick => {
            let row = (v * 3.0).floor();
            let off = if (row as i32) % 2 == 0 { 0.0 } else { 0.5 };
            let (bu, bv) = (((u * 3.0 + off).fract()), (v * 3.0).fract());
            if bv < 0.10 || bu < 0.06 || bu > 0.94 {
                mix(m.tint, 0.45) // mortar
            } else {
                let j = fbm(u * 4.0, v * 4.0, m.seed) * 0.3;
                mix(m.tint, 0.80 + j)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mat(s: Surface) -> Material { Material { surface: s, seed: 7, tint: (180, 140, 90) } }

    #[test]
    fn sample_is_deterministic() {
        let m = mat(Surface::Wood);
        assert_eq!(sample(&m, 1.3, 2.1), sample(&m, 1.3, 2.1));
    }

    #[test]
    fn sample_varies_across_surface() {
        // a real texture is not flat: distinct coords give distinct colors
        let m = mat(Surface::Marble);
        let a = sample(&m, 0.2, 0.5);
        let b = sample(&m, 2.7, 1.1);
        assert_ne!(a, b);
    }

    #[test]
    fn seed_changes_the_pattern() {
        let m1 = Material { surface: Surface::Wood, seed: 1, tint: (180, 140, 90) };
        let m2 = Material { surface: Surface::Wood, seed: 2, tint: (180, 140, 90) };
        // same coordinate, different seed -> generally different color
        let mut diffs = 0;
        for i in 0..20 {
            let u = i as f32 * 0.37;
            if sample(&m1, u, 1.0) != sample(&m2, u, 1.0) { diffs += 1; }
        }
        assert!(diffs > 10, "seed should vary the pattern (got {diffs}/20)");
    }

    #[test]
    fn all_surfaces_produce_valid_color() {
        for s in [Surface::Wood, Surface::Marble, Surface::Tile, Surface::Plaster, Surface::Brick] {
            let c = sample(&mat(s), 1.1, 0.7);
            let _ = c; // u8 tuple is always valid; just exercise every arm
        }
    }
}
