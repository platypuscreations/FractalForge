; SPDX-License-Identifier: GPL-3.0-or-later
; Symmetry qualifier tests (gen-sym.py)
y00 {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
y01(XAXIS) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
y02(YAXIS) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
y03(XYAXIS) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
y04(ORIGIN) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
y05(PI_SYM) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
y06(XAXIS_NOPARM) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
y07(XAXIS_NOREAL) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
y08(XAXIS_NOIMAG) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
y09(YAXIS_NOPARM) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
y10(XYAXIS_NOPARM) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
y11(ORIGIN_NOPARM) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
y12(PI_SYM_NOPARM) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
y13(NOSYM) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
y14(BOGUS) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
y15(xaxis) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
y16(NONE) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
y17(X AXIS) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
y18(xaxis junk) {
  z = pixel:
  z = z*z + (0.3,0.4)*pixel + (0.1,0.1) + 0.01*p1
  |z| <= 4
}
