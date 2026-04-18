//! Space primitive.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 { pub x: f32, pub y: f32 }

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self { Self { x, y } }
    pub fn from_angle(angle: f32) -> Self { Self { x: angle.cos(), y: angle.sin() } }
    pub fn length(self) -> f32 { (self.x * self.x + self.y * self.y).sqrt() }
    pub fn rotate(self, angle: f32) -> Self {
        let (s, c) = (angle.sin(), angle.cos());
        Self { x: self.x * c - self.y * s, y: self.x * s + self.y * c }
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, o: Self) -> Self { Self::new(self.x + o.x, self.y + o.y) }
}
impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, o: Self) -> Self { Self::new(self.x - o.x, self.y - o.y) }
}
impl std::ops::Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, s: f32) -> Self { Self::new(self.x * s, self.y * s) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tile(pub u8);

impl Tile {
    pub const FLOOR: Tile = Tile(0);
    pub const BEDROOM: Tile = Tile(1);
    pub const KITCHEN: Tile = Tile(2);
    pub const DOORFRAME: Tile = Tile(3);
    pub const BATHROOM: Tile = Tile(4);
    pub fn is_solid(self) -> bool { self.0 != 0 }
}

pub struct TileGrid {
    pub width: usize,
    pub height: usize,
    cells: Vec<Tile>,
}

impl TileGrid {
    pub fn from_ascii(rows: &[&str]) -> Self {
        let height = rows.len();
        let width = rows.first().map(|r| r.len()).unwrap_or(0);
        let mut cells = Vec::with_capacity(width * height);
        for row in rows {
            assert_eq!(row.len(), width, "ascii rows must all be same length");
            for ch in row.chars() {
                cells.push(match ch {
                    '.' | 'R' | 'P' => Tile::FLOOR,
                    '1' => Tile::BEDROOM,
                    '2' => Tile::KITCHEN,
                    '3' => Tile::DOORFRAME,
                    '4' => Tile::BATHROOM,
                    other => panic!("unknown tile char: {}", other),
                });
            }
        }
        Self { width, height, cells }
    }
    pub fn at(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            Tile::BEDROOM
        } else {
            self.cells[y as usize * self.width + x as usize]
        }
    }
    pub fn is_solid_at(&self, x: f32, y: f32) -> bool {
        self.at(x.floor() as i32, y.floor() as i32).is_solid()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RayHit {
    pub perp_dist: f32,
    pub tile: Tile,
    pub side: RaySide,
    pub hit_x: f32,
    pub hit_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaySide { NorthSouth, EastWest }

pub fn cast_ray(
    grid: &TileGrid,
    origin: Vec2, dir: Vec2, plane: Vec2,
    camera_x: f32, max_steps: usize,
) -> RayHit {
    let ray_dir = Vec2::new(dir.x + plane.x * camera_x, dir.y + plane.y * camera_x);
    let mut map_x = origin.x.floor() as i32;
    let mut map_y = origin.y.floor() as i32;
    let delta_dist_x = if ray_dir.x == 0.0 { f32::INFINITY } else { (1.0 / ray_dir.x).abs() };
    let delta_dist_y = if ray_dir.y == 0.0 { f32::INFINITY } else { (1.0 / ray_dir.y).abs() };
    let (step_x, mut side_dist_x) = if ray_dir.x < 0.0 {
        (-1, (origin.x - map_x as f32) * delta_dist_x)
    } else {
        (1, (map_x as f32 + 1.0 - origin.x) * delta_dist_x)
    };
    let (step_y, mut side_dist_y) = if ray_dir.y < 0.0 {
        (-1, (origin.y - map_y as f32) * delta_dist_y)
    } else {
        (1, (map_y as f32 + 1.0 - origin.y) * delta_dist_y)
    };
    let mut side = RaySide::NorthSouth;
    let mut tile = Tile::FLOOR;
    for _ in 0..max_steps {
        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
            side = RaySide::NorthSouth;
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
            side = RaySide::EastWest;
        }
        tile = grid.at(map_x, map_y);
        if tile.is_solid() { break; }
    }
    let perp_dist = match side {
        RaySide::NorthSouth => side_dist_x - delta_dist_x,
        RaySide::EastWest => side_dist_y - delta_dist_y,
    }.max(0.01);
    let hit_x = origin.x + ray_dir.x * perp_dist;
    let hit_y = origin.y + ray_dir.y * perp_dist;
    RayHit { perp_dist, tile, side, hit_x, hit_y }
}
