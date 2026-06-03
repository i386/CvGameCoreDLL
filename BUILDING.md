# Building on GitHub Actions

The GitHub Actions workflow uses a hosted `windows-2022` runner and a private
VC7.1 toolchain archive. GitHub's hosted runners do not include Visual Studio
.NET 2003 or the VC7.1 compiler.

Configure the repository secret `VC71_TOOLCHAIN_URL` with an HTTPS URL to a ZIP
archive containing the command-line toolchain. The build script accepts these
layouts:

```text
vc71/
  VC7/
    bin/cl.exe
    bin/link.exe
    bin/nmake.exe
    include/
    lib/
  PlatformSDK/
    bin/rc.exe
    include/windows.h
    include/winres.h
    lib/
```

or equivalent `VC/`, `SDK/`, or root-level `bin`, `include`, and `lib`
directories.

The workflow downloads and extracts that archive, then runs:

```cmd
scripts\build_vc71_nmake.bat "%RUNNER_TEMP%\vc71"
```

The output artifact is:

```text
artifacts/CvGameCoreDLL.dll
```

