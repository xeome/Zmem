# Zmem: Linux Memory Monitoring
Zmem is a tool for monitoring virtual memory on Linux systems, focused specifically on swap usage. With detailed per-process swap usage and zswap statistics like compression ratio and zswap compressed cache, zmem helps keep track of your system's memory usage and optimize performance.

The name Zmem comes from the fact that it is inspired by Linux kernel features zswap and zram, both of which start with the letter Z. Zmem similarly focuses on memory-related information, hence the name.

**NOTE:** This program is a work in progress and may contain bugs. Use at your own risk.

# Installation
TODO

```
git clone http://github.com/xeome/Zmem
cd Zmem
cargo build -r
cp ./target/release/zmem /usr/local/bin/
```

# Usage

To use Zmem, simply run the command below in your terminal:

```bash
zmem
```

![](https://cdn.discordapp.com/attachments/739162076886597715/1101525847376134215/zmem.png)

# Contributing
We welcome contributions to Zmem! If you have an idea for a new feature or have found a bug, please open an issue or submit a pull request.

# License
Zmem is licensed under the GPL3 License. See [LICENSE](LICENSE) for more information.

Thank you for using Zmem!
