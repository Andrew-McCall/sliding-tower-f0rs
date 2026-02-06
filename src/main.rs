#![no_std]
#![no_main]
#![allow(static_mut_refs)]

extern crate flipperzero_rt;

use core::ffi::c_void;
use core::sync::atomic::{AtomicBool, Ordering};
use core::usize;

use flipperzero_rt::{entry, manifest};
use flipperzero_sys::*;

manifest!(name = "Sliding Tower", app_version = 1, has_icon = false,);

static EXIT: AtomicBool = AtomicBool::new(false);
static PLACED: AtomicBool = AtomicBool::new(false);
static PLACED_UP: AtomicBool = AtomicBool::new(true);

// Box state
struct BoxState {
    x: i32,
    y: i32,
    w: i32,
    dx: i32,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum PlacedBoxSlot {
    Used(PlacedBoxData), // WIDTH
    Empty
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct PlacedBoxData{
    w: usize,
    x: i32,
}
const BOX_HEIGHT: usize = 8;
const PLACED_HEIGHT: usize = 7;

// Initial state: top-left, moving right
static mut BOX_STATE: BoxState = BoxState {
    x: 0,
    y: 8,
    w: 64,
    dx: 1, // SPEED
};

static mut PREVIOUS: [PlacedBoxSlot; 5] = [PlacedBoxSlot::Empty,PlacedBoxSlot::Empty,PlacedBoxSlot::Empty,PlacedBoxSlot::Empty,PlacedBoxSlot::Used(PlacedBoxData { w: 64, x: 32 })];

unsafe extern "C" fn draw_cb(canvas: *mut Canvas, _: *mut c_void) {
    unsafe {
        #[allow(static_mut_refs)]
        let s = &mut BOX_STATE;

  if PLACED.load(Ordering::Relaxed) {
    PLACED.store(false, Ordering::Relaxed);

    // Find the top-most non-empty slot
    let mut top_box = None;
    for (index, slot) in PREVIOUS.iter().enumerate() {
        if !matches!(slot, PlacedBoxSlot::Empty) {
            top_box = Some(index);
        }
    }

    let Some(mut top_box) = top_box else {
        return; // Silent exit if all slots are empty
    };

    let new_box = PlacedBoxData {
        x: s.x,
        w: s.w.try_into().unwrap(),
    };

    // Shift all elements to the right to make space
    for i in (1..PREVIOUS.len()).rev() {
        PREVIOUS[i] = PREVIOUS[i - 1];
    }
    PREVIOUS[0] = PlacedBoxSlot::Empty;

    // Place the new box in the first non-empty slot after shift
    PREVIOUS[0] = PlacedBoxSlot::Used(new_box);
}



        // Move box
        s.x += s.dx;

        // Bounce off screen edges (screen: 128x64)
        if s.x <= 0 || s.x + s.w >= 128 {
            s.dx = -s.dx;
        }

        // Draw box
        canvas_clear(canvas);
        canvas_draw_box(
            canvas,
            s.x,
            s.y,
            s.w.try_into().unwrap(),
            BOX_HEIGHT,
        );

        // Draw Previous
        for (index, slot) in PREVIOUS.iter().enumerate(){
            let slot = match slot {
                PlacedBoxSlot::Used(data) => data,
                PlacedBoxSlot::Empty => continue,
            };
            canvas_draw_box(canvas, slot.x, (18+(index*(PLACED_HEIGHT+1))).try_into().unwrap(), slot.w, PLACED_HEIGHT);
        }
    } 
}

unsafe extern "C" fn input_cb(event: *mut InputEvent, _: *mut c_void) {
    let ev = unsafe { &*event };
    if ev.key == InputKeyOk{ //&& PLACED_UP.load(Ordering::Relaxed) {

        PLACED.store(true, Ordering::Relaxed);
    }

    if ev.type_ == InputTypePress && ev.key == InputKeyBack {
            EXIT.store(true, Ordering::Relaxed);
            return
    }

    // if ev.key == InputKeyOk && ev.type_ == InputTypeRelease {
    //     PLACED_UP.store(true, Ordering::Relaxed);
    // }
}

entry!(main);

fn main(_args: Option<&core::ffi::CStr>) -> i32 {
    unsafe {
        // Allocate viewport
        let viewport = view_port_alloc();
        view_port_draw_callback_set(viewport, Some(draw_cb), core::ptr::null_mut());
        view_port_input_callback_set(viewport, Some(input_cb), core::ptr::null_mut());

        // Open GUI
        let gui = furi_record_open(b"gui\0".as_ptr() as _) as *mut Gui;
        gui_add_view_port(gui, viewport, GuiLayerFullscreen);
        view_port_enabled_set(viewport, true);

        // Main loop
        while !EXIT.load(Ordering::Relaxed) {
            view_port_update(viewport);
            furi_delay_ms(12); // ~80 FPS
        }

        // Cleanup
        view_port_enabled_set(viewport, false);
        gui_remove_view_port(gui, viewport);
        view_port_free(viewport);
    }

    0
}
