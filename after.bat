@echo off

echo [*] Collecting after state...

tasklist > C:\analysis\tasklist_after.txt
netstat -ano | findstr ESTABLISHED > C:\analysis\netstat_after.txt
wmic startup get caption,command > C:\analysis\startup_after.txt
dir C:\Users\ /s /b *.exe > C:\analysis\files_after.txt
reg query HKCU\Software\Microsoft\Windows\CurrentVersion\Run > C:\analysis\reg_after.txt
reg query HKLM\Software\Microsoft\Windows\CurrentVersion\Run >> C:\analysis\reg_after.txt
schtasks /query /fo LIST /v > C:\analysis\tasks_after.txt

echo [*] After snapshot done.
pause