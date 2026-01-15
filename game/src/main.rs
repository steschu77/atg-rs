mod game;
mod gameplay;

#[cfg(target_os = "windows")]
mod win32 {
    use engine::core::clock::Clock;
    use engine::core::game_loop::GameLoop;
    use engine::core::input;
    use engine::error::{Error, Result};
    use engine::sys::win32::Win32GLContext;
    use windows::Win32::UI::Input::{
        GetRawInputData, HRAWINPUT, RAWINPUT, RAWINPUTHEADER, RID_INPUT, RIM_TYPEKEYBOARD,
        RIM_TYPEMOUSE,
    };
    use windows::Win32::{
        Foundation::*,
        UI::Input::{RAWINPUTDEVICE, RIDEV_INPUTSINK, RegisterRawInputDevices},
        UI::WindowsAndMessaging::*,
    };

    struct GameWindowParams {}

    struct GameWindow {
        clock: Clock,
        win32: Win32GLContext,
        game: super::game::Game,
        game_loop: GameLoop,
        input: input::Input,
    }

    impl engine::sys::win32::window::IWindow for GameWindow {
        type Params = GameWindowParams;
        fn create(hwnd: HWND, _params: &GameWindowParams) -> Result<Self> {
            let rid_mouse = RAWINPUTDEVICE {
                usUsagePage: 0x01,
                usUsage: 0x02, // Mouse
                dwFlags: RIDEV_INPUTSINK,
                hwndTarget: hwnd,
            };
            let rid_keyboard = RAWINPUTDEVICE {
                usUsagePage: 0x01,
                usUsage: 0x06, // Keyboard
                dwFlags: RIDEV_INPUTSINK,
                hwndTarget: hwnd,
            };
            unsafe {
                RegisterRawInputDevices(
                    &[rid_mouse, rid_keyboard],
                    size_of::<RAWINPUTDEVICE>() as u32,
                )
                .map_err(Error::from)?
            };

            let t_update = std::time::Duration::from_millis(10);
            let win32 = Win32GLContext::from_hwnd(hwnd).unwrap();
            let game_loop = GameLoop::new(t_update);
            let gl = win32.load()?;
            Ok(Self {
                clock: Clock::new(),
                win32,
                game: super::game::Game::new(gl, t_update)?,
                game_loop,
                input: input::Input::new(),
            })
        }

        fn on_create(&mut self) -> LRESULT {
            LRESULT(0)
        }

        fn on_destroy(&mut self) -> LRESULT {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }

        fn on_size(&mut self, cx: i32, cy: i32) -> LRESULT {
            self.game.resize(cx, cy);
            LRESULT(0)
        }

        fn on_gameloop(&mut self) -> LRESULT {
            if let Err(e) = self
                .game_loop
                .step(&mut self.game, &self.clock, &mut self.input)
            {
                eprintln!("Game loop exited with: {e:?}");
                unsafe { PostQuitMessage(0) };
                return LRESULT(0);
            }

            self.win32.swap_buffers();
            LRESULT(0)
        }

        fn on_key_event(&mut self, msg: u32, key: u32) -> LRESULT {
            match msg {
                WM_KEYDOWN => self.input.add_event(input::Event::KeyDown { key }),
                WM_KEYUP => self.input.add_event(input::Event::KeyUp { key }),
                _ => {}
            }
            LRESULT(0)
        }

        fn on_mouse_event(
            &mut self,
            msg: u32,
            _x: i32,
            _y: i32,
            _keys: u32,
            delta: i32,
        ) -> LRESULT {
            match msg {
                WM_MOUSEWHEEL => self.input.add_event(input::Event::Wheel { delta }),
                WM_LBUTTONDOWN => self.input.add_event(input::Event::ButtonDown { button: 1 }),
                WM_LBUTTONUP => self.input.add_event(input::Event::ButtonUp { button: 1 }),
                WM_RBUTTONDOWN => self.input.add_event(input::Event::ButtonDown { button: 2 }),
                WM_RBUTTONUP => self.input.add_event(input::Event::ButtonUp { button: 2 }),
                WM_MBUTTONDOWN => self.input.add_event(input::Event::ButtonDown { button: 3 }),
                WM_MBUTTONUP => self.input.add_event(input::Event::ButtonUp { button: 3 }),
                _ => {}
            }
            LRESULT(0)
        }

