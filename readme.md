# silly08 
<sup>the silliest pico-8 emulator • AI-free codebase</sup>

silly08 is a pico-8 runtime available as a libretro core.

it notably features:
* 100% compatibility with the real pico-8
* save states

neither of which i have found in any other pico-8 libretro core.

the way it achieves 100% compatibility is that it runs the real pico-8 education edition in a sandboxed javascript runtime. this is super stupid and the namesake for the entire project :3

the project is written in rust and uses deno as the javscript runtime as its written in native rust. since its all native rust, it means  it can compile for most architectures, including ARM, meaning it can run on handhelds like the Anbernic RG SP:

<img src="https://cdn.hackclub.com/01a199de-ebdd-7914-bc7e-65287166eabd/picture.webp" width=300px>

## should you use this?

probably no? effectively, silly08 is a rust program, running a javascript runtime, running an emscripten compiled c++ program, running a lua runtime, running your pico-8 cartridge.

in practice, it will have quite awful performance on low-end devices, like the RG SP in the image above. it runs at a *playable* but uncomfortable framerate.

on a regular computer the performance is great, and on a more high-end handheld it might actually work great too, but this is the only one i have.

furthermore, as it stands today, the project is super early in development. there is still much low hanging fruit when it comes to performance optimizations, so it might improve over time.

this project was born from just a silly idea i had, and the want to prove it was possible. 

i might make a future version which ditches the education edition and JS runtime and acts as a thin wrapper around a real Pico-8 linux binary, which could realistically actually be really good!

## contributions !!

feel super free to contribute! though, **please dont use generative AI** for code or any potential assets <3

i want to keep this project AI-free, thats all :)

## building

the project can both be built as a libretro-core sharedlibrary (sillycore), or as a standalone program that runs directly on your computer (sillyapp)

to build you will need Rust installed.

to run the standalone program you should be able to just do: 
```bash
cargo run -p sillyapp
```

and to build the sharedlibrary simply do:
```bash
cargo build -p sillycore
```

now, to build for 64bit ARM (like what the RG SP and most handhelds use), use the taskfile. run `task build:arm64`. you will need docker installed (easiest with docker desktop as it will handle dependency management for you)