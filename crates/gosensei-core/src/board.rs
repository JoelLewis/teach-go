use std::collections::HashSet;

use crate::types::{BoardSize, Color, Point};

#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    size: BoardSize,
    grid: Vec<Option<Color>>,
}

impl Board {
    pub fn new(size: BoardSize) -> Self {
        let n = size.size() as usize;
        Self {
            size,
            grid: vec![None; n * n],
        }
    }

    pub fn size(&self) -> BoardSize {
        self.size
    }

    pub fn dimension(&self) -> u8 {
        self.size.size()
    }

    fn index(&self, point: Point) -> usize {
        point.row as usize * self.dimension() as usize + point.col as usize
    }

    pub fn get(&self, point: Point) -> Option<Color> {
        self.grid[self.index(point)]
    }

    pub fn set(&mut self, point: Point, color: Option<Color>) {
        let idx = self.index(point);
        self.grid[idx] = color;
    }

    pub fn is_empty(&self, point: Point) -> bool {
        self.get(point).is_none()
    }

    pub fn group_at(&self, point: Point) -> Option<Group> {
        let color = self.get(point)?;
        let mut stones = HashSet::new();
        let mut liberties = HashSet::new();
        let mut stack = vec![point];

        while let Some(p) = stack.pop() {
            if !stones.insert(p) {
                continue;
            }
            for neighbor in p.neighbors(self.dimension()) {
                match self.get(neighbor) {
                    None => {
                        liberties.insert(neighbor);
                    }
                    Some(c) if c == color => {
                        if !stones.contains(&neighbor) {
                            stack.push(neighbor);
                        }
                    }
                    Some(_) => {}
                }
            }
        }

        Some(Group {
            color,
            stones,
            liberties,
        })
    }

    pub fn all_points(&self) -> impl Iterator<Item = Point> {
        let dim = self.dimension();
        (0..dim).flat_map(move |r| (0..dim).map(move |c| Point::new(r, c)))
    }

    pub fn remove_group(&mut self, group: &Group) -> usize {
        let count = group.stones.len();
        for &stone in &group.stones {
            self.set(stone, None);
        }
        count
    }
}

#[derive(Debug, Clone)]
pub struct Group {
    pub color: Color,
    pub stones: HashSet<Point>,
    pub liberties: HashSet<Point>,
}

impl Group {
    pub fn liberty_count(&self) -> usize {
        self.liberties.len()
    }

    pub fn is_captured(&self) -> bool {
        self.liberties.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_board_is_empty() {
        let board = Board::new(BoardSize::Nine);
        for point in board.all_points() {
            assert!(board.is_empty(point));
        }
    }

    #[test]
    fn place_and_read_stone() {
        let mut board = Board::new(BoardSize::Nine);
        let p = Point::new(3, 3);
        board.set(p, Some(Color::Black));
        assert_eq!(board.get(p), Some(Color::Black));
    }

    #[test]
    fn group_liberties_corner() {
        let mut board = Board::new(BoardSize::Nine);
        let p = Point::new(0, 0);
        board.set(p, Some(Color::Black));
        let group = board.group_at(p).unwrap();
        assert_eq!(group.liberty_count(), 2);
    }

    #[test]
    fn group_liberties_center() {
        let mut board = Board::new(BoardSize::Nine);
        let p = Point::new(4, 4);
        board.set(p, Some(Color::Black));
        let group = board.group_at(p).unwrap();
        assert_eq!(group.liberty_count(), 4);
    }

    #[test]
    fn connected_group() {
        let mut board = Board::new(BoardSize::Nine);
        board.set(Point::new(4, 4), Some(Color::Black));
        board.set(Point::new(4, 5), Some(Color::Black));
        let group = board.group_at(Point::new(4, 4)).unwrap();
        assert_eq!(group.stones.len(), 2);
        assert_eq!(group.liberty_count(), 6);
    }
}
