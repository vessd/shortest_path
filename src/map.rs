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
    Visited,
    InQueue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MapPos {
    pub x: usize,
    pub y: usize,
}

impl MapPos {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PosState {
    pos: MapPos,
    cost: f64,
}

impl Eq for PosState {}

impl Ord for PosState {
    fn cmp(&self, other: &PosState) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for PosState {
    fn partial_cmp(&self, other: &PosState) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
        if self[pos.x][pos.y] == Cell::Passable {
            match cell {
                Cell::Passable | Cell::Visited => (),
                Cell::Impassable | Cell::InQueue => self[pos.x][pos.y] = cell,
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
        } else {
            match cell {
                Cell::Passable => {
                    if self[pos.x][pos.y] == Cell::Impassable {
                        self[pos.x][pos.y] = cell;
                    }
                }
                Cell::Visited => {
                    if self[pos.x][pos.y] == Cell::InQueue {
                        self[pos.x][pos.y] = cell;
                    }
                }
                _ => (),
            }
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                if !(self[i][j] == Cell::Start || self[i][j] == Cell::Finish) {
                    self[i][j] = Cell::Passable;
                }
            }
        }
    }

    pub fn clear_path(&mut self) {
        self.data
            .iter_mut()
            .filter(|c| **c == Cell::Visited || **c == Cell::InQueue)
            .for_each(|c| *c = Cell::Passable)
    }

    //евклидово расстояние
    fn distance(p: MapPos, q: MapPos) -> f64 {
        ((p.x as f64 - q.x as f64).powi(2) + (p.y as f64 - q.y as f64).powi(2)).sqrt()
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

    pub fn replace_from(&mut self, map: &Map) {
        self.cols = map.cols;
        self.data.clear();
        self.data.extend_from_slice(&map.data);
        self.start = map.start;
        self.finish = map.finish;
    }
}

#[derive(PartialEq)]
pub enum SearchStatus {
    Found(f64),
    NotFound,
    Searching,
}

#[derive(Debug, PartialEq)]
pub enum Algorithm {
    BreadthFirstSearch,
    Dijkstra,
    AStar,
}

pub struct ShortestPath {
    pub map: Map,
    queue: BinaryHeap<PosState>,
    visited: HashMap<MapPos, PosState>,
    algorithm: Algorithm,
}
impl ShortestPath {
    pub fn new(map: Map, algorithm: Algorithm) -> Self {
        let queue = BinaryHeap::new();
        let visited = HashMap::new();
        Self {
            map,
            queue,
            visited,
            algorithm,
        }
    }

    pub fn next(&mut self) -> SearchStatus {
        if let Some(current) = self.queue.pop() {
            if current.pos == self.map.finish {
                return SearchStatus::Found(self.visited[&current.pos].cost);
            }
            for pos in self.map.neighbors(current.pos) {
                let new_cost = self.visited[&current.pos].cost + Map::distance(current.pos, pos);

                if let Some(info) = self.visited.get(&pos) {
                    if self.algorithm == Algorithm::BreadthFirstSearch || new_cost >= info.cost {
                        continue;
                    }
                }

                let cost = match self.algorithm {
                    Algorithm::BreadthFirstSearch => current.cost + (self.queue.len() + 1) as f64,
                    Algorithm::Dijkstra => new_cost,
                    Algorithm::AStar => new_cost + Map::distance(pos, self.map.finish),
                };

                self.queue.push(PosState { pos, cost });
                self.visited.insert(
                    pos,
                    PosState {
                        pos: current.pos,
                        cost: new_cost,
                    },
                );
                self.map.set_cell(Cell::InQueue, pos);
            }
            self.map.set_cell(Cell::Visited, current.pos);
            SearchStatus::Searching
        } else {
            SearchStatus::NotFound
        }
    }

    pub fn path(&self) -> Option<Vec<MapPos>> {
        if let Some(info) = self.visited.get(&self.map.finish) {
            let mut vec = Vec::with_capacity(info.cost as usize);
            let mut current = self.map.finish;
            while self.visited[&current].pos != current {
                vec.push(current);
                current = self.visited[&current].pos;
            }
            vec.push(current);
            Some(vec)
        } else {
            None
        }
    }

    pub fn init(&mut self) {
        self.queue.clear();
        self.visited.clear();
        let cost = match self.algorithm {
            Algorithm::AStar => Map::distance(self.map.start, self.map.finish),
            _ => 0f64,
        };
        self.queue.push(PosState {
            pos: self.map.start,
            cost,
        });
        self.visited.insert(
            self.map.start,
            PosState {
                pos: self.map.start,
                cost: 0f64,
            },
        );
    }
}
