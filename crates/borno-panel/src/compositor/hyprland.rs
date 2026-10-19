//! Hyprland: window and monitor geometry over its command socket (`j/activewindow`, `j/monitors`).

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

use serde_json::Value;

use super::{Compositor, CursorRect, Located};
use crate::geometry::Rect;

const TIMEOUT: Duration = Duration::from_millis(150);

pub struct Hyprland {
    socket: PathBuf,
}

impl Hyprland {
    /// From `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock`; `None` outside Hyprland.
    pub fn from_env() -> Option<Self> {
        let runtime = std::env::var_os("XDG_RUNTIME_DIR")?;
        let signature = std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE")?;
        let socket = PathBuf::from(runtime)
            .join("hypr")
            .join(signature)
            .join(".socket.sock");
        socket.exists().then_some(Self { socket })
    }

    /// Talks to the socket at `path`.
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self {
            socket: path.into(),
        }
    }

    fn request(&self, command: &str) -> Option<Value> {
        let mut stream = UnixStream::connect(&self.socket).ok()?;
        stream.set_read_timeout(Some(TIMEOUT)).ok()?;
        stream.set_write_timeout(Some(TIMEOUT)).ok()?;
        stream.write_all(command.as_bytes()).ok()?;
        let mut reply = String::new();
        stream.read_to_string(&mut reply).ok()?;
        serde_json::from_str(&reply).ok()
    }
}

impl Compositor for Hyprland {
    fn locate(&mut self, cursor: CursorRect) -> Option<Located> {
        let global = if cursor.surface_relative {
            let window = self.request("j/activewindow")?;
            // X11 clients run through XWayland report global coordinates whatever the frontend says.
            if window.get("xwayland").and_then(Value::as_bool) == Some(true) {
                cursor.rect
            } else {
                let at = window.get("at")?.as_array()?;
                let (x, y) = (at.first()?.as_f64()? as f32, at.get(1)?.as_f64()? as f32);
                Rect::new(
                    x + cursor.rect.x,
                    y + cursor.rect.y,
                    cursor.rect.width,
                    cursor.rect.height,
                )
            }
        } else {
            cursor.rect
        };
        let monitors = self.request("j/monitors")?;
        let monitors = monitors.as_array()?;
        let mut found = monitors
            .iter()
            .filter_map(monitor)
            .filter(|(rect, _, _)| rect.contains(global.x, global.y));
        let (rect, name, scale) = found
            .next()
            .or_else(|| monitors.iter().filter_map(monitor).next())?;
        Some(Located {
            cursor: global,
            monitor: rect,
            monitor_name: name,
            scale,
        })
    }
}

/// A monitor entry as (logical rectangle, name, scale). Sizes are physical pixels divided by the scale, and a
/// 90 or 270 degree transform swaps the sides.
fn monitor(entry: &Value) -> Option<(Rect, String, f32)> {
    let scale = entry.get("scale")?.as_f64()? as f32;
    let (mut width, mut height) = (
        entry.get("width")?.as_f64()? as f32,
        entry.get("height")?.as_f64()? as f32,
    );
    if entry
        .get("transform")
        .and_then(Value::as_i64)
        .is_some_and(|t| t % 2 == 1)
    {
        std::mem::swap(&mut width, &mut height);
    }
    let (x, y) = (
        entry.get("x")?.as_f64()? as f32,
        entry.get("y")?.as_f64()? as f32,
    );
    let name = entry.get("name")?.as_str()?.to_owned();
    (scale > 0.0).then(|| (Rect::new(x, y, width / scale, height / scale), name, scale))
}

#[cfg(test)]
mod tests {
    use std::os::unix::net::UnixListener;
    use std::thread;

    use super::*;

    const MONITORS: &str = r#"[
        {"name":"DP-1","x":0,"y":0,"width":2560,"height":1440,"scale":1.25,"transform":0},
        {"name":"HDMI-A-1","x":2048,"y":0,"width":1080,"height":1920,"scale":1.0,"transform":1}
    ]"#;

    /// A fake Hyprland socket answering `j/activewindow` and `j/monitors`, one connection per request.
    fn serve(name: &str, window: &'static str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("borno-hypr-{name}-{}.sock", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path).unwrap();
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { return };
                let mut command = [0u8; 64];
                let count = stream.read(&mut command).unwrap_or(0);
                let reply = match &command[..count] {
                    b"j/activewindow" => window,
                    b"j/monitors" => MONITORS,
                    _ => "unknown request",
                };
                let _ = stream.write_all(reply.as_bytes());
            }
        });
        path
    }

    #[test]
    fn a_wayland_cursor_is_moved_by_the_window_origin() {
        let path = serve(
            "window",
            r#"{"at":[100,200],"size":[800,600],"xwayland":false}"#,
        );
        let mut compositor = Hyprland::at(&path);
        let located = compositor
            .locate(CursorRect {
                rect: Rect::new(30.0, 40.0, 2.0, 18.0),
                surface_relative: true,
            })
            .unwrap();
        assert_eq!(located.cursor, Rect::new(130.0, 240.0, 2.0, 18.0));
        assert_eq!(located.monitor_name, "DP-1");
        assert_eq!(located.scale, 1.25);
        assert_eq!(located.monitor, Rect::new(0.0, 0.0, 2048.0, 1152.0));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn an_xwayland_window_reports_global_coordinates_already() {
        let path = serve(
            "xwayland",
            r#"{"at":[100,200],"size":[800,600],"xwayland":true}"#,
        );
        let located = Hyprland::at(&path)
            .locate(CursorRect {
                rect: Rect::new(530.0, 640.0, 2.0, 18.0),
                surface_relative: true,
            })
            .unwrap();
        assert_eq!(located.cursor, Rect::new(530.0, 640.0, 2.0, 18.0));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn a_global_cursor_is_used_as_is_and_picks_its_monitor() {
        let path = serve("global", "{}");
        let mut compositor = Hyprland::at(&path);
        let located = compositor
            .locate(CursorRect {
                rect: Rect::new(2500.0, 300.0, 2.0, 18.0),
                surface_relative: false,
            })
            .unwrap();
        assert_eq!(located.cursor.x, 2500.0);
        assert_eq!(located.monitor_name, "HDMI-A-1");
        // A rotated monitor swaps its sides: 1080x1920 becomes 1920x1080 logical pixels.
        assert_eq!(located.monitor, Rect::new(2048.0, 0.0, 1920.0, 1080.0));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn a_dead_socket_or_a_bad_reply_gives_none() {
        let mut gone = Hyprland::at("/nonexistent/hypr.sock");
        let cursor = CursorRect {
            rect: Rect::new(1.0, 1.0, 1.0, 1.0),
            surface_relative: true,
        };
        assert_eq!(gone.locate(cursor), None);
        let path = serve("bad", "not json");
        assert_eq!(Hyprland::at(&path).locate(cursor), None);
        let _ = std::fs::remove_file(path);
    }
}
