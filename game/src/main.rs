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
    use engine::util::logger;
    use windows::Win32::UI::Input::{
        GetRawInputData, HRAWINPUT, RAWINPUT, RAWINPUTHEADER, RID_INPUT, RIM_TYPEKEYBOARD,
        RIM_TYPEMOUSE,
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

            log::info!("Game is ready.");
            Ok(Self {
                clock: Clock::new(),
                win32,
                game: super::game::Game::new(gl)?,
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
            let events = self.input.take_events();
            let state = self.input.take_state();
            if let Err(e) = self
                .game_loop
                .step(&mut self.game, &self.clock, &events, &state)
            {
                log::info!("Game loop exited with: {e:?}");
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

    const VK_MAP: [Option<Key>; 256] = {
        let mut m = [None; 256];
        macro_rules! key_map {
            ($vk:expr, $key:expr) => {
                m[$vk.0 as usize] = Some($key);
            };
        }
        use windows::Win32::UI::Input::KeyboardAndMouse::*;
        key_map!(VK_ESCAPE, Key::k_Escape);
        key_map!(VK_F1, Key::k_F1);
        key_map!(VK_F2, Key::k_F2);
        key_map!(VK_F3, Key::k_F3);
        key_map!(VK_F4, Key::k_F4);
        key_map!(VK_F5, Key::k_F5);
        key_map!(VK_F6, Key::k_F6);
        key_map!(VK_F7, Key::k_F7);
        key_map!(VK_F8, Key::k_F8);
        key_map!(VK_F9, Key::k_F9);
        key_map!(VK_F10, Key::k_F10);
        key_map!(VK_F11, Key::k_F11);
        key_map!(VK_F12, Key::k_F12);
        key_map!(VK_RETURN, Key::k_Return);
        key_map!(VK_SPACE, Key::k_Space);
        key_map!(VK_BACK, Key::k_Backspace);
        key_map!(VK_TAB, Key::k_Tab);
        key_map!(VK_INSERT, Key::k_Insert);
        key_map!(VK_DELETE, Key::k_Delete);
        key_map!(VK_HOME, Key::k_Home);
        key_map!(VK_END, Key::k_End);
        key_map!(VK_PRIOR, Key::k_PageUp);
        key_map!(VK_NEXT, Key::k_PageDown);
        key_map!(VK_UP, Key::k_Up);
        key_map!(VK_DOWN, Key::k_Down);
        key_map!(VK_LEFT, Key::k_Left);
        key_map!(VK_RIGHT, Key::k_Right);
        key_map!(VK_LSHIFT, Key::k_LeftShift);
        key_map!(VK_LCONTROL, Key::k_LeftCtrl);
        key_map!(VK_LMENU, Key::k_LeftAlt);
        key_map!(VK_LWIN, Key::k_LeftSuper);
        key_map!(VK_RSHIFT, Key::k_RightShift);
        key_map!(VK_RCONTROL, Key::k_RightCtrl);
        key_map!(VK_RMENU, Key::k_RightAlt);
        key_map!(VK_RWIN, Key::k_RightSuper);
        key_map!(VK_0, Key::k_0);
        key_map!(VK_1, Key::k_1);
        key_map!(VK_2, Key::k_2);
        key_map!(VK_3, Key::k_3);
        key_map!(VK_4, Key::k_4);
        key_map!(VK_5, Key::k_5);
        key_map!(VK_6, Key::k_6);
        key_map!(VK_7, Key::k_7);
        key_map!(VK_8, Key::k_8);
        key_map!(VK_9, Key::k_9);
        key_map!(VK_A, Key::k_A);
        key_map!(VK_B, Key::k_B);
        key_map!(VK_C, Key::k_C);
        key_map!(VK_D, Key::k_D);
        key_map!(VK_E, Key::k_E);
        key_map!(VK_F, Key::k_F);
        key_map!(VK_G, Key::k_G);
        key_map!(VK_H, Key::k_H);
        key_map!(VK_I, Key::k_I);
        key_map!(VK_J, Key::k_J);
        key_map!(VK_K, Key::k_K);
        key_map!(VK_L, Key::k_L);
        key_map!(VK_M, Key::k_M);
        key_map!(VK_N, Key::k_N);
        key_map!(VK_O, Key::k_O);
        key_map!(VK_P, Key::k_P);
        key_map!(VK_Q, Key::k_Q);
        key_map!(VK_R, Key::k_R);
        key_map!(VK_S, Key::k_S);
        key_map!(VK_T, Key::k_T);
        key_map!(VK_U, Key::k_U);
        key_map!(VK_V, Key::k_V);
        key_map!(VK_W, Key::k_W);
        key_map!(VK_X, Key::k_X);
        key_map!(VK_Y, Key::k_Y);
        key_map!(VK_Z, Key::k_Z);

        m
    };

    // ------------------------------------------------------------------------
    fn vk_to_key(vk: u32) -> Option<Key> {
        VK_MAP.get(vk as usize).copied().flatten()
    }

    // ------------------------------------------------------------------------
    pub fn main() -> Result<()> {
        let _ = logger::init_logger(log::LevelFilter::Info);

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
    use engine::util::logger;
    use std::ptr::NonNull;
    use x11::xlib::{
        XCloseDisplay, XCreateSimpleWindow, XDefaultScreen, XDestroyWindow, XEvent, XLookupKeysym,
        XMapWindow, XNextEvent, XOpenDisplay, XPending, XRaiseWindow, XRootWindow, XSelectInput,
    };
    //use x11::xlib::{XDisplayHeight, XDisplayWidth};
    use std::collections::HashMap;

    pub fn main() -> Result<()> {
        let _ = logger::init_logger(log::LevelFilter::Info);

        let display = unsafe { XOpenDisplay(std::ptr::null()) };
        let display = NonNull::new(display).ok_or(Error::InvalidDisplay)?;

        let screen = unsafe { XDefaultScreen(display.as_ptr()) };
        let root = unsafe { XRootWindow(display.as_ptr(), screen) };

        let cx = 1280; // unsafe { XDisplayWidth(display.as_ptr(), screen) as u32 };
        let cy = 720; // unsafe { XDisplayHeight(display.as_ptr(), screen) as u32 };
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
        let mut game = super::game::Game::new(gl)?;
        let mut input = input::Input::new();

        game.resize(cx as i32, cy as i32);

        let key_map = key_map();
        loop {
            while unsafe { XPending(display.as_ptr()) } > 0 {
                let mut event: XEvent = unsafe { std::mem::zeroed() };
                unsafe { XNextEvent(display.as_ptr(), &mut event) };

                match unsafe { event.type_ } {
                    x11::xlib::Expose => {}
                    x11::xlib::KeyPress => {
                        let keysym = unsafe { XLookupKeysym(&mut event.key as *mut _, 0) } as u32;
                        if let Some(key) = key_map.get(&keysym).copied() {
                            input.add_event(input::Event::KeyDown { key });
                        }
                    }
                    _ => {}
                }
            }

            let events = input.take_events();
            let state = input.take_state();

            if let Err(e) = game_loop.step(&mut game, &clock, &events, &state) {
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
    fn key_map() -> HashMap<u32, Key> {
        use x11::keysym::*;
        HashMap::from([
            (XK_Escape, Key::k_Escape),
            (XK_F1, Key::k_F1),
            (XK_F2, Key::k_F2),
            (XK_F3, Key::k_F3),
            (XK_F4, Key::k_F4),
            (XK_F5, Key::k_F5),
            (XK_F6, Key::k_F6),
            (XK_F7, Key::k_F7),
            (XK_F8, Key::k_F8),
            (XK_F9, Key::k_F9),
            (XK_F10, Key::k_F10),
            (XK_F11, Key::k_F11),
            (XK_F12, Key::k_F12),
            (XK_Return, Key::k_Return),
            (XK_space, Key::k_Space),
            (XK_BackSpace, Key::k_Backspace),
            (XK_Tab, Key::k_Tab),
            (XK_Insert, Key::k_Insert),
            (XK_Delete, Key::k_Delete),
            (XK_Home, Key::k_Home),
            (XK_End, Key::k_End),
            (XK_Page_Up, Key::k_PageUp),
            (XK_Page_Down, Key::k_PageDown),
            (XK_Up, Key::k_Up),
            (XK_Down, Key::k_Down),
            (XK_Left, Key::k_Left),
            (XK_Right, Key::k_Right),
            (XK_Shift_L, Key::k_LeftShift),
            (XK_Control_L, Key::k_LeftCtrl),
            (XK_Alt_L, Key::k_LeftAlt),
            (XK_Super_L, Key::k_LeftSuper),
            (XK_Shift_R, Key::k_RightShift),
            (XK_Control_R, Key::k_RightCtrl),
            (XK_Alt_R, Key::k_RightAlt),
            (XK_Super_R, Key::k_RightSuper),
            (XK_0, Key::k_0),
            (XK_1, Key::k_1),
            (XK_2, Key::k_2),
            (XK_3, Key::k_3),
            (XK_4, Key::k_4),
            (XK_5, Key::k_5),
            (XK_6, Key::k_6),
            (XK_7, Key::k_7),
            (XK_8, Key::k_8),
            (XK_9, Key::k_9),
            (XK_A, Key::k_A),
            (XK_B, Key::k_B),
            (XK_C, Key::k_C),
            (XK_D, Key::k_D),
            (XK_E, Key::k_E),
            (XK_F, Key::k_F),
            (XK_G, Key::k_G),
            (XK_H, Key::k_H),
            (XK_I, Key::k_I),
            (XK_J, Key::k_J),
            (XK_K, Key::k_K),
            (XK_L, Key::k_L),
            (XK_M, Key::k_M),
            (XK_N, Key::k_N),
            (XK_O, Key::k_O),
            (XK_P, Key::k_P),
            (XK_Q, Key::k_Q),
            (XK_R, Key::k_R),
            (XK_S, Key::k_S),
            (XK_T, Key::k_T),
            (XK_U, Key::k_U),
            (XK_V, Key::k_V),
            (XK_W, Key::k_W),
            (XK_X, Key::k_X),
            (XK_Y, Key::k_Y),
            (XK_Z, Key::k_Z),
        ])
    }
}
