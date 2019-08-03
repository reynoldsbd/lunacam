The portal provides a unified UI for vieweing and managing all cameras in a LunaCam installation.


# Developer Instructions

Prerequisites:

* Rust
* npm
* Sass CLI

Install npm dependencies:

```shell
npm install
```

Compile stylesheets (add `--watch` for active development):

```shell
sass -I node_modules/bulma -I node_modules/bulma-switch/dist/css style:static/css
```

Running:

```shell
cargo run
```
