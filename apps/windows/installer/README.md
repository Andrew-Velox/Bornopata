# Bornopata Windows installer

An [Inno Setup](https://jrsoftware.org/isinfo.php) installer. It puts the text service DLL (a 64 bit and a 32 bit one), the
Server, the settings program and the data into `C:\Program Files\Bornopata`, registers the text service, starts the Server at
every login and starts it once when the wizard closes.

```text
C:\Program Files\Bornopata\
    bornopata_tsf-<version>.dll        the text service (64 bit, loaded into every application)
    bornopata_tsf-<version>-x86.dll    the same for 32 bit applications
    bornopata-server.exe               the engine, outside the applications
    bornopata-settings.exe             the settings program
    bornopata.ico
    data\words.bnd lm.bnd gloss-en.bnd
    licenses\
```

The user's data stays in `%APPDATA%\Bornopata` (`config.toml`, what it learned, `phrases.tsv`), the logs in
`%LOCALAPPDATA%\Bornopata\logs` (`server.<date>.log` and `tsf.<date>.log`). Uninstalling leaves them alone.

The comments at the top of `bornopata.iss` explain the parts that are easy to get wrong: why the DLLs are installed side by
side by version, why the installer holds a mutex, and why the Server is renamed before it is stopped.

## Building it

On a machine with the MSVC toolchain and Inno Setup 7 (the CI workflow `windows.yml` does exactly this):

```powershell
bash tools/windows-fetch-data.sh                              # words.bnd, lm.bnd, gloss-en.bnd into target\release-inputs
powershell -ExecutionPolicy Bypass -File apps\windows\installer\build.ps1
```

The result is `target\installer\bornopata-<version>-windows-x86_64-setup.exe`. `-SkipBuild` reuses the binaries.

## Not signed yet

The installer and the binaries have no code signature, so Windows SmartScreen warns before the first run ("Windows protected
your PC": More info, Run anyway). Without a signature the Server cannot ask for `uiAccess` either, so the candidate window
can be covered by the Start menu and the search box.
