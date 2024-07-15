# Zmem: Linux Memory Monitoring

Zmem is a tool for monitoring virtual memory on Linux systems, focused specifically on swap usage. With detailed per-process swap usage and zswap statistics like compression ratio and zswap compressed cache, zmem helps keep track of your system's memory usage and optimize performance.

The name Zmem comes from the fact that it is inspired by Linux kernel features zswap and zram, both of which start with the letter Z. Zmem similarly focuses on memory-related information, hence the name.

> [!WARNING]
> This program is a work in progress and may contain bugs. Use at your own risk.

## Installation

```sh
git clone http://github.com/xeome/Zmem
cd Zmem
cargo install --path .
```

## Usage

To use Zmem, simply run the command below in your terminal:

```bash
zmem
```

or if you want per-process swap usage:

```bash
zmem -p
```

![zmem](assets/zmem.png)

## Contributing

We welcome contributions from the community to improve Zmem. If you have any ideas for new features, suggestions for improvements, or you have discovered a bug, please feel free to open an issue or submit a pull request.

When submitting pull requests, please make sure that your code adheres to the project's coding standards and guidelines. This project is performance-sensitive, so it is important to ensure that any changes do not negatively impact performance.

You should benchmark your changes to ensure that they do not introduce any performance regressions. If you are adding a new feature, please include tests to ensure that it works as expected and does not break existing functionality.

Example benchmark:

```sh
hyperfine 'zmem -p'
```

results:

![benchmark](assets/hyperfine.png)

Before making any significant changes to the project, it is best to open an issue and discuss your proposal with the project maintainers. This will help you get feedback, ensure that your changes align with the project's goals, and avoid duplicating work.

Thank you for your interest in contributing to Zmem. Your help is appreciated, and we look forward to working with you.

## License

Zmem is licensed under the GPL3 License. See [LICENSE](LICENSE) for more information.

Thank you for using Zmem!
