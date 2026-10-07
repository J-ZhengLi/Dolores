@echo off
setlocal EnableExtensions DisableDelayedExpansion
title Dolores
if not "%~1"=="" if not "%~1"=="--check" goto usage
if not "%~2"=="" goto usage
for %%F in (dolores_flutter.exe dolores-update-launcher.exe dolores-desktop-helper.exe dolores_flutter_bridge.dll flutter_windows.dll file_selector_windows_plugin.dll pasteboard_plugin.dll screen_retriever_windows_plugin.dll window_manager_plugin.dll native_assets.json data\app.so data\icudtl.dat data\flutter_assets\AssetManifest.bin data\flutter_assets\FontManifest.json data\flutter_assets\NativeAssetsManifest.json data\flutter_assets\NOTICES.Z data\flutter_assets\fonts\MaterialIcons-Regular.otf data\flutter_assets\shaders\ink_sparkle.frag data\flutter_assets\shaders\stretch_effect.frag) do (
  if not exist "%~dp0%%F" goto incomplete
  for %%S in ("%~dp0%%F") do if %%~zS LEQ 0 goto incomplete
)
set "DoloresRuntime=%SystemRoot%\System32"
if defined PROCESSOR_ARCHITEW6432 set "DoloresRuntime=%SystemRoot%\Sysnative"
for %%F in (msvcp140.dll vcruntime140.dll vcruntime140_1.dll) do if not exist "%DoloresRuntime%\%%F" goto runtime
if "%~1"=="--check" (
  echo Dolores: required app files and C++ runtime files are present.
  echo This check does not verify runtime versions or replace a clean-machine launch test.
  exit /b 0
)
start "" "%~dp0dolores_flutter.exe"
exit /b 0
:incomplete
echo Dolores cannot start because the extracted app is incomplete.
echo Extract the whole ZIP into a new folder and keep all files together, then open Start-Dolores.cmd again.
echo Your history and settings are stored separately and have not been removed.
if not "%~1"=="--check" pause
exit /b 2
:runtime
echo Dolores needs the Microsoft Visual C++ Redistributable for x64.
echo Install or repair it from Microsoft's page, then open Start-Dolores.cmd again:
echo https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist
echo Your history and settings have not been removed. Nothing was downloaded or installed.
if not "%~1"=="--check" pause
exit /b 3
:usage
echo Usage: Start-Dolores.cmd [--check]
exit /b 4
