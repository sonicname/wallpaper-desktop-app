//! A Slint platform that draws into memory. Shared by the offscreen example and the tests.
#![allow(dead_code)]

use std::{
    collections::VecDeque,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use slint::{
    EventLoopError, PhysicalSize, PlatformError,
    platform::{
        EventLoopProxy, Platform, WindowAdapter, WindowEvent,
        software_renderer::{MinimalSoftwareWindow, PremultipliedRgbaColor, RepaintBufferType},
    },
};

type Task = Box<dyn FnOnce() + Send>;

#[derive(Clone, Default)]
struct Tasks(Arc<Mutex<VecDeque<Task>>>);

impl EventLoopProxy for Tasks {
    fn quit_event_loop(&self) -> Result<(), EventLoopError> {
        Ok(())
    }

    fn invoke_from_event_loop(&self, task: Task) -> Result<(), EventLoopError> {
        self.0.lock().unwrap().push_back(task);
        Ok(())
    }
}

struct Offscreen {
    window: Rc<MinimalSoftwareWindow>,
    tasks: Tasks,
    start: Instant,
}

impl Platform for Offscreen {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> Duration {
        self.start.elapsed()
    }

    fn new_event_loop_proxy(&self) -> Option<Box<dyn EventLoopProxy>> {
        Some(Box::new(self.tasks.clone()))
    }
}

pub struct Stage {
    pub window: Rc<MinimalSoftwareWindow>,
    tasks: Tasks,
    pub buffer: Vec<PremultipliedRgbaColor>,
    pub width: u32,
    pub height: u32,
}

impl Stage {
    pub fn new(width: u32, height: u32, scale: f32) -> Self {
        let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
        let tasks = Tasks::default();
        let platform = Offscreen {
            window: window.clone(),
            tasks: tasks.clone(),
            start: Instant::now(),
        };
        slint::platform::set_platform(Box::new(platform)).unwrap();
        window.dispatch_event(WindowEvent::ScaleFactorChanged {
            scale_factor: scale,
        });
        window.set_size(PhysicalSize::new(width, height));
        let buffer = vec![PremultipliedRgbaColor::default(); (width * height) as usize];
        Self {
            window,
            tasks,
            buffer,
            width,
            height,
        }
    }

    /// Runs timers and answers from worker threads, then draws. Returns the drawing time.
    pub fn frame(&mut self) -> Option<Duration> {
        slint::platform::update_timers_and_animations();
        loop {
            let task = self.tasks.0.lock().unwrap().pop_front();
            let Some(task) = task else { break };
            task();
        }
        let (buffer, stride) = (&mut self.buffer, self.width as usize);
        let mut spent = None;
        self.window.draw_if_needed(|renderer| {
            let start = Instant::now();
            renderer.render(buffer, stride);
            spent = Some(start.elapsed());
        });
        spent
    }

    pub fn run_until(&mut self, limit: Duration, mut done: impl FnMut() -> bool) -> bool {
        let start = Instant::now();
        while start.elapsed() < limit {
            self.frame();
            if done() {
                self.frame();
                return true;
            }
            thread::sleep(Duration::from_millis(4));
        }
        false
    }

    pub fn run_for(&mut self, time: Duration) {
        self.run_until(time, || false);
    }
}
