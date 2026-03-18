@echo off
setlocal enabledelayedexpansion

:: ── Read current version from Cargo.toml ─────────────────────────────────────
for /f "tokens=3 delims= " %%v in ('findstr /r "^version" Cargo.toml') do (
    set raw=%%v
    goto :found
)
:found
set current=%raw:"=%

for /f "tokens=1,2,3 delims=." %%a in ("%current%") do (
    set major=%%a
    set minor=%%b
    set patch=%%c
)

set /a major_bump=%major%+1
set /a minor_bump=%minor%+1
set /a patch_bump=%patch%+1

echo Current version: %current%
echo.
echo Bump type:
echo   1^) major  (%current% -^> %major_bump%.0.0)
echo   2^) minor  (%current% -^> %major%.%minor_bump%.0)
echo   3^) patch  (%current% -^> %major%.%minor%.%patch_bump%)
echo.
set /p choice="Choice [1/2/3]: "

if "%choice%"=="1" set new_version=%major_bump%.0.0
if "%choice%"=="2" set new_version=%major%.%minor_bump%.0
if "%choice%"=="3" set new_version=%major%.%minor%.%patch_bump%

if not defined new_version (
    echo Invalid choice. Aborting.
    exit /b 1
)

echo.
echo Bumping %current% -^> %new_version%
set /p confirm="Confirm? [y/N] "
if /i not "%confirm%"=="y" (
    echo Aborted.
    exit /b 1
)

:: ── Bump version in Cargo.toml ────────────────────────────────────────────────
powershell -NoProfile -Command "(Get-Content Cargo.toml) -replace '^version = \"%current%\"', 'version = \"%new_version%\"' | Set-Content Cargo.toml"

:: Update Cargo.lock
cargo build -q 2>nul

:: ── Commit ────────────────────────────────────────────────────────────────────
git add Cargo.toml Cargo.lock
git commit -m "Bump version to %new_version%"

:: ── Tag ───────────────────────────────────────────────────────────────────────
git tag v%new_version%

:: ── Push ──────────────────────────────────────────────────────────────────────
git push origin HEAD
git push origin v%new_version%

echo.
echo Released v%new_version% -- GitHub Actions will build and publish the release.
endlocal
