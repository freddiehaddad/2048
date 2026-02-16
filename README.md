# 2048

A TUI Rust clone of the single-player sliding tile puzzle video game written by
Italian web developer Gabriele Cirulli. The objective of the game is to slide
numbered tiles on a grid to combine them to create a tile with the number 2048.

https://github.com/user-attachments/assets/483799e4-80bc-462f-9be7-8946324f6c94

## Quick Start

### Prerequisites

- Rust 1.93.0 or later

### How to Build

```console
# Clone the repository
git clone https://github.com/freddiehaddad/2048.git
cd 2048

# Build the project
cargo build --release
```

### Running the Game

```console
# Via Cargo
cargo run --release

# Directly after How to Build steps (Linux/Mac)
./target/release/2048

# Directly after How to Build steps (Windows)
./target/release/2048.exe
```

## Controls

| Action         | Keybindings     |
|----------------|-----------------|
| **Move Up**    | `↑` / `W` / `K` |
| **Move Down**  | `↓` / `S` / `J` |
| **Move Left**  | `←` / `A` / `H` |
| **Move Right** | `→` / `D` / `L` |
| **Restart**    | `R`             |
| **Quit**       | `Q`             |

## License

This project is licensed under the [MIT License](LICENSE).

## Contact

For questions or feedback, please open an issue on GitHub.

---

## Resources

- [Wikipedia](https://en.wikipedia.org/wiki/2048_(video_game))
- [2048.org](https://www.2048.org/)
- [GitHub](https://github.com/gabrielecirulli/2048)

---

**Built with ❤️ and Rust**
