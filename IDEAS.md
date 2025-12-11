# IDEAS for Digging Game

This document outlines potential improvements and new features for the Digging Game, categorized for clarity.

## I. Core Game Mechanics & World Generation

1.  **Chunk Unloading/Garbage Collection:**
    *   **Problem:** Currently, once a chunk is generated, it remains in memory (`WorldManager.chunks`). For an infinite world, this will eventually lead to excessive memory consumption.
    *   **Idea:** Implement a mechanism to unload chunks that are far from the player's current camera view. This would involve saving their modified state (if any) and removing them from `WorldManager.chunks`.
    *   **Impact:** Improved memory management, enabling truly larger or infinite worlds.

2.  **More Sophisticated World Generation:**
    *   **Biomes:** Introduce different biomes (e.g., forest, desert, tundra, deep caves) with unique block distributions, surface features, and potentially distinct ore types.
    *   **Structures:** Implement procedural generation for natural structures like caves, underground lakes, or even simple ruins/dungeons.
    *   **Resource Distribution:** Refine ore distribution logic (e.g., ores appearing in veins, rarity increasing with depth, specific ores appearing in certain biomes).
    *   **Impact:** Increased exploration value, visual variety, and strategic depth.

3.  **Block Types as Objects/Data-Driven:**
    *   **Problem:** Adding new block types with unique properties (sprite, hardness, sound, item drops) requires modifying `Block` logic.
    *   **Idea:** Define block types as separate data structures or classes (e.g., `BlockType` class holding properties like `sprite_info`, `base_hardness`, `drop_item_id`). `Block` instances would then reference a `BlockType` object instead of hardcoding properties.
    *   **Impact:** Easier to add and manage new block types, improved extensibility.

4.  **Item Drops & Inventory System:**
    *   **Problem:** When blocks are broken, they only generate visual particles.
    *   **Idea:** Implement an item system where broken blocks drop collectible items (e.g., dirt, stone, coal, rare gems). This would necessitate:
        *   An `Item` class with properties (ID, name, stackable).
        *   An `Inventory` system for the player to store items.
        *   A mechanism for players to pick up items (e.g., walking over them).
        *   An `Inventory UI` to view and manage collected items.
    *   **Impact:** Introduces a core loop of mining, collecting, and managing resources, which is fundamental to digging games.

5.  **Crafting System:**
    *   **Prerequisite:** Item drops and inventory.
    *   **Idea:** Allow players to combine collected items into new tools, blocks, or other useful objects. This would involve:
        *   A `Recipe` system defining input items and output items.
        *   A `Crafting UI`.
    *   **Impact:** Adds progression, utility for collected resources, and more complex gameplay.

## II. User Interface & Experience (UI/UX)

1.  **Expanded Main Menu:**
    *   **Problem:** The current title screen is minimal.
    *   **Idea:** Add "Continue Game" (loads last save automatically), "Options", and "Exit Game" buttons directly on the title screen.
    *   **Impact:** Improved user flow and convenience.

2.  **Options Menu - Sub-menus:**
    *   **Problem:** As settings grow, the single pause menu will become cluttered.
    *   **Idea:** Introduce sub-menus for categories like "Audio Settings" (sound, music volume), "Video Settings" (screen resolution, fullscreen toggle), and "Controls" (keybindings).
    *   **Impact:** Better organized settings, easier for users to find what they need.

3.  **Keybinding Customization UI:**
    *   **Problem:** Keybindings are currently hardcoded.
    *   **Idea:** A dedicated UI in the options menu to allow users to remap game controls to their preferred keys. This would involve storing keybindings persistently (via `PersistenceManager`).
    *   **Impact:** Greatly enhanced accessibility and player comfort.

4.  **Save Slot Selection UI:**
    *   **Problem:** Only one save file is currently supported.
    *   **Idea:** Implement a UI for multiple save slots, allowing players to create, load, and delete different game saves.
    *   **Impact:** Prevents accidental overwrites, supports multiple playthroughs.

5.  **Confirmation Dialogs:**
    *   **Problem:** Actions like "Quit Game" or "Overwrite Save" happen immediately without warning.
    *   **Idea:** Add confirmation dialogs before executing irreversible actions.
    *   **Impact:** Prevents accidental data loss or premature exits.

6.  **Persistent Settings:**
    *   **Problem:** Settings like `se_on`, `bgm_on`, and chosen `current_language_code` are not saved between game sessions.
    *   **Idea:** Extend `PersistenceManager` to save and load these global settings.
    *   **Impact:** Settings persist, providing a consistent experience for the player.

7.  **Hot-reload Language in-game:**
    *   **Problem:** Changing language in the menu might require a save/load or restart to fully apply (depending on how `pyxel.title` and other strings are updated).
    *   **Idea:** Ensure that changing the language via the menu immediately updates all displayed text elements without needing a reload. (`diggame.update_window_title()` is good, but other UI elements need to re-render with new strings).
    *   **Impact:** Seamless language switching experience.

## III. Visuals & Feedback

1.  **More Particle Types/Effects:**
    *   **Idea:** Introduce different visual particles or more complex effects for specific events (e.g., sparkling particles for rare ore, larger dust clouds for hard blocks).
    *   **Impact:** Enhanced visual feedback and game immersion.

2.  **Sound/Music Volume Control:**
    *   **Prerequisite:** Options menu.
    *   **Idea:** Implement sliders or toggle buttons in the options menu to control the volume of sound effects and background music independently.
    *   **Impact:** Improved player control over audio experience.

3.  **Debug Visualizations:**
    *   **Chunk Borders:** Add an F3 debug mode overlay to show chunk borders, helping visualize the chunk loading/unloading system.
    *   **Impact:** Aids development and debugging of world management.

## IV. Technical & Performance

1.  **Error Logging:**
    *   **Problem:** `traceback` is imported but not explicitly used for logging in `diggame.py` or `managers.py` (beyond `traceback.print_exc()`).
    *   **Idea:** Implement a simple logging system (e.g., writing errors to a file) to capture runtime exceptions and warnings for easier debugging by players/developers.
    *   **Impact:** Improved diagnosability of issues.

2.  **Further Optimization of Drawing (if needed):**
    *   **Problem:** If the game becomes very complex or has many on-screen elements, Pyxel's drawing performance might become a bottleneck.
    *   **Idea:** Investigate techniques like sprite batching (if Pyxel supports it or a custom solution), dirty rectangle rendering (only redrawing changed areas), or further optimizing `get_active_blocks_in_view()`.
    *   **Impact:** Smoother framerates for more demanding scenarios.

## V. Testing

1.  **Expand WorldManager Tests:**
    *   **Problem:** `WorldManager` currently has limited direct unit tests.
    *   **Idea:** Add dedicated tests for chunk generation logic, `get_block_at_world_coords`, and `regenerate_world_from_chunks_and_apply_mods` to ensure its core functionality is robust.
    *   **Impact:** Increased confidence in world generation and persistence logic.

2.  **GameMenu Interaction Tests:**
    *   **Problem:** `GameMenu` has complex UI interaction logic (checkboxes, dropdowns, buttons).
    *   **Idea:** Write more comprehensive unit tests using mocks to simulate various user inputs and verify correct menu state changes and actions.
    *   **Impact:** Reduces potential bugs in menu navigation and option handling.

3.  **Integration Tests:**
    *   **Idea:** Develop a suite of higher-level tests that verify the interaction between multiple components (e.g., a save-load cycle that modifies blocks and checks if they are correctly restored).
    *   **Impact:** Catches bugs that might arise from component interactions.
