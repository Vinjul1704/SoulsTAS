# SoulsTAS - TAS tool for multiple FromSoftware games

This is a tool to create Tool-Assisted Speedruns (TAS) for Elden Ring, Sekiro, Nightreign and Dark Souls 3 (not implemented yet).

It is run in a command line interface and works with script files that include the TAS actions:
```
soulstas.exe (game) (path/to/script)
```

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
- Press or release a mouse button: `mouse button (down/up) (button)`
- Scroll the mouse wheel: `mouse scroll (down/up) (amount)`
- Move the mouse: `mouse move (x) (y)`
- Wait for character control: `await control`
- Wait for no character control: `await no_control`
- Wait for cutscene: `await cutscene`
- Wait for no cutscene: `await no_cutscene`
- Wait for save active¹: `await save_active`
- Wait for no save active¹: `await no_save_active`

Additionally, there are actions that affect the behaviour of the TAS tool:
- Do nothing: `nothing`
- Set the FPS limit (use 0 to reset): `fps (fps)`
- Wait until you are tabbed in: `await focus`
- Set the TAS frame: `frame (frame)`
- Pause for an amount of milliseconds: `pause ms (ms)`
- Pause until you press enter in the terminal window: `pause input`

¹Note about "save active": This is a value you can use for now to check if you are back in the main menu, since a save is always "active" unless you are in the main menu.

<details>
<summary>Pixels required for a full rotation:</summary>
<br>
Here's a table of the amount of pixels of mouse movement required to do a full camera rotation.

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
soulstas.exe eldenring my-tas.txt
```


## Roadmap and future plans:
- v0.1: Initial version, support for Elden Ring
- v0.2: Code improvements, script syntax tweaks and added `await` actions
- v0.3: Support for Sekiro
- v0.4 (current): Support for Nightreign
- v0.5: Support for Dark Souls 3
- v0.6: Send inputs directly to the game and improvements/fixes for actions
- Beyond v0.6: Loading patch, fixed RNG, user-friendly interface, input recorder, game speed, pause/unpause...


## Compiling
To compile the program yourself, use latest rust nightly: `cargo build --release --target x86_64-pc-windows-msvc`
If you are using Linux, cross-compile for Windows using xwin: `cargo xwin build --release --target x86_64-pc-windows-msvc`


## Special thanks
- Massive thanks to wasted (https://github.com/FrankvdStam) for all his help with my stupid and often basic questions, and creating the building blocks that make this possible, especially SoulSplitter and mem-rs.
- Huge thanks to everyone on the reversing server and in particular MetalCrow for pointing me in the right direction for the FPS and frame advance patches.
