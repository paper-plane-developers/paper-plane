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

Telegrand is an in-development project and it isn't considered stable software yet. Also, the included API credentials are very limited and, in some cases, your account may end up banned by Telegram (check the `Telegram API Credentials` section below). You can avoid that by using a custom built version of Telegrand with provided API credentials via meson options, like [this AUR package](https://aur.archlinux.org/packages/telegrand-git) which you may prefer using if you use Arch Linux. But, if you still feel brave enough, there's a CI that automatically generates the latest flatpak build with the test API credentials: just download the [latest artifact](https://nightly.link/melix99/telegrand/workflows/ci/main) and install it locally using `flatpak install telegrand.flatpak`.

## Telegram API Credentials

Telegram requires custom clients to set some credentials for using their API. Telegrand doesn't provide official API credentials, so the packagers are expected to set their own credentials for distributing the app, obtainable at https://my.telegram.org/. However, Telegrand includes the Telegram's test credentials by default, which are very limited, but usable (especially for development). However, it's known that Telegram sometimes decides to ban accounts that use such credentials (especially newer accounts). For that reason, it's suggested to use your own API credentials, which can be set by using meson options (see the `Build Instructions` section below).

## Build Instructions

### Gnome Builder

Using Gnome Builder is the easiest way to get the app built without even using the terminal: just clone the repository and press the big "Run" button at the top and it will automatically build all the required dependencies together with the app.

### Meson

#### Prerequisites

The following packages are required to build Telegrand:

- meson
- cargo
- GTK >= 4.5.0
- libadwaita
- TDLib 1.7.10
- [Telegram API Credentials](https://my.telegram.org/) (optional, but recommended)

#### Instructions

```shell
meson . _build -Dtg_api_id=ID -Dtg_api_hash=HASH
ninja -C _build
sudo ninja -C _build install
```

## Acknowledgment

The general code architecture was heavily inspired by [fractal-next](https://gitlab.gnome.org/GNOME/fractal/-/tree/fractal-next).
