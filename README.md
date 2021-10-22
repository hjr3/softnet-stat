# softnet-stat

[![Build Status](https://travis-ci.org/hjr3/softnet-stat.svg?branch=master)](https://travis-ci.org/hjr3/softnet-stat)

Parse the `/proc/net/softnet_stat` file into something more readable. For more information about `/proc/net/softnet_stat`, I suggest reading [Monitoring and Tuning the Linux Networking Stack: Receiving Data](http://blog.packagecloud.io/eng/2016/06/22/monitoring-tuning-linux-networking-stack-receiving-data/#help-with-linux-networking-or-other-systems).

## Compatible Systems

This parser should work on all Linux kernels since v2.6.32. In later versions of the kernel, more fields were added. Currently, the parser will default these fields to a value of `0` if they are not found in the file.

## Distribution

The `softnet-stat` binary is distributed via the [hjr3/softnet-stat](https://packagecloud.io/hjr3/softnet-stat) repository on packagecloud.io

## Build

This software was written in Rust using v1.10.0. All dependencies are listed in `Cargo.toml`. To build: `$ cargo build`.

### Static Binary

To build a completely static binary for production use, the binary can use musl as a target.

For Linux:

```shell
$ cargo build --target=x86_64-unknown-linux-musl
```

For macOS:

```shell
docker run --rm -it -v "$(pwd)":/home/rust/src ekidd/rust-musl-builder cargo build --release
```

## Tests

This program has been tested against `/proc/net/softnet_stat` files from these Linux versions:

* `v2.6.32`
* `v2.6.36`
* `v3.11.x`
* `v5.10.47`

To exercise these tests: `$ cargo test`.

## Examples

### Formatted

```shell
./softnet-stat

Cpu            Processed      Dropped        Time Squeezed  Cpu Collision  Received RPS   Flow Limit Count
0              1842008611     0              1              0              0              0
1              1863193957     0              2              0              0              0
2              1711764716     0              3              0              0              0
3              1640600369     0              0              0              0              0
4              1737798067     0              5              0              0              0
5              1686686610     0              1              0              0              0
```

### Json

```shell
./softnet-stat --json
[{"processed":1842008611,"dropped":0,"time_squeeze":1,"cpu_collision":0,"received_rps":null,"flow_limit_count":null},{"processed":1863193957,"dropped":0,"time_squeeze":2,"cpu_collision":0,"received_rps":null,"flow_limit_count":null},{"processed":1711764716,"dropped":0,"time_squeeze":3,"cpu_collision":0,"received_rps":null,"flow_limit_count":null},{"processed":1640600369,"dropped":0,"time_squeeze":0,"cpu_collision":0,"received_rps":null,"flow_limit_count":null},{"processed":1737798067,"dropped":0,"time_squeeze":5,"cpu_collision":0,"received_rps":null,"flow_limit_count":null},{"processed":1686686610,"dropped":0,"time_squeeze":1,"cpu_collision":0,"received_rps":null,"flow_limit_count":null}]
```

### Read From Stdin

```
$ cat /path/to/file | ./target/debug/softnet-stat -s
Cpu            Processed      Dropped        Time Squeezed  Cpu Collision  Received RPS   Flow Limit Count
0              1842008611     0              1              0              0              0
1              1863193957     0              2              0              0              0
2              1711764716     0              3              0              0              0
3              1640600369     0              0              0              0              0
4              1737798067     0              5              0              0              0
5              1686686610     0              1              0              0              0
```
