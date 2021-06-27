use futures::executor::block_on;
use image::ImageFormat;
use std::cmp::max;
use time::Duration;
use wgpu::SwapChainError;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

use crate::{
    load_font_image, App, Builder, Font, KeyState, PresentInput, PresentResult, RenderState,
    Result, TickInput, TickResult,
};

/// Start the main loop.
///
/// This function does not exit unless an error occurs during start up.
///
/// # Arguments
///
/// * __app__ - An object that implements the App trait.  `mterm` will call the
///   App methods to allow it to run logic and control presentation.
/// * __builder__ - An instance of `Builder` to provide configuration
///   information for the window.
///
/// # Returns
///
/// Returns a `TermResult`.

pub fn run(app: Box<dyn App>, builder: Builder) -> Result<()> {
    block_on(run_internal(app, builder))
}

pub async fn run_internal(mut app: Box<dyn App>, builder: Builder) -> Result<()> {
    let font_data = match builder.font {
        Font::Default => load_font_image(include_bytes!("font1.png"), ImageFormat::Png)?,
        Font::Custom(font) => font,
    };

    // Adjust the dimensions of the window to fit character cells exactly.
    let width =
        max(20 * font_data.width, builder.inner_size.0 as u32) / font_data.width * font_data.width;
    let height = max(20 * font_data.height, builder.inner_size.1 as u32) / font_data.height
        * font_data.height;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(width, height))
        .with_title(builder.title)
        .with_min_inner_size(PhysicalSize::new(
            20 * font_data.width,
            20 * font_data.height,
        ))
        .build(&event_loop)?;

    let mut render = RenderState::new(&window, &font_data).await?;

    let mut key_state = KeyState {
        vkey: None,
        pressed: false,
        alt: false,
        ctrl: false,
        shift: false,
        code: None,
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            //
            // Windowed Events
            //
            Event::WindowEvent { event, window_id } if window.id() == window_id => {
                match event {
                    //
                    // Closing the window
                    //
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                    //
                    // Keyboard Events
                    //
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state,
                                virtual_keycode,
                                ..
                            },
                        ..
                    } => {
                        key_state.pressed = state == ElementState::Pressed;
                        key_state.vkey = virtual_keycode;

                        //
                        // Check for system keys
                        //
                        match key_state {
                            KeyState {
                                pressed: true,
                                vkey: Some(VirtualKeyCode::Escape),
                                ..
                            } => {
                                //
                                // Exit
                                //
                                *control_flow = ControlFlow::Exit;
                            }
                            KeyState {
                                pressed: true,
                                shift: false,
                                ctrl: false,
                                alt: true,
                                vkey: Some(VirtualKeyCode::Return),
                                code: None,
                            } => {
                                //
                                // Toggle fullscreen
                                //
                                if window.fullscreen().is_some() {
                                    window.set_fullscreen(None);
                                } else if let Some(monitor) = window.current_monitor() {
                                    if let Some(video_mode) = monitor.video_modes().next() {
                                        if cfg!(any(target_os = "macos", unix)) {
                                            window.set_fullscreen(Some(Fullscreen::Borderless(
                                                Some(monitor),
                                            )));
                                        } else {
                                            window.set_fullscreen(Some(Fullscreen::Exclusive(
                                                video_mode,
                                            )));
                                        }
                                    };
                                };
                            }
                            _ => {}
                        }
                    }
                    //
                    // Modifier keys
                    //
                    WindowEvent::ModifiersChanged(mods) => {
                        key_state.alt = mods.alt();
                        key_state.ctrl = mods.ctrl();
                        key_state.shift = mods.shift();
                    }
                    //
                    // Resizing
                    //
                    WindowEvent::Resized(new_size) => render.resize(new_size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        render.resize(*new_inner_size)
                    }

                    _ => {} // No more windowed events
                }
            }
            //
            // Idle
            //
            Event::MainEventsCleared => {
                if let TickResult::Stop = tick(app.as_mut(), &render, &key_state) {
                    *control_flow = ControlFlow::Exit;
                }
                key_state.pressed = false;
                key_state.vkey = None;
                window.request_redraw();
            }
            //
            // Redraw
            //
            Event::RedrawRequested(_) => {
                if let PresentResult::Changed = present(app.as_ref(), &mut render) {
                    match render.render() {
                        Ok(_) => {}
                        Err(SwapChainError::Lost) => render.resize(window.inner_size()),
                        Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(e) => eprintln!("{:?}", e),
                    };
                }
            }

            _ => {} // No more events
        }
    });
}

fn tick(app: &mut dyn App, render: &RenderState, key_state: &KeyState) -> TickResult {
    let (width, height) = render.chars_size();
    let sim_input = TickInput {
        dt: Duration::zero(),
        width,
        height,
        key: (*key_state).clone(),
        mouse: None,
    };

    app.tick(sim_input)
}

fn present(app: &dyn App, render: &mut RenderState) -> PresentResult {
    let (width, height) = render.chars_size();
    let (fore_image, back_image, text_image) = render.images();

    let present_input = PresentInput {
        width: width as usize,
        height: height as usize,
        fore_image,
        back_image,
        text_image,
    };

    app.present(present_input)
}
