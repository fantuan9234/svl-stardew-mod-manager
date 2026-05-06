@echo off
taskkill /f /im explorer.exe
del /f /s /q %localappdata%\IconCache.db
del /f /s /q %localappdata%\Microsoft\Windows\Explorer\iconcache_*.db
del /f /s /q %localappdata%\Microsoft\Windows\Explorer\thumbcache_*.db
start explorer.exe
