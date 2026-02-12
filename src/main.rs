#![no_std]
#![no_main]

extern crate flipperzero_rt;

use core::ffi::c_void;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::usize;

use flipperzero::furi::sync::{Mutex, MutexGuard};
use flipperzero_rt::{entry, manifest};
use flipperzero_sys::*;

manifest!(name = "Sliding Tower", app_version = 1, has_icon = false,);

static mut APP_STATE: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Menu,
    GameOver,
    Playing,
    Pause,
    Quit,
    Err,
}

pub fn get_app_state() -> AppState {
    #[allow(static_mut_refs)]
    unsafe {
        APP_STATE.load(Ordering::Relaxed).into()
    }
}
pub fn update_app_state(new_state: AppState) {
    #[allow(static_mut_refs)]
    unsafe {
        APP_STATE.store(new_state.into(), Ordering::Relaxed)
    };
}

pub fn get_game_state() -> Option<MutexGuard<'static, GameState>> {
    unsafe {
        #[allow(static_mut_refs)]
        let res = GAME_STATE.try_lock();
        if res.is_none() {
            ERROR_CODE = 2;
        }
        res
    }
}

impl From<usize> for AppState {
    fn from(n: usize) -> Self {
        match n {
            0 => AppState::Menu,
            1 => AppState::GameOver,
            2 => AppState::Playing,
            3 => AppState::Pause,
            4 => AppState::Quit,
            _ => unsafe {
                ERROR_CODE = 1;
                AppState::Err
            },
        }
    }
}

impl Into<usize> for AppState {
    fn into(self) -> usize {
        match self {
            AppState::Menu => 0,
            AppState::GameOver => 1,
            AppState::Playing => 2,
            AppState::Pause => 3,
            AppState::Quit => 4,
            _ => 9,
        }
    }
}

// 1 = appstate load (from) error
// 2 = gamestate load error
static mut ERROR_CODE: u8 = 0;
static mut GAME_STATE: Mutex<GameState> = Mutex::new(GameState {
    x: 0,
    y: 10,
    w: 64,
    speed: 1,
    drop_btn: false,
    drop_btn_release: true,
    tower: STARTING_TOWER,
    score: 0,
});

// Box state
pub struct GameState {
    x: i32,
    y: i32,
    w: i32,
    speed: i32,
    drop_btn: bool,
    drop_btn_release: bool,
    tower: [PlacedBoxSlot; 5],
    score: i32,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum PlacedBoxSlot {
    Used(PlacedBoxData), // WIDTH
    Empty,
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct PlacedBoxData {
    w: usize,
    x: i32,
}

const BOX_HEIGHT: usize = 8;
const PLACED_HEIGHT: usize = 7;

static STARTING_TOWER: [PlacedBoxSlot; 5] = [
    PlacedBoxSlot::Empty,
    PlacedBoxSlot::Empty,
    PlacedBoxSlot::Empty,
    PlacedBoxSlot::Empty,
    PlacedBoxSlot::Used(PlacedBoxData { w: 64, x: 32 }),
];

unsafe extern "C" fn draw_cb(canvas: *mut Canvas, _: *mut c_void) {
    let app_state = get_app_state();
    let game_state = get_game_state();
    if app_state == AppState::Err || game_state.is_none() {
        unsafe {
            canvas_clear(canvas);
            canvas_draw_str(canvas, 20, 20, b"There was an error: ".as_ptr());
            let mut buf = [0u8; 7];
            buf[..5].copy_from_slice(b"CODE ");
            buf[5] = b'0' + ERROR_CODE.max(9);
            buf[6] = 0;

            canvas_draw_str(canvas, 20, 28, buf.as_ptr());
            return;
        }
    }
    let mut s = game_state.unwrap();

    if s.drop_btn {
        // All full
        let replace_index = if !matches!(s.tower[0], PlacedBoxSlot::Empty) {
            let tower_height = s.tower.len();
            s.tower.copy_within(..tower_height - 1, 1);
            0
        } else {
            s.tower
                .iter()
                .enumerate()
                .rev()
                .find(|(_, d)| matches!(d, PlacedBoxSlot::Empty))
                .unwrap()
                .0
        };
        let last_box = match s.tower[replace_index + 1] {
            PlacedBoxSlot::Used(placed_box_data) => placed_box_data,
            PlacedBoxSlot::Empty => PlacedBoxData { w: 64, x: 32 },
        };

        if last_box.x > s.x + s.w || s.x > last_box.x + last_box.w as i32 {
            update_app_state(AppState::GameOver);
            unsafe { canvas_draw_box(canvas, 10, 10, 10, 10) };
            return;
        }

        let new_box = if s.x == last_box.x {
            PlacedBoxData {
                w: s.w.try_into().unwrap(),
                x: s.x,
            }
        } else if s.x < last_box.x {
            // left overhang
            let new_w = s.w.abs_diff(last_box.x.abs_diff(s.x) as i32) as i32;
            s.x += (s.w + new_w) / 2;
            s.w = new_w;
            PlacedBoxData {
                w: new_w.try_into().unwrap(),
                x: last_box.x,
            }
        } else {
            // right overhang
            let new_w = (s.x + s.w) - (last_box.x + last_box.w as i32);
            s.x += (s.w + new_w) / 2;
            s.w = new_w;
            PlacedBoxData {
                w: new_w.try_into().unwrap(),
                x: s.x,
            }
        };
        s.tower[replace_index] = PlacedBoxSlot::Used(new_box);
        s.score += 1;
    }

    s.x += s.speed;

    // Bounce off screen edges (screen: 128x64)
    if s.x <= 0 || s.x + s.w >= 128 {
        s.speed = -s.speed;
    }

    // Draw box
    unsafe {
        canvas_clear(canvas);
        canvas_draw_box(canvas, s.x, s.y, s.w.try_into().unwrap(), BOX_HEIGHT);

        // let mut buf = [0u8; 7];
        // buf[..5].copy_from_slice(b"SCORE: ");
        // buf[5] = b'0' + s.score.max(9) as u8;
        // buf[6] = 0;
        // canvas_draw_str(canvas, 120, 10, buf.as_ptr());

        // Draw Previous
        for (index, slot) in s.tower.iter().enumerate() {
            let slot = match slot {
                PlacedBoxSlot::Used(data) => data,
                PlacedBoxSlot::Empty => continue,
            };
            canvas_draw_box(
                canvas,
                slot.x,
                (18 + (index * (PLACED_HEIGHT + 1))).try_into().unwrap(),
                slot.w,
                PLACED_HEIGHT,
            );
        }
    };
}

unsafe extern "C" fn input_cb(event: *mut InputEvent, _: *mut c_void) {
    let ev = unsafe { &*event };

    if ev.type_ == InputTypeLong && ev.key == InputKeyBack {
        update_app_state(AppState::Quit);
        return;
    }

    let app_state = get_app_state();
    let mut game_state = match get_game_state() {
        Some(s) => s,
        None => {
            update_app_state(AppState::Err);
            return;
        }
    };

    if app_state == AppState::Playing {
        if ev.key == InputKeyOk && ev.type_ == InputTypePress && game_state.drop_btn_release == true
        {
            game_state.drop_btn = true;
            return;
        }

        if ev.key == InputKeyOk && ev.type_ == InputTypeRelease {
            game_state.drop_btn_release = true;
            return;
        }
    } else {
        game_state.drop_btn_release = true;
    }
}

entry!(main);

fn main(_args: Option<&core::ffi::CStr>) -> i32 {
    let app_state = get_app_state();
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
        while app_state != AppState::Quit {
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
