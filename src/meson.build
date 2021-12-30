global_conf = configuration_data()
global_conf.set_quoted('APP_ID', application_id)
global_conf.set_quoted('PKGDATADIR', pkgdatadir)
global_conf.set_quoted('PROFILE', profile)
global_conf.set_quoted('VERSION', version + version_suffix)
global_conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
global_conf.set_quoted('LOCALEDIR', localedir)
global_conf.set('TG_API_ID', tg_api_id)
global_conf.set_quoted('TG_API_HASH', tg_api_hash)
config = configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: global_conf
)
# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.build_root() / 'src' / 'config.rs',
  meson.source_root() / 'src' / 'config.rs',
  check: true
)

rust_sources = files(
  'application.rs',
  'config.rs',
  'login.rs',
  'main.rs',
  'preferences_window.rs',
  'utils.rs',
  'window.rs',
  'session/avatar.rs',
  'session/basic_group.rs',
  'session/basic_group_list.rs',
  'session/chat_list.rs',
  'session/mod.rs',
  'session/supergroup.rs',
  'session/supergroup_list.rs',
  'session/user.rs',
  'session/user_list.rs',
  'session/chat/history.rs',
  'session/chat/item.rs',
  'session/chat/message.rs',
  'session/chat/mod.rs',
  'session/chat/sponsored_message.rs',
  'session/components/avatar.rs',
  'session/components/mod.rs',
  'session/content/chat_action_bar.rs',
  'session/content/chat_history.rs',
  'session/content/event_row.rs',
  'session/content/item_row.rs',
  'session/content/mod.rs',
  'session/content/user_dialog.rs',
  'session/content/message_row/mod.rs',
  'session/content/message_row/sticker.rs',
  'session/content/message_row/sticker_paintable.rs',
  'session/content/message_row/text.rs',
  'session/sidebar/avatar.rs',
  'session/sidebar/mod.rs',
  'session/sidebar/row.rs',
)

sources = [rust_sources, cargo_sources]

custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  depends: resources,
  command: [
    cargo_script,
    meson.build_root(),
    meson.source_root(),
    '@OUTPUT@',
    profile,
    meson.project_name(),
  ]
)
