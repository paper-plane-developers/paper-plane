# Telegrand

A GTK Telegram client built to be well integrated with the GNOME desktop environment.

<!-- <div align="center">
![Main Window](data/resources/screenshots/screenshot1.png "Main Window")
</div> -->

## Build Instructions

```shell
meson . _build --prefix=/usr/local
ninja -C _build
sudo ninja -C _build install
```
