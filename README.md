# dot

Data-over-time

This tool takes a directory and recursively walks it, summing the sizes of files that were modified between the start and end date.

In it's current form `fd` or `find` can do what this does, I just didn't trust their output - turns out they were right though!

## Build

```bash
cargo build --release
```

### Cross Compile for Linux

```bash
cross build --release --target x86_64-unknown-linux-gnu
```
