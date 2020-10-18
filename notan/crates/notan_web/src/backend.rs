use crate::utils::request_animation_frame;
use crate::window::WebWindowBackend;
use notan_app::{App, Backend, InitializeFn};
use std::cell::RefCell;
use std::panic;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;

pub struct WebBackend {
    window: WebWindowBackend,
    exit_requested: bool,
}

impl WebBackend {
    pub fn new() -> Result<Self, String> {
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        Ok(Self {
            window: WebWindowBackend::new()?,
            exit_requested: false,
        })
    }
}

impl Backend for WebBackend {
    type Impl = WebBackend;
    type Window = WebWindowBackend;

    fn get_impl(&mut self) -> &mut Self::Impl {
        self
    }

    fn initialize<B, S, R>(&mut self) -> Result<Box<InitializeFn<B, S, R>>, String>
    where
        B: Backend<Impl = Self::Impl> + 'static,
        S: 'static,
        R: FnMut(&mut App<B>, &mut S) + 'static,
    {
        Ok(Box::new(move |mut app: App<B>, mut state: S, mut cb: R| {
            let callback = Rc::new(RefCell::new(None));
            let inner_callback = callback.clone();

            *callback.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                cb(&mut app, &mut state);

                let backend = app.backend.get_impl();
                if !backend.exit_requested {
                    request_animation_frame(
                        &backend.window.window,
                        inner_callback.borrow().as_ref().unwrap(),
                    );
                }
            }) as Box<dyn FnMut()>));

            let window = web_sys::window().unwrap();
            request_animation_frame(&window, callback.borrow().as_ref().unwrap());
            Ok(())
        }))
    }

    fn window(&mut self) -> &mut Self::Window {
        &mut self.window
    }

    fn exit(&mut self) {
        self.exit_requested = true;
    }
}

unsafe impl Send for WebBackend {}
unsafe impl Sync for WebBackend {}
