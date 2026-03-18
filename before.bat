@echo off
mkdir C:\analysis 2>nul

echo [*] Collecting baseline...

tasklist > C:\analysis\tasklist_before.txt
netstat -ano | findstr ESTABLISHED > C:\analysis\netstat_before.txt
wmic startup get caption,command > C:\analysis\startup_before.txt
dir C:\Users\ /s /b *.exe > C:\analysis\files_before.txt
reg query HKCU\Software\Microsoft\Windows\CurrentVersion\Run > C:\analysis\reg_before.txt
reg query HKLM\Software\Microsoft\Windows\CurrentVersion\Run >> C:\analysis\reg_before.txt
schtasks /query /fo LIST /v > C:\analysis\tasks_before.txt

echo [*] Baseline done.
pause