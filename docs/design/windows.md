# Windows: the Server, the text service and the installer

How Bornopata runs on Windows, and why. The Linux counterpart is [../notes/linux-fcitx5.md](../notes/linux-fcitx5.md); the rule
for both is that a shell only forwards keys and draws what the Core returns ([../contributing.md](../contributing.md)).

**State.** Everything here builds for Windows and the engine layer is tested against the real Router on Linux, but **nothing has
been run on a Windows machine yet**. Treat the behaviour below as the intended design until a Windows machine has tried it.
It is ported from Qingjian's Windows shell (GPL, [../../NOTICE.md](../../NOTICE.md)), whose Windows-specific lessons are kept.

## Parts

```text
application A -- bornopata_tsf.dll --+
application B -- bornopata_tsf.dll --+-- named pipe \\.\pipe\bornopata -->  bornopata-server.exe
application C -- bornopata_tsf.dll --+                                       (one Router, one candidate window)
```

| Package | Product | What it does |
|---|---|---|
| `apps/windows/server` (`borno-windows-server`) | `bornopata-server.exe` | Holds the engines through `borno-server`, listens on the pipe, draws the candidate window. |
| `apps/windows/tsf` (`borno-windows-tsf`) | `bornopata_tsf.dll` | A TSF text service loaded into every application: forwards keys, writes text into the document. |
| `apps/windows/installer` | `bornopata-<version>-windows-x86_64-setup.exe` | Inno Setup installer. |

**Why the engine is outside the DLL.** TSF loads the text service into every application with a text box. An engine in the DLL
would be copied into dozens of processes with no shared learning, and a crash would take the host application with it. So the DLL
is thin and the engine lives in one Server process, like Weasel and Fcitx5. The two are separate packages so the DLL cannot pull
the dictionary and the learning code into its dependencies.

**What is shared with Linux.** `borno-server` (Router, sessions, the connection loop, startup), `borno-window` (placement and
theme lookup), `borno-render` (the bitmap of a frame), and the wire protocol in `borno-platform`. The Windows Server is a few
hundred lines around them: the named pipe and the window.

## The text service

`ITfTextInputProcessor` + `ITfKeyEventSink` (+ a display attribute provider for the underline). One service per thread; the
session number is the thread id (unique on the system, unlike TSF client ids).

- **Keys.** TSF asks `OnTestKeyDown` first ("do you want this key?") and only then calls `OnKeyDown`. A key must reach the Server
  once (a letter would be typed twice), so the answer is asked in the test and kept for `OnKeyDown`. Lone modifier keys never
  reach the Server (a lone Shift would settle the word). A password field (`GUID_COMPARTMENT_KEYBOARD_DISABLED`) lets every key
  through.
- **The word being typed** is a TSF composition: the Bangla text of the highlighted candidate, underlined (the counterpart of
  Fcitx5's client preedit). The candidates are drawn by the Server.
- **Writing** is always an asynchronous edit session: the text stores of immersive applications (the new Notepad) are across a
  process boundary and a synchronous write crashes the host.
- **A key the Server lets through after settling the word** (Enter, an arrow, Ctrl+C) must reach the application after the text.
  The text can only be written after the key has been answered, so the key is eaten and sent again with `SendInput` once the
  text is in; the service lets its own copy through once.
- **The cursor.** After every update the service measures the composition start (collapsed, because some applications answer a
  non-empty range with a one pixel fragment) and sends it as `Event::Cursor`; the Server moves its window. No cursor, no window.
- **Focus.** Losing the focus (or deactivating) asks the Server for the word with `Event::Deactivate` and writes it into the
  last document, so a word is never lost. If the application ended the composition itself, the Server just forgets the word.
- **Connection.** One pipe per thread, opened on activation. If the Server is not running the service starts it (a normal desktop
  application only, never from the sign-in screen, a sandbox or the installer) and retries every two seconds.

## The Server

- **Pipe.** One instance per client on its own thread, so a newly focused application never waits for another. The first instance
  claims the name, so a second Server fails to start. The access rules let sandboxed applications connect.
- **Router thread.** The Router stays on one thread; connections hand it requests over a channel (`borno-server::ipc`).
- **Window.** A layered, topmost, no-activate tool window on its own thread, per-monitor DPI aware. The renderer's bitmap (shadow
  and decorations included) is put on it with `UpdateLayeredWindow`; it is placed with `borno-window`'s `place` in logical pixels
  on the monitor of the cursor, and the theme's animations run on a timer. The mouse goes through it (no clicking candidates yet).
- **Config changes.** The Router watches `config.toml`; on a change the Server starts a new copy of itself that takes over the pipe.

## Files

| What | Where |
|---|---|
| Program, data | `C:\Program Files\Bornopata` (`data\words.bnd`, `lm.bnd`, `gloss-en.bnd`) |
| Config, learned data, phrases | `%APPDATA%\Bornopata` |
| Logs (`server.<date>.log`, `tsf.<date>.log`) | `%LOCALAPPDATA%\Bornopata\logs`, seven days; the typed text is never logged |

## Installing and upgrading

The installer registers the DLL (`regsvr32`, a 64 bit and a 32 bit one) and puts a Startup shortcut for the Server in. DLLs are
named by version and installed side by side, because a loaded DLL cannot be overwritten; only the new one is registered and the
old ones are deleted afterwards or at restart. The installer holds the mutex `Global\BornopataInstaller` so no DLL starts the
Server while its file is being replaced. Details are at the top of `bornopata.iss`.

## Testing without a Windows machine

- `cargo check --target x86_64-pc-windows-gnu` from Linux compiles everything.
- `apps/windows/tsf/tests/protocol_loop.rs` runs the DLL's client against the real Router over a socket pair, including the
  decision logic (eat, commit, re-send).
- `.github/workflows/windows.yml` runs clippy and the tests on a Windows runner and, when started by hand, builds the installer
  as an artifact.
- Real behaviour needs a person on Windows. Both programs write their logs to `%LOCALAPPDATA%\Bornopata\logs`, which is what a
  tester sends back.

## Known limits

- No code signing: SmartScreen warns, and without `uiAccess` the candidate window can be covered by the Start menu and the
  search box.
- Candidates cannot be clicked, and there is no Bangla/English indicator in the language bar (the keys switch, `Ctrl+Alt+Space`).
- Applications that are not DPI aware report cursor rectangles in scaled coordinates, so the window may sit off.
- The settings program builds for Windows but is untried there.
