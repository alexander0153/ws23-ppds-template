# Programmierpraktikum Datensysteme

This repository contains the project template for a transactional, in-memory, index server.

## Installation

This project uses development containers to ensure a consistent development environment. Alternatively, installing Rust and the `build-essential` package (for `make`, `gcc`, etc.) should be sufficient to compile the project.

## Usage

Running `make test` will compile our implementation and run both `unittest.c` and `speed_test.c` drivers for testing and benchmarking. Individual tests can be run using `test_unit` and `test_speed` respectively.

Setting the environment var `IMPLEMENTATION` to `speedLinux` will run the same drivers using the provided reference implementation. This can be achieved by running `IMPLEMENTATION=speedLinux make test`.

### Examples

```bash
# Compile and run tests
$ make test        # run all tests
$ make test_unit   # only run tests
$ make test_speed  # only run benchmarks

# Compile and run tests using reference implementation
$ IMPLEMENTATION=speedLinux make test
$ IMPLEMENTATION=speedLinux make test_unit
$ IMPLEMENTATION=speedLinux make test_speed
```
