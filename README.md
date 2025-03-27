# ✨ gperf2flamegraph

<div align="center">
<img alt="GitHub" src="https://img.shields.io/badge/github-%23121011.svg?style=for-the-badge&amp;logo=github&amp;logoColor=white">
<img alt="Rust" src="https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&amp;logo=rust&amp;logoColor=white">
<img alt="Flamegraph" src="https://img.shields.io/badge/flamegraph-%23FF6F61.svg?style=for-the-badge&amp;logo=flamegraph&amp;logoColor=white">
<img alt="Gprof" src="https://img.shields.io/badge/gprof-%23FF6F61.svg?style=for-the-badge&amp;logo=gprof&amp;logoColor=white">

![GitHub](https://img.shields.io/github/license/Emin017/gperf2flamegraph)
![GitHub workflows](https://img.shields.io/github/actions/workflow/status/Emin017/gperf2flamegraph/build.yml)
![GitHub issues](https://img.shields.io/github/issues/Emin017/gperf2flamegraph)
![GitHub pull requests](https://img.shields.io/github/issues-pr/Emin017/gperf2flamegraph)
![GitHub release (latest by date)](https://img.shields.io/github/v/release/Emin017/gperf2flamegraph)

![GitHub stars](https://img.shields.io/github/stars/Emin017/gperf2flamegraph?style=social)
![GitHub forks](https://img.shields.io/github/forks/Emin017/gperf2flamegraph?style=social)

[![English](https://img.shields.io/badge/English-README-2ea44f?style=for-the-badge)](README.md)
<!-- [![中文](https://img.shields.io/badge/中文-介绍-FF6F61?style=for-the-badge)](README_CN.md) -->
</div>

A Rust tool for converting Google gperftools CPU profiler output to FlameGraph visualization format.

## 🔥 Features

* 📊 Parse binary output from gperftools CPU profiler
* 🔍 Resolve symbols with demangling support
* 🧰 Customizable output options for visualization
* 📝 Generate text format for debugging or further processing
* 🚀 Performance optimized with Rust

## 🛠️ Installation

build from source:
```shell
cargo build --release
```

## 📖 Usage
Basic syntax:
```shell
gperf2flamegraph <EXECUTABLE> <PROFILE_FILE> [OPTIONS]
```

Parameters
* `<EXECUTABLE>`: Path to the executable binary that was profiled
* `<PROFILE_FILE>`: Path to gperftools CPU profiler result file

Options
| Option | Description |
|--------|-------------|
| `--help` | Show help message |
| `--svg-output <PATH>`  | Path for SVG flamegraph output |
| `--text-output <PATH>` | Path for text format output |
| `--simplify-symbol`	 | Simplify symbol names (remove template/function args) |
| `--executable-only`	 | Only resolve symbols from the executable (ignore libraries) |
| `--annotate-libname` | Add library name annotations like `[libname.so]` |
| `--to-microsecond` | Use microseconds as time unit (default is sample count) |
| `--flamegraph-path <PATH>` | Path to flamegraph.pl script (default: "flamegraph.pl") |

## 🧪 Example

### Basic Usage
```shell
gperf2flamegraph <gprof_output_file> <output_file>
```

### With Options
```shell
gperf2flamegraph <EXECUTABLE> gprof.prof --svg-output gprof.svg --text-output gprof.txt
```

---

***Note: The generated SVG files are best viewed in a modern browser. For large profiles, consider using the `--simplify-symbol` option to improve readability.***
## 🔧 Troubleshooting
### Missing flamegraph.pl
If you encounter "Failed to start flamegraph.pl" error:
```shell
# Option 1: Install FlameGraph and specify path
git clone https://github.com/brendangregg/FlameGraph.git
./gperf2flamegraph myapp cpu.prof --svg-output out.svg --flamegraph-path ./FlameGraph/flamegraph.pl

# Option 2: Add FlameGraph to your PATH
export PATH=$PATH:$(pwd)/FlameGraph
```

### Symbol Resolution Issues

If you see too many "???" unknown symbols:

1. Ensure your binary was compiled with debug information (-g flag)
2. Check that you're providing the correct executable path

## 🤝 Contributing
Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License
This project is licensed under the Mulan PSL v2 License - see the [LICENSE](LICENSE) file for details.