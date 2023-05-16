<h1 align="center">
  <img src="data/icons/app.drey.PaperPlane.svg" alt="Paper Plane" width="192" height="192"/>
  <br>
  Paper Plane
</h1>

<p align="center"><strong>Chat over Telegram on a modern and elegant client</strong></p>

<p align="center">
  <a href="https://hosted.weblate.org/engage/paper-plane/">
    <img src="https://hosted.weblate.org/widgets/paper-plane/-/main/svg-badge.svg" alt="Translation status" />
  </a>
  <a href="https://github.com/paper-plane-developers/paper-plane/actions/workflows/ci.yml">
    <img src="https://github.com/paper-plane-developers/paper-plane/actions/workflows/ci.yml/badge.svg" alt="CI status"/>
  </a>
  <a href="https://t.me/paperplanechat">
    <img src="https://img.shields.io/static/v1?label=Chat&message=@paperplanechat&color=blue&logo=telegram" alt="Telegram group">
  </a>
</p>

<br>

<p align="center">
  <img width=600 src="data/resources/screenshots/screenshot1.png" alt="Screenshot"/>
</p>

Paper Plane is an alternative Telegram client.
It uses libadwaita for its user interface and strives to meet the design principles of the GNOME desktop.

Paper Plane is still under development and not yet feature-complete.
However, the following things are already working:

- The use of multiple accounts at the same time.
- Viewing text messages, images, stickers and files.
- Sending text messages and images.
- Replying to messages.
- Searching for groups and persons.

## Installation Instructions

Paper Plane is an in-development project and it isn't considered stable software yet. Also, the included API credentials are very limited and, in some cases, your account may end up banned by Telegram (check the `Telegram API Credentials` section below). You can avoid that by using a custom built version of Paper Plane with provided API credentials via meson options, like [this AUR package](https://aur.archlinux.org/packages/paper-plane-git) which you may prefer using if you use Arch Linux. But, if you still feel brave enough, there's a CI that automatically generates the latest flatpak build with the test API credentials: just download the [latest artifact](https://nightly.link/paper-plane-developers/paper-plane/workflows/ci/main) and install it locally using `flatpak install paper-plane.flatpak`.

## Telegram API Credentials

Telegram requires custom clients to set some credentials for using their API. Paper Plane doesn't provide official API credentials, so the packagers are expected to set their own credentials for distributing the app, obtainable at https://my.telegram.org/. However, Paper Plane includes the Telegram's test credentials by default, which are very limited, but usable (especially for development). However, it's known that Telegram sometimes decides to ban accounts that use such credentials (especially newer accounts). For that reason, it's suggested to use your own API credentials, which can be set by using meson options (see the `Build Instructions` section below).

## Build Instructions

### Gnome Builder

Using Gnome Builder is the easiest way to get the app built without even using the terminal: just clone the repository and press the big "Run" button at the top and it will automatically build all the required dependencies together with the app.

### Meson

#### Prerequisites

The following packages are required to build Paper Plane:

- meson
- cargo
- GTK >= 4.10 (with the patch included in the build-aux directory)
- libadwaita >= 1.4
- [TDLib 1.8.14](https://github.com/tdlib/td/commit/8517026415e75a8eec567774072cbbbbb52376c1)
- [Telegram API Credentials](https://my.telegram.org/) (optional, but recommended)

Additionally, Paper Plane requires the following GStreamer plugins installed in your system to correctly show all media files:

- gstreamer-libav
- gstreamer-plugins-good

#### Instructions

```shell
meson . _build -Dtg_api_id=ID -Dtg_api_hash=HASH
ninja -C _build
sudo ninja -C _build install
```

## Acknowledgment

The general code architecture was heavily inspired by [Fractal](https://gitlab.gnome.org/GNOME/fractal).

Also, some logic is inspired by [Telegram X](https://github.com/TGX-Android/Telegram-X), which helps to understand how to use some TDLib features correctly and to their fullest potential.
