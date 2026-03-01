# ROG Control Center

Terminal UI for managing ASUS ROG laptops on Linux, built on top of [asusctl](https://gitlab.com/asus-linux/asusctl) and [supergfxctl](https://gitlab.com/asus-linux/supergfxctl).

## Features

Six tabs covering all major hardware controls:

| Tab | What it does |
|---|---|
| **Overview** | System dashboard — temps, fan RPMs, profile, GPU mode, battery |
| **Performance** | Switch power profiles, adjust APU/Platform PPT sliders |
| **Fans** | View and edit 8-point fan curves per fan, apply or reset to defaults |
| **Aura** | Cycle keyboard lighting effects and set custom RGB colors |
| **GPU** | Switch between Integrated / Hybrid / MUX dGPU (with reboot confirmation) |
| **Battery** | Charge limit slider, one-shot full charge, boot sound, panel overdrive, keyboard brightness |

- Sensors (CPU/GPU temp, fan RPM, battery) refresh every 2 seconds via sysfs
- CLI state (profiles, armoury attributes, GPU mode) refreshes every 10 seconds
- Vim-style navigation (`hjkl`) alongside arrow keys
- Automatic [omarchy](https://github.com/nicholasgasior/omarchy) theme integration, with Tokyo Night + Catppuccin Mocha fallback

## Requirements

- Linux with an ASUS ROG laptop
- [asusctl](https://gitlab.com/asus-linux/asusctl) v6+
- [supergfxctl](https://gitlab.com/asus-linux/supergfxctl) v5+
- Rust 1.70+

## Build & Install

```sh
cargo build --release
cp target/release/rog-control-center ~/.local/bin/
```

## Usage

```sh
rog-control-center
```

### Keybindings

| Key | Action |
|---|---|
| `q` / `Ctrl+C` | Quit |
| `Tab` / `Shift+Tab` | Next / previous tab |
| `1`–`6` | Jump to tab |
| `↑↓` / `jk` | Navigate items |
| `←→` / `hl` | Adjust values |
| `Enter` | Apply / select |
| `Esc` | Cancel / back |
| `?` | Toggle help overlay |

Tab-specific keys are shown in the footer bar.

## Theming

The app automatically reads the active [omarchy](https://github.com/nicholasgasior/omarchy) theme from `~/.config/omarchy/current/theme/colors.toml`. All 17 built-in omarchy themes are supported, including light themes like `catppuccin-latte`.

If the file is missing or can't be parsed, the app falls back to its built-in Tokyo Night + Catppuccin Mocha palette.

To apply a new theme, switch it with `omarchy theme set <name>` and restart the app.

## Waybar Integration

A status script is included for Waybar:

```sh
cp scripts/waybar-rog-status.sh ~/.local/bin/
chmod +x ~/.local/bin/waybar-rog-status.sh
```

Add to your Waybar config:

```jsonc
"custom/rog": {
    "exec": "~/.local/bin/waybar-rog-status.sh",
    "return-type": "json",
    "interval": 5,
    "format": "{}",
    "on-click": "~/.local/bin/rog-control-center"
}
```

The module outputs profile-colored classes (`quiet`, `balanced`, `performance`) for styling.

## Project Structure

```
src/
  main.rs           Entry point, terminal setup, main loop
  app.rs            Application state, keybindings, actions
  error.rs          Error types
  event.rs          Crossterm event handler with tick timer
  backend/
    mod.rs          SystemState and action dispatcher
    asusctl.rs      asusctl CLI wrapper (profiles, fans, aura, armoury)
    supergfxctl.rs  supergfxctl CLI wrapper (GPU mode switching)
    sysfs.rs        Direct hwmon reads (temps, fans, battery)
  ui/
    mod.rs          Layout, header, tabs bar, footer, help overlay
    theme.rs        Color palette (omarchy theme or Tokyo Night + Catppuccin Mocha fallback)
    tabs/
      overview.rs   System dashboard
      performance.rs Profile & PPT controls
      fans.rs       Fan curve editor
      aura.rs       Keyboard lighting
      gpu.rs        GPU mode selector
      battery.rs    Battery & misc toggles
scripts/
  waybar-rog-status.sh  Waybar custom module
```

## License

MIT
