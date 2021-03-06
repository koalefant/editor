use crate::math::Rect;
use glam::{ivec2, IVec2, Vec2};
use tracy_client::span;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Grid<T: Copy> {
    #[serde(default)]
    pub default_value: T,
    pub bounds: [IVec2; 2],
    pub cells: Vec<T>,
}

impl<T> Grid<T>
where
    T: Copy + Default + PartialEq,
{
    pub fn new(default_value: T) -> Grid<T> {
        Grid {
            default_value,
            bounds: Rect::zero(),
            cells: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.bounds = Rect::zero();
        self.cells.clear();
    }

    pub fn size(&self) -> IVec2 {
        self.bounds.size()
    }

    pub fn find_used_bounds(&self) -> [IVec2; 2] {
        let _span = span!("Grid::find_used_bounds");
        let mut b = self.bounds;
        for x in (b[0].x..b[1].x).rev() {
            let mut used = false;
            for y in self.bounds[0].y..self.bounds[1].y {
                if self.cells[self.grid_pos_index(x, y)] != self.default_value {
                    used = true;
                    break;
                }
            }
            if used {
                break;
            }
            b[1].x = x + 1;
        }

        for x in b[0].x..b[1].x {
            let mut used = false;
            for y in b[0].y..b[1].y {
                if self.cells[self.grid_pos_index(x, y)] != self.default_value {
                    used = true;
                    break;
                }
            }
            if used {
                break;
            }
            b[0].x = x + 1;
        }

        for y in (b[0].y..b[1].y).rev() {
            let mut used = false;
            for x in b[0].x..b[1].x {
                if self.cells[self.grid_pos_index(x, y)] != self.default_value {
                    used = true;
                    break;
                }
            }
            if used {
                break;
            }
            b[1].y = y + 1;
        }

        for y in b[0].y..b[1].y {
            let mut used = false;
            for x in b[0].x..b[1].x {
                if self.cells[self.grid_pos_index(x, y)] != self.default_value {
                    used = true;
                    break;
                }
            }
            if used {
                break;
            }
            b[0].y = y + 1;
        }

        b
    }

    pub fn resize(&mut self, new_bounds: [IVec2; 2]) {
        if self.bounds == new_bounds {
            return;
        }
        let old_bounds = self.bounds;
        let old_size = old_bounds.size();
        let new_size = new_bounds.size();
        let mut new_cells = vec![self.default_value; new_size[0] as usize * new_size[1] as usize];
        let common = old_bounds.intersect(new_bounds).unwrap_or(Rect::zero());
        let y_range = common[0].y..common[1].y;
        let x_range = common[0].x..common[1].x;
        for y in y_range {
            let old_start =
                ((y - old_bounds[0].y) * old_size.x + (x_range.start - old_bounds[0].x)) as usize;
            let new_start =
                ((y - new_bounds[0].y) * new_size.x + (x_range.start - new_bounds[0].x)) as usize;
            let old_range = old_start..old_start + x_range.len();
            let new_range = new_start..new_start + x_range.len();
            new_cells[new_range].copy_from_slice(&self.cells[old_range]);
        }
        self.bounds = new_bounds;
        assert!(self.bounds.contains(new_bounds));
        self.cells = new_cells;
    }

    pub fn resize_to_include_amortized(&mut self, bounds: [IVec2; 2]) {
        if self.bounds.contains(bounds) {
            return;
        }
        let tile_size_cells = 64;
        let tile_l = bounds[0].x.div_euclid(tile_size_cells);
        let tile_t = bounds[0].y.div_euclid(tile_size_cells);
        let tile_r = bounds[1].x.div_euclid(tile_size_cells);
        let tile_b = bounds[1].y.div_euclid(tile_size_cells);

        let tile_bounds = [
            ivec2(tile_l * tile_size_cells, tile_t * tile_size_cells),
            ivec2(
                (tile_r + 1) * tile_size_cells,
                (tile_b + 1) * tile_size_cells,
            ),
        ];

        let bounds = self.bounds.union(tile_bounds);

        self.resize(bounds);
    }

    pub fn world_to_grid_pos(&self, point: Vec2, cell_size: i32) -> anyhow::Result<IVec2, IVec2> {
        let grid_pos = point / Vec2::splat(cell_size as f32);
        let pos = grid_pos.floor().as_ivec2();
        if !self.bounds.contains_point(pos) {
            return Err(pos);
        }
        Ok(pos)
    }
    pub fn world_to_grid_rect(rect: [Vec2; 2], cell_size: i32) -> [IVec2; 2] {
        let start = rect[0] / Vec2::splat(cell_size as f32);
        let end = rect[1] / Vec2::splat(cell_size as f32);
        let start = start.floor().as_ivec2();
        let end = end.ceil().as_ivec2();
        [start, end]
    }

    pub fn flood_fill(cells: &mut [T], rect: [IVec2; 2], start: IVec2, value: T, empty_value: T) {
        let size = rect.size();
        let w = size.x;
        let h = size.y;
        let start_x = start.x - rect[0].x;
        let start_y = start.y - rect[0].y;
        let old_value = cells[(start_y * w + start_x) as usize];
        if old_value == value {
            return;
        }
        let mut stack = Vec::new();
        stack.push([start_x, start_y]);
        let fill_diagonals = old_value != empty_value;
        while let Some([mut x, y]) = stack.pop() {
            while x >= 0 && cells[(y * w + x) as usize] == old_value {
                x -= 1;
            }
            let mut span_above = false;
            let mut span_below = false;

            if fill_diagonals && x > 0 {
                if y > 0 && cells[((y - 1) * w + x) as usize] == old_value {
                    stack.push([x, y - 1]);
                    span_above = true;
                }
                if y < h - 1 && cells[((y + 1) * w + x) as usize] == old_value {
                    stack.push([x, y + 1]);
                    span_above = true;
                }
            }
            x += 1;

            while x < w && cells[(y * w + x) as usize] == old_value {
                cells[(y * w + x) as usize] = value;
                if !span_above && y > 0 && cells[((y - 1) * w + x) as usize] == old_value {
                    stack.push([x, y - 1]);
                    span_above = true;
                } else if span_above && y > 0 && cells[((y - 1) * w + x) as usize] != old_value {
                    span_above = false;
                }

                if !span_below && y < h - 1 && cells[((y + 1) * w + x) as usize] == old_value {
                    stack.push([x, y + 1]);
                    span_below = true;
                } else if span_below && y < h - 1 && cells[((y + 1) * w + x) as usize] != old_value
                {
                    span_below = false;
                }
                x += 1;
            }

            if fill_diagonals && x < w {
                if !span_above && y > 0 && cells[((y - 1) * w + x) as usize] == old_value {
                    stack.push([x, y - 1]);
                }
                if !span_below && y < h - 1 && cells[((y + 1) * w + x) as usize] == old_value {
                    stack.push([x, y + 1]);
                }
            }
        }
    }

    pub fn grid_pos_index(&self, x: i32, y: i32) -> usize {
        ((y - self.bounds[0].y) * (self.bounds[1].x - self.bounds[0].x) + x - self.bounds[0].x)
            as usize
    }

    pub fn rectangle_outline(&mut self, [min, max]: [IVec2; 2], value: T) {
        let l = min.x;
        let r = max.x;
        let t = min.y;
        let b = max.y;
        for x in l..r {
            let index = self.grid_pos_index(x, t);
            self.cells[index] = value;
        }

        for y in t..b {
            let index = self.grid_pos_index(l, y);
            self.cells[index] = value;
            let index = self.grid_pos_index(r - 1, y);
            self.cells[index] = value;
        }
        for x in l..r {
            let index = self.grid_pos_index(x, b - 1);
            self.cells[index] = value;
        }
    }
}
