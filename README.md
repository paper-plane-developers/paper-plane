<h1 align="center">
  Telegrand
</h1>

<p align="center">
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

A Telegram client built to be well integrated with the GNOME desktop environment.

## Build Instructions

```shell
meson . _build --prefix=/usr/local
ninja -C _build
sudo ninja -C _build install
```
