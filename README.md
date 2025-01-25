# Slint Rust Template

A template for a Rust application that's using [Slint](https://slint.rs/) for the user interface.

## About

This template helps you get started developing a Rust application with Slint as toolkit
for the user interface. It demonstrates the integration between the `.slint` UI markup and
Rust code, how to react to callbacks, get and set properties, and use basic widgets.

## Usage

1. Install Rust by following its [getting-started guide](https://www.rust-lang.org/learn/get-started).
   Once this is done, you should have the `rustc` compiler and the `cargo` build system installed in your `PATH`.
2. Download and extract the [ZIP archive of this repository](https://github.com/slint-ui/slint-rust-template/archive/refs/heads/main.zip).
3. Rename the extracted directory and change into it:
    ```
    mv slint-rust-template-main my-project
    cd my-project    
    ```
4. Build with `cargo`:
    ```
    cargo build
    ```
5. Run the application binary:
    ```
    cargo run
    ```

We recommend using an IDE for development, along with our [LSP-based IDE integration for `.slint` files](https://github.com/slint-ui/slint/blob/master/tools/lsp/README.md). You can also load this project directly in [Visual Studio Code](https://code.visualstudio.com) and install our [Slint extension](https://marketplace.visualstudio.com/items?itemName=Slint.slint).

## Next Steps

We hope that this template helps you get started, and that you enjoy exploring making user interfaces with Slint. To learn more
about the Slint APIs and the `.slint` markup language, check out our [online documentation](https://slint.dev/docs).

Don't forget to edit this readme to replace it by yours, and edit the `name =` field in `Cargo.toml` to match the name of your
project.


#![allow(non_local_definitions)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]

use rand::seq::SliceRandom; // For shuffling tiles
use slint::Model; // Import the Model trait

slint::include_modules!(); // Include Slint-generated modules

pub fn main() {
    // Create the main window
    let main_window = MainWindow::new();

    // Initialize the tiles with the provided images
    let mut tiles: Vec<TileData> = main_window.get_memory_tiles().iter().collect();

    // Duplicate the tiles to make pairs
    tiles.extend(tiles.clone());
    
    // Shuffle the tiles
    let mut rng = rand::thread_rng();
    tiles.shuffle(&mut rng);

    // Wrap the shuffled tiles into a Slint VecModel
    let tiles_model = std::rc::Rc::new(slint::VecModel::from(tiles));

    // Track the number of wrong choices
    let wrong_choices = std::rc::Rc::new(std::cell::Cell::new(0));
    const MAX_WRONG_CHOICES: u32 = 8; // Allow 5 wrong choices; exit on the 6th

    // Set the shuffled tiles to the main window's model
    main_window.set_memory_tiles(tiles_model.clone().into());
    let main_window_weak = main_window.as_weak();

    main_window.on_check_if_pair_solved(move || {
        let mut flipped_tiles =
            tiles_model.iter().enumerate().filter(|(_, tile)| tile.image_visible && !tile.solved);

        if let (Some((t1_idx, mut t1)), Some((t2_idx, mut t2))) =
            (flipped_tiles.next(), flipped_tiles.next())
        {
            let is_pair_solved = t1 == t2;
            if is_pair_solved {
                t1.solved = true;
                tiles_model.set_row_data(t1_idx, t1);
                t2.solved = true;
                tiles_model.set_row_data(t2_idx, t2);
            } else {
                // Increment wrong choices counter
                let current_wrong_choices = wrong_choices.get() + 1;
                wrong_choices.set(current_wrong_choices);

                let main_window = main_window_weak.unwrap();
                main_window.set_disable_tiles(true);
                let tiles_model = tiles_model.clone();
                slint::Timer::single_shot(std::time::Duration::from_secs(1), move || {
                    main_window.set_disable_tiles(false);
                    t1.image_visible = false;
                    tiles_model.set_row_data(t1_idx, t1);
                    t2.image_visible = false;
                    tiles_model.set_row_data(t2_idx, t2);
                });

                // Check if maximum wrong choices reached
                if current_wrong_choices >= MAX_WRONG_CHOICES {
                    println!("Maximum wrong choices reached. Exiting the game.");
                    std::process::exit(0); // Terminate the application
                } else {
                    println!(
                        "Wrong choice {} out of {} allowed.",
                        current_wrong_choices, MAX_WRONG_CHOICES
                    );
                }
            }
        }
    });

    // Start the Slint event loop
    main_window.run(); // No need for unwrap here
}

struct TileData {
    image: image,
    image_visible: bool,
    solved: bool,
}

component MemoryTile inherits Rectangle {
    callback clicked;
    in property <bool> open_curtain;
    in property <bool> solved;
    in property <image> icon;

    height: 64px;
    width: 64px;
    background: solved ? #54b2bd : #565f7d;
    animate background { duration: 800ms; }

    Image {
        source: icon;
        width: parent.width;
        height: parent.height;
    }

    // Left curtain
    Rectangle {
        background: #4d8f75;
        x: 0px;
        width: open_curtain ? 0px : (parent.width / 2);
        height: parent.height;
        animate width { duration: 250ms; easing: ease-in; }
    }

    // Right curtain
    Rectangle {
        background: #4d8f75;
        x: open_curtain ? parent.width : (parent.width / 2);
        width: open_curtain ? 0px : (parent.width / 2);
        height: parent.height;
        animate width { duration: 250ms; easing: ease-in; }
        animate x { duration: 250ms; easing: ease-in; }
    }

    TouchArea {
        clicked => {
            // Delegate to the user of this element
            root.clicked();
        }
    }
}

export component MainWindow inherits Window {
    width: 326px;
    height: 326px;

    callback check_if_pair_solved(); // Added
    in property <bool> disable_tiles; // Added

    in property <[TileData]> memory_tiles: [
        { image: @image-url("icons/at.png") },
        { image: @image-url("icons/balance-scale.png") },
        { image: @image-url("icons/bicycle.png") },
        { image: @image-url("icons/bus.png") },
        { image: @image-url("icons/cloud.png") },
        { image: @image-url("icons/cogs.png") },
        { image: @image-url("icons/motorcycle.png") },
        { image: @image-url("icons/video.png") },
    ];
    for tile[i] in memory_tiles : MemoryTile {
        x: mod(i, 4) * 74px;
        y: floor(i / 4) * 74px;
        width: 64px;
        height: 64px;
        icon: tile.image;
        open_curtain: tile.image_visible || tile.solved;
        // propagate the solved status from the model to the tile
        solved: tile.solved;
        clicked => {
            tile.image_visible = !tile.image_visible;
        }
    }
    for tile[i] in memory_tiles : MemoryTile {
        x: mod(i, 4) * 74px;
        y: floor(i / 4) * 74px;
        width: 64px;
        height: 64px;
        icon: tile.image;
        open_curtain: tile.image_visible || tile.solved;
        // propagate the solved status from the model to the tile
        solved: tile.solved;
        clicked => {
            // old: tile.image_visible = !tile.image_visible;
            // new:
            if (!root.disable_tiles) {
                tile.image_visible = true;
                root.check_if_pair_solved();
            }
        }
    }
}