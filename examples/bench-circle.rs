extern crate ggez;
use ggez::conf;
use ggez::event;
use ggez::{GameResult, Context};
use ggez::graphics;
use ggez::timer;
use ggez::graphics::{ DrawMode, Point };
use std::time::Duration;

struct Item {
    position: Point,
    radius: f32,
}

struct MainState {
    items: Vec<Item>,
    fps: f64,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState {
            items: Vec::new(),
            fps: 60.0,
        };

        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        self.fps = self.fps * 0.9 + timer::get_fps(ctx) * 0.1;

        if self.fps > 59.5 {
            let d = timer::get_time_since_start(ctx);
            let t = d.as_secs() ^ (d.subsec_nanos() as u64);
            let x = ((t ^ 2305843009213693951) % 800) as f32;
            let y = ((t ^ 461168601842738789) % 600) as f32;
            let r = 30.0 + ((t ^ 133978850655919) % 60) as f32;
            self.items.push(Item {
                position: Point { x: x, y: y },
                radius: r,
            });
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        for item in &self.items {
            graphics::circle(ctx, DrawMode::Fill, item.position, item.radius, 32)?;
        }

        graphics::present(ctx);

        timer::sleep_until_next_frame(ctx, 60);

        println!("{} items = {} fps", self.items.len(), timer::get_fps(ctx));

        Ok(())
    }
}

pub fn main() {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("helloworld", c).unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = event::run(ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
