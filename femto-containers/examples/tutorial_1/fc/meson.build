project('tutorial_1_fc',
        'c')

sources = [files('increment.c')]
clang_bin = find_program('clang')
llc_bin = find_program('llc')

clang_flags = [
  '-nostdinc',
  '-Wno-unused-value',
  '-Wno-pointer-sign',
  '-Wno-compare-distinct-pointer-types',
  '-Wno-gnu-variable-sized-type-not-at-end',
  '-Wno-address-of-packed-member',
  '-Wno-tautological-compare',
  '-Wno-unknown-warning-option',
  '-g3',
  '-Os',
  '--emit-llvm',
  ]

llc_flags = [
  '-march=bpf',
  '-mcpu=v2',
  '-filetype=obj',
  ]

llvm_gen = generator(clang_bin,
  arguments: [clang_flags, '-c @INPUT@', '-o @OUTPUT@'],
  outfile: '@BASENAME@.llvm',
  )

llvm_bins = llvm_gen.process(sources)


cust
