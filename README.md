# SoulsTAS - TAS tool for Elden Ring, Sekiro and Dark Souls 3

This is a tool to create Tool-Assisted Speedruns (TAS) for Elden Ring, Sekiro and Dark Souls 3.
As of version 0.1, only Elden Ring support is implemented.

It is run in a command line interface and works with script files that include the TAS actions:
```
soulstas.exe (game) (path/to/script)
```

If you have any questions, issues or suggestions, feel free to make a Github issue or message me via Discord: `virazy`


## Script creation
The tool is based around the use of TAS script files, which include the actions performed by the TAS, and at which frame.

It follows the syntax `(frame) (action) (arguments)`, with comments being supported via semicolons (`;`).

The `(frame)` field can optionally have a `+` or `++` prefix:
- `+` means it will be done n frames after the last action found before it.
- `++` means it will be done n frames after the last action without a `+` or `++` prefix found before it.

Possible in-game actions:
- Press a key down: `key_down (key)`
- Release a key: `key_up (key)`
- Press a mouse button down: `mouse_down (button)`
- Release a mouse button: `mouse_up (button)`
- Scroll the mouse wheel down: `scroll_down (amount)`
- Scroll the mouse wheel up: `scroll_up (amount)`
- Move the mouse: `mouse_move (x) (y)`
- Wait for character control: `await_control`
- Wait for no character control: `await_no_control`
- Wait for cutscene: `await_cutscene`
- Wait for no cutscene: `await_no_cutscene`

Additionally, there are actions that affect the behaviour of the TAS tool:
- Do nothing: `nothing`
- Set the FPS limit (20-60*): `fps_limit (fps)`
- Wait until you are tabbed in: `await_focus`
- Set the TAS frame: `set_frame (frame)`
- Pause for an amount of milliseconds: `pause_ms (ms)`
- Pause until you press enter in the terminal window: `pause_input`

*IMPORTANT NOTE about the FPS limit: ALWAYS use "0" as the limit if you plan to use the default 60 FPS limit. Setting 60 manually is slightly different than the default limit and will break FPS-sensitive glitches like zips.*

Here is an example script:
```
; This is a comment (notice the semicolon at the start)
0 nothing ; Comments can be after actions too

; Wait until you are tabbed in. "await_" commands never pause the game, but also don't increment the frame count
0 await_focus

; 30 frames into the script (so half a second after tabbing in), press down the w key to walk forward
30 key_down w
+60 key_up w ; A second later, release the w key again to stop walking. This happens 90 frames into the TAS, due to the "+" syntax

; Move the camera to the right, walk forward and attack
120 mouse_move 1000 0
150 key_down w
180 mouse_down left
+1 mouse_up left
210 key_up w
```

Simply save it to a file, for example `example.soulstas` and run the following command while the game (here Elden Ring) is running:
```
soulstas.exe eldenring example.soulstas
```


## Roadmap and future plans:
- v0.1 (current): Initial version, support for Elden Ring
- v0.2: Code and stability improvements, fixed `await_control` action, additional actions (feel free to request some!)
- v0.3: Send inputs directly to the game and don't require you to be tabbed in
- v0.4: Support for Sekiro and Dark Souls 3
- v0.5: Fixed RNG
- v0.6-v1.0: GUI, input recorder, game speed, pause/unpause...


## Compiling
To compile the program yourself, use latest rust nightly: `cargo build --release`
If you are using Linux, cross-compile for Windows using xwin: `cargo xwin build --release --target x86_64-pc-windows-msvc`


## Special thanks
- Massive thanks to wasted (https://github.com/FrankvdStam) for all his help with my stupid and often basic questions, and creating the building blocks that make this possible, especially SoulSplitter and mem-rs.
- Huge thanks to everyone on the reversing server and in particular MetalCrow for pointing me in the right direction to put together the FPS and frame advance patches.
