use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::ops::Index;
use std::vec::IntoIter;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Cell {
    Passable,
    Impassable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapPos {
    x: usize,
    y: usize,
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
}

impl Index<usize> for Map {
    type Output = [Cell];

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        let i = index * self.cols;
        &self.data[i..i + self.cols]
    }
}

impl Map {
    pub fn new(cols: usize, rows: usize) -> Self {
        //Увеличиваем рамер карты, для того чтобы сделать стену по периметру,
        //это позволит не проверять границы при поиске соседей.
        let cols = cols + 2;
        let rows = rows + 2;
        let mut data = vec![Cell::Passable; cols * rows];
        //Стена сверху и снизу
        for i in 0..cols {
            data[i] = Cell::Impassable;
            data[cols * (rows - 1) + i] = Cell::Impassable;
        }
        //Стена слева и справа
        for i in 0..rows {
            data[cols * i] = Cell::Impassable;
            data[cols * (i + 1) - 1] = Cell::Impassable;
        }
        Map { cols, data }
    }

    pub fn set_wall(&mut self, pos: MapPos) {
        self.data[(pos.x + 1) * self.cols + pos.y + 1] = Cell::Impassable;
    }

    pub fn print_map(&self) {
        for i in 1..self.cols - 1 {
            for j in 1..self.data.len() / self.cols - 1 {
                match self[i][j] {
                    Cell::Passable => print!("0 "),
                    Cell::Impassable => print!("1 "),
                }
            }
            println!("");
        }
    }

    pub fn print_path(&self, path: Vec<MapPos>) {
        let mut matrix = vec![vec![""; self.cols - 2]; self.data.len() / self.cols - 2];
        for i in 1..self.cols - 1 {
            for j in 1..self.data.len() / self.cols - 1 {
                match self[i][j] {
                    Cell::Passable => matrix[i - 1][j - 1] = "0",
                    Cell::Impassable => matrix[i - 1][j - 1] = "1",
                }
            }
        }
        for pos in path {
            matrix[pos.x][pos.y] = "2";
        }
        for row in matrix {
            for cell in row {
                print!("{} ", cell);
            }
            println!("");
        }
    }

    //евклидово расстояние
    fn distance(p: MapPos, q: MapPos) -> f64 {
        ((p.x as f64 - q.x as f64).powi(2) + (p.y as f64 - q.y as f64).powi(2)).sqrt()
    }

    fn reconstruct_path(mut map: HashMap<MapPos, PosInfo>, goal: MapPos) -> Vec<MapPos> {
        let mut vec = Vec::with_capacity(map[&goal].cost as usize);
        let mut current = goal;
        while let Some(info) = map.remove(&current) {
            vec.push(current);
            if current == info.parent {
                break;
            }
            current = info.parent;
        }
        vec
    }

    fn neighbors(&self, pos: MapPos) -> IntoIter<MapPos> {
        let mut vec = Vec::with_capacity(8);
        let mut s = [false; 4];
        let mut d = [false; 4];
        let pos = MapPos::new(pos.x + 1, pos.y + 1);
        if self[pos.x - 1][pos.y] == Cell::Passable {
            s[0] = true;
            vec.push(MapPos::new(pos.x - 2, pos.y - 1));
        }
        if self[pos.x][pos.y + 1] == Cell::Passable {
            s[1] = true;
            vec.push(MapPos::new(pos.x - 1, pos.y));
        }
        if self[pos.x + 1][pos.y] == Cell::Passable {
            s[2] = true;
            vec.push(MapPos::new(pos.x, pos.y - 1));
        }
        if self[pos.x][pos.y - 1] == Cell::Passable {
            s[3] = true;
            vec.push(MapPos::new(pos.x - 1, pos.y - 2));
        }

        d[0] = s[3] || s[0];
        d[1] = s[0] || s[1];
        d[2] = s[1] || s[2];
        d[3] = s[2] || s[3];

        if d[0] && self[pos.x - 1][pos.y - 1] == Cell::Passable {
            vec.push(MapPos::new(pos.x - 2, pos.y - 2));
        }
        if d[1] && self[pos.x - 1][pos.y + 1] == Cell::Passable {
            vec.push(MapPos::new(pos.x - 2, pos.y));
        }
        if d[2] && self[pos.x + 1][pos.y + 1] == Cell::Passable {
            vec.push(MapPos::new(pos.x, pos.y));
        }
        if d[3] && self[pos.x + 1][pos.y - 1] == Cell::Passable {
            vec.push(MapPos::new(pos.x, pos.y - 2));
        }
        vec.into_iter()
    }

    pub fn shortest_path(&self, start: MapPos, goal: MapPos) -> Vec<MapPos> {
        let mut heap = BinaryHeap::new();
        let mut map = HashMap::new();
        heap.push(PosState {
            position: start,
            cost: Map::distance(start, goal),
        });
        map.insert(
            start,
            PosInfo {
                parent: start,
                cost: 0f64,
            },
        );
        while let Some(current) = heap.pop() {
            if current.position == goal {
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
                    cost: new_cost + Map::distance(pos, goal),
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
        Map::reconstruct_path(map, goal)
    }
}
