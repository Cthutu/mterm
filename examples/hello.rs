//
// Hello world example
//

use mterm::*;

fn main() {
    let hello = Box::new(HelloApp {});
    let app_builder = Builder::new()
        .with_inner_size(100, 100)
        .with_title("Hello!")
        .build();
    run(hello, app_builder).unwrap();
}

struct HelloApp {}

impl App for HelloApp {
    fn tick(&mut self, _tick_input: TickInput) -> TickResult {
        TickResult::Continue
    }

    fn present(&self, mut present_input: PresentInput) -> PresentResult {
        let mut image = Image::new(present_input.width, present_input.height);
        image.clear(Colour::White.into(), Colour::Black.into());
        image.draw_string(
            Point::new(1, 1),
            "Hello",
            Colour::Yellow.into(),
            Colour::Blue.into(),
        );
        image.draw_string(
            Point::new(
                present_input.width as i32 - 7,
                present_input.height as i32 - 2,
            ),
            "World!",
            Colour::Blue.into(),
            Colour::Yellow.into(),
        );
        present_input.blit_screen(&image);
        PresentResult::Changed
    }
}
