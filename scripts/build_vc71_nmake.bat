@echo off
setlocal

set "TOOLCHAIN_ROOT=%~1"
if "%TOOLCHAIN_ROOT%"=="" set "TOOLCHAIN_ROOT=%VC71_ROOT%"
if "%TOOLCHAIN_ROOT%"=="" set "TOOLCHAIN_ROOT=%RUNNER_TEMP%\vc71"

set "ROOT=%~dp0.."
pushd "%ROOT%" || exit /b 1

if not exist "%TOOLCHAIN_ROOT%" (
  echo VC7.1 toolchain root not found: %TOOLCHAIN_ROOT%
  popd
  exit /b 1
)

if exist "%TOOLCHAIN_ROOT%\Common7\Tools\vsvars32.bat" call "%TOOLCHAIN_ROOT%\Common7\Tools\vsvars32.bat"
if exist "%TOOLCHAIN_ROOT%\VC7\bin\vcvars32.bat" call "%TOOLCHAIN_ROOT%\VC7\bin\vcvars32.bat"
if exist "%TOOLCHAIN_ROOT%\bin\vcvars32.bat" call "%TOOLCHAIN_ROOT%\bin\vcvars32.bat"

set "VCROOT="
for %%D in ("%TOOLCHAIN_ROOT%\VC7" "%TOOLCHAIN_ROOT%\VC" "%TOOLCHAIN_ROOT%") do (
  if exist "%%~D\bin\cl.exe" set "VCROOT=%%~D"
)

if "%VCROOT%"=="" (
  echo Could not find cl.exe below %TOOLCHAIN_ROOT%.
  echo Expected one of VC7\bin\cl.exe, VC\bin\cl.exe, or bin\cl.exe.
  popd
  exit /b 1
)

set "SDKROOT="
for %%D in ("%TOOLCHAIN_ROOT%\PlatformSDK" "%TOOLCHAIN_ROOT%\SDK" "%TOOLCHAIN_ROOT%\Microsoft Platform SDK" "%TOOLCHAIN_ROOT%") do (
  if exist "%%~D\include\windows.h" set "SDKROOT=%%~D"
)

if "%SDKROOT%"=="" (
  echo Could not find a Platform SDK below %TOOLCHAIN_ROOT%.
  echo Expected include\windows.h under PlatformSDK, SDK, or the toolchain root.
  popd
  exit /b 1
)

set "PATH=%VCROOT%\bin;%SDKROOT%\bin;%PATH%"
set "INCLUDE=%CD%\Boost-1.32.0\include;%CD%\Python24\include;%VCROOT%\include;%SDKROOT%\include;%INCLUDE%"
set "LIB=%CD%\Boost-1.32.0\libs;%CD%\Python24\libs;%VCROOT%\lib;%SDKROOT%\lib;%LIB%"

where cl.exe || goto :missing_tool
where link.exe || goto :missing_tool
where nmake.exe || goto :missing_tool
where rc.exe || goto :missing_tool
goto :have_tools

:missing_tool
  echo Required VC7.1 build tools are missing.
  popd
  exit /b 1

:have_tools

nmake /nologo /f Makefile.vc71 CFG=FinalRelease
set "BUILD_EXIT=%ERRORLEVEL%"

popd
exit /b %BUILD_EXIT%
