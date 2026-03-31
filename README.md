# PAPI_OS

## pre-requisites

- qemu

## how to run

```sh
$ cargo install bootimage
```

```sh
$ cargo bootimage && qemu-system-x86_64 -drive format=raw,file=target/x86_64-papi_os/debug/bootimage-papi_os.bin
```

## RustRover / IntelliJ note

This project uses a custom JSON target: `x86_64-papi_os.json`.

JetBrains IDEs sometimes run `cargo metadata` outside the project root, so the local `.cargo/config.toml` is not picked up. When that happens, the IDE can show an error like this:

```text
error: `.json` target specs require -Zjson-target-spec
```

If you see that error in RustRover or IntelliJ, add this file:

`~/.cargo/config.toml`

```toml
[unstable]
json-target-spec = true
```

Then reload the Cargo project in the IDE.
