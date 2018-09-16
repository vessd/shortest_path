use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::ops::Index;
use std::ops::IndexMut;
use std::vec::IntoIter;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Cell {
    Passable,
    Impassable,
    Start,
    Finish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MapPos {
    pub x: usize,
    pub y: usize,
}

impl Ord for MapPos {
    fn cmp(&self, other: &MapPos) -> Ordering {
        self.x.cmp(&other.x).then_with(|| self.y.cmp(&other.y))
    }
}

impl PartialOrd for MapPos {
    fn partial_cmp(&self, other: &MapPos) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl MapPos {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PosState {
    position: MapPos,
    cost: f64,
}

impl Eq for PosState {}

impl Ord for PosState {
    fn cmp(&self, other: &PosState) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap()
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for PosState {
    fn partial_cmp(&self, other: &PosState) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
struct PosInfo {
    parent: MapPos,
    cost: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Map {
    cols: usize,
    data: Vec<Cell>,
    start: MapPos,
    finish: MapPos,
}

impl Index<usize> for Map {
    type Output = [Cell];

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        let i = (index + 1) * self.cols;
        &self.data[i + 1..i + self.cols - 1]
    }
}

impl IndexMut<usize> for Map {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let i = (index + 1) * self.cols;
        &mut self.data[i + 1..i + self.cols - 1]
    }
}

impl Map {
    pub fn new(rows: usize, cols: usize) -> Self {
        //Увеличиваем рамер карты, для того чтобы сделать стену по периметру,
        //это позволит не проверять границы при поиске соседей.
        if cols < 2 && rows < 2 {
            panic!("");
        }
        let start = MapPos::new(0, 0);
        let finish = MapPos::new(rows - 1, cols - 1);
        let cols = cols + 2;
        let rows = rows + 2;
        let data = vec![Cell::Passable; cols * rows];
        let mut map = Map {
            cols,
            data,
            start,
            finish,
        };
        //Стена слева и справа
        for i in 0..rows {
            *map.get_mut(i, 0) = Cell::Impassable;
            *map.get_mut(i, cols - 1) = Cell::Impassable;
        }
        //Стена сверху и снизу
        for j in 0..cols {
            *map.get_mut(0, j) = Cell::Impassable;
            *map.get_mut(rows - 1, j) = Cell::Impassable;
        }
        map[start.x][start.y] = Cell::Start;
        map[finish.x][finish.y] = Cell::Finish;
        map
    }

    fn get(&self, x: usize, y: usize) -> Cell {
        self.data[x * self.cols + y]
    }

    fn get_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.data[x * self.cols + y]
    }

    pub fn rows(&self) -> usize {
        self.data.len() / self.cols - 2
    }

    pub fn cols(&self) -> usize {
        self.cols - 2
    }

    pub fn set_cell(&mut self, cell: Cell, pos: MapPos) {
        if cell == Cell::Passable || self[pos.x][pos.y] == Cell::Passable {
            match cell {
                Cell::Passable => {
                    if self[pos.x][pos.y] == Cell::Impassable {
                        self[pos.x][pos.y] = Cell::Passable;
                    }
                }
                Cell::Impassable => {
                    self[pos.x][pos.y] = Cell::Impassable;
                }
                Cell::Start => {
                    let start = self.start;
                    self[start.x][start.y] = Cell::Passable;
                    self[pos.x][pos.y] = Cell::Start;
                    self.start = pos;
                }
                Cell::Finish => {
                    let finish = self.finish;
                    self[finish.x][finish.y] = Cell::Passable;
                    self[pos.x][pos.y] = Cell::Finish;
                    self.finish = pos;
                }
            }
        }
    }

    //евклидово расстояние
    fn distance(p: MapPos, q: MapPos) -> f64 {
        ((p.x as f64 - q.x as f64).powi(2) + (p.y as f64 - q.y as f64).powi(2)).sqrt()
    }

    fn reconstruct_path(&self, map: HashMap<MapPos, PosInfo>) -> Vec<MapPos> {
        if let Some(info) = map.get(&self.finish) {
            let mut vec = Vec::with_capacity(info.cost as usize);
            let mut current = self.finish;
            while let Some(info) = map.get(&current) {
                vec.push(current);
                if current == info.parent {
                    break;
                }
                current = info.parent;
            }
            vec
        } else {
            Vec::new()
        }
    }

    fn neighbors(&self, pos: MapPos) -> IntoIter<MapPos> {
        let mut vec = Vec::with_capacity(8);
        let mut s = [false; 4];
        let mut d = [false; 4];
        let pos = MapPos::new(pos.x + 1, pos.y + 1);
        if self.get(pos.x - 1, pos.y) != Cell::Impassable {
            s[0] = true;
            vec.push(MapPos::new(pos.x - 2, pos.y - 1));
        }
        if self.get(pos.x, pos.y + 1) != Cell::Impassable {
            s[1] = true;
            vec.push(MapPos::new(pos.x - 1, pos.y));
        }
        if self.get(pos.x + 1, pos.y) != Cell::Impassable {
            s[2] = true;
            vec.push(MapPos::new(pos.x, pos.y - 1));
        }
        if self.get(pos.x, pos.y - 1) != Cell::Impassable {
            s[3] = true;
            vec.push(MapPos::new(pos.x - 1, pos.y - 2));
        }

        d[0] = s[3] || s[0];
        d[1] = s[0] || s[1];
        d[2] = s[1] || s[2];
        d[3] = s[2] || s[3];

        if d[0] && self.get(pos.x - 1, pos.y - 1) != Cell::Impassable {
            vec.push(MapPos::new(pos.x - 2, pos.y - 2));
        }
        if d[1] && self.get(pos.x - 1, pos.y + 1) != Cell::Impassable {
            vec.push(MapPos::new(pos.x - 2, pos.y));
        }
        if d[2] && self.get(pos.x + 1, pos.y + 1) != Cell::Impassable {
            vec.push(MapPos::new(pos.x, pos.y));
        }
        if d[3] && self.get(pos.x + 1, pos.y - 1) != Cell::Impassable {
            vec.push(MapPos::new(pos.x, pos.y - 2));
        }
        vec.into_iter()
    }

    pub fn shortest_path(&self) -> Vec<MapPos> {
        let mut heap = BinaryHeap::new();
        let mut map = HashMap::new();
        heap.push(PosState {
            position: self.start,
            cost: Map::distance(self.start, self.finish),
        });
        map.insert(
            self.start,
            PosInfo {
                parent: self.start,
                cost: 0f64,
            },
        );
        while let Some(current) = heap.pop() {
            if current.position == self.finish {
                break;
            }
            for pos in self.neighbors(current.position) {
                let new_cost = map[&current.position].cost + Map::distance(current.position, pos);
                if let Some(info) = map.get(&pos) {
                    if new_cost >= info.cost {
                        continue;
                    }
                }
                heap.push(PosState {
                    position: pos,
                    cost: new_cost + Map::distance(pos, self.finish),
                });
                map.insert(
                    pos,
                    PosInfo {
                        parent: current.position,
                        cost: new_cost,
                    },
                );
            }
        }
        self.reconstruct_path(map)
    }
}
