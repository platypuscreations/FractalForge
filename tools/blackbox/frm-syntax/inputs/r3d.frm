ps(PI_SYM) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1)
  |z| <= 4
}
pn {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1)
  |z| <= 4
}
pxy(XYAXIS) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1)
  |z| <= 4
}
qs(PI_SYM) {
  z = sin(pixel) + (0.2,0.1):
  z = z*z + (0.3,0.4)
  |z| <= 4
}
qn {
  z = sin(pixel) + (0.2,0.1):
  z = z*z + (0.3,0.4)
  |z| <= 4
}
