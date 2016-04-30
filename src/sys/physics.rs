use std::collections::HashMap;
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
    grid: HashMap<Cell, Vec<(specs::Entity, u16)>>,
}

impl System {
    pub fn new() -> System {
        System {
            grid: HashMap::new(),
        }
    }
}

impl specs::System<super::Delta> for System {
    fn run(&mut self, arg: specs::RunArg, _: super::Delta) {
        use specs::Join;
        let mut empty = Vec::new();
        let (space, mut collision, entities) = arg.fetch(|w|
            (w.read::<w::Spatial>(), w.write::<w::Collision>(), w.entities())
        );
        for (sp, col, ent) in (&space, &collision, &entities).iter() {
            let cell = Cell((sp.pos.x / CELL_SIZE) as i32, (sp.pos.y / CELL_SIZE) as i32);
            let mut damage = 0;
            for &(ofx, ofy) in OFFSETS.iter() {
                let cell2 = Cell(cell.0 + ofx, cell.1 + ofy);
                for &mut (e2, ref mut dam2) in self.grid.get_mut(&cell2).unwrap_or(&mut empty).iter_mut() {
                    let s2 = space.get(e2).unwrap();
                    let c2 = collision.get(e2).unwrap();
                    let dist_sq = (sp.pos - s2.pos).magnitude2();
                    let diam = col.radius + c2.radius;
                    assert!(diam <= CELL_SIZE);
                    if c2.health > *dam2 && dist_sq < diam*diam {
                        *dam2 += col.damage;
                        damage += c2.damage;
                    }
                }
            }
            if col.health > damage {
                self.grid.entry(cell).or_insert(Vec::new()).push((ent, damage));
            }else {
                arg.delete(ent);
            }
        }
        // clean up and delete more stuff
        for (_, vec) in self.grid.iter_mut() {
            for (e, damage) in vec.drain(..) {
                let c = collision.get_mut(e).unwrap();
                if c.health > damage {
                    c.health -= damage;
                }else {
                    arg.delete(e)
                }
            }
        }
    }
}
