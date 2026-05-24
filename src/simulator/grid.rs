use rand::Rng;
use rand::rngs::ThreadRng;

use crate::simulator::element::{Cell, ElementType};

pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
    pub generation: u8,
    pub row_active: Vec<bool>,
    pub next_row_active: Vec<bool>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![Cell::new(ElementType::Air); width * height];
        let row_active = vec![true; height];
        let next_row_active = vec![false; height];
        Grid {
            width,
            height,
            cells,
            generation: 0,
            row_active,
            next_row_active,
        }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Cell {
        self.cells[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        self.cells[y * self.width + x] = cell;
    }

    pub fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            *cell = Cell::new(ElementType::Air);
        }
        self.row_active.fill(false);
    }

    #[inline]
    fn activate_row(&mut self, y: usize) {
        self.next_row_active[y] = true;
        if y > 0 {
            self.next_row_active[y - 1] = true;
        }
        if y + 1 < self.height {
            self.next_row_active[y + 1] = true;
        }
    }

    pub fn draw_circle(&mut self, cx: usize, cy: usize, radius: usize, element: ElementType) {
        let r = radius as i32;
        let cx = cx as i32;
        let cy = cy as i32;

        for dy in -r..=r {
            let py = cy + dy;
            if py < 0 || py >= self.height as i32 {
                continue;
            }
            let ty = py as usize;
            
            // Wake up row and neighbors on draw
            self.row_active[ty] = true;
            if ty > 0 {
                self.row_active[ty - 1] = true;
            }
            if ty + 1 < self.height {
                self.row_active[ty + 1] = true;
            }

            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    let px = cx + dx;
                    if px >= 0 && px < self.width as i32 {
                        let tx = px as usize;
                        let current_elem = self.get(tx, ty).element;
                        
                        // Prevent overwriting Stone walls unless drawing Stone or Air (Eraser)
                        if current_elem == ElementType::Stone {
                            if element == ElementType::Stone || element == ElementType::Air {
                                self.set(tx, ty, Cell::new(element));
                            }
                        } else if element.is_gas() && current_elem != ElementType::Air && !current_elem.is_flammable() {
                            // Prevent gas elements (Fire/Smoke/Steam) from deleting solid/liquid targets (Lava/Sand/Water/Ice)
                        } else {
                            self.set(tx, ty, Cell::new(element));
                        }
                    }
                }
            }
        }
    }

    pub fn get_color_buffer(&self, buffer: &mut [u8]) {
        for (i, cell) in self.cells.iter().enumerate() {
            let offset = i * 4;
            let mut color = cell.color;
            if cell.element == ElementType::Fire {
                let mut rng = rand::thread_rng();
                color[1] = (color[1] as i16 + rng.gen_range(-15..15)).clamp(50, 220) as u8;
                color[2] = (color[2] as i16 + rng.gen_range(-5..5)).clamp(0, 100) as u8;
            } else if cell.element == ElementType::Lava {
                let mut rng = rand::thread_rng();
                color[0] = (color[0] as i16 + rng.gen_range(-10..10)).clamp(200, 255) as u8;
                color[1] = (color[1] as i16 + rng.gen_range(-10..10)).clamp(40, 110) as u8;
            }

            buffer[offset] = color[0];
            buffer[offset + 1] = color[1];
            buffer[offset + 2] = color[2];
            buffer[offset + 3] = 255;
        }
    }

    pub fn generate_terrain(&mut self) {
        self.clear();
        let mut rng = rand::thread_rng();
        
        // 1. Bedrock contours
        for y in 0..self.height {
            for x in 0..self.width {
                if y == self.height - 1 || x == 0 || x == self.width - 1 {
                    self.set(x, y, Cell::new(ElementType::Stone));
                    continue;
                }

                let platform_height = (self.height as f32 * 0.72) 
                    + ((x as f32 * 0.045).sin() * 8.0) 
                    + ((x as f32 * 0.12).cos() * 4.0);
                
                if y as f32 > platform_height && y < self.height - 5 {
                    if rng.gen_bool(0.75) {
                        self.set(x, y, Cell::new(ElementType::Stone));
                    } else {
                        self.set(x, y, Cell::new(ElementType::Gravel));
                    }
                    continue;
                }

                if y as f32 > platform_height + 5.0 && rng.gen_bool(0.06) {
                    self.draw_circle(x, y, 2, ElementType::Coal);
                }
            }
        }

        // Platforms
        let left_plat_y = self.height * 2 / 5;
        for x in 10..45 {
            self.set(x, left_plat_y, Cell::new(ElementType::Wood));
            if x == 10 || x == 44 {
                for dy in 1..5 {
                    self.set(x, left_plat_y - dy, Cell::new(ElementType::Wood));
                }
            }
        }
        for x in 11..44 {
            for y in (left_plat_y - 4)..left_plat_y {
                self.set(x, y, Cell::new(ElementType::Oil));
            }
        }

        let center_plat_y = self.height * 3 / 5;
        for x in (self.width / 2 - 25)..(self.width / 2 + 25) {
            self.set(x, center_plat_y, Cell::new(ElementType::Wood));
            if x == (self.width / 2 - 25) || x == (self.width / 2 + 24) {
                for dy in 1..6 {
                    self.set(x, center_plat_y - dy, Cell::new(ElementType::Wood));
                }
            }
        }
        let box_cx = self.width / 2;
        let box_cy = center_plat_y - 1;
        for dy in 0..5 {
            for dx in -12..12 {
                let px = (box_cx as i32 + dx) as usize;
                let py = (box_cy as i32 - dy) as usize;
                
                if dx == -12 || dx == 11 || dy == 4 || dy == 0 {
                    self.set(px, py, Cell::new(ElementType::Wood));
                } else {
                    self.set(px, py, Cell::new(ElementType::Gunpowder));
                }
            }
        }

        let right_plat_y = self.height * 2 / 5;
        for x in (self.width - 45)..(self.width - 10) {
            self.set(x, right_plat_y, Cell::new(ElementType::Stone));
            if x == (self.width - 45) || x == (self.width - 11) {
                for dy in 1..5 {
                    self.set(x, right_plat_y - dy, Cell::new(ElementType::Stone));
                }
            }
        }
        for x in (self.width - 44)..(self.width - 11) {
            for y in (right_plat_y - 4)..right_plat_y {
                if x % 4 == 0 || y % 2 == 0 {
                    self.set(x, y, Cell::new(ElementType::Ice));
                } else {
                    self.set(x, y, Cell::new(ElementType::Water));
                }
            }
        }

        let pool_y_start = self.height * 4 / 5;
        for y in pool_y_start..(self.height - 6) {
            for x in (self.width / 3)..(self.width * 2 / 3) {
                let current_elem = self.get(x, y).element;
                if current_elem == ElementType::Air || current_elem == ElementType::Gravel {
                    self.set(x, y, Cell::new(ElementType::Water));
                }
            }
        }

        for _ in 0..12 {
            let gx = rng.gen_range(5..(self.width - 5));
            let gy = rng.gen_range((self.height - 4)..self.height - 1);
            self.draw_circle(gx, gy, 1, ElementType::Gold);
        }
        
        for _ in 0..8 {
            let gx = rng.gen_range(5..(self.width - 5));
            let gy = rng.gen_range((self.height - 12)..(self.height - 4));
            self.draw_circle(gx, gy, 2, ElementType::Gravel);
        }

        self.row_active.fill(true);
    }

    pub fn tick(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        let mut rng = rand::thread_rng();

        self.next_row_active.fill(false);

        // Scan from bottom to top so falling items update correctly
        for y in (0..self.height).rev() {
            // Simplified row check (neighbor activation was handled during write phase)
            if !self.row_active[y] {
                continue;
            }

            let left_to_right = rng.gen_bool(0.5);
            if left_to_right {
                for x in 0..self.width {
                    self.update_cell(x, y, &mut rng);
                }
            } else {
                for x in (0..self.width).rev() {
                    self.update_cell(x, y, &mut rng);
                }
            }
        }

        std::mem::swap(&mut self.row_active, &mut self.next_row_active);
    }

    #[inline]
    fn check_fire_neighbor(&mut self, idx: usize, y: usize, nx: usize, ny: usize, n_idx: usize, rng: &mut ThreadRng) -> Option<bool> {
        let n_cell = self.cells[n_idx];
        if n_cell.element.is_flammable() {
            self.activate_row(ny);
            match n_cell.element {
                ElementType::Gunpowder => {
                    self.explode(nx, ny, 10);
                    return Some(true); // Exploded
                }
                ElementType::Oil => {
                    if rng.gen_bool(0.4) {
                        self.cells[n_idx] = Cell::new(ElementType::Fire);
                    }
                }
                ElementType::Wood => {
                    if rng.gen_bool(0.12) {
                        self.cells[n_idx] = Cell::new(ElementType::Fire);
                    }
                }
                ElementType::Coal => {
                    if rng.gen_bool(0.04) {
                        self.cells[n_idx] = Cell::new(ElementType::Fire);
                    }
                }
                _ => {}
            }
        } else if n_cell.element == ElementType::Ice {
            self.activate_row(ny);
            if rng.gen_bool(0.2) {
                self.cells[n_idx] = Cell::new(ElementType::Water);
            }
        } else if n_cell.element == ElementType::Water {
            // Fire gets extinguished
            self.cells[idx] = Cell::new(ElementType::Steam);
            self.activate_row(y);
            return Some(true); // Extinguished
        } else if n_cell.element.is_falling_solid() || n_cell.element == ElementType::Lava {
            // Smothered
            self.cells[idx] = Cell::new(ElementType::Smoke);
            self.activate_row(y);
            return Some(true); // Smothered
        }
        None
    }

    #[inline]
    fn check_lava_neighbor(&mut self, idx: usize, nx: usize, ny: usize, n_idx: usize) -> Option<bool> {
        let n_cell = self.cells[n_idx];
        if n_cell.element.is_flammable() {
            self.activate_row(ny);
            if n_cell.element == ElementType::Gunpowder {
                self.explode(nx, ny, 10);
                return Some(true); // Exploded
            } else {
                self.cells[n_idx] = Cell::new(ElementType::Fire);
            }
        } else if n_cell.element == ElementType::Ice {
            self.activate_row(ny);
            self.cells[n_idx] = Cell::new(ElementType::Water);
        } else if n_cell.element == ElementType::Water {
            self.activate_row(ny);
            self.cells[idx] = Cell::new(ElementType::Stone);
            self.cells[n_idx] = Cell::new(ElementType::Steam);
            return Some(true); // Hardened (Lava turns to Stone)
        }
        None
    }

    fn update_cell(&mut self, x: usize, y: usize, rng: &mut ThreadRng) {
        let idx = y * self.width + x;
        let mut cell = self.cells[idx];
        
        // Skip empty cells, static solids, or already processed cells
        if cell.element == ElementType::Air 
            || cell.element.is_static_solid() 
            || cell.generation == self.generation 
        {
            return;
        }

        // 1. Reactions & State Updates
        match cell.element {
            ElementType::Fire => {
                self.activate_row(y);

                if cell.life == 0 {
                    if rng.gen_bool(0.4) {
                        self.cells[idx] = Cell::new(ElementType::Smoke);
                    } else {
                        self.cells[idx] = Cell::new(ElementType::Air);
                    }
                    return;
                }
                cell.life -= 1;
                self.cells[idx] = cell;

                // Ignite flammable neighbors with unrolled boundary checks (no index multiplications)
                if x > 0 {
                    if let Some(stop) = self.check_fire_neighbor(idx, y, x - 1, y, idx - 1, rng) {
                        if stop { return; }
                    }
                }
                if x + 1 < self.width {
                    if let Some(stop) = self.check_fire_neighbor(idx, y, x + 1, y, idx + 1, rng) {
                        if stop { return; }
                    }
                }
                if y > 0 {
                    if let Some(stop) = self.check_fire_neighbor(idx, y, x, y - 1, idx - self.width, rng) {
                        if stop { return; }
                    }
                }
                if y + 1 < self.height {
                    if let Some(stop) = self.check_fire_neighbor(idx, y, x, y + 1, idx + self.width, rng) {
                        if stop { return; }
                    }
                }
            }

            ElementType::Smoke => {
                self.activate_row(y);

                if cell.life == 0 {
                    self.cells[idx] = Cell::new(ElementType::Air);
                    return;
                }
                cell.life -= 1;
                self.cells[idx] = cell;
            }

            ElementType::Steam => {
                self.activate_row(y);

                if cell.life == 0 {
                    if rng.gen_bool(0.2) {
                        self.cells[idx] = Cell::new(ElementType::Water);
                    } else {
                        self.cells[idx] = Cell::new(ElementType::Air);
                    }
                    return;
                }
                cell.life -= 1;
                self.cells[idx] = cell;
            }

            ElementType::Acid => {
                self.activate_row(y);

                if cell.life == 0 {
                    self.cells[idx] = Cell::new(ElementType::Air);
                    return;
                }
                cell.life -= 1;
                self.cells[idx] = cell;

                // Corrosion: select one random neighbor directly without index multiplications
                let dir = rng.gen_range(0..4);
                let opt_target = match dir {
                    0 if x > 0 => Some((x - 1, y, idx - 1)),
                    1 if x + 1 < self.width => Some((x + 1, y, idx + 1)),
                    2 if y > 0 => Some((x, y - 1, idx - self.width)),
                    3 if y + 1 < self.height => Some((x, y + 1, idx + self.width)),
                    _ => None,
                };

                if let Some((nx, ny, n_idx)) = opt_target {
                    let n_cell = self.cells[n_idx];

                    if n_cell.element != ElementType::Air 
                        && n_cell.element != ElementType::Acid 
                        && n_cell.element != ElementType::Stone 
                    {
                        self.activate_row(ny);
                        if n_cell.element == ElementType::Gunpowder {
                            self.explode(nx, ny, 8);
                        } else {
                            self.cells[n_idx] = Cell::new(ElementType::Air);
                        }
                        self.cells[idx] = Cell::new(ElementType::Air);
                        return;
                    }
                }
            }

            ElementType::Lava => {
                self.activate_row(y);

                // Check neighbors with unrolled boundary checks (no index multiplications)
                if x > 0 {
                    if let Some(stop) = self.check_lava_neighbor(idx, x - 1, y, idx - 1) {
                        if stop { return; }
                    }
                }
                if x + 1 < self.width {
                    if let Some(stop) = self.check_lava_neighbor(idx, x + 1, y, idx + 1) {
                        if stop { return; }
                    }
                }
                if y > 0 {
                    if let Some(stop) = self.check_lava_neighbor(idx, x, y - 1, idx - self.width) {
                        if stop { return; }
                    }
                }
                if y + 1 < self.height {
                    if let Some(stop) = self.check_lava_neighbor(idx, x, y + 1, idx + self.width) {
                        if stop { return; }
                    }
                }
            }

            _ => {}
        }

        // Reload cell after reaction phase
        let cell = self.cells[idx];
        if cell.element == ElementType::Air || cell.element.is_static_solid() {
            return;
        }

        // 2. Physical movement
        if cell.element.is_falling_solid() {
            let dy = y + 1;
            if dy < self.height {
                // Down
                let down_cell = self.get(x, dy);
                if self.can_displace(cell.element, down_cell.element) {
                    self.move_cell(x, y, x, dy);
                    self.activate_row(y);
                    self.activate_row(dy);
                    return;
                }

                // Down-left / Down-right
                let go_left = rng.gen_bool(0.5);
                let offsets = if go_left { [-1, 1] } else { [1, -1] };
                for dx in offsets {
                    let tx = x as i32 + dx;
                    if tx >= 0 && tx < self.width as i32 {
                        let tx = tx as usize;
                        let diag_cell = self.get(tx, dy);
                        if self.can_displace(cell.element, diag_cell.element) {
                            self.move_cell(x, y, tx, dy);
                            self.activate_row(y);
                            self.activate_row(dy);
                            return;
                        }
                    }
                }
            }
        } else if cell.element.is_liquid() {
            let dy = y + 1;
            if dy < self.height {
                // Down
                let down_cell = self.get(x, dy);
                if self.can_displace(cell.element, down_cell.element) {
                    self.move_cell(x, y, x, dy);
                    self.activate_row(y);
                    self.activate_row(dy);
                    return;
                }

                // Down-left / Down-right
                let go_left = rng.gen_bool(0.5);
                let offsets = if go_left { [-1, 1] } else { [1, -1] };
                for dx in offsets {
                    let tx = x as i32 + dx;
                    if tx >= 0 && tx < self.width as i32 {
                        let tx = tx as usize;
                        let diag_cell = self.get(tx, dy);
                        if self.can_displace(cell.element, diag_cell.element) {
                            self.move_cell(x, y, tx, dy);
                            self.activate_row(y);
                            self.activate_row(dy);
                            return;
                        }
                    }
                }
            }

            // Flow left / right (horizontal spread)
            let go_left = rng.gen_bool(0.5);
            let offsets = if go_left { [-1, 1] } else { [1, -1] };
            for dx in offsets {
                let tx = x as i32 + dx;
                if tx >= 0 && tx < self.width as i32 {
                    let tx = tx as usize;
                    let side_cell = self.get(tx, y);
                    if self.can_displace(cell.element, side_cell.element) {
                        self.move_cell(x, y, tx, y);
                        self.activate_row(y);
                        return;
                    }
                }
            }
        } else if cell.element.is_gas() {
            if y > 0 {
                let uy = y - 1;
                // Up
                let up_cell = self.get(x, uy);
                if self.can_displace(cell.element, up_cell.element) {
                    self.move_cell(x, y, x, uy);
                    self.activate_row(y);
                    self.activate_row(uy);
                    return;
                }

                // Up-left / Up-right
                let go_left = rng.gen_bool(0.5);
                let offsets = if go_left { [-1, 1] } else { [1, -1] };
                for dx in offsets {
                    let tx = x as i32 + dx;
                    if tx >= 0 && tx < self.width as i32 {
                        let tx = tx as usize;
                        let diag_cell = self.get(tx, uy);
                        if self.can_displace(cell.element, diag_cell.element) {
                            self.move_cell(x, y, tx, uy);
                            self.activate_row(y);
                            self.activate_row(uy);
                            return;
                        }
                    }
                }
            }

            // Drift left / right
            let go_left = rng.gen_bool(0.5);
            let offsets = if go_left { [-1, 1] } else { [1, -1] };
            for dx in offsets {
                let tx = x as i32 + dx;
                if tx >= 0 && tx < self.width as i32 {
                    let tx = tx as usize;
                    let side_cell = self.get(tx, y);
                    if self.can_displace(cell.element, side_cell.element) {
                        self.move_cell(x, y, tx, y);
                        self.activate_row(y);
                        return;
                    }
                }
            }
        }
    }

    fn can_displace(&self, src: ElementType, dst: ElementType) -> bool {
        if dst.is_static_solid() {
            return false;
        }
        if src == dst {
            return false;
        }
        if src.is_gas() {
            // Fire flames cannot rise through liquids (only Smoke and Steam can bubble up)
            if src == ElementType::Fire {
                return dst == ElementType::Air;
            }
            return dst == ElementType::Air || dst.is_liquid();
        }
        src.density() > dst.density()
    }

    fn move_cell(&mut self, x: usize, y: usize, tx: usize, ty: usize) {
        let idx = y * self.width + x;
        let tidx = ty * self.width + tx;

        let mut c1 = self.cells[idx];
        let mut c2 = self.cells[tidx];

        c1.generation = self.generation;
        c2.generation = self.generation;

        self.cells[tidx] = c1;
        self.cells[idx] = c2;
    }

    fn explode(&mut self, cx: usize, cy: usize, radius: usize) {
        let r = radius as i32;
        let cx = cx as i32;
        let cy = cy as i32;
        let mut rng = rand::thread_rng();

        // Wake up rows in explosion radius
        for dy in -r..=r {
            let py = cy + dy;
            if py >= 0 && py < self.height as i32 {
                self.activate_row(py as usize);
            }
        }

        for dy in -r..=r {
            for dx in -r..=r {
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= r * r {
                    let px = cx + dx;
                    let py = cy + dy;
                    if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                        let tx = px as usize;
                        let ty = py as usize;
                        let t_idx = ty * self.width + tx;
                        let target = self.cells[t_idx];

                        if target.element == ElementType::Stone {
                            if dist_sq < 4 && rng.gen_bool(0.3) {
                                self.cells[t_idx] = Cell::new(ElementType::Fire);
                            }
                        } else if target.element == ElementType::Gunpowder {
                            self.cells[t_idx] = Cell::new(ElementType::Fire);
                            if rng.gen_bool(0.5) {
                                self.explode(tx, ty, radius / 2);
                            }
                        } else if target.element.is_falling_solid() 
                            || target.element == ElementType::Lava 
                            || target.element == ElementType::Acid
                        {
                            // Sand, Gravel, Gold, Lava, and Acid resist vaporization and are left intact
                        } else {
                            if target.element == ElementType::Water {
                                self.cells[t_idx] = Cell::new(ElementType::Steam);
                            } else {
                                if dist_sq < (r * r) / 2 {
                                    self.cells[t_idx] = Cell::new(ElementType::Fire);
                                } else {
                                    if rng.gen_bool(0.6) {
                                        self.cells[t_idx] = Cell::new(ElementType::Smoke);
                                    } else {
                                        self.cells[t_idx] = Cell::new(ElementType::Fire);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lava_fire_interaction() {
        let mut grid = Grid::new(3, 5);
        grid.set(1, 1, Cell::new(ElementType::Lava));
        grid.set(1, 2, Cell::new(ElementType::Fire));

        for _ in 0..10 {
            grid.tick();
        }

        let has_lava = grid.cells.iter().any(|c| c.element == ElementType::Lava);
        assert!(has_lava, "Lava should not disappear!");
    }

    #[test]
    fn test_sand_fire_interaction() {
        let mut grid = Grid::new(3, 5);
        grid.set(1, 1, Cell::new(ElementType::Sand));
        grid.set(1, 2, Cell::new(ElementType::Fire));

        for _ in 0..10 {
            grid.tick();
        }

        let has_sand = grid.cells.iter().any(|c| c.element == ElementType::Sand);
        assert!(has_sand, "Sand should not disappear!");
    }

    #[test]
    fn test_fire_above_lava() {
        let mut grid = Grid::new(3, 5);
        grid.set(1, 2, Cell::new(ElementType::Lava));
        grid.set(1, 1, Cell::new(ElementType::Fire));

        for _ in 0..10 {
            grid.tick();
        }

        let has_lava = grid.cells.iter().any(|c| c.element == ElementType::Lava);
        assert!(has_lava, "Lava should not disappear when Fire is above it!");
    }

    #[test]
    fn test_large_lava_fire_retention() {
        let mut grid = Grid::new(10, 10);
        let mut initial_lava = 0;
        for y in 0..5 {
            for x in 0..10 {
                grid.set(x, y, Cell::new(ElementType::Lava));
                initial_lava += 1;
            }
        }
        for y in 5..9 {
            for x in 0..10 {
                grid.set(x, y, Cell::new(ElementType::Fire));
            }
        }
        for x in 0..10 {
            grid.set(x, 9, Cell::new(ElementType::Stone));
        }

        for _ in 0..30 {
            grid.tick();
        }

        let final_lava = grid.cells.iter().filter(|c| c.element == ElementType::Lava).count();
        assert_eq!(initial_lava, final_lava, "Lava should not disappear during large-scale interaction with Fire!");
    }

    #[test]
    fn test_large_sand_fire_retention() {
        let mut grid = Grid::new(10, 10);
        let mut initial_sand = 0;
        for y in 0..5 {
            for x in 0..10 {
                grid.set(x, y, Cell::new(ElementType::Sand));
                initial_sand += 1;
            }
        }
        for y in 5..9 {
            for x in 0..10 {
                grid.set(x, y, Cell::new(ElementType::Fire));
            }
        }
        for x in 0..10 {
            grid.set(x, 9, Cell::new(ElementType::Stone));
        }

        for _ in 0..30 {
            grid.tick();
        }

        let final_sand = grid.cells.iter().filter(|c| c.element == ElementType::Sand).count();
        assert_eq!(initial_sand, final_sand, "Sand should not disappear during large-scale interaction with Fire!");
    }
}

