# SoulsTAS - TAS tool for multiple FromSoftware games

This is a tool to create Tool-Assisted Speedruns (TAS) for multiple FromSoftware games. It is run in a command line interface and works with script files that include the TAS actions:
```
soulstas_x64.exe (dsr/sotfs/ds3/sekiro/er/nr) path/to/tas/script.txt
soulstas_x86.exe (ds1/ds2) path/to/tas/script.txt
```

| Game | Overall | Consistency | Input | Actions/Flags | FPS Limit | Versions | RNG |
| - | - | - | - | - | - | - | - |
| Dark Souls 1 (PTDE) | 🟢 | 🟠¹ | 🟢 | 🟢 | N/A | 🟢 | 🔴 |
| Dark Souls 1 (Remastered) | 🟢 | 🟠¹ | 🟠² | 🟢 | N/A | 🟢 | 🔴 |
| Dark Souls 2 (Original) | 🟠³ | 🟠 | 🟠 | 🟠 | 🟢 | 🟢 | 🔴 |
| Dark Souls 2 (SOTFS) | 🟠³ | 🟠 | 🟠 | 🟠 | 🟢 | 🟢 | 🔴 |
| Dark Souls 3 | 🟢 | 🟠¹ | 🟢 | 🟢 | 🟢 | 🟢 | 🔴 |
| Sekiro | 🟢 | 🟠¹ | 🟢 | 🟢 | 🟢 | 🟢 | 🔴 |
| Elden Ring | 🟢 | 🟠¹ | 🟢 | 🟢 | 🟢 | 🟢 | 🔴 |
| Nightreign | 🟠 | 🟠¹ | 🟠⁴ | 🟢 | 🟢 | 🟠⁵ | 🔴 |
| Armored Core 6 | 🟠 | 🟠¹ | 🟠⁴ | 🟠⁶ | 🟢 | 🟢 | 🔴 |

<details>
<summary>Notes:</summary>
  
  - ¹: Loading of dynamic objects in-game currently depends on hardware. This is particularly noticable when suddenly teleporting larger distances and having to wait for things like boxes and crates to load in.
  - ²: Gamepad input only works with a physical gamepad plugged in.
  - ³: DS2 support in general is kinda meh, due to it using a different engine. It's not 100% consistent like the other games, but it should work "okay enough" for glitch testing (certainly better than macros). Not ready for proper TASing.
  - ⁴: Gamepad input is currently not supported.
  - ⁵: Due to the nature of it being a primarily online-focused game with frequent updates, support for newer versions is not guaranteed. Last checked on v1.02.1.
  - ⁶: Cutscene flags might not be perfect and some cutscenes might not be handled. Will be improved in the future.
</details>

If you have any questions, issues or suggestions, feel free to make a Github issue or message me via Discord: `virazy`

## Script creation
The tool is based around the use of TAS script files, which include the actions performed by the TAS, and at which frame.

It follows the syntax `(frame) (action) (arguments)`, with comments being supported via semicolons (`;`) or hashtags (`#`).

The `(frame)` field can optionally have a `+` or `++` prefix:
- `+` means it will be done n frames after the last action found before it.
- `++` means it will be done n frames after the last action without a `+` or `++` prefix found before it.

Possible in-game actions:
- Press or release a key: `key (down/up) (key)`
- Press or release a key (alternative, for the character name box specifically): `key_alternative (down/up) (key)`
- Press or release a gamepad button: `gamepad button (down/up) (button)`
- Set a gamepad stick position: `gamepad stick (left/right) (angle) (amount, 0-1)`
- Set a gamepad axis position: `gamepad axis (axis) (amount)`
- Press or release a mouse button: `mouse button (down/up) (button)`
- Scroll the mouse wheel: `mouse scroll (down/up) (amount)`
- Move the mouse: `mouse move (x) (y)`
- Wait for being loaded in with character control: `await ingame`
- Wait for not being loaded in with character control: `await ingame`
- Wait for cutscene¹: `await cutscene`
- Wait for no cutscene¹: `await no_cutscene`
- Wait for being in the main menu: `await mainmenu`
- Wait for not being in the main menu: `await no_mainmenu`
- Wait for the character to be near a given position²: `await position (x) (y) (z) (range)`
- Wait for the character to be near a given position using alternative coordinates²: `await position_alternative (x) (y) (z) (range)`

