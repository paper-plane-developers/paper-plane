project(
  'telegrand',
  'rust',
  version: '0.1.0',
  meson_version: '>= 0.50',
)

i18n = import('i18n')
gnome = import('gnome')

base_id = 'com.github.melix99.telegrand'

dependency('glib-2.0', version: '>= 2.66')
dependency('gio-2.0', version: '>= 2.66')
dependency('gtk4', version: '>= 4.4.0')
dependency('libadwaita-1', version: '>= 1.0.0')
dependency('tdjson', version: '>= 1.7.0')

glib_compile_resources = find_program('glib-compile-resources', required: true)
glib_compile_schemas = find_program('glib-compile-schemas', required: true)
desktop_file_validate = find_program('desktop-file-validate', required: false)
appstream_util = find_program('appstream-util', required: false)
cargo = find_program('cargo', required: true)
cargo_script = find_program('build-aux/cargo.sh')

version = meson.project_version()

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
localedir = prefix / get_option('localedir')

datadir = prefix / get_option('datadir')
pkgdatadir = datadir / meson.project_name()
iconsdir = datadir / 'icons'
podir = meson.source_root() / 'po'
gettext_package = meson.project_name()

tg_api_id = get_option('tg_api_id')
tg_api_hash = get_option('tg_api_hash')

if get_option('profile') == 'development'
  profile = 'Devel'
  vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD').stdout().strip()
  if vcs_tag == ''
    version_suffix = '-devel'
  else
    version_suffix = '-@0@'.format(vcs_tag)
  endif
  application_id = '@0@.@1@'.format(base_id, profile)
else
  profile = ''
  version_suffix = ''
  application_id = base_id
endif

meson.add_dist_script(
  'build-aux/dist-vendor.sh',
  meson.build_root() / 'meson-dist' / meson.project_name() + '-' + version,
  meson.source_root()
)

cargo_sources = files(
  'Cargo.lock',
  'Cargo.toml',
)

subdir('data')
subdir('po')
subdir('src')

meson.add_install_script('build-aux/meson_post_install.py')
