use ggez::{event, Context, GameResult};

pub struct TestState {}

impl event::EventHandler for TestState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }
}

fn main() -> GameResult {
    Ok(())
} 