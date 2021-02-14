tg_api_hash = get_option('tg_api_hash')
tg_api_id = get_option('tg_api_id')

config = configuration_data()
config.set_quoted('PKGDATADIR', pkgdatadir)
config.set_quoted('TG_API_HASH', tg_api_hash)
config.set('TG_API_ID', tg_api_id)

configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: config,
)

# Copy the config.rs output to the source directory.
run_command(
  'cp',
  join_paths(meson.build_root(), 'src', 'config.rs'),
  join_paths(meson.source_root(), 'src', 'config.rs'),
  check: true,
)

sources = files(
  'add_account_window.rs',
  'config.rs',
  'main.rs',
  'window.rs',
)

cargo_script = find_program(join_paths(meson.source_root(), 'build-aux/cargo.sh'))
custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    cargo_script,
    meson.build_root(),
    meson.source_root(),
    '@OUTPUT@',
    get_option('buildtype'),
    meson.project_name(),
  ]
)
