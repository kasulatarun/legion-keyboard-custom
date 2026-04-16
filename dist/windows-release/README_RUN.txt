==================================================
 Lenovo LOQ LightSync - Windows Release Instructions
==================================================

=== LAUNCH INSTRUCTIONS ===

1) Open `dist\windows-release\`.
2) Double-click `LenovoLOQ-LightSync.exe`.
3) The application launches as a native desktop UI (no terminal window).

=== ADMINISTRATOR ACCESS (IF REQUIRED) ===

- If keyboard control does not apply, right-click `LenovoLOQ-LightSync.exe` and choose `Run as administrator`.
- Accept the UAC prompt to allow direct HID/EC communication.

=== DRIVER / HID REQUIREMENTS ===

- Lenovo keyboard control relies on standard Windows USB HID access.
- No extra third-party RGB drivers are required.
- Close other RGB control tools (for example Lenovo Vantage, Legion Toolkit, OpenRGB) to avoid command conflicts.

=== TROUBLESHOOTING ===

Keyboard not detected:
- Confirm the device is a supported Lenovo LOQ/Legion keyboard controller.
- Run the app as administrator.
- Disconnect/reconnect power and restart Windows if HID is locked by another process.

RGB not applying:
- Select a profile and effect, then change zone color or brightness once to force a fresh write.
- Disable competing software that might continuously overwrite keyboard lighting.

Profile save failure:
- Ensure Windows account has write access to `%LOCALAPPDATA%\legion-kb-rgb\settings.json`.
- If needed, launch once as administrator so initial settings files can be created.

Support:
- Lenovo Support Portal: https://support.lenovo.com/
