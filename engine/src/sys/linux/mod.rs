use super::opengl::*;
use crate::error::{Error, Result};
use std::ptr::NonNull;
use x11::glx::*;
use x11::xlib::*;

pub struct LinuxGLContext {
    display: NonNull<Display>,
    window: Window,
    context: x11::glx::GLXContext,
}

impl LinuxGLContext {
    pub fn from_window(
        display: NonNull<Display>,
        screen: std::os::raw::c_int,
        window: Window,
    ) -> Result<Self> {
        let mut attribs = [GLX_RGBA, GLX_DOUBLEBUFFER, GLX_DEPTH_SIZE, 24, 0];
        let visual_info =
            unsafe { glXChooseVisual(display.as_ptr(), screen, attribs.as_mut_ptr()) };
        if visual_info.is_null() {
            return Err(Error::InvalidVisualInfo);
        }

        let context =
            unsafe { glXCreateContext(display.as_ptr(), visual_info, std::ptr::null_mut(), 1) };
        if context.is_null() {
            return Err(Error::InvalidContext);
        }

        unsafe { glXMakeCurrent(display.as_ptr(), window, context) };
        Ok(Self {
            display,
            window,
            context,
        })
    }

    pub fn load(&self) -> Result<OpenGlFunctions> {
        OpenGlFunctions::load(|fn_name| {
            let fn_ptr = unsafe { glXGetProcAddress(fn_name.as_ptr() as *const _) };
            fn_ptr.map(|f| f as FnOpenGL)
        })
    }

    pub fn swap_buffers(&self) {
        unsafe { glXSwapBuffers(self.display.as_ptr(), self.window) };
    }
}

impl Drop for LinuxGLContext {
    fn drop(&mut self) {
        unsafe { glXDestroyContext(self.display.as_ptr(), self.context) };
    }
}
