balance ⚖️

This is a little project I made where you type in what you have eaten and it recommends you some foods to help make sure you get all your nutrients :)

https://oscarsaharoy.github.io/balance

This uses the nutrition data from the [Composition of foods integrated dataset](https://www.gov.uk/government/publications/composition-of-foods-integrated-dataset-cofid)

![app screenshot](assets/app_screenshot.png)

Run locally:
- [install rust](https://www.rust-lang.org/tools/install) - you may need to install using rustup so that you can easily install the webassembly target with `rustup target add wasm32-unknown-unknown`
- install [trunk](https://trunkrs.dev/): `cargo install trunk`
- go into this repo root and `trunk serve`
