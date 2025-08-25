# 🚀 Introduction to Bevy and ECS Architecture

Bevy is a game engine based on the **Entity-Component-System (ECS)** model, an architecture that promotes modularity, flexibility, and a clear separation of responsibilities in code. This model is especially well-suited for this project, where different features are developed by multiple interns. Since all project code relies on this model, understanding how it works is essential to work efficiently.

---

## 🧩 The ECS Architecture: Three Main Concepts

- **Entity:**  
  A unique identifier associated with a group of components. An entity contains no data itself but serves as an abstract container to which components are attached.

- **Component:**  
  Data structures (structs in Rust) that hold data related to an entity. Each component must implement the `Component` trait.

- **System:**  
  Rust functions that manipulate components. A system reads or modifies the state of components to perform actions such as handling physics or game behavior.

---

## ⏰ System Scheduling in Bevy

Systems are registered in Bevy’s `App` through **Schedules** that determine when and in what order they run. Bevy provides several basic schedules, executed in a loop every frame:

- **Startup:** Runs once at application startup. ⚡
- **PreUpdate**, **Update**, **PostUpdate:** Run each frame, in this order, structuring different processing phases (e.g., game logic, UI, etc.). 🔄

A special schedule, **FixedUpdate**, runs at regular intervals regardless of the machine's refresh rate. This is useful for game logic systems like movement or physics. 🕹️

You can also create **system sets** within schedules to finely organize execution order. For example, a `PhysicsUpdate` set might group systems managing spaceship trajectories and time progression, running within the `FixedUpdate` schedule. 🚀

---

## 🛠️ Additional ECS Concepts in Bevy

- **Resources:**  
  Global, unique structures that store data shared across the entire game, like the simulation tick or the currently selected ship in the editor. Resources implement the `Resource` trait. 📦

- **Events:**  
  For communication between systems, Bevy provides an event system:  
  - A system with `EventWriter` sends events (e.g., when a key is pressed). ⌨️  
  - A system with `EventReader` reacts to those events (e.g., moving a player when a movement event is detected). 🎮

- **Queries:**  
  Systems use queries to access entity data, specifying which components must be present on returned entities, enabling targeted and efficient processing. 🔍

---

## 🔌 Modularity with Plugins

Bevy is designed to be modular through **Plugins**, which group sets of systems and their scheduling, resources, and components. For example, a physics plugin (`PhysicsPlugin`) can bundle all physics-related systems and resources, easily integrated into the main `App`. 🧩

---

## 📚 Further Reading

For more information, you can consult:

- The [official Bevy documentation](https://bevyengine.org/learn/book/) 📖  
- The [Bevy Quickstart Guide](https://bevyengine.org/learn/book/introduction/) 🚀  
- The [Unofficial Bevy Cheat Book](https://bevy-cheatbook.github.io/) (Note: this resource may not always be fully up-to-date with the latest Bevy version.) ⚠️

