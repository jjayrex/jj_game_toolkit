use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock, mpsc};

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VK_BACK, VK_CLEAR, VK_DECIMAL, VK_DELETE, VK_DOWN, VK_END, VK_HOME, VK_INSERT, VK_LEFT,
    VK_NEXT, VK_NUMPAD0, VK_NUMPAD9, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED,
    MSG, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, WH_KEYBOARD_LL, WM_KEYDOWN,
    WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEvent {
    NumEnter,
    NumDigit(u8),
    NumDecimal,
    Backspace,
}

pub struct InputHandle {
    pub rx: mpsc::Receiver<KeyEvent>,
    pub capture: Arc<AtomicBool>,
}

pub fn start(ctx: egui::Context) -> InputHandle {
    let (tx, rx) = mpsc::channel();
    let capture = Arc::new(AtomicBool::new(false));
    spawn_hook_thread(tx, ctx, Arc::clone(&capture));
    InputHandle { rx, capture }
}

struct Shared {
    tx: mpsc::Sender<KeyEvent>,
    ctx: egui::Context,
    capture: Arc<AtomicBool>,
}

static SHARED: OnceLock<Shared> = OnceLock::new();

static SWALLOWED_DOWN: [AtomicBool; 256] = [const { AtomicBool::new(false) }; 256];
static IS_DOWN: [AtomicBool; 256] = [const { AtomicBool::new(false) }; 256];

pub fn spawn_hook_thread(tx: mpsc::Sender<KeyEvent>, ctx: egui::Context, capture: Arc<AtomicBool>) {
    if SHARED.set(Shared { tx, ctx, capture }).is_err() {
        panic!("input::start must only be called once");
    }

    std::thread::Builder::new()
        .name("keyboard-hook".into())
        .spawn(|| unsafe {
            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), None, 0)
                .expect("failed to install WH_KEYBOARD_LL hook");

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let _ = UnhookWindowsHookEx(hook);
        })
        .expect("failed to spawn keyboard hook thread");
}

fn classify(vk: u16, extended: bool, capturing: bool) -> (Option<KeyEvent>, bool) {
    if vk == VK_RETURN.0 {
        return if extended {
            (Some(KeyEvent::NumEnter), true)
        } else {
            (None, false)
        };
    }

    if (VK_NUMPAD0.0..=VK_NUMPAD9.0).contains(&vk) {
        let d = (vk - VK_NUMPAD0.0) as u8;
        return (Some(KeyEvent::NumDigit(d)), capturing);
    }
    if vk == VK_DECIMAL.0 {
        return (Some(KeyEvent::NumDecimal), capturing);
    }

    if !extended {
        let digit = match vk {
            v if v == VK_INSERT.0 => Some(0),
            v if v == VK_END.0 => Some(1),
            v if v == VK_DOWN.0 => Some(2),
            v if v == VK_NEXT.0 => Some(3),
            v if v == VK_LEFT.0 => Some(4),
            v if v == VK_CLEAR.0 => Some(5),
            v if v == VK_RIGHT.0 => Some(6),
            v if v == VK_HOME.0 => Some(7),
            v if v == VK_UP.0 => Some(8),
            v if v == VK_PRIOR.0 => Some(9),
            _ => None,
        };
        if let Some(d) = digit {
            return (Some(KeyEvent::NumDigit(d)), capturing);
        }
        if vk == VK_DELETE.0 {
            return (Some(KeyEvent::NumDecimal), capturing);
        }
    }

    if vk == VK_BACK.0 && capturing {
        return (Some(KeyEvent::Backspace), true);
    }

    (None, false)
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        let msg = wparam.0 as u32;
        let vk = (kb.vkCode & 0xFF) as usize;
        let injected = kb.flags.contains(LLKHF_INJECTED);

        if !injected {
            if let Some(shared) = SHARED.get() {
                match msg {
                    WM_KEYDOWN | WM_SYSKEYDOWN => {
                        let repeat = IS_DOWN[vk].swap(true, Ordering::Relaxed);
                        let extended = kb.flags.contains(LLKHF_EXTENDED);
                        let capturing = shared.capture.load(Ordering::Relaxed);
                        let (event, swallow) = classify(vk as u16, extended, capturing);

                        if !repeat {
                            if let Some(ev) = event {
                                let _ = shared.tx.send(ev);
                                shared.ctx.request_repaint();
                            }
                        }
                        if swallow {
                            SWALLOWED_DOWN[vk].store(true, Ordering::Relaxed);
                            return LRESULT(1);
                        }
                    }
                    WM_KEYUP | WM_SYSKEYUP => {
                        IS_DOWN[vk].store(false, Ordering::Relaxed);
                        // If we ate the key-down, eat the key-up too.
                        if SWALLOWED_DOWN[vk].swap(false, Ordering::Relaxed) {
                            return LRESULT(1);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}
