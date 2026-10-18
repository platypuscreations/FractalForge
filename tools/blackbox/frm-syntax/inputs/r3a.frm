za {
  n = 0, z = 3, k = 100 + 10 * real(z):
  n = n + 1
  real(n) <= real(k)
}
zb {
  n = 0, z = 0, z = 3, k = 100 + 10 * real(z):
  n = n + 1
  real(n) <= real(k)
}
zc {
  n = 0, w = 0, w = 3, k = 100 + 10 * real(w):
  n = n + 1
  real(n) <= real(k)
}
zd {
  n = 0, z = 5, k = 100 + 10 * real(z):
  n = n + 1
  real(n) <= real(k)
}
ze {
  n = 0, z = 3
  k = 100 + 10 * real(z):
  n = n + 1
  real(n) <= real(k)
}
zf {
  n = 0, z = (1,2), k = 100 + 10 * real(z) + 30 * imag(z):
  n = n + 1
  real(n) <= real(k)
}
zg {
  n = 0, z = pixel, k = 100 + 10 * real(z):
  n = n + 1
  real(n) <= real(k)
}
zh {
  n = 0, z = 3, w = z, k = 100 + 10 * real(w):
  n = n + 1
  real(n) <= real(k)
}
zi {
  n = 0, z = 2 + 1, k = 100 + 10 * real(z):
  n = n + 1
  real(n) <= real(k)
}
zj {
  n = 0, a = 3, z = a, k = 100 + 10 * real(z):
  n = n + 1
  real(n) <= real(k)
}
zk {
  n = 0, k = 100 + 10 * real(z):
  n = n + 1
  real(n) <= real(k)
}
zl {
  n = 0, z = 3, k = 100 + 10 * real(z) + 0 * z:
  n = n + 1
  real(n) <= real(k)
}
zm {
  n = 0, z = 3, k = 100:
  n = n + 1
  real(n) <= real(k) + 10 * real(z)
}
zn {
  n = 0, k = 100:
  n = n + 1
  z = 3
  real(n) <= real(k) + 10 * real(z)
}
