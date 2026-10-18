quad {
  z = pixel:
  z = z*z + pixel
  |z| <= 4
}
bad1 {
  z = pixel:
  z = z*+ pixel
  |z| <= 4
}
unkfn {
  z = pixel:
  z = foo(z) + pixel
  |z| <= 4
}
noinit {
  z = z*z + pixel
  |z| <= 4
}
undef {
  z = pixel:
  z = z*z + qq
  |z| <= 4
}
