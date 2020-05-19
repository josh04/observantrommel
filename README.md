# Observant Rommel
## A card-counting tool for the card game 'Secret Hitler'.

Hosted version at https://josh04.computer/dropbox/sechit/

Built with Rust and wasm-pack. 

### Build instructions

To build, install Rust via [Rustup](https://rustup.rs/) followed by [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/). 
You will also need a version of [npm](https://www.npmjs.com/get-npm).

Run `wasm-pack build` in the root followed by moving into the `www` directory and running `npm install` (you only need to do this command once).
Finally, run either `npm run start` to host a local webserver running the app, or `npm run build` to package it for hosting in `www/dist`.