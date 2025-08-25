## Project Architecture

### Plugin Architecture

The project architecture is organized around **features** rather than strictly by Bevy types (components, systems, etc.). Each feature or subsystem of the game has its own folder, containing its specific elements: components, Bevy systems, resources, and events. This organization naturally follows the **Entity-Component-System (ECS)** model used by Bevy, promoting separation of concerns and modularity.

The entry point of the application is the file `src/bin/client.rs`, which initializes a Bevy instance and registers the `ClientPlugin`, `TuiPlugin`, and `GuiPlugin` (see Figure 1). These plugins constitute the main building blocks of the client-side application.

*Figure 1: Architecture of the main project plugins (architecture.png)*

- **ClientPlugin**: Configures the game environment depending on the mode (solo, multiplayer, exploration). Currently, only solo mode is available.  
- **TuiPlugin**: Initializes the terminal user interface using the `bevy_ratatui` library.  
- **GuiPlugin**: Manages the graphical interface of the game, including trajectory editing.

The `ClientPlugin` also registers the central game plugin: **GamePlugin**. This plugin groups the main systems related to the simulation core via several thematic sub-plugins:

- **PhysicsPlugin**: Handles the temporal and physical simulation of the game (orbits, gravitational fields, numerical integration, etc.).  
- **BodiesPlugin**: Manages the creation and behavior of celestial bodies (stars, planets, moons…).  
- **ShipsPlugin**: Manages ship behavior.

This modular organization ensures **good scalability** of the project. Each feature is isolated in its own plugin, facilitating testing and the addition of new behaviors.

Moreover, this plugin-based architecture is also reflected in the **project folder structure**. The source code is organized by functional domains, where each folder contains the systems, components, and resources necessary for a given feature (see Figure 2). This maintains consistency between the logical code structure and the physical structure on disk.

---

### Folder Organization

Each sub-folder corresponds to a part of the game, usually structured around a plugin. It contains the components, Bevy systems, resources, and events specific to that feature. This separation improves code readability and facilitates project evolution.

*Figure 2: Partial physical organization of the `src/` folder with associated Bevy plugins (partial_folder_organization.txt)*

For example, the `objects/` folder contains elements related to game entities:

- `objects/ships/`: contains files related to ship management.  
- `objects/ships/trajectories.rs`: defines systems that read trajectory modifications and consequently update ship velocity at each `FixedUpdate`.  
- `objects/ships.rs`: contains event management systems, such as creating a new ship, and functions associated with instantiating or modifying ship entities in the Bevy world.

Other folders follow the same logic, for example:

- `physics/`: contains modules related to physical simulation (orbits, influence forces, numerical integration…).  
- `ui/`: groups files necessary for the user interface, both in graphical and terminal modes.

This organization ensures a **clear, scalable, and coherent structure** in line with the Bevy plugin model: each major feature has its own space while interacting with others via shared resources and events.

**Note:** This overview covers the main plugins and folder structure. For details on additional sub-plugins or modules, please refer directly to the source code.