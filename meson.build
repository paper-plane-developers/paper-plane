project(
  'telegrand',
  'rust',
  version: '0.1.0',
  license: 'GPLv3',
  meson_version : '>= 0.50',
)

i18n = import('i18n')
gnome = import('gnome')

dependency('glib-2.0', version: '>= 2.66')
dependency('gio-2.0', version: '>= 2.66')
dependency('gtk4', version: '>= 4.0.0')
dependency('libadwaita-1', version: '>= 1.0.0')

desktop_file_validate = find_program('desktop-file-validate', required: false)
cargo_script = find_program('build-aux/cargo.sh')

if get_option('profile') == 'development'
  profile = 'Devel'
else
  profile = ''
endif

application_id = 'com.github.melix99.telegrand@0@'.format(profile)
tg_api_hash = get_option('tg_api_hash')
tg_api_id = get_option('tg_api_id')

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')

datadir = prefix / get_option('datadir')
pkgdatadir = datadir / meson.project_name()
podir = meson.source_root() / 'po'

subdir('data')
subdir('src')
