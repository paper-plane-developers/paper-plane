# Telegrand

A GTK4 telegram client built to be well integrated with the GNOME desktop environment.

## What can it do?

- Authentication (without 2FA)
- Show user dialogs
- Notify for new messages
- Send text messages

## Planned features (short run)

- Show messages list
- Multilanguage support
- Flatpak support

## Planned features (long run)

- Show stickers in chat
- Send stickers
- Download media
- Send media

## How to build?

Before building you need to obtain your own telegram api key and hash. You can obtain them [here](https://my.telegram.org/).

Then you need to have gtk4 and libadwaita installed on your system, use your package manager or build this dependencies from source.

Now you can configure and then build the project:

```meson
meson _build -Dtg_api_id=ID -Dtg_api_hash=HASH
ninja -C _build install
```
