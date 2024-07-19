# cetrainer-unpacker

A simple tool to unpack/decrypt Cheat Engine's trainers

# Supported Formats

- [x] Protected `.CETRAINER` files
- [x] Unprotected `.CETRAINER` files (does nothing)
- [x] Protected `Gigantic` trainers (.exe that has the entire cheat engine bundled)
- [x] Protected `Tiny` trainers (.exe that has only the trainer code)

# Supported versions

- [x] Cheat Engine 6.4 - 7.5 (and possibly newer versions)

# Usage

```bash
cetrainer-unpacker <trainer_file>
```

> [!NOTE]  
> Files will be extracted to the same directory as the trainer file

> [!TIP]
> On Windows, you can drag and drop the trainer file onto the executable

# building

install rust and run `cargo build --release`

# License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details
