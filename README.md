# PAPI_OS

## pre-requisites

- qemu

## how to run

```sh
$ cargo install bootimage
$ cargo bootimage
$ qemu-system-x86_64 -drive format=raw,file=target/x86_64-papi_os/debug/bootimage-papi_os.bin
```

```sh
$ qemu-system-x86_64 -drive format=raw,file=target/x86_64-papi_os/debug/bootimage-papi_os.bin
```