config = configuration_data()
config.set_quoted('APP_ID', application_id)
config.set_quoted('PKGDATADIR', pkgdatadir)
config.set_quoted('TG_API_HASH', tg_api_hash)
config.set('TG_API_ID', tg_api_id)
config.set_quoted('VERSION', version + version_suffix)
configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: config,
)
# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.build_root() / 'src' / 'config.rs',
  meson.source_root() / 'src' / 'config.rs',
  check: true,
)

sources = files(
  'add_account_window.rs',
  'application.rs',
  'chat_page.rs',
  'config.rs',
  'dialog_data.rs',
  'dialog_model.rs',
  'dialog_row.rs',
  'main.rs',
  'message_row.rs',
  'telegram.rs',
  'window.rs',
)

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
