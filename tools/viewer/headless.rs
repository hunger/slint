// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

use core::time::Duration;
use std::{
    collections::VecDeque,
    rc::Rc,
    result::Result,
    sync::{Arc, Mutex},
    thread::Thread,
};

use i_slint_core::{
    api::EventLoopError,
    platform::{duration_until_next_timer_update, EventLoopProxy, Platform, WindowAdapter},
    software_renderer::MinimalSoftwareWindow,
};
use slint_interpreter::PlatformError;

enum Event {
    Quit,
    Event(Box<dyn FnOnce() + Send>),
}

#[derive(Clone)]
struct Queue(Arc<Mutex<VecDeque<Event>>>, Thread);

impl EventLoopProxy for Queue {
    fn quit_event_loop(&self) -> Result<(), EventLoopError> {
        self.0.lock().unwrap().push_back(Event::Quit);
        self.1.unpark();
        Ok(())
    }

    fn invoke_from_event_loop(
        &self,
        event: Box<dyn FnOnce() + Send>,
    ) -> Result<(), EventLoopError> {
        self.0.lock().unwrap().push_back(Event::Event(event));
        self.1.unpark();
        Ok(())
    }
}

pub struct HeadlessBackend {
    window: Rc<MinimalSoftwareWindow>,
    queue: Option<Queue>,
}

impl Platform for HeadlessBackend {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> Duration {
        Duration::from_millis(i_slint_core::animations::current_tick().0)
    }

    fn run_event_loop(&self) -> Result<(), PlatformError> {
        let queue = match self.queue.as_ref() {
            Some(queue) => queue.clone(),
            None => return Err(PlatformError::NoEventLoopProvider),
        };

        loop {
            let e = queue.0.lock().unwrap().pop_front();
            i_slint_core::platform::update_timers_and_animations();
            match e {
                Some(Event::Quit) => break Ok(()),
                Some(Event::Event(e)) => e(),
                None => match duration_until_next_timer_update() {
                    _ => std::thread::park(),
                },
            }
        }
    }

    fn new_event_loop_proxy(&self) -> Option<Box<dyn EventLoopProxy>> {
        self.queue.as_ref().map(|q| Box::new(q.clone()) as Box<dyn EventLoopProxy>)
    }
}

pub fn init() {
    let window = MinimalSoftwareWindow::new(
        i_slint_core::software_renderer::RepaintBufferType::ReusedBuffer,
    );

    i_slint_core::platform::set_platform(Box::new(HeadlessBackend {
        window,
        queue: Some(Queue(Default::default(), std::thread::current())),
    }))
    .unwrap();
}
