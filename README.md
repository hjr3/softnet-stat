# softnet-stat

Parse the `/proc/net/softnet_stat` file into something more readable. For more information about `/proc/net/softnet_stat`, I suggest reading [Monitoring and Tuning the Linux Networking Stack: Receiving Data](http://blog.packagecloud.io/eng/2016/06/22/monitoring-tuning-linux-networking-stack-receiving-data/#help-with-linux-networking-or-other-systems).

## Compatible Systems

This parser should work on all Linux kernels since v2.6.32. In later versions of the kernel, more fields were added. Currenty, the parser will default these fields to a value of `0` if they are not found in the file.

## Build

This software was written in Rust using v1.10.0. All dependencies are listed in `Cargo.toml`. To build: `$ cargo build`.


## Tests

This program has been tested against 3 versions of the `/proc/net/softnet_stat` file. To excercise these tests: `$ cargo test`.
