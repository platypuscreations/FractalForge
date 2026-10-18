it1 {
  z = 0, k = 120:
  z = z + 1
  if (real(z) > real(k))
    t = 0
  else
    t = 1
  endif
}
it2 {
  z = 0, k = 121:
  z = z + 1
  if (1)
    real(z) <= real(k)
  endif
}
it3 {
  z = 0, k = 122:
  z = z + 1
  w = real(z) <= real(k)
  if (0)
    w = 1
  endif
}
it4 {
  z = 0, k = 123
  if (1)
  :
  endif
  z = z + 1
  real(z) <= real(k)
}
it5 {
  z = 0, k = 124:
  z = z + 1
  if (real(z) <= real(k))
    t = 5
  endif
}
it6 {
  z = 0, k = 125:
  if (1)
    z = z + 1
  endif
  real(z) <= real(k)
}
it7 {
  z = 0, k = 126:
  z = z + 1
  if (real(z) > real(k))
    t = 0
  endif
}