Additionally, there are actions that affect the behaviour of the TAS tool:
- Do nothing: `nothing`
- Set the FPS limit (use 0 to reset): `fps (fps)`
- Wait until you are tabbed in: `await focus`
- Set the TAS frame: `frame (frame)`
- Pause for an amount of milliseconds: `pause ms (ms)`
- Pause until you press enter in the terminal window: `pause input`

¹: When you have 2 cutscenes in a row (for example, the intro in most games) and you try to do `await no_cutscene` into `await cutscene` between them, try to delay `await cutscene` by one frame if you're running into issues.
²: Only implemented for Elden Ring currently. In this case, `await position` uses your map coordinates (seen in JDSD practice tool), while `await position_alternative` uses the more accurate havok coordinates, in case that's needed. A negative range means it checks if you're *outside* of that range, as in if you are `(range)` units away from `(x) (y) (z)`.

<details>
<summary>Key/Button/Axis names:</summary>
  
<br>
  
| Keyboard Key | Description |
| - | - |
| a-z, 0-9, f1-f12 | Self-Explanatory |
| shift / shift_left / shift_l | Left shift key |
| shift_right / shift_r | Right shift key |
| control / ctrl / control_left / ctrl_left / control_l / ctrl_l | Left control key |
| control_right / ctrl_right / control_r / ctrl_r | Right control key |
| alt / alt_left / alt_l | Left alt key |
| alt_right / alt_r | Right alt key |
| tab | Tab key |
| back / backspace | Backspace key |
| enter / return | Enter key |
| caps / capslock | Caps lock key |
| space | Space key |
| escape / esc | Escape key |
| arrow_up / up | Up arrow key |
| arrow_down / down | Down arrow key |
| arrow_left / left | Left arrow key |
| arrow_right / right | Right arrow key |

<br>

| Mouse Button | Description |
| - | - |
| left / l | Left mouse button |
| right / r | Right mouse button |
| middle / m | Middle mouse button |
| extra1 / e1 | First extra mouse button |
| extra2 / e2 | Second extra mouse button |

<br>

| Gamepad Button | Description |
| - | - |
| dpad_up / up | Up directional button |
| dpad_down / down | Down directional button |
| dpad_left / left | Left directional button |
| dpad_right / right | Right directional button |
| a / cross | A or "Cross" face button |
| b / circle | B or "Circle" face button |
| x / square | X or "Square" face button |
| y / triangle | Y or "Triangle" face button |
| start / options | Start or Options face button |
| select / share | Select or Share face button |
| stick_left / stick_l / l3 | Left stick press |
| stick_right / stick_r / r3 | Right stick press |
| shoulder_left / shoulder_l / l1 | Left shoulder button |
| shoulder_right / shoulder_r / r1 | Right shoulder button |

<br>

| Gamepad Axis | Limits | Description |
| - | - | - |
| stick_left_x / stick_l_x / left_x / l_x | -32768 to 32767 | Left stick, Horizontal axis |
| stick_left_y / stick_l_y / left_y / l_y | -32768 to 32767 | Left stick, Vertical axis |
| stick_right_x / stick_r_x / right_x / r_x | -32768 to 32767 | Right stick, Horizontal axis |
| stick_right_y / stick_r_y / right_y / r_y | -32768 to 32767 | Right stick, Vertical axis |
| trigger_left / trigger_l / l2 | 0 to 255 | Left trigger |
| trigger_right / trigger_r / r2 | 0 to 255 | Right trigger |
</details>

