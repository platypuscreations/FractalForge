um1 {
  z = 0, a = 3, j = -(2 + a), k = 100 + 10 * real(j):
  z = z + 1
  real(z) <= real(k)
}
um2 {
  z = 0, a = 3, j = -(a + 2), k = 100 + 10 * real(j):
  z = z + 1
  real(z) <= real(k)
}
um3 {
  z = 0, j = -(2), k = 100 + 10 * real(j):
  z = z + 1
  real(z) <= real(k)
}
um4 {
  z = 0, j = 0 ^ 2, k = 100 + 10 * real(j):
  z = z + 1
  real(z) <= real(k)
}
um5 {
  z = 0, j = (0) ^ 2, k = 100 + 10 * real(j):
  z = z + 1
  real(z) <= real(k)
}
um6 {
  z = 0, a = 0, j = a ^ 2, k = 100 + 10 * real(j):
  z = z + 1
  real(z) <= real(k)
}
um7 {
  z = 0, j = -(.5 + 1), k = 100 + 10 * real(j):
  z = z + 1
  real(z) <= real(k)
}
um8 {
  z = 0, j = -(1e1 + 1), k = 100 + 10 * real(j):
  z = z + 1
  real(z) <= real(k)
}
um9 {
  z = 0, j = -((1, 2)) + (0, 0), k = 100 + 10 * real(j):
  z = z + 1
  real(z) <= real(k)
}
um10 {
  z = 0, a = 2, j = -(a) * 3, k = 100 + 10 * real(j):
  z = z + 1
  real(z) <= real(k)
}
um11 {
  z = 0, k = 100:
  z = z + 1
  j = -(2)
  real(z) <= real(k) + 10 * real(j)
}
