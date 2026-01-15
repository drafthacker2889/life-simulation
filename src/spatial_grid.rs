pub struct SpatialGrid {
    cell_size: f64,
    cols: usize,
    rows: usize,
    // A 1D Vector of Vectors. Index = y * cols + x
    cells: Vec<Vec<usize>>,
}

impl SpatialGrid {
    pub fn new(width: f64, height: f64, cell_size: f64) -> SpatialGrid {
        let cols = (width / cell_size).ceil() as usize;
        let rows = (height / cell_size).ceil() as usize;
        let cells = vec![Vec::new(); cols * rows];
        
        SpatialGrid { cell_size, cols, rows, cells }
    }

    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.clear();
        }
    }

    pub fn insert(&mut self, x: f64, y: f64, index: usize) {
        let col = (x / self.cell_size).floor() as usize;
        let row = (y / self.cell_size).floor() as usize;
        
        if col < self.cols && row < self.rows {
            self.cells[row * self.cols + col].push(index);
        }
    }

    pub fn query(&self, x: f64, y: f64) -> Vec<usize> {
        let mut neighbors = Vec::new();
        let col_idx = (x / self.cell_size).floor() as i32;
        let row_idx = (y / self.cell_size).floor() as i32;

        for dy in -1..=1 {
            for dx in -1..=1 {
                let c = col_idx + dx;
                let r = row_idx + dy;
                
                if c >= 0 && c < self.cols as i32 && r >= 0 && r < self.rows as i32 {
                    let idx = (r as usize) * self.cols + (c as usize);
                    neighbors.extend(&self.cells[idx]);
                }
            }
        }
        neighbors
    }
}