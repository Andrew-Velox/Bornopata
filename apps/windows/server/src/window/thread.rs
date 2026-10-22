//! The window thread: makes the window, then draws what the Router sends until the Router is gone.

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use borno_platform::protocol::CursorRect;
use borno_render::{FontLibrary, Frame, Layout, Renderer, Theme};
use borno_window::placement::{pinned, place};
use borno_window::{PanelConfig, Pin, Rect, load_theme};
use windows::Win32::Foundation::{E_FAIL, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetCursorPos, GetMessageW,
    HWND_TOPMOST, IDC_SIZEALL, KillTimer, LoadCursorW, MA_NOACTIVATE, MSG, SW_HIDE, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SetTimer, SetWindowPos, ShowWindow,
    TranslateMessage, WM_CAPTURECHANGED, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEACTIVATE,
    WM_MOUSEMOVE, WM_RBUTTONDOWN, WM_TIMER, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};
use windows::core::{Error, PCWSTR, Result, w};

use super::class::WindowClass;
use super::drag::{self, Moved};
use super::layered::present;
use super::monitor::screen_at;
use super::{Command, WM_WAKE};

const CLASS_NAME: PCWSTR = w!("BornopataCandidateWindow");
static CLASS: WindowClass = WindowClass::new();

/// The id of the timer that asks for the next animation frame.
const ANIMATION_TIMER: usize = 1;

/// Starts the thread and waits until the window exists. Returns what the Router needs to reach it, or why it could not
/// be made.
pub(super) fn spawn(config: PanelConfig, pin: Arc<Pin>) -> Result<(Sender<Command>, u32)> {
    // The thread id instead of a `Result`, so no Windows error has to cross threads.
    let (ready, started) = mpsc::channel::<Option<u32>>();
    let (sender, commands) = mpsc::channel::<Command>();
    thread::Builder::new()
        .name("bornopata-candidates".to_owned())
        .spawn(move || run(&config, pin, &commands, &ready))
        .map_err(|_| Error::from(E_FAIL))?;
    match started.recv() {
        Ok(Some(thread_id)) => Ok((sender, thread_id)),
        _ => Err(Error::from(E_FAIL)),
    }
}

/// The window and what it draws with, all on the window thread.
struct Window {
    hwnd: HWND,
    renderer: Renderer,
    theme: Theme,
    layout: Layout,

    /// Where the bitmap's top-left corner is on the screen, so an animation frame lands on the same place.
    position: (i32, i32),

    pin: Arc<Pin>,

    /// The card inside the bitmap, for keeping a dragged window on the screen.
    card: Option<Card>,
}

struct Card {
    /// Where the card starts inside the bitmap, and its size, in physical pixels.
    offset: (i32, i32),
    size: (i32, i32),
}

