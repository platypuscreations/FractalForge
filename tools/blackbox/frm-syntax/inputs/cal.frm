; SPDX-License-Identifier: GPL-3.0-or-later
; Calibration: the iteration count encodes a constant k.
k0 {
  z = 0, k = 0:
  z = z + 1
  real(z) <= real(k)
}
k1 {
  z = 0, k = 1:
  z = z + 1
  real(z) <= real(k)
}
k2 {
  z = 0, k = 2:
  z = z + 1
  real(z) <= real(k)
}
k3 {
  z = 0, k = 3:
  z = z + 1
  real(z) <= real(k)
}
k5 {
  z = 0, k = 5:
  z = z + 1
  real(z) <= real(k)
}
k10 {
  z = 0, k = 10:
  z = z + 1
  real(z) <= real(k)
}
k2p5 {
  z = 0, k = 2.5:
  z = z + 1
  real(z) <= real(k)
}
km1 {
  z = 0, k = -1:
  z = z + 1
  real(z) <= real(k)
}
k100 {
  z = 0, k = 100:
  z = z + 1
  real(z) <= real(k)
}
k200 {
  z = 0, k = 200:
  z = z + 1
  real(z) <= real(k)
}
k300 {
  z = 0, k = 300:
  z = z + 1
  real(z) <= real(k)
}
