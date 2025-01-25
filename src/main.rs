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