fn run(
    config: &PanelConfig,
    pin: Arc<Pin>,
    commands: &Receiver<Command>,
    ready: &Sender<Option<u32>>,
) {
    // Positions are in physical pixels, like the cursor rectangles the applications report. Already set is no problem.
    let _ = unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
    let mut window = match Window::new(config, pin) {
        Ok(window) => window,
        Err(error) => {
            tracing::error!(%error, "cannot make the candidate window");
            let _ = ready.send(None);
            return;
        }
    };
    // After the window exists, so the thread has a message queue to post to.
    if ready.send(Some(unsafe { GetCurrentThreadId() })).is_err() {
        return;
    }
    let mut message = MSG::default();
    while unsafe { GetMessageW(&mut message, None, 0, 0) }.0 > 0 {
        if message.message == WM_WAKE {
            // One wake-up drains the queue, which keeps the order of Hide and Show.
            while let Ok(command) = commands.try_recv() {
                window.apply(command);
            }
        } else if message.message == WM_TIMER && message.wParam.0 == ANIMATION_TIMER {
            window.animation_frame();
        } else {
            unsafe {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            if let Some(moved) = drag::take() {
                window.dragged(moved);
            }
        }
    }
}

impl Window {
    fn new(config: &PanelConfig, pin: Arc<Pin>) -> Result<Self> {
        let library = FontLibrary::system("en").map_err(|error| {
            tracing::error!(%error, "no usable fonts");
            Error::from(E_FAIL)
        })?;
        let theme = load_theme(config).map_err(|error| {
            tracing::error!(%error, "no usable theme");
            Error::from(E_FAIL)
        })?;
        let mut renderer = Renderer::new(library);
        renderer.load_theme_fonts(&theme, |_| Vec::new());
        CLASS.ensure(|| WNDCLASSEXW {
            lpfnWndProc: Some(window_proc),
            hInstance: unsafe { GetModuleHandleW(None) }.unwrap_or_default().into(),
            hCursor: unsafe { LoadCursorW(None, IDC_SIZEALL) }.unwrap_or_default(),
            lpszClassName: CLASS_NAME,
            ..Default::default()
        })?;
        // NOACTIVATE: showing it must not take the focus from the application the user types in.
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
                CLASS_NAME,
                w!("Bornopata candidates"),
                WS_POPUP,
                0,
                0,
                0,
                0,
                None,
                None,
                Some(GetModuleHandleW(None).unwrap_or_default().into()),
                None,
            )?
        };
        Ok(Self {
            hwnd,
            renderer,
            theme,
            layout: config.layout,
            position: (0, 0),
            pin,
            card: None,
        })
    }

    fn apply(&mut self, command: Command) {
        match command {
            Command::Show(frame, cursor) => self.show(&frame, cursor),
            Command::Hide => self.hide(),
        }
    }

    /// Draws `frame` next to the text cursor: below it, above when there is no room below, always on the monitor.
    fn show(&mut self, frame: &Frame, cursor: CursorRect) {
        if frame.is_empty() {
            self.hide();
            return;
        }
        let corner = self.pin.get();
        let anchor = match corner {
            Some((x, y)) => CursorRect {
                x,
                y,
                width: 0.0,
                height: 0.0,
                surface_relative: false,
            },
            None if cursor.is_known() => cursor,
            None => mouse_cursor(),
        };
        let screen = screen_at(POINT {
            x: anchor.x as i32,
            y: anchor.y as i32,
        });
        let rendered = match self
            .renderer
            .render(frame, self.layout, &self.theme, screen.scale)
        {
            Ok(rendered) => rendered,
            Err(error) => {
                tracing::warn!(%error, "could not draw the candidate window");
                return;
            }
        };
        // `place` works in logical pixels (its gap is one), so the rectangles go in divided by the scale and the
        // answer comes out multiplied by it.
        let factor = screen.scale;
        let logical = |rect: Rect| {
            Rect::new(
                rect.x / factor,
                rect.y / factor,
                rect.width / factor,
                rect.height / factor,
            )
        };
        let card = (
            rendered.content_width as f32 / factor,
            rendered.content_height as f32 / factor,
        );
        let cursor_rect = Rect::new(anchor.x, anchor.y, anchor.width, anchor.height);
        let (x, y) = match corner {
            Some((cx, cy)) => pinned((cx / factor, cy / factor), card, logical(screen.work)),
            None => place(logical(cursor_rect), card, logical(screen.work)),
        };
        self.card = Some(Card {
            offset: (rendered.content_x as i32, rendered.content_y as i32),
            size: (
                rendered.content_width as i32,
                rendered.content_height as i32,
            ),
        });
        // The bitmap reaches beyond the card (shadow, decoration): its corner is the card's minus that margin.
        self.position = (
            (x * factor).round() as i32 - rendered.content_x as i32,
            (y * factor).round() as i32 - rendered.content_y as i32,
        );
        if let Err(error) = present(self.hwnd, &rendered.pixmap, self.position) {
            tracing::warn!(%error, "could not show the candidate window");
            return;
        }
        self.schedule(rendered.next_frame);
        // The Start menu and the search box are topmost too; whoever came last is on top, so go to the front each time.
        if let Err(error) = unsafe {
            SetWindowPos(
                self.hwnd,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            )
        } {
            tracing::warn!(%error, "could not bring the candidate window to the front");
        }
    }

    /// The mouse moved the window: the animation draws it at the new place, and where it is dropped is pinned.
    fn dragged(&mut self, moved: Moved) {
        match moved {
            Moved::To(x, y) => self.position = (x, y),
            Moved::Dropped(x, y) => {
                self.position = (x, y);
                self.keep_on_screen();
                let Some(card) = &self.card else { return };
                self.pin.move_to((
                    (self.position.0 + card.offset.0) as f32,
                    (self.position.1 + card.offset.1) as f32,
                ));
                if let Err(error) = self.pin.save() {
                    tracing::warn!(%error, "could not save where the window was dropped");
                }
            }
            Moved::Freed => {
                if let Err(error) = self.pin.clear() {
                    tracing::warn!(%error, "could not forget where the window was pinned");
                }
            }
        }
    }

    /// A window dropped half off the screen is pulled back onto the monitor it is mostly on.
    fn keep_on_screen(&mut self) {
        let Some(card) = &self.card else { return };
        let corner = (
            (self.position.0 + card.offset.0) as f32,
            (self.position.1 + card.offset.1) as f32,
        );
        let size = (card.size.0 as f32, card.size.1 as f32);
        let point = POINT {
            x: corner.0 as i32,
            y: corner.1 as i32,
        };
        let work = screen_at(point).work;
        let (x, y) = pinned(corner, size, work);
        self.position = (x as i32 - card.offset.0, y as i32 - card.offset.1);
        let _ = unsafe {
            SetWindowPos(
                self.hwnd,
                None,
                self.position.0,
                self.position.1,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
    }

    fn hide(&mut self) {
        let _ = unsafe { ShowWindow(self.hwnd, SW_HIDE) };
        self.stop_animation();
        self.renderer.forget();
    }

    /// The next animation frame goes on the same place; when the animation is over the timer stops.
    fn animation_frame(&mut self) {
        match self.renderer.tick() {
            Ok(Some(rendered)) => {
                let _ = present(self.hwnd, &rendered.pixmap, self.position);
                self.schedule(rendered.next_frame);
            }
            Ok(None) => self.stop_animation(),
            Err(error) => {
                tracing::warn!(%error, "animation frame failed");
                self.stop_animation();
            }
        }
    }

    /// Asks for the next frame after the renderer's interval; `None` stops. Setting the same timer again changes it.
    fn schedule(&self, next_frame: Option<Duration>) {
        match next_frame {
            Some(interval) => {
                let milliseconds = interval.as_millis().clamp(10, 1000) as u32;
                unsafe { SetTimer(Some(self.hwnd), ANIMATION_TIMER, milliseconds, None) };
            }
            None => self.stop_animation(),
        }
    }

    fn stop_animation(&self) {
        // A timer that is not running makes this fail, which is fine.
        let _ = unsafe { KillTimer(Some(self.hwnd), ANIMATION_TIMER) };
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        let _ = unsafe { DestroyWindow(self.hwnd) };
    }
}

/// A one line tall rectangle at the mouse, for applications that do not say where their cursor is.
fn mouse_cursor() -> CursorRect {
    let mut point = POINT::default();
    let _ = unsafe { GetCursorPos(&mut point) };
    CursorRect {
        x: point.x as f32,
        y: point.y as f32,
        width: 0.0,
        height: 16.0,
        surface_relative: false,
    }
}

/// A layered window needs no painting. It never takes the focus. Where its bitmap is transparent the mouse goes
/// through to the application; elsewhere it drags the window (left button) or lets it follow the text cursor again
/// (right button).
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_LBUTTONDOWN => {
            drag::press(hwnd);
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            drag::motion(hwnd);
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            drag::release(hwnd);
            LRESULT(0)
        }
        WM_CAPTURECHANGED => {
            drag::lost(hwnd);
            LRESULT(0)
        }
        WM_RBUTTONDOWN => {
            drag::free();
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}
