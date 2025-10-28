@echo off

:: Store some paths
set ORIG_DIR=%CD%
set SCRIPT_DIR=%~dp0


:: Build the main EXEs
cd %SCRIPT_DIR%\..\

cargo build --profile dev --target x86_64-pc-windows-msvc
if errorlevel 1 (
  set ERR_CODE=%errorlevel%
  cd %ORIG_DIR%
  exit /B %ERR_CODE%
)

cargo build --profile dev --target i686-pc-windows-msvc
if errorlevel 1 (
  set ERR_CODE=%errorlevel%
  cd %ORIG_DIR%
  exit /B %ERR_CODE%
)


:: Build the soulstas-patches DLLs
cd %SCRIPT_DIR%\..\lib\soulstas-patches\

cargo build --profile dev --target x86_64-pc-windows-msvc
if errorlevel 1 (
  set ERR_CODE=%errorlevel%
  cd %ORIG_DIR%
  exit /B %ERR_CODE%
)

cargo build --profile dev --target i686-pc-windows-msvc
if errorlevel 1 (
  set ERR_CODE=%errorlevel%
  cd %ORIG_DIR%
  exit /B %ERR_CODE%
)


:: Build the soulmods DLL
cd %SCRIPT_DIR%\..\lib\SoulSplitter\src\soulmods\

cargo build --profile dev --target x86_64-pc-windows-msvc
if errorlevel 1 (
  set ERR_CODE=%errorlevel%
  cd %ORIG_DIR%
  exit /B %ERR_CODE%
)


:: Create fresh build dir
rmdir /S /Q %SCRIPT_DIR%\build-debug 2>nul
mkdir %SCRIPT_DIR%\build-debug

:: Copy built files
copy %SCRIPT_DIR%\..\target\x86_64-pc-windows-msvc\debug\soulstas.exe %SCRIPT_DIR%\build-debug\soulstas_x64.exe
copy %SCRIPT_DIR%\..\target\i686-pc-windows-msvc\debug\soulstas.exe %SCRIPT_DIR%\build-debug\soulstas_x86.exe

copy %SCRIPT_DIR%\..\lib\soulstas-patches\target\x86_64-pc-windows-msvc\debug\soulstas_patches.dll %SCRIPT_DIR%\build-debug\soulstas_patches_x64.dll
copy %SCRIPT_DIR%\..\lib\soulstas-patches\target\i686-pc-windows-msvc\debug\soulstas_patches.dll %SCRIPT_DIR%\build-debug\soulstas_patches_x86.dll

copy %SCRIPT_DIR%\..\lib\SoulSplitter\target\x86_64-pc-windows-msvc\debug\soulmods.dll %SCRIPT_DIR%\build-debug\soulmods_x64.dll


:: Go back into original dir
cd %ORIG_DIR%