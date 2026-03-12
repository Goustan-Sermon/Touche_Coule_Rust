# Touche-Coulé: Network Battleship in Rust
[![Ask DeepWiki](https://devin.ai/assets/askdeepwiki.png)](https://deepwiki.com/Goustan-Sermon/Touche_Coule_Rust)

This repository contains the source code for a classic Battleship game (`Touche-Coulé` in French) implemented in Rust. The game is played between two players over a network, directly within the terminal, featuring an interactive, cursor-based interface.

## Features
*   **Network Gameplay**: Play against another person on a local network. Simply choose to host a game or join by providing the host's IP address.
*   **Interactive Terminal UI**: Utilizes `crossterm` to provide a rich, in-terminal experience. No more typing coordinates!
*   **Cursor-Based Controls**: Use arrow keys to move a cursor for both placing your ships and targeting your opponent's grid.
*   **Dynamic Ship Placement**: Place your fleet manually, with the ability to move and rotate ships before finalizing their position. A "ghost" ship shows you the placement before you confirm.
*   **Randomized Fleet Deployment**: Want to get into the action quickly? Choose the automatic placement option to deploy your fleet instantly.
*   **Real-time Game State**: Get immediate feedback on your shots: `Miss`, `Hit`, or `Sunk!`.

## How to Play

1.  **Launch the Game**: Run the application from your terminal.
2.  **Enter Your Name**: Choose a name for your commander.
3.  **Connect**:
    *   **Host**: Select `1` to host a game. The application will wait for an opponent to connect.
    *   **Join**: Select `2` to join a game and enter the IP address of the host.
4.  **Deploy Your Fleet**:
    *   Choose `1` for **Manual Placement**:
        *   Use the **Arrow Keys** to move the current ship.
        *   Press **'R'** to rotate the ship between horizontal and vertical orientations.
        *   Press **Enter** to confirm the placement.
        *   Repeat for all ships in your fleet.
    *   Choose `2` for **Random Placement** to have the game set up your board automatically.
5.  **Combat Phase**:
    *   The host takes the first turn.
    *   When it's your turn, use the **Arrow Keys** to move the targeting cursor on the enemy's grid (your "radar").
    *   Press **Enter** to fire at the selected coordinates.
    *   The result of the shot will be displayed, and your radar will be updated.
6.  **Winning**: The game ends when a player successfully sinks all of the opponent's ships.

## Getting Started

### Prerequisites
*   [Rust](https://www.rust-lang.org/tools/install) and Cargo must be installed on your system.

### Installation & Launch
1.  Clone the repository:
    ```sh
    git clone https://github.com/goustan-sermon/touche_coule_rust.git
    ```
2.  Navigate to the project directory:
    ```sh
    cd touche_coule_rust
    ```
3.  Run the game:
    ```sh
    cargo run
    ```
The game will build and launch in your terminal.

## Project Structure
The codebase is organized into three main modules:

*   `src/main.rs`: The application's entry point. Manages the main game loop, handles user input, and orchestrates the different game phases (menu, placement, combat).
*   `src/modele.rs`: Defines the core data structures and logic of the Battleship game. This includes `Grille` (Grid), `Navire` (Ship), `Coordonnee` (Coordinates), and the rules for placing ships and processing shots.
*   `src/reseau.rs`: Implements all networking functionality. It defines a simple text-based protocol (`MessageReseau`) for communication and handles setting up a TCP server (host) and client (join).

## Dependencies
*   [`crossterm`](https://github.com/crossterm-rs/crossterm): For cross-platform terminal manipulation, enabling the interactive UI, cursor control, and screen clearing.
*   [`rand`](https://github.com/rust-random/rand): Used for the random fleet placement feature.