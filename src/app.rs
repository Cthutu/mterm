use time::Duration;
use winit::event::VirtualKeyCode;

/// Application trait for hooking into the main loop of `mterm`.
///
/// `mterm` manages `Winit` and `wgpu` for you to provide an interface between
/// it and your application.  The application author is required to implement 2
/// methods from the trait: `tick` and `present`.
///
/// `tick` is called everytime an event is triggered in the main loop.  It
/// received a `TickInput` structure that contains various information such as
/// any key presses, mouse movements and clicks, and some stats.  Using this
/// information the applicaiton can execute its logic.  This method should also
/// return a `TickResult` value to tell the main loop whether to continue or
/// stop and exit the application.
///
/// `present` is called whenever the application is required to render itself.
/// `mterm` provides the method with a few `u32` arrays via a PresentInput
/// structure.  These arrays can be mutated to change what appears in the
/// window. This method should return a `PresentResult` to tell the main loop if
/// anything changed and needs to be rendered.

pub trait App {
    fn tick(&mut self, tick_input: TickInput) -> TickResult;
    fn present(&self, present_input: PresentInput) -> PresentResult;
}

/// Provides feedback to `mterm`'s main loop instructing it whether to keep
/// ticking or to stop and exit the application.

pub enum TickResult {
    /// Instructs the main loop to continue ticking.
    Continue,
    /// Instructs the main loop to stop and exit the applicaiton.
    Stop,
}

/// Provides feedback to `mterm`'s main loop instructing whether the images were
/// written to.
///

pub enum PresentResult {
    /// Return this from the `present` method to signify that changes were made
    /// and the window will need to be redrawn.
    Changed,
    /// Return this from the `present` method to signify that no changes were
    /// made so that the main loop does not have to redraw the window.
    NoChanges,
}

/// Contains information for the tick method in `App`.

pub struct TickInput {
    /// This is the delta time since last time `tick` was called.
    pub dt: Duration,
    /// Current width of the window in characters.
    pub width: u32,
    /// Current height of the window in characters.
    pub height: u32,
    /// May contain information on a key pressed or released, along with shift
    /// modifiers.
    pub key: KeyState,
    /// May contain information on a mouse event such as a click or mouse movement.
    pub mouse: Option<MouseState>,
}

/// Can provide information about a key press or release, and will maintain the
/// current state of shift modifiers at all time.

#[derive(Debug, Copy, Clone)]
pub struct KeyState {
    /// If `KeyState::vkey` is not `None`, this will be true if the key was
    /// pressed, otherwise it was released.
    pub pressed: bool,
    /// True if either shift key is being held down.
    pub shift: bool,
    /// True if either ctrl key is being held down.
    pub ctrl: bool,
    /// True if the alt key is being held down.
    pub alt: bool,
    /// If a key has been pressed or released, this will contains its virtual
    /// key code as defined by the `winit` crate.
    pub vkey: Option<VirtualKeyCode>,
    /// If a key was pressed, and is mappable to a character, this will contain
    /// the character.
    pub code: Option<char>,
}

/// Provides information about the position of the mouse pointer, its buttons
/// and scroll wheel.
pub struct MouseState {
    /// True if the mouse pointer is currently on the application window.
    pub on_window: bool,
    /// True if the mouse's primary mouse button was clicked.
    pub primary_pressed: bool,
    /// True if the mouse's secondary mouse button was clicked.
    pub secondary_pressed: bool,
    /// The X coordinate of the mouse pointer, relative to the top left corner
    /// of the application window.
    pub x: i32,
    /// The Y coordinate of the mouse pointer, relative to the top left corner
    /// of the application window.
    pub y: i32,
}

/// Provides presentation information and contains the arrays that can be
/// mutated to update the window's contents.
///
/// The window's contents is split into 3 arrays: `fore_image`, `back_image` and
/// `text_image`.  The `fore_image` is an array of `u32`s containing the colour
/// codes for the foreground colour (or ink colour) of each character on the
/// window. Each `u32` represents a single character.  Similarly, the
/// `back_image` contains an array of `u32`s representing all the background
/// colours (or paper colour) for each character on the window.  Finally,
/// `text_image` contains all the ASCII character codes for each character on
/// the window.  This also contains `u32`s but currently, only the lower 8 bits
/// is considered for rendering.  In a future version, higher bits might be used
/// for other effects (such as bold, underline etc).

pub struct PresentInput<'a> {
    /// The current width, in chars, of the application window.
    pub width: usize,
    /// The current height, in chars, of the application window.
    pub height: usize,
    /// The array (of size width*height) of u32 values representing the ink
    /// colours (or foreground colours) of each character on the window.
    pub fore_image: &'a mut Vec<u32>,
    /// The array (of size width*height) of u32 values representing the paper
    /// colours (or background colours) of each character on the window.
    pub back_image: &'a mut Vec<u32>,
    /// The array (of size width*height) of u32 values representing the ASCII
    /// character codes of each character on the window.  Only the lower 8-bits
    /// are currently used.
    pub text_image: &'a mut Vec<u32>,
}
