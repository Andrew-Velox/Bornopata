//! The popup render thread against a socket pair standing in for the shell: pools with their descriptor, presented
//! bitmaps, scale changes, animation pacing and the pool of slots.

use std::io::{Read, Seek, SeekFrom};
use std::os::unix::net::UnixStream;
use std::time::Duration;

mod support;

use borno_linux_server::popup::{PopupHub, PopupSink};
use borno_platform::PanelConfig;
use borno_platform::protocol::{PopupFrame, PopupServerMessage};
use borno_render::sample::bangl;
use support::shell::Shell;

const TOKEN: u64 = 5;

/// A hub on `theme` with a shell attached, or `None` when this machine has no usable font.
fn rig(theme: &str) -> Option<(PopupHub, Shell)> {
    let config = PanelConfig {
        theme: theme.to_owned(),
        ..PanelConfig::default()
    };
    let hub = match PopupHub::start(&config) {
        Ok(hub) => hub,
        Err(error) => {
            eprintln!("skipped: {error}");
            return None;
        }
    };
    let (server, shell) = UnixStream::pair().unwrap();
    shell
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    hub.attach(TOKEN, server);
    Some((hub, Shell::new(shell)))
}

#[test]
fn a_frame_arrives_as_a_pool_with_its_descriptor_and_a_cropped_opaque_card() {
    let Some((mut hub, mut shell)) = rig("bornopata") else {
        return;
    };
    assert!(hub.attached(TOKEN) && !hub.attached(TOKEN + 1));
    assert!(hub.show(TOKEN, "ctx", bangl()));
    let Some(PopupServerMessage::Pool { pool, size }) = shell.next() else {
        panic!("the first message is the pool");
    };
    let frame = shell.present();
    assert_eq!((frame.context.as_str(), frame.pool), ("ctx", pool));
    assert_eq!((frame.scale_120, frame.stride), (120, frame.width * 4));
    assert_eq!(
        (frame.logical_width, frame.logical_height),
        (frame.width, frame.height)
    );
    assert!(frame.width > 100 && frame.height > 60);
    assert!(u64::from(frame.offset) + u64::from(frame.stride * frame.height) <= size);

    let mut memory = shell
        .fds
        .pop_front()
        .expect("the pool's descriptor came along");
    let mut pixels = vec![0u8; (frame.stride * frame.height) as usize];
    memory
        .seek(SeekFrom::Start(u64::from(frame.offset)))
        .unwrap();
    memory.read_exact(&mut pixels).unwrap();
    // The card is cropped out of the bitmap, so its middle is opaque (alpha is the fourth byte, ARGB8888 little-endian).
    let middle = ((frame.height / 2 * frame.width + frame.width / 2) * 4) as usize;
    assert_eq!(pixels[middle + 3], 255);
}

#[test]
fn a_new_scale_draws_the_frame_again_bigger() {
    let Some((mut hub, mut shell)) = rig("bornopata") else {
        return;
    };
    assert!(hub.show(TOKEN, "ctx", bangl()));
    let first = shell.present();
    hub.released(TOKEN, first.buffer);
    hub.scale(TOKEN, 240);
    let second = shell.present();
    assert_eq!(second.scale_120, 240);
    assert!(second.width > first.width * 3 / 2);
    // The logical size stays about the same: that is the size the compositor lays out.
    assert!(second.logical_width.abs_diff(first.logical_width) <= 1);
}

#[test]
fn hiding_tells_the_shell_and_an_empty_frame_hides_too() {
    let Some((mut hub, mut shell)) = rig("bornopata") else {
        return;
    };
    assert!(hub.show(TOKEN, "ctx", bangl()));
    shell.present();
    hub.hide(TOKEN);
    loop {
        match shell.next().unwrap() {
            PopupServerMessage::Hide => break,
            PopupServerMessage::Present(_) | PopupServerMessage::Pool { .. } => {}
        }
    }
    assert!(hub.show(TOKEN, "ctx", bangl()));
    shell.present();
    assert!(hub.show(TOKEN, "ctx", borno_render::Frame::default()));
    assert_eq!(shell.next(), Some(PopupServerMessage::Hide));
}

#[test]
fn a_shell_that_left_is_not_attached_any_more() {
    let Some((mut hub, shell)) = rig("bornopata") else {
        return;
    };
    drop(shell);
    // The first write to the closed socket detaches the channel.
    for _ in 0..50 {
        hub.show(TOKEN, "ctx", bangl());
        if !hub.attached(TOKEN) {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("the channel stayed attached");
}

#[test]
fn an_animated_theme_keeps_drawing_until_the_slots_are_used_and_waits_for_a_release() {
    let Some((mut hub, mut shell)) = rig("sakura") else {
        return;
    };
    assert!(hub.show(TOKEN, "ctx", bangl()));
    // Nobody releases anything: the first frame and two animation frames fill the three slots.
    let frames: Vec<PopupFrame> = (0..3).map(|_| shell.present()).collect();
    let mut buffers: Vec<u64> = frames.iter().map(|frame| frame.buffer).collect();
    buffers.sort_unstable();
    buffers.dedup();
    assert_eq!(buffers.len(), 3);
    assert!(frames.iter().all(|frame| frame.pool == frames[0].pool));
    shell
        .stream
        .set_read_timeout(Some(Duration::from_millis(400)))
        .unwrap();
    assert!(shell.next().is_none(), "a fourth frame found a slot");
    shell
        .stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    hub.released(TOKEN, frames[0].buffer);
    let after = shell.present();
    assert_eq!(after.offset, frames[0].offset);
}

#[test]
fn a_static_theme_goes_quiet_after_the_frame() {
    let Some((mut hub, mut shell)) = rig("bornopata") else {
        return;
    };
    assert!(hub.show(TOKEN, "ctx", bangl()));
    shell.present();
    shell
        .stream
        .set_read_timeout(Some(Duration::from_millis(400)))
        .unwrap();
    assert!(shell.next().is_none(), "an idle window drew again");
}
