# Project Introduction

This project focuses on the development of a space exploration and trading game built upon physical simulations of moving objects within the solar system. It continues prior foundational work aimed at establishing a robust simulation core.

The game is developed entirely in **Rust**, a systems programming language known for its performance and strong compile-time safety guarantees. The architecture leverages **Bevy**, an open-source, minimalist game engine that employs the Entity-Component-System (ECS) model to promote modularity and maintainability.

Version control and collaboration are managed using **Git** and **GitHub**, ensuring efficient source management and smooth project handover.

## General Overview  
The application runs partly in the terminal (controlled entirely via keyboard) and partly in a graphical interface for the spatial map (controlled entirely via mouse). Please note that keyboard inputs do not work when the graphical window is focused; you must switch back to the terminal to use the keyboard controls.

Input handling is completely separate: the terminal interface uses Ratatui, while the graphical display and game loop are managed by Bevy.

### Keyboard Shortcuts  
All keyboard shortcuts for the terminal interface are defined in `keymap.toml`.  
Refer to `keyboard.rs` for an overview of how input handling works.

For mouse actions (applicable only within the GUI), see `src/ui/gui.rs` and `src/ui/editor_gui.rs` for the list of shortcuts.

### Other  
The entry point of the application is `src/bin/client.rs`, which contains command-line arguments and the activated Bevy plugins.

Several Cargo features are implemented:  
- Asteroids support (see `src/bin/client.rs`)  
- Graphical debug display (see `src/ui/gui.rs`)  

Please consult `Cargo.toml` for more details on enabled features.

The project is entirely in English, except for celestial body IDs, which are currently in French because the database is managed by a French contributor. This is temporary, so please avoid French comments or variable names.

For open issues and tasks, please check the [main repository issues](https://github.com/ben3dninja/solar4x/issues).

## Fleet Screen  
Create a new ship, select it, then edit its trajectory.  
Inputs in the ship creation menu are very strict (incorrect input may cause the program to crash). Be sure to check `src/ui/screen/fleet.rs`, especially the `CreateShipContext.to_info` function.

Note: Celestial body identifiers are currently in French (e.g., `soleil`, `terre`), see `main_objects.json` for the list of IDs and `src/objects/bodies/` for more details.

## Trajectory Editor Screen  
Almost all controls are managed via the GUI, so make sure to select the window before attempting to use the mouse wheel or other inputs.
