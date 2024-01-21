project('femto-containers',
        'c',
        license: 'MIT',
        default_options : ['werror=true'])

project_sources = []

incdir = include_directories('include')

subdir('src')

lib = library('fc', project_sources, include_directories : incdir) 

if get_option('enable-tests')
  subdir('tests')
endif
if get_option('enable-examples')
  subdir('examples')
endif

