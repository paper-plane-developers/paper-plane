<h1 align="center">
  Telegrand
</h1>

<p align="center"><strong>A Telegram client optimized for the GNOME desktop</strong></p>

<p align="center">
  <a href="https://hosted.weblate.org/engage/telegrand/">
    <img src="https://hosted.weblate.org/widgets/telegrand/-/telegrand/svg-badge.svg" alt="Translation status" />
  </a>
  <a href="https://github.com/melix99/telegrand/actions/workflows/ci.yml">
    <img src="https://github.com/melix99/telegrand/actions/workflows/ci.yml/badge.svg" alt="CI status"/>
  </a>
  <a href="https://t.me/telegrandchat">
    <img src="https://img.shields.io/static/v1?label=Chat&message=@telegrandchat&color=blue&logo=telegram" alt="Telegram group">
  </a>
</p>

<!--
<p align="center">
  <img src="data/resources/screenshots/screenshot1.png" alt="Preview"/>
</p>
-->

## Installation Instructions

Telegrand is an in-development project and it isn't considered stable software yet. But if you still want to try it out, there's a [CI](https://github.com/melix99/telegrand/actions?query=branch%3Amain) that automatically generates the latest flatpak build. Just download the artifact of the latest build and install it locally using `flatpak install telegrand.flatpak`.

## Build Instructions

### Gnome Builder

Using Gnome Builder is the easiest way to get the app built without even using the terminal: just clone the repository and press the big "Run" button at the top and it will automatically build all the required dependencies together with the app.

### Meson

We use TDLib as the backend for the Telegram API, so you'll need to have TDLib already installed in your system (together with GTK4 and libadwaita). Then, you'll need Meson and Rust to actually build the app.

```shell
meson . _build --prefix=/usr/local
ninja -C _build
sudo ninja -C _build install
```

## Telegram API Credentials

Telegram requires custom clients to set some credentials for using their API. Telegrand doesn't provide official credentials, so the packagers are expected to set their own credentials for distributing the app. Anyway, Telegrand does include by default the official credentials that Telegram provides for testing purposes, which are very limited, but usable (expecially for development).

## Acknowledgment

The general code architecture was heavily inspired by [fractal-next](https://gitlab.gnome.org/GNOME/fractal/-/tree/fractal-next).
