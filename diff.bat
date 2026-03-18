@echo off
setlocal enabledelayedexpansion

echo =========================
echo NEW PROCESSES
echo =========================
for /f "tokens=1,*" %%a in (C:\analysis\tasklist_after.txt) do (
	findstr /i "%%a" C:\analysis\tasklist_before.txt >nul
	if errorlevel 1 (
		echo [+] %%a %%b
	)
)

echo.
echo =========================
echo NEW NETWORK CONNECTIONS (ESTABLISHED ONLY)
echo =========================
for /f "tokens=1,2,3,4,5" %%a in ('type C:\analysis\netstat_after.txt ^| findstr ESTABLISHED') do (
	findstr /i "%%c %%d" C:\analysis\netstat_before.txt >nul
	if errorlevel 1 (
		echo [+] %%a %%b %%c %%d %%e
	)
)

echo.
echo =========================
echo NEW STARTUP ENTRIES
echo =========================
for /f "tokens=*" %%a in (C:\analysis\startup_after.txt) do (
	findstr /i "%%a" C:\analysis\startup_before.txt >nul
	if errorlevel 1 (
		echo [+] %%a
	)
)

echo.
echo =========================
echo NEW EXECUTABLE FILES
echo =========================
for /f "tokens=*" %%a in (C:\analysis\files_after.txt) do (
	findstr /i "%%a" C:\analysis\files_before.txt >nul
	if errorlevel 1 (
		echo [+] %%a
	)
)

echo.
echo =========================
echo NEW REGISTRY STARTUP
echo =========================
for /f "tokens=*" %%a in (C:\analysis\reg_after.txt) do (
	findstr /i "%%a" C:\analysis\reg_before.txt >nul
	if errorlevel 1 (
		echo [+] %%a
	)
)

echo.
echo =========================
echo NEW SCHEDULED TASKS
echo =========================
for /f "tokens=*" %%a in (C:\analysis\tasks_after.txt) do (
	findstr /i "%%a" C:\analysis\tasks_before.txt >nul
	if errorlevel 1 (
		echo [+] %%a
	)
)

echo.
echo =========================
echo ANALYSIS HINTS
echo =========================
echo Look for:
echo - .exe in AppData or Temp
echo - Random names (asdf123.exe)
echo - Unknown IP addresses
echo - Anything persisting after reboot
echo.

pause