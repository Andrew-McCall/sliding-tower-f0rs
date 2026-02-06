#![no_std]
#![no_main]

extern crate flipperzero_rt;

use core::ffi::c_void;
use core::sync::atomic::{AtomicBool, Ordering, AtomicI32};

use flipperzero_rt::{entry, manifest};
use flipperzero_sys::*;

manifest!(
    name = "Hello Rust",
    app_version = 1,
    has_icon = false,
);

static EXIT: AtomicBool = AtomicBool::new(false);
static COUNTER: AtomicI32 = AtomicI32::new(0);

unsafe extern "C" fn draw_cb(canvas: *mut Canvas, _: *mut c_void) {
    let count =  COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut buf = [0u8; 12];
    let mut i = buf.len() - 1;
    buf[i] = 0;
    let mut n = count;

    loop {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 { break; }
    }

    unsafe {
        canvas_clear(canvas);
        canvas_draw_str(canvas, 2, 12, b"Katya is a cutiepie <3\0".as_ptr() as _);
        canvas_draw_str(canvas, 2, 28, b"Hold BACK to exit\0".as_ptr() as _);
        canvas_draw_str(canvas, 2, 44, buf[i..].as_ptr() as _);
    }
}

unsafe extern "C" fn input_cb(event: *mut InputEvent, _: *mut c_void) {
    unsafe {
        let ev = &*event;

        if ev.key == InputKeyBack && ev.type_ == InputTypePress {
            EXIT.store(true, Ordering::Relaxed);
        }
    }
}

entry!(main);

fn main(_args: Option<&core::ffi::CStr>) -> i32 {
    unsafe {
        let viewport = view_port_alloc();
        view_port_draw_callback_set(viewport, Some(draw_cb), core::ptr::null_mut());
        view_port_input_callback_set(viewport, Some(input_cb), core::ptr::null_mut());

        let gui = furi_record_open(b"gui\0".as_ptr() as _) as *mut Gui;
        gui_add_view_port(gui, viewport, GuiLayerFullscreen);

        view_port_enabled_set(viewport, true);
        view_port_update(viewport);

        while !EXIT.load(Ordering::Relaxed) {
            furi_delay_ms(15);
        }

        gui_remove_view_port(gui, viewport);
        view_port_free(viewport);
    }

    0
}
