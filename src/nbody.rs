#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<f32>,
    vel: Vec2<f32>,
    force: Vec2<f32>,
    mass: f32,
}

impl Bot {
    fn handle(&mut self) {
        let b = self;

        b.pos += b.vel;

        //F=MA
        //A=F/M
        let acc = b.force / b.mass;

        b.vel += acc;

        b.force = vec2same(0.0);
    }
}

#[derive(Copy, Clone, Debug)]
struct NodeMass {
    center: Vec2<f32>,
    mass: f32,
    force: Vec2<f32>,
}

use core::default::Default;
impl Default for NodeMass {
    fn default() -> NodeMass {
        NodeMass {
            center: vec2(Default::default(), Default::default()),
            mass: Default::default(),
            force: vec2(Default::default(), Default::default()),
        }
    }
}


use broccoli::pmut::*;
use broccoli::query::nbody::*;
use core::marker::PhantomData;

#[derive(Clone, Copy)]
struct Bla<'a> {
    _num_pairs_checked: usize,
    _p: PhantomData<&'a usize>,
}
impl<'a> broccoli::build::Splitter for Bla<'a> {
    fn div(&mut self) -> (Self, Self) {
        (
            Bla {
                _p: PhantomData,
                _num_pairs_checked: 0,
            },
            Bla {
                _p: PhantomData,
                _num_pairs_checked: 0,
            },
        )
    }
    fn add(&mut self, _: Self, _: Self) {
        //do nothing
    }
}
impl<'b> broccoli::query::nbody::Nbody for Bla<'b> {
    type Mass = NodeMass;
    type T = BBox<f32, &'b mut Bot>;
    type N = f32;

    //return the position of the center of mass
    fn compute_center_of_mass(&mut self, a: &[Self::T]) -> Self::Mass {
        let mut total_x = 0.0;
        let mut total_y = 0.0;
        let mut total_mass = 0.0;

        for i in a.iter() {
            let m = i.inner.mass;
            total_mass += m;
            total_x += m * i.inner.pos.x;
            total_y += m * i.inner.pos.y;
        }

        let center = if total_mass != 0.0 {
            vec2(total_x / total_mass, total_y / total_mass)
        } else {
            vec2same(0.0)
        };
        NodeMass {
            center,
            mass: total_mass,
            force: vec2same(0.0),
        }
    }

    fn is_close_half(&mut self, m: &Self::Mass, line: Self::N, a: impl Axis) -> bool {
        if a.is_xaxis() {
            (m.center.x - line).abs() < 200.0
        } else {
            (m.center.y - line).abs() < 200.0
        }
    }

    fn is_close(&mut self, m: &Self::Mass, line: Self::N, a: impl Axis) -> bool {
        if a.is_xaxis() {
            (m.center.x - line).abs() < 400.0
        } else {
            (m.center.y - line).abs() < 400.0
        }
    }

    #[inline(always)]
    fn gravitate(&mut self, a: GravEnum<Self::T, Self::Mass>, b: GravEnum<Self::T, Self::Mass>) {
        match (a, b) {
            (GravEnum::Mass(a), GravEnum::Mass(b)) => {
                let _ = duckduckgeo::gravitate(
                    [
                        (a.center, a.mass, &mut a.force),
                        (b.center, b.mass, &mut b.force),
                    ],
                    0.0001,
                    0.004,
                );
            }
            (GravEnum::Mass(a), GravEnum::Bot(b)) | (GravEnum::Bot(b), GravEnum::Mass(a)) => {
                for b in b.iter_mut() {
                    let b = b.unpack_inner();

                    let _ = duckduckgeo::gravitate(
                        [
                            (a.center, a.mass, &mut a.force),
                            (b.pos, b.mass, &mut b.force),
                        ],
                        0.0001,
                        0.004,
                    );
                }
            }
            (GravEnum::Bot(b1), GravEnum::Bot(mut b2)) => {
                for mut a in b1.iter_mut() {
                    for b in b2.borrow_mut().iter_mut() {
                        let (a, b) = (a.borrow_mut().unpack_inner(), b.unpack_inner());

                        let _ = duckduckgeo::gravitate(
                            [(a.pos, a.mass, &mut a.force), (b.pos, b.mass, &mut b.force)],
                            0.0001,
                            0.004,
                        );
                    }
                }
            }
        }
    }

    fn gravitate_self(&mut self, a: PMut<[Self::T]>) {
        broccoli::query::nbody::naive_mut(a, |a, b| {
            let (a, b) = (a.unpack_inner(), b.unpack_inner());

            let _ = duckduckgeo::gravitate(
                [(a.pos, a.mass, &mut a.force), (b.pos, b.mass, &mut b.force)],
                0.0001,
                0.004,
            );
        })
    }

    fn apply_a_mass<'a>(&mut self, a: Self::Mass, b: PMut<[Self::T]>) {
        if a.mass > 0.000_000_1 {
            let indforce = vec2(a.force.x / b.len() as f32, a.force.y / b.len() as f32);

            //TODO counteract the added fudge here, but dividing by 3 on bot on bot cases
            let fudge = 3.0;
            for i in b.iter_mut() {
                let i = i.unpack_inner();
                let forcex = indforce.x * fudge;
                let forcey = indforce.y * fudge;

                i.force += vec2(forcex, forcey);
            }
        }
    }

    fn combine_two_masses(&mut self, a: &Self::Mass, b: &Self::Mass) -> Self::Mass {
        NodeMass {
            center: (a.center + b.center) / 2.0,
            mass: a.mass + b.mass,
            force: vec2same(0.0),
        }
    }
}
