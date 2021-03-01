use crate::support::prelude::*;
use duckduckgeo;

use axgeom::Rect;


pub const ROCK:Team=Team(0b100001);
pub const PAPER:Team=Team(0b001010);
pub const SCISSOR:Team=Team(0b010100);

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub struct Team(u8);


#[derive(Copy, Clone, Debug)]
pub struct Bot {
    pub team:Team,
    pub pos: Vec2<f32>,
    pub vel: Vec2<f32>
}


#[derive(Debug)]
enum Res{
    Equal,
    Attack,
    Avoid
}

//returns whether or not to attack or avoid b from the perspective of a.
fn func(a:Team,b:Team)->Res{
    let (Team(a),Team(b))=(a,b);
    if a==b
    {
        Res::Equal
    }
    else
    {
        if (a & (b<<3)) != 0
        {
            Res::Attack
        }
        else
        {
            Res::Avoid
        }
    }
}




impl Bot {
    pub fn new(team:Team,pos: Vec2<f32>) -> Bot {
        let z = vec2same(0.0);

        Bot {
            team,
            pos,
            vel: z,
        }
    }

    pub fn solve(&mut self, b: &mut Self, radius: f32){
        let a=self;



        let diff = b.pos - a.pos;

        let dis_sqr = diff.magnitude2();

        if dis_sqr < 0.00001 {
            a.vel += vec2(1.0, 0.0);
            b.vel -= vec2(1.0, 0.0);
            return;
        }

        if dis_sqr >= (2. * RADIUS_PROXY) * (2. * RADIUS_PROXY) {
            return;
        }

        let dis = dis_sqr.sqrt();

        let norm=diff/dis;

        let mag=(RADIUS_PROXY*2.0-dis)/(RADIUS_PROXY*2.0);
        assert!(mag>=0.0,"{:?}",(mag,dis)); 
        
        let mag=mag*mag;
        let mag=mag*0.1;
        
        match func(a.team,b.team){
            Res::Attack=>{
                let foo=norm*mag;
                a.vel+=foo;
                b.vel+=foo;
            }
            Res::Avoid=>{
                let foo=norm*mag;
                a.vel-=foo;
                b.vel-=foo;
            }
            Res::Equal=>{
                let foo=norm*0.003;
                a.vel+=foo;
                b.vel-=foo;
            }
        }
    }
}


pub const RADIUS:f32=8.0;
pub const RADIUS_PROXY:f32=30.0;


pub fn make_demo(dim: Rect<f32>) -> Demo {
    
    let num_bots=1000;

    let mut bots:Vec<_>=crate::dists::rand2_iter(dim)
    .map(|[a, b]| axgeom::vec2(a as f32, b as f32))
    .enumerate()
    .map(|(i,a)| {
        let t=if i<num_bots/3{
            ROCK
        }else if i<num_bots*2/3{
            PAPER
        }else{
            SCISSOR
        };

        Bot::new(t,a)
    })
    .take(num_bots)
    .collect();
    
    let mut solver=seq_impulse::CollisionVelocitySolver::new();

    Demo::new(move |cursor, canvas, _check_naive| {

        {
         
            let mut base=broccoli::container::TreeIndBase::new(&mut bots,|bot|{
                let p = bot.pos;
                let r = RADIUS;
                Rect::new(p.x - r, p.x + r, p.y - r, p.y + r)
            });
            let mut tree=base.build();
            use duckduckgeo::grid;
            let g=grid::GridViewPort{spacing:50.0,origin:vec2(0.0,0.0)};
            let walls=grid::Grid2D::new(vec2(5,5));
            solver.solve(RADIUS*0.8,&g,&walls,&mut tree,|a|&a.pos,|a|&mut a.vel,|a,b|{
                match func(a.team,b.team){
                    Res::Attack=>{
                        b.team=a.team
                    }
                    Res::Avoid=>{
                        a.team=b.team
                    }
                    _=>{}
                }
            });
        }

        let mut k = support::distribute(&mut bots, |bot| {
            let p = bot.pos;
            let r = RADIUS_PROXY;
            Rect::new(p.x - r, p.x + r, p.y - r, p.y + r)
        });

        let mut tree = broccoli::new_par(RayonJoin, &mut k);

        tree.find_colliding_pairs_mut_par(RayonJoin, move |a, b| {
            let (a, b) = (a.unpack_inner(), b.unpack_inner());
            a.solve(b, RADIUS_PROXY);
        });

        let vv = vec2same(100.0);



        tree.for_all_not_in_rect_mut(&dim, move |a| {
            let a = a.unpack_inner();
            duckduckgeo::collide_with_border(&mut a.pos, &mut a.vel, &dim, 0.5);
        });

        for b in bots.iter_mut() {
            b.pos += b.vel;
            //b.vel.y+=0.01;
            //b.vel=b.vel*0.999; //fake friction
        }


        let mut circle = canvas.circles();
        for bot in bots.iter().filter(|x|x.team==ROCK) {
            circle.add(bot.pos.into());
        }
        circle
            .send_and_uniforms(canvas, RADIUS)
            .with_color([1.0, 0.0, 0.0, 0.5])
            .draw();


        let mut circle = canvas.circles();
        for bot in bots.iter().filter(|x|x.team==PAPER) {
            circle.add(bot.pos.into());
        }
        circle
            .send_and_uniforms(canvas, RADIUS)
            .with_color([0.0, 1.0, 0.0, 0.5])
            .draw();
            


        let mut circle = canvas.circles();
        for bot in bots.iter().filter(|x|x.team==SCISSOR) {
            circle.add(bot.pos.into());
        }
        circle
            .send_and_uniforms(canvas, RADIUS)
            .with_color([0.3, 0.3, 1.0, 0.5])
            .draw();
    })
}



