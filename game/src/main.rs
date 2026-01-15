mod game;
mod gameplay;

// ----------------------------------------------------------------------------
#[cfg(target_os = "windows")]
pub fn main() {
    if let Err(e) = win32::main() {
        eprintln!("Error: {e:?}");
    }
}

// ----------------------------------------------------------------------------
#[cfg(target_os = "linux")]
pub fn main() {
    if let Err(e) = linux::main() {
        eprintln!("Error: {e:?}");
    }
}

#[cfg(target_os = "windows")]
mod win32 {
    use engine::core::clock::Clock;
    use engine::core::game_loop::GameLoop;
    use engine::core::input;
    use engine::core::input::Key;
    use engine::error::{Error, Result};
    use engine::sys::win32::Win32GLContext;
    use windows::Win32::UI::Input::{
        GetRawInputData, HRAWINPUT, KeyboardAndMouse, RAWINPUT, RAWINPUTHEADER, RID_INPUT,
        RIM_TYPEKEYBOARD, RIM_TYPEMOUSE,
    };
    use windows::Win32::{
        Foundation::*,
        UI::Input::{RAWINPUTDEVICE, RIDEV_INPUTSINK, RegisterRawInputDevices},
        UI::WindowsAndMessaging::*,
    };

    // ----------------------------------------------------------------------------
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
            if let Some(key) = vk_to_key(key) {
                match msg {
                    WM_KEYDOWN => self.input.add_event(input::Event::KeyDown { key }),
                    WM_KEYUP => self.input.add_event(input::Event::KeyUp { key }),
                    _ => {}
                }
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
                    if let Some(key) = vk_to_key(kb.VKey as u32) {
                        match kb.Message {
                            WM_KEYDOWN | WM_SYSKEYDOWN => {
                                self.input.set_state(key, 0x80);
                            }
                            WM_KEYUP | WM_SYSKEYUP => {
                                self.input.set_state(key, 0x00);
                            }
                            _ => {}
                        }
                    }
                }
            }
            LRESULT(0)
        }
    }

    // ------------------------------------------------------------------------
    fn vk_to_key(vk: u32) -> Option<Key> {
        const VK_ESCAPE: u32 = KeyboardAndMouse::VK_ESCAPE.0 as u32;
        const VK_LEFT: u32 = KeyboardAndMouse::VK_LEFT.0 as u32;
        const VK_RIGHT: u32 = KeyboardAndMouse::VK_RIGHT.0 as u32;
        const VK_UP: u32 = KeyboardAndMouse::VK_UP.0 as u32;
        const VK_DOWN: u32 = KeyboardAndMouse::VK_DOWN.0 as u32;
        const VK_W: u32 = KeyboardAndMouse::VK_W.0 as u32;
        const VK_A: u32 = KeyboardAndMouse::VK_A.0 as u32;
        const VK_S: u32 = KeyboardAndMouse::VK_S.0 as u32;
        const VK_D: u32 = KeyboardAndMouse::VK_D.0 as u32;

        match vk {
            VK_ESCAPE => Some(Key::Exit),
            VK_LEFT => Some(Key::LookLeft),
            VK_RIGHT => Some(Key::LookRight),
            VK_UP => Some(Key::LookUp),
            VK_DOWN => Some(Key::LookDown),
            VK_W => Some(Key::MoveForward),
            VK_S => Some(Key::MoveBackward),
            VK_A => Some(Key::StrafeLeft),
            VK_D => Some(Key::StrafeRight),

            _ => None,
        }
    }

    // ------------------------------------------------------------------------
    pub fn main() -> Result<()> {
        let hwnd = engine::sys::win32::window::WindowProc::<GameWindow>::create(
            "Game",
            "GameWindow",
            WS_POPUP | WS_VISIBLE,
            GameWindowParams {},
        );

        if let Ok(hwnd) = hwnd {
            engine::sys::win32::window::run_message_loop(hwnd);
        }

        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use engine::core::clock::Clock;
    use engine::core::game_loop::GameLoop;
    use engine::core::input;
    use engine::core::input::Key;
    use engine::error::{Error, Result};
    use engine::sys::linux::LinuxGLContext;
    use std::ptr::NonNull;
    use x11::xlib::{
        XCloseDisplay, XCreateSimpleWindow, XDefaultScreen, XDestroyWindow, XDisplayHeight,
        XDisplayWidth, XEvent, XLookupKeysym, XMapWindow, XNextEvent, XOpenDisplay, XPending,
        XRaiseWindow, XRootWindow, XSelectInput,
    };

    pub fn main() -> Result<()> {
        let display = unsafe { XOpenDisplay(std::ptr::null()) };
        let display = NonNull::new(display).ok_or(Error::InvalidDisplay)?;
            
        let screen = unsafe { XDefaultScreen(display.as_ptr()) };
        let root = unsafe { XRootWindow(display.as_ptr(), screen) };

        let cx = unsafe { XDisplayWidth(display.as_ptr(), screen) as u32 };
        let cy = unsafe { XDisplayHeight(display.as_ptr(), screen) as u32 };
        let win = unsafe { XCreateSimpleWindow(display.as_ptr(), root, 0, 0, cx, cy, 0, 0, 0) };

        unsafe {
            XSelectInput(
                display.as_ptr(),
                win,
                x11::xlib::ExposureMask | x11::xlib::KeyPressMask,
            );
            XMapWindow(display.as_ptr(), win);
            XRaiseWindow(display.as_ptr(), win);
        }

        let context = LinuxGLContext::from_window(display, screen, win)?;
        let gl = context.load()?;
        let clock = Clock::new();

        let t_update = std::time::Duration::from_millis(10);
        let mut game_loop = GameLoop::new(t_update);
        let mut game = super::game::Game::new(gl, t_update)?;
        let mut input = input::Input::new();
        
        game.resize(cx as i32, cy as i32);

        loop {
            while unsafe { XPending(display.as_ptr()) } > 0 {
                let mut event: XEvent = unsafe { std::mem::zeroed() };
                unsafe { XNextEvent(display.as_ptr(), &mut event) };

                match unsafe { event.type_ } {
                    x11::xlib::Expose => {}
                    x11::xlib::KeyPress => {
                        let keysym = unsafe { XLookupKeysym(&mut event.key as *mut _, 0) };
                        if let Some(key) = xkey_to_key(keysym as u32) {
                            input.add_event(input::Event::KeyDown { key });
                        }
                    }
                    _ => {}
                }
            }

            if let Err(e) = game_loop.step(&mut game, &clock, &mut input) {
                eprintln!("Game loop exited with: {e:?}");
                unsafe {
                    XDestroyWindow(display.as_ptr(), win);
                    XCloseDisplay(display.as_ptr());
                }
                return Ok(());
            }

            context.swap_buffers();
        }
    }

    #[allow(non_upper_case_globals)]
    fn xkey_to_key(keysym: u32) -> Option<Key> {
        use x11::keysym::{XK_A, XK_D, XK_Down, XK_Escape, XK_Left, XK_Right, XK_S, XK_Up, XK_W};
        match keysym {
            XK_Escape => Some(Key::Exit),
            XK_Left => Some(Key::LookLeft),
            XK_Right => Some(Key::LookRight),
            XK_Up => Some(Key::LookUp),
            XK_Down => Some(Key::LookDown),
            XK_W => Some(Key::MoveForward),
            XK_S => Some(Key::MoveBackward),
            XK_A => Some(Key::StrafeLeft),
            XK_D => Some(Key::StrafeRight),
            _ => None,
        }
    }
}