        fn on_input(&mut self, raw_input: HRAWINPUT) -> LRESULT {
            let mut data_size = 0u32;
            unsafe {
                GetRawInputData(
                    raw_input,
                    RID_INPUT,
                    None,
                    &mut data_size,
                    size_of::<RAWINPUTHEADER>() as u32,
                );
            }

            let mut raw_input_bytes = vec![0u8; data_size as usize];
            unsafe {
                GetRawInputData(
                    raw_input,
                    RID_INPUT,
                    Some(raw_input_bytes.as_mut_ptr() as *mut _),
                    &mut data_size,
                    size_of::<RAWINPUTHEADER>() as u32,
                )
            };

            unsafe {
                let raw: &RAWINPUT = &*(raw_input_bytes.as_ptr() as *const RAWINPUT);
                if raw.header.dwType == RIM_TYPEMOUSE.0 {
                    let mouse = raw.data.mouse;
                    if (mouse.lLastX != 0) || (mouse.lLastY != 0) {
                        self.input.add_event(input::Event::MouseMove {
                            x: mouse.lLastX,
                            y: mouse.lLastY,
                        });
                    }
                }
                if raw.header.dwType == RIM_TYPEKEYBOARD.0 {
                    let kb = raw.data.keyboard;
                    let vk = kb.VKey as usize;
                    match kb.Message {
                        WM_KEYDOWN | WM_SYSKEYDOWN => {
                            self.input.set_state(vk, 0x80);
                        }
                        WM_KEYUP | WM_SYSKEYUP => {
                            self.input.set_state(vk, 0x00);
                        }
                        _ => {}
                    }
                }
            }
            LRESULT(0)
        }
    }

    pub fn main() {
        let hwnd = engine::sys::win32::window::WindowProc::<GameWindow>::create(
            "Game",
            "GameWindow",
            WS_POPUP | WS_VISIBLE,
            &GameWindowParams {},
        );

        if let Ok(hwnd) = hwnd {
            engine::sys::win32::window::run_message_loop(hwnd);
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use fantasia::core::clock::Clock;
    use fantasia::core::game_loop::GameLoop;
    use fantasia::sys::linux::LinuxGLContext;

    pub fn main() {
        let display = unsafe { x11::xlib::XOpenDisplay(std::ptr::null()) };
        let screen = unsafe { x11::xlib::XDefaultScreen(display) };
        let root = unsafe { x11::xlib::XRootWindow(display, screen) };

        let win = unsafe { x11::xlib::XCreateSimpleWindow(display, root, 0, 0, 800, 600, 0, 0, 0) };

        unsafe {
            x11::xlib::XSelectInput(
                display,
                win,
                x11::xlib::ExposureMask | x11::xlib::KeyPressMask,
            );
            x11::xlib::XMapWindow(display, win);
        }

        let context = unsafe { LinuxGLContext::from_window(display, screen, win).unwrap() };
        let gl = context.load().unwrap();
        let clock = Clock::new();

        let t_update = std::time::Duration::from_millis(10);
        let mut game_loop = GameLoop::new(t_update);
        let mut game = super::game::Game::new(gl, t_update);

        loop {
            unsafe {
                let mut event: x11::xlib::XEvent = std::mem::zeroed();
                x11::xlib::XNextEvent(display, &mut event);
                match event.type_ {
                    x11::xlib::Expose => {
                        game_loop.step(&mut game, &clock).unwrap();
                        context.swap_buffers();
                    }
                    x11::xlib::KeyPress => break,
                    _ => (),
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn main() {
    win32::main();
}

#[cfg(target_os = "linux")]
pub fn main() {
    linux::main();
}