<details>
<summary>Pixels required for a full rotation (Elden Ring):</summary>
<br>
Here's a table of the amount of pixels of mouse movement required to do a full camera rotation, tested in Elden Ring.

Keep in mind the values don't always match up perfectly.
If you are using Windows, you need to double the value.

I recommend using 0 sensitivity for the best accuracy.

| Sensitivity | Pixels |
| - | - |
| 0 | 36000 |
| 1 | 12857 |
| 2 | 7826 |
| 3 | 5625 |
| 4 | 4390 |
| 5 | 3600 |
| 6 | 3051 |
| 7 | 2647 |
| 8 | 2338 |
| 9 | 2093 |
| 10 | 1895 |
</details>

It is highly recommended to reset your game settings to default for TASing, since that way it's easy to share with others. If you made tweaks, you can mention them in the script as comments.

You should also always use this while offline and in the case of ER, with EAC disabled (see here: https://soulsspeedruns.com/eldenring/eac-bypass).

Keep in mind it is not yet compatible with SoulSplitter (the Livesplit autosplitter).


### Example:
```
; This is a comment (notice the semicolon at the start)
0 nothing ; Comments can be after actions too..

# ..or done with hashtags

; Wait until you are tabbed in. "await" commands never pause the game, but also don't increment the frame count
0 await focus

; 30 frames into the script (so half a second after tabbing in), press down the w key to walk forward
30 key down w
+60 key up w ; A second later, release the w key again to stop walking. This happens 90 frames into the TAS, due to the "+" syntax

; Move the camera to the right, walk forward and attack
120 mouse move 1000 0
150 key down w
180 mouse button down left
+1 mouse button up left
210 key up w
```

Simply save it to a file, for example `my-tas.txt` and run the following command while the game (here Elden Ring) is running:
```
soulstas_x64.exe eldenring my-tas.txt
```

## Future plans (may change):
- Support DATA.exe for DS1 (old + GFWL versions).
- Gamepad support for Nightreign + AC6 and improvements for DSR.
- Make use of dearxan (https://github.com/tremwil/dearxan) and replace the scuffed DS3 FPS patch.
- Enforce offline-mode and patch out low-FPS popups.
- Patches to make dynamic loading of assets, as well as RNG, consistent.
- Some sort of basic user interface? Pause/Unpause? Speedup/Slowdown? Input recording? ~~Improved DS2?~~


## Compiling
To compile the program yourself, you can use the included build scripts in the `build-helpers` directory. Keep in mind even when compiling from Linux, it will build an EXE to run through Proton. A native Linux binary is not supported.

Install the latest rust stable version (https://rust-lang.org/tools/install/) and add the necessary targets:
```
rustup target add x86_64-pc-windows-msvc
rustup target add i686-pc-windows-msvc
```

If you are compiling from Linux, install cargo-xwin as well (https://github.com/rust-cross/cargo-xwin):
```
cargo install --locked cargo-xwin
```

Clone the repository with submodules and `cd` into it:
```
git clone --recurse-submodules https://github.com/Vinjul1704/SoulsTAS
cd SoulsTAS
```

Run the build script for your platform and desired profile (release recommended):
- Windows, Release: `.\build-helpers\build_release_windows.bat`
- Windows, Debug: `.\build-helpers\build_debug_windows.bat`
- Linux, Release: `./build-helpers/build_release_linux.sh`
- Linux, Debug: `./build-helpers/build_debug_linux.sh`

If everything compiled correctly, the build will be found in a folder next to the build scripts.


## Special thanks
- Massive thanks to wasted (https://github.com/FrankvdStam) for all his help with my stupid and often basic questions, and creating the building blocks that make this possible, especially SoulSplitter and mem-rs.
- Big thanks to Radai (https://github.com/LordRadai) for giving me functions and memory values necessary for DS2.
- Huge thanks to everyone in the modding community, reversing server and in particular MetalCrow for pointing me in the right direction for the FPS and frame advance patches.
