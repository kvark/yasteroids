use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use cgmath::{EuclideanVector};
use specs;
use world as w;


const CELL_SIZE: f32 = 1.0;
const OFFSETS: [(i32, i32); 9] = [(0, 0),
    (1, 0), (0, 1), (-1, 0), (0, -1),
    (1, 1), (1, -1), (-1, 1), (-1, -1)];

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Cell(i32, i32);

pub struct System {
    grid: Arc<Mutex<HashMap<Cell, Vec<specs::Entity>>>>,
}

impl System {
    pub fn new() -> System {
        let map = HashMap::new();
        System {
            grid: Arc::new(Mutex::new(map)),
        }
    }
}

impl super::System for System {
    fn process(&mut self, plan: &mut specs::Planner, _: super::Delta) {
        let grid = self.grid.clone();
        plan.run(move |arg| {
            use specs::Storage;
            let (space, mut collision, entities) = arg.fetch(|w|
                (w.read::<w::Spatial>(), w.write::<w::Collision>(), w.entities())
            );
            let mut grid = grid.lock().unwrap();
            for ent in entities {
                let (sp, mut col) = match (space.get(ent), collision.get(ent)) {
                    (Some(s), Some(c)) => (s, c.clone()),
                    _ => continue,
                };
                let cell = Cell((sp.pos.x / CELL_SIZE) as i32, (sp.pos.y / CELL_SIZE) as i32);
                for &(ofx, ofy) in OFFSETS.iter() {
                    let empty = Vec::new();
                    let cell2 = Cell(cell.0 + ofx, cell.1 + ofy);
                    for &e2 in grid.get(&cell2).unwrap_or(&empty).iter() {
                        let s2 = space.get(e2).unwrap();
                        let mut c2 = collision.get_mut(e2).unwrap();
                        let dist_sq = (sp.pos - s2.pos).magnitude2();
                        let diam = col.radius + c2.radius;
                        assert!(diam <= CELL_SIZE);
                        if c2.health > 0 && dist_sq < diam*diam {
                            c2.hit(col.damage);
                            col.hit(c2.damage);
                        }
                    }
                }
                if col.health > 0 {
                    grid.entry(cell).or_insert(Vec::new()).push(ent);
                }else {
                    arg.delete(ent);
                }
                *collision.get_mut(ent).unwrap() = col;
            }
            // clean up and delete more stuff
            for (_, vec) in grid.iter_mut() {
                for e in vec.drain(..) {
                    if collision.get(e).unwrap().health == 0 {
                        arg.delete(e);
                    }
                }
            }
        });
    }
}
