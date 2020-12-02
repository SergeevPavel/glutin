mod support;

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::{WindowBuilder, WindowId};
use glutin::ContextBuilder;
use support::{ContextCurrentWrapper, ContextTracker, ContextWrapper};
use glutin::platform::desktop::EventLoopExtDesktop;
use std::collections::{HashMap, HashSet};
use glutin::dpi::PhysicalSize;
use std::time::Duration;

fn main() {
    let mut el = EventLoop::new();
    let mut ct = ContextTracker::default();
    let mut isWindowsInitialized = false;

    let mut windows = HashMap::new();
    let mut is_resize_requested: HashSet<WindowId> = Default::default();

    el.run_return(move |event, window_target, control_flow| {
        if !isWindowsInitialized {
            isWindowsInitialized = true;
            for index in 0..1 {
                let title = format!("Charming Window #{}", index + 1);
                let wb = WindowBuilder::new().with_title(title);
                let windowed_context =
                    ContextBuilder::new().build_windowed(wb, window_target).unwrap();
                let windowed_context =
                    unsafe { windowed_context.make_current().unwrap() };
                let gl = support::load(&windowed_context.context());
                let window_id = windowed_context.window().id();
                let context_id = ct.insert(ContextCurrentWrapper::PossiblyCurrent(
                    ContextWrapper::Windowed(windowed_context),
                ));
                windows.insert(window_id, (context_id, gl, index));
            }
        }
        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, window_id } => match event {
                WindowEvent::Resized(physical_size) => {
                    let windowed_context = ct.get_current(windows[&window_id].0).unwrap();
                    let windowed_context = windowed_context.windowed();

                    is_resize_requested.insert(window_id);

                    windowed_context.window().request_redraw();

                }
                WindowEvent::CloseRequested => {
                    if let Some((cid, _, _)) = windows.remove(&window_id) {
                        ct.remove(cid);
                        println!(
                            "Window with ID {:?} has been closed",
                            window_id
                        );
                    }
                }
                _ => (),
            },

            Event::MainEventsCleared => {
                std::thread::sleep(Duration::from_millis(10));
            }

            Event::RedrawRequested(window_id) => {
                std::thread::sleep(Duration::from_millis(8));
                let window = windows.get_mut(&window_id).unwrap();
                let windowed_context = ct.get_current(window.0).unwrap();

                if is_resize_requested.remove(&window_id) {
                    let new_physical_size = windowed_context.windowed().window().inner_size();
                    windowed_context.windowed().resize(new_physical_size);
                    window.1.resize(new_physical_size.width, new_physical_size.height);
                }

                let mut color = [1.0, 0.5, 0.7, 1.0];
                color.swap(0, window.2 % 3);

                window.1.draw_frame(color);
                windowed_context.windowed().swap_buffers().unwrap();
            }
            _ => (),
        }

        if windows.is_empty() {
            *control_flow = ControlFlow::Exit
        } else {
            *control_flow = ControlFlow::Wait
        }
    });
}
