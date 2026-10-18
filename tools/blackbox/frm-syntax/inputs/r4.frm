; SPDX-License-Identifier: GPL-3.0-or-later
ie1 {
  z = 0, k = 5:
  z = z + 1
  if (1)
  real(z) <= real(k)
}
ie2 {
  z = 0, k = 5:
  z = z + 1
  endif
  real(z) <= real(k)
}
ie3 {
  z = 0, k = 5:
  z = z + 1
  if (1)
  else
  elseif (1)
  endif
  real(z) <= real(k)
}
ie4 {
  z = 0, k = 5:
  z = z +* 0
  z = z +* 1
  z = z +* 2
  z = z +* 3
  z = z +* 4
  z = z +* 5
  z = z +* 6
  z = z +* 7
  z = z +* 8
  z = z +* 9
  z = z +* 10
  z = z +* 11
  real(z) <= real(k)
}
ie5 {
  z = 0, k = 5:
  z = z + 1
  real(z) <= real(k)
  if (0)
  endif
}
ie6 {
  z = 0, k = 5
  if (1)
  k = 6
  endif
  :
  z = z + 1
  real(z) <= real(k)
}
