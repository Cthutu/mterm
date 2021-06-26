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

    fn present(&self, present_input: PresentInput) -> PresentResult {
        present_input
            .fore_image
            .iter_mut()
            .for_each(|x| *x = Colour::Red.into());
        present_input
            .back_image
            .iter_mut()
            .for_each(|x| *x = Colour::Black.into());
        present_input
            .text_image
            .iter_mut()
            .for_each(|x| *x = b'H' as u32);
        //present_input.at(1, 1).pr("Hello, World!");
        PresentResult::Changed
    }
}
