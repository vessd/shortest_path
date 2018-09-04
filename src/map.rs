use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PosState {
    position: MapPos,
    cost: usize,
}

impl Ord for PosState {
    fn cmp(&self, other: &PosState) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
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
    cost: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Map {
    cols: usize,
    data: Vec<Cell>,
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
                match self.data[i * self.cols + j] {
                    Cell::Passable => print!("0 "),
                    Cell::Impassable => print!("1 "),
                }
            }
            println!("");
        }
    }

    pub fn print_path(&self, path: Vec<MapPos>) {
        let mut matrix = vec![vec![""; self.cols - 1]; self.data.len() / self.cols - 1];
        for i in 1..self.cols - 1 {
            for j in 1..self.data.len() / self.cols - 1 {
                match self.data[i * self.cols + j] {
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
    //Расстояние Чебышёва
    fn distance(p: MapPos, q: MapPos) -> usize {
        let abs_sub = |a: usize, b: usize| if a > b { a - b } else { b - a };
        abs_sub(p.x, q.x).max(abs_sub(p.y, q.y))
    }

    fn reconstruct_path(mut map: HashMap<MapPos, PosInfo>, goal: MapPos) -> Vec<MapPos> {
        let mut vec = Vec::with_capacity(map[&goal].cost);
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
        for i in pos.x..pos.x + 3 {
            for j in pos.y..pos.y + 3 {
                if !(i == pos.x + 1 && j == pos.y + 1) {
                    if self.data[i * self.cols + j] == Cell::Passable {
                        //Возвращаем координаты без учёта стены по периметру
                        vec.push(MapPos::new(i - 1, j - 1));
                    }
                }
            }
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
                cost: 0,
            },
        );
        while let Some(current) = heap.pop() {
            if current.position == goal {
                break;
            }
            for pos in self.neighbors(current.position) {
                let new_cost = map[&current.position].cost + Map::distance(start, pos);
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
