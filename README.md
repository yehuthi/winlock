# WinLock <img src="https://github.com/yehuthi/winlock/blob/0a5dd2c85d33b8d1e8e906ee48491a6cb186c174/winlock.svg?raw=true" width=100 height=100 align=left />

A utility to customize the keyboard shortcut for session locking on Windows.

## Why

- Free <kbd>Win</kbd>+<kbd>L</kbd> for other uses.<sup>1</sup>
- Use the shortcut that you want instead of the Microsoft-mandated shortcut.

## Usage

```shell
winlock -d       # Disables the lock screen (and Win+L with it)
winlock -r       # Restores the lock screen (and Win+L with it)
winlock -cwk j   # Sets a shortcut Ctrl+Win+J to lock the screen (Win+L still functional)
winlock -drcwk j # Replaces Win+L with Ctrl+Win+J

winlock --help   # Describes usage with more detail
```

> Note: experimental, subject to change.

To exit gracefully send an interrupt signal (press <kbd>Ctrl</kbd>+<kbd>C</kbd> to the program). Ungraceful exits (e.g. process termination) will impede the `-r` flag from functioning.

---

<sup>1</sup> My own reason for making this is wanting to use <kbd>Win</kbd>+<kbd>L</kbd> inside a Windows-hosted virtual-machine (in my [i3](https://i3wm.org/) config, where such bindings are popular and very handy).
Its elevated priority means that even when the virtual-machine captures the button press, Windows does too and simultaneously locks the host screen.
