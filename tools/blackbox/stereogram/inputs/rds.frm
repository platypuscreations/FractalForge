rdsflat {
  k = 0, t = real(p1):
  k = k + 1
  k < t
}
rdsbox {
  k = 0, x = real(scrnpix), y = imag(scrnpix)
  t = real(p1)
  if (x >= real(p2) && x < imag(p2) && y >= real(p3) && y < imag(p3))
    t = imag(p1)
  endif
  :
  k = k + 1
  k < t
}
rdsrows {
  k = 0, t = imag(scrnpix):
  k = k + 1
  k < t
}
