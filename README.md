# CvGameCoreDLL

This repository contains the `CvGameCoreDLL` source from **Sid Meier's Civilization IV: Beyond the Sword**.

`CvGameCoreDLL.dll` is the native C++ game logic DLL loaded by Civilization IV: Beyond the Sword. It contains core gameplay systems, AI, map logic, XML loading, Python bindings, event reporting, and related engine-facing interfaces.

## Repository layout

- `CvGameCoreDLL/` - native DLL source, VC7.1 project, command-line makefile, Boost, Python 2.4, and C++ vendored dependencies.
- `crates/` - Rust workspace crates for the Civ4 bridge client/protocol.
- `scripts/` - helper scripts for preparing/building with the legacy compiler.
- `BUILDING.md` - GitHub Actions toolchain preparation details.

## Build requirements

This project targets the original Civilization IV: Beyond the Sword DLL ABI and should be built as a 32-bit Windows DLL with the Visual C++ 7.1 compiler from Visual Studio .NET 2003.

Required components:

- Visual C++ 7.1 / Visual Studio .NET 2003 toolchain
- Windows Server 2003 SP1 Platform SDK or compatible Platform SDK
- In-tree Boost 1.32.0 and Python 2.4 libraries
- `nmake.exe` for the command-line build

## Building with nmake

From a Windows command prompt with a prepared VC7.1 toolchain:

```cmd
scripts\build_vc71_nmake.bat C:\path\to\vc71
```

If no argument is provided, the script checks `VC71_ROOT`, then `%RUNNER_TEMP%\vc71`.

The `nmake` build currently supports:

```cmd
cd CvGameCoreDLL
nmake /nologo /f Makefile.vc71 CFG=FinalRelease
```

The output DLL is written to:

```text
CvGameCoreDLL/artifacts/CvGameCoreDLL.dll
```

## GitHub Actions build

The workflow in `.github/workflows/build-cvgamedll.yml` builds the native DLL first, then builds the Rust workspace.

Because hosted runners do not include the VC7.1 compiler, CI prepares the legacy toolchain with:

```cmd
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\prepare_vc71_toolchain.ps1 -OutputDir "%RUNNER_TEMP%\vc71"
```

See `BUILDING.md` for details on toolchain archive layouts, repository variables, and the optional `VC71_TOOLCHAIN_URL` secret.

## Building the Rust workspace

From the repository root:

```cmd
cargo build --workspace --all-targets
```

## Running the bridge companion

The DLL bridge is disabled by default. Set `CVGAME_BRIDGE=1` before launching
the game to create the control and callback named pipes. Set
`CVGAME_BRIDGE_AUTOLAUNCH=1` as well to let the DLL launch a companion process
after the pipes are created. See `CIV4_BRIDGE_PROTOCOL.md` for executable
discovery, pipe environment variables, and the Rust `BridgeClient` handshake.

## Installing the DLL

To use a built DLL with Civilization IV: Beyond the Sword, copy `CvGameCoreDLL.dll` into the target `Assets` directory for the game or mod.

For the base game layout used by the project file:

```text
Beyond the Sword\Assets\CvGameCoreDLL.dll
```

For a mod, place it under that mod's `Assets` directory instead.

## Copyright

Sid Meier's Civilization IV: Beyond the Sword and the original `CvGameCoreDLL` source are copyright Take-Two Interactive Software, Inc. and its subsidiaries. Civilization, 2K Games, Firaxis Games, Take-Two Interactive Software, and related names and logos are trademarks or registered trademarks of their respective owners.

This repository does not claim ownership of the original game code or assets.

## Notes

- This is a legacy C++ codebase and depends on compiler behavior from Visual C++ 7.1.
- The DLL must remain compatible with the game executable's expected binary interface.
- There is no standalone test runner in this repository; validation is primarily through successful DLL compilation and in-game testing.
