# nix-forecast

Check the forecast for today's Nix builds with a blazingly fast (🚀🔥🦀) CLI

## Usage

```
Check the forecast for today's Nix builds

Usage: nix-forecast [OPTIONS] [INSTALLABLES]...

Arguments:
  [INSTALLABLES]...  A list of Nix installables to look for. If not given, all paths in nixpkgs are checked

Options:
  -c, --configuration <CONFIGURATION>  Flake reference pointing to a NixOS or nix-darwin configuration
  -o, --home <HOME>                    Flake reference pointing to a standalone home-manager configuration
  -b, --binary-caches <BINARY_CACHES>  URLs of the substituters to check (can be passed more than once) [default: https://cache.nixos.org]
  -f, --flake <FLAKE>                  Flake reference of nixpkgs (or other package repository) [default: nixpkgs]
  -s, --show-missing                   Show a list of store paths not found in the substituter
  -h, --help                           Print help
  -V, --version                        Print version
```

## Examples

### Flake installables

```sh
nix-forecast nixpkgs#{hello,gcc,clang,nrr}
```

### NixOS configuration


```sh
nix-forecast -c ".#nixosConfigurations.myMachine"
```

### nix-darwin configuration

```sh
nix-forecast -c ".#darwinConfigurations.myMac"
```

## Why?

Finding out if paths are cached can be a bit troublesome in Nix, with commands like `nix build --dry-run`
being the only solution a lot of the time. Meanwhile in the world of [Guix](https://guix.gnu.org/), they have
had the `guix weather` command to do this for ages! This project aims to bring the power of that command right
to Nix

### What about `nix-weather`?

`nix-weather` is another project with a similar goal of bringing some of the features of `guix weather` to
Nix. However, it does introduce it's own spin on things and has much more of a focus on NixOS configurations.
In contrast, `nix-forecast` aims to be as close to the Guix command as possible, while also introducing more
generic support for [Flake](https://nix.dev/concepts/flakes) references, NixOS *and* nix-darwin
configurations, better error messages, and a (subjectively) more comfortable interface

I've also made it slightly faster :p

```
$ hyperfine --warmup 1 './result/bin/nix-weather --name glados --config ~/flake' './target/release/nix-forecast --configuration ~/flake#nixosConfigurations.glados'
Benchmark 1: ./result/bin/nix-weather --name glados --config ~/flake
  Time (mean ± σ):     11.387 s ±  0.646 s    [User: 3.422 s, System: 1.663 s]
  Range (min … max):   10.490 s … 12.569 s    10 runs

Benchmark 2: ./target/release/nix-forecast --configuration ~/flake#nixosConfigurations.glados
  Time (mean ± σ):      6.395 s ±  0.229 s    [User: 0.992 s, System: 0.967 s]
  Range (min … max):    6.231 s …  7.030 s    10 runs

  Warning: Statistical outliers were detected. Consider re-running this benchmark on a quiet system without any interferences from other programs. It might help to use the '--warmup' or '--prepare' options.

Summary
  ./target/release/nix-forecast --configuration ~/flake#nixosConfigurations.glados ran
    1.78 ± 0.12 times faster than ./result/bin/nix-weather --name glados --config ~/flake
```

## Inspired by...

- [guix weather](https://guix.gnu.org/manual/en/html_node/Invoking-guix-weather.html)
- [nix-weather](https://github.com/cafkafk/nix-weather/)
- [My much slower version in Fish](https://discourse.nixos.org/t/how-to-find-uncached-dependencies-of-a-closure/45385)
