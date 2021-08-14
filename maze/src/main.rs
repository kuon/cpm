use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MazeError {
    #[error("io error")]
    IOError(#[from] std::io::Error),
    #[error("xml error")]
    XMLError,
}

#[derive(Debug)]
struct Rect {
    id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Debug)]
struct Circle {
    id: String,
    cx: f64,
    cy: f64,
    r: f64,
}

#[derive(Debug)]
enum Attr {
    Rect(Rect),
    Circle(Circle),
}

#[derive(Debug)]
struct Maze {
    origin: (f64, f64),
    start: (f64, f64),
    end: (f64, f64),
    size: (f64, f64),
    walls: Vec<(f64, f64, f64, f64)>,
}

impl Maze {
    fn new() -> Self {
        Self {
            origin: (0.0, 0.0),
            start: (0.0, 0.0),
            end: (0.0, 0.0),
            size: (0.0, 0.0),
            walls: vec![],
        }
    }

    fn grid(&self) -> Grid {
        let mut walls = HashSet::new();

        for w in self.walls.iter() {
            let min_x = (w.0 - self.origin.0).floor() as i32;
            let min_y = (w.1 - self.origin.1).floor() as i32;
            let max_x = (w.0 - self.origin.0 + w.2).ceil() as i32;
            let max_y = (w.1 - self.origin.1 + w.3).ceil() as i32;

            for y in min_y..max_y {
                for x in min_x..max_x {
                    walls.insert((x, y));
                }
            }
        }

        Grid {
            start: (
                (self.start.0 - self.origin.0).round() as i32,
                (self.start.1 - self.origin.1).round() as i32,
            ),
            end: (
                (self.end.0 - self.origin.0).round() as i32,
                (self.end.1 - self.origin.1).round() as i32,
            ),
            width: self.size.0.ceil() as i32,
            height: self.size.1.ceil() as i32,
            walls,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Node {
    coord: (i32, i32),
    f_score: i32,
    g_score: i32,
    came_from: (i32, i32),
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f_score
            .cmp(&self.f_score)
            .then_with(|| self.g_score.cmp(&other.g_score))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn d(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0) + (a.1 - b.1)
}

#[derive(Debug)]
struct Grid {
    start: (i32, i32),
    end: (i32, i32),
    width: i32,
    height: i32,
    walls: HashSet<(i32, i32)>,
}

impl Grid {
    fn path(&self) -> Vec<(i32, i32)> {
        let mut open_set = BinaryHeap::new();
        let mut open_set_map = HashSet::new();
        let start = Node {
            f_score: d(self.start, self.end),
            g_score: 0,
            came_from: self.start,
            coord: self.start,
        };
        open_set_map.insert(start.coord);
        open_set.push(start.clone());

        let mut nodes = HashMap::new();
        nodes.insert(start.coord, start);

        let neighbors = |node: &Node| {
            let c = node.coord;
            let neighbors = vec![
                (c.0 - 1, c.1),
                (c.0 + 1, c.1),
                (c.0, c.1 - 1),
                (c.0, c.1 + 1),
            ];

            let coords: Vec<_> = neighbors
                .into_iter()
                .filter(|c| {
                    if self.walls.contains(c) {
                        false
                    } else if c.0 < 0
                        || c.1 < 0
                        || c.0 >= self.width
                        || c.1 >= self.height
                    {
                        false
                    } else {
                        true
                    }
                })
                .collect();

            coords
        };

        let mut found = None;
        while !open_set.is_empty() {
            let current = open_set.pop().unwrap();
            open_set_map.remove(&current.coord);
            if current.coord == self.end {
                found = Some(current);
                break;
            }

            for ncoord in neighbors(&current) {
                let tentative = current.g_score + 1;
                let n = nodes.entry(ncoord).or_insert(Node {
                    f_score: i32::MAX,
                    g_score: i32::MAX,
                    coord: ncoord,
                    came_from: current.coord,
                });
                if tentative < n.g_score {
                    n.came_from = current.coord;
                    n.g_score = tentative;
                    n.f_score = tentative + d(self.end, n.coord);
                    if !open_set_map.contains(&n.coord) {
                        open_set_map.insert(n.coord);
                        open_set.push(n.clone());
                    }
                }
            }
        }

        let reconstruct = |node: &Node| {
            let mut path = Vec::new();
            let mut coord = node.coord;
            while coord != self.start {
                path.push(coord);
                coord = nodes.get(&coord).unwrap().came_from;
            }
            path.push(self.start);
            path
        };

        if let Some(found) = found {
            println!("Finished path: {:?}", found);
            return reconstruct(&found);
        } else {
            println!("No solution found!!!");
            return vec![];
        }
    }
}

use svg::Document;

use svg::node::element;
use svg::node::element::tag;
use svg::parser::Event;

fn main() -> Result<(), MazeError> {
    let mut content = String::new();
    let mut attrs = Vec::new();
    for event in svg::open("../maze.svg", &mut content)? {
        match event {
            Event::Tag(tag::Rectangle, _, attributes) => {
                let r = Attr::Rect(Rect {
                    id: attributes.get("id").unwrap().to_string(),
                    x: attributes.get("x").unwrap().parse().unwrap(),
                    y: attributes.get("y").unwrap().parse().unwrap(),
                    width: attributes.get("width").unwrap().parse().unwrap(),
                    height: attributes.get("height").unwrap().parse().unwrap(),
                });
                attrs.push(r);
            }
            Event::Tag(tag::Circle, _, attributes) => {
                let r = Attr::Circle(Circle {
                    id: attributes.get("id").unwrap().to_string(),
                    cx: attributes.get("cx").unwrap().parse().unwrap(),
                    cy: attributes.get("cy").unwrap().parse().unwrap(),
                    r: attributes.get("r").unwrap().parse().unwrap(),
                });
                attrs.push(r);
            }
            _ => {}
        }
    }

    let mut maze = Maze::new();

    for attr in attrs {
        match attr {
            Attr::Circle(c) => {
                if c.id == "start" {
                    maze.start = (c.cx, c.cy)
                } else if c.id == "end" {
                    maze.end = (c.cx, c.cy)
                }
            }
            Attr::Rect(r) => {
                if r.id == "bg" {
                    maze.origin = (r.x, r.y);
                    maze.size = (r.width, r.height);
                } else {
                    maze.walls.push((r.x, r.y, r.width, r.height));
                }
            }
        }
    }

    let grid = maze.grid();
    let path = grid.path();

    let mut document =
        Document::new().set("viewBox", (0, 0, maze.size.0, maze.size.1));

    let rect = element::Rectangle::new()
        .set("fill", "rgba(220, 220, 220, 1")
        .set("stroke", "transparent")
        .set("x", 0)
        .set("y", 0)
        .set("width", maze.size.0)
        .set("height", maze.size.1);
    document = document.add(rect);

    for (x, y) in grid.walls.iter() {
        let rect = element::Rectangle::new()
            .set("fill", "transparent")
            .set("stroke", "rgba(0, 0, 0, 0.5")
            .set("stroke-width", "0.1")
            .set("x", *x)
            .set("y", *y)
            .set("width", 1)
            .set("height", 1);
        document = document.add(rect);
    }

    for (x, y) in path.iter() {
        let rect = element::Rectangle::new()
            .set("stroke", "rgba(255, 0, 0, 0.5")
            .set("fill", "rgba(255, 0, 0, 0.5")
            .set("stroke-width", "0.1")
            .set("x", *x)
            .set("y", *y)
            .set("width", 1)
            .set("height", 1);
        document = document.add(rect);
    }

    for (x, y, w, h) in maze.walls.iter() {
        let rect = element::Rectangle::new()
            .set("fill", "rgba(0, 0, 255, 0.2)")
            .set("stroke", "transparent")
            .set("x", *x - maze.origin.0)
            .set("y", *y - maze.origin.1)
            .set("width", *w)
            .set("height", *h);
        document = document.add(rect);
    }

    let start = element::Circle::new()
        .set("fill", "transparent")
        .set("stroke", "green")
        .set("stroke-width", "0.5")
        .set("cx", maze.start.0 - maze.origin.0)
        .set("cy", maze.start.1 - maze.origin.1)
        .set("r", 2);
    document = document.add(start);

    let end = element::Circle::new()
        .set("fill", "transparent")
        .set("stroke", "green")
        .set("stroke-width", "0.5")
        .set("cx", maze.end.0 - maze.origin.0)
        .set("cy", maze.end.1 - maze.origin.1)
        .set("r", 2);
    document = document.add(end);

    svg::save("../grid.svg", &document).unwrap();

    Ok(())
}
