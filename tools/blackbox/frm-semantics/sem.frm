; SPDX-License-Identifier: GPL-3.0-or-later
; Formula-semantics probes (our own). One entry per case in cases.txt.
quad {
  z = pixel:
  z = z*z + pixel
  |z| <= 4
}
startt {
  z = 3*pixel:
  z = pixel
  |z| <= 4
}
count {
  z = 0:
  z = z + 1
  real(z) < real(pixel)
}
tz {
  z = pixel:
  z = z
  z
}
tre {
  z = pixel:
  z = z
  real(z)
}
tim {
  z = pixel:
  z = z
  imag(z)
}
clt {
  z = pixel:
  z = z
  z < (10,20)
}
cle {
  z = pixel:
  z = z
  z <= (10,20)
}
cgt {
  z = pixel:
  z = z
  z > (10,20)
}
cge {
  z = pixel:
  z = z
  z >= (10,20)
}
ceq {
  z = pixel:
  z = z
  z == (10,20)
}
cne {
  z = pixel:
  z = z
  z != (10,20)
}
land {
  z = pixel:
  z = z
  real(z) < 0 && imag(z) < 0
}
lor {
  z = pixel:
  z = z
  real(z) < 0 || imag(z) < 0
}
lnot {
  z = pixel:
  z = z
  !(real(z) < 0)
}
modb {
  z = pixel:
  z = z
  |z| < 4
}
zun {
  c = pixel:
  z = z*z + c
  |z| <= 4
}
bogus {
  z = pixel:
  z = z*z + qqq(z)
  |z| <= 4
}
mz {
  v = z:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mp1 {
  v = p1:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mp2 {
  v = p2:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mp3 {
  v = p3:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mp4 {
  v = p4:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mp5 {
  v = p5:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mpi {
  v = pi:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
me {
  v = e:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mmaxit {
  v = maxit:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mscrnmax {
  v = scrnmax:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mscrnpix {
  v = scrnpix:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mwhitesq {
  v = whitesq:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mismand {
  v = ismand:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mcenter {
  v = center:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mmagxmag {
  v = magxmag:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mrotskew {
  v = rotskew:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mlastsqr {
  v = lastsqr:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mrand {
  v = rand:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mlit {
  v = (1.5,-2.25):
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mtrue {
  v = 1<2:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mfalse {
  v = 2<1:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mand {
  v = (1<2)&&(1<2):
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mor {
  v = (2<1)||(1<2):
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mdiv0 {
  v = (1,1)/0:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mpow00 {
  v = 0^0:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mpow0n {
  v = 0^2:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mpowneg {
  v = (-1)^0.5:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mpowneg2 {
  v = (-4)^(0.5,0):
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mcmplt {
  v = (3,1)<(2,5):
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mmod {
  v = |(3,4)|:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mxpow {
  v = (0,1)^(0,1):
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
mfn1 {
  v = fn1(1):
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
zv {
  c = pixel:
  z = z
  real(pixel) < real(z) && imag(pixel) < imag(z)
}
zzv {
  w = pixel:
  w = w*w + pixel
  z = (0.5,-1.5)*w
  |w| <= 4
}
symq {
  z = pixel:
  z = z*z + pixel + (0.2,0.3)
  |z| <= 4
}
sq_XAXIS(XAXIS) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3)
  |z| <= 4
}
sq_YAXIS(YAXIS) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3)
  |z| <= 4
}
sq_XYAXIS(XYAXIS) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3)
  |z| <= 4
}
sq_ORIGIN(ORIGIN) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3)
  |z| <= 4
}
sq_PI_SYM(PI_SYM) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3)
  |z| <= 4
}
sq_XAXIS_NOPARM(XAXIS_NOPARM) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3)
  |z| <= 4
}
sq_XAXIS_NOREAL(XAXIS_NOREAL) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3)
  |z| <= 4
}
sq_XAXIS_NOIMAG(XAXIS_NOIMAG) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3)
  |z| <= 4
}
sq_XYAXIS_NOPARM(XYAXIS_NOPARM) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3)
  |z| <= 4
}
sq_YAXIS_NOPARM(YAXIS_NOPARM) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3)
  |z| <= 4
}
sq_ORIGIN_NOPARM(ORIGIN_NOPARM) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3)
  |z| <= 4
}
sq_PI_SYM_NOPARM(PI_SYM_NOPARM) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3)
  |z| <= 4
}
sq_NOSYM(NOSYM) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3)
  |z| <= 4
}
sq_FOO(FOO) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3)
  |z| <= 4
}
f_sin {
  c = sin(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_cos {
  c = cos(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_cosxx {
  c = cosxx(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_tan {
  c = tan(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_cotan {
  c = cotan(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_cot {
  c = cot(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_sinh {
  c = sinh(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_cosh {
  c = cosh(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_tanh {
  c = tanh(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_cotanh {
  c = cotanh(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_coth {
  c = coth(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_asin {
  c = asin(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_acos {
  c = acos(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_atan {
  c = atan(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_asinh {
  c = asinh(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_acosh {
  c = acosh(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_atanh {
  c = atanh(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_exp {
  c = exp(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_log {
  c = log(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_sqr {
  c = sqr(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_sqrt {
  c = sqrt(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_abs {
  c = abs(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_cabs {
  c = cabs(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_conj {
  c = conj(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_real {
  c = real(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_imag {
  c = imag(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_flip {
  c = flip(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_ident {
  c = ident(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_zero {
  c = zero(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_one {
  c = one(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_recip {
  c = recip(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_floor {
  c = floor(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_ceil {
  c = ceil(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_trunc {
  c = trunc(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_round {
  c = round(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_srand {
  c = srand(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_fn1 {
  c = fn1(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_fn2 {
  c = fn2(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_fn3 {
  c = fn3(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
f_fn4 {
  c = fn4(pixel)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
o_neg {
  c = -pixel
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
o_div {
  c = 1/pixel
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
o_divm {
  c = (0.5,0.25)/pixel
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
o_sub {
  c = (0.25,0.5)-pixel
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
o_pw2 {
  c = pixel^2
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
o_pw3 {
  c = pixel^3
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
o_pwm1 {
  c = pixel^-1
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
o_pwh {
  c = pixel^0.5
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
o_pw25 {
  c = pixel^2.5
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
o_pwc {
  c = pixel^(0.5,0.5)
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
o_pwcb {
  c = (0.5,0.5)^pixel
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
o_pwpp {
  c = pixel^pixel
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
o_pw0 {
  c = pixel^0
  z = (0.3,0.2):
  z = z*z + c
  |z| <= 4
}
carry {
  t = k
  k = 0
  z = 0:
  k = k + 1
  z = z + 1
  real(z) < real(pixel) + (real(t) - 200)*(real(pixel) < 0.5)
}
k_div0 {
  v = (1,1)/0
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
k_div00 {
  v = (0,0)/0
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
k_pow02 {
  v = 0^2
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
k_pow00 {
  v = 0^0
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
k_pow0c {
  v = 0^(0.5,0.5)
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
k_log0 {
  v = log(0)
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
k_atanh1 {
  v = atanh(1)
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
k_atani {
  v = atan((0,1))
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
k_rand {
  v = rand
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
k_srand {
  v = srand(1)
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
k_huge {
  v = 1e300*1e300
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
k_one {
  v = (1,1)
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
k_zero {
  v = (0,0)
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
k_sqrtm1 {
  v = sqrt(-1)
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
k_recip0 {
  v = 1/(0,0)
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
k_cotan0 {
  v = cotan(0)
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
k_tan {
  v = tan((1.5707963267948966,0))
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
k_flog {
  v = log(-1)
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
tnan {
  v = (1,1)/0:
  z = z
  v
}
tnan2 {
  v = 0^2:
  z = z
  v
}
tnan3 {
  v = (1,1)/0:
  z = z
  |v| <= 4
}
tnan4 {
  v = (1,1)/0:
  z = z
  real(v) < 4
}
lsq1 {
  z = pixel:
  q = |z*0.5|
  real(lastsqr) < 1
}
lsq2 {
  z = pixel:
  z = z
  real(lastsqr) < 1
}
lsq3 {
  z = pixel:
  z = z*z + pixel
  |z| <= 4
  ;
}
lsq4 {
  z = pixel
  q = |z*0.5|
  v = lastsqr:
  z = z
  real(v) < 1
}
lsq5 {
  z = pixel:
  q = |z*0.5|
  z = z
  imag(lastsqr) < 0.5 && imag(lastsqr) > -0.5
}
ifz {
  z = pixel:
  if (z)
    w = 1
  else
    w = 0
  endif
  real(w)
}
ifim {
  z = pixel:
  if (imag(z))
    w = 1
  else
    w = 0
  endif
  real(w)
}
ifel {
  z = pixel:
  if (real(z) < -50)
    w = 1
  elseif (imag(z) < 0)
    w = 0
  elseif (real(z) < 50)
    w = 1
  else
    w = 0
  endif
  real(w)
}
apix {
  pixel = 2*pixel
  z = pixel:
  z = z*z + pixel
  |z| <= 4
}
apix2 {
  pixel = pixel + 0.01
  z = 0:
  z = pixel
  real(pixel) < 0.5 + 0.01*0
}
ap1 {
  p1 = p1 + (0.001,0):
  z = z
  real(p1) < 50
}
api {
  pi = 3
  v = pi:
  z = pixel
  real(pixel) < real(v)
}
ae {
  e = 2
  v = e:
  z = pixel
  real(pixel) < real(v)
}
apx2 {
  z = pixel:
  pixel = pixel + 1
  z = z
  real(pixel) < 3
}
rnd1 {
  v = rand:
  z = pixel
  real(pixel) < real(v)
}
rnd2 {
  v = rand:
  z = pixel
  imag(pixel) < imag(v)
}
fnx {
  z = pixel:
  z = fn1(z)*fn2(z) + pixel
  |z| <= 4
}
g_acos {
  c = acos(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_acosh {
  c = acosh(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_asin {
  c = asin(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_asinh {
  c = asinh(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_atan {
  c = atan(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_atanh {
  c = atanh(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_log {
  c = log(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_sqrt {
  c = sqrt(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_tan {
  c = tan(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_cotan {
  c = cotan(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_cotanh {
  c = cotanh(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_tanh {
  c = tanh(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_exp {
  c = exp(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_cosxx {
  c = cosxx(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_round {
  c = round(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_floor {
  c = floor(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_ceil {
  c = ceil(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_trunc {
  c = trunc(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_cabs {
  c = cabs(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_abs {
  c = abs(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_flip {
  c = flip(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_imag {
  c = imag(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_real {
  c = real(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
g_conj {
  c = conj(pixel)
  z = (-0.1,0.35):
  z = z*z + 0.5*c
  |z| <= 4
}
sp_XAXIS_NOPARM(XAXIS_NOPARM) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3) + p1 + p2
  |z| <= 4
}
sp_XAXIS_NOREAL(XAXIS_NOREAL) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3) + p1 + p2
  |z| <= 4
}
sp_XAXIS_NOIMAG(XAXIS_NOIMAG) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3) + p1 + p2
  |z| <= 4
}
sp_XYAXIS_NOPARM(XYAXIS_NOPARM) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3) + p1 + p2
  |z| <= 4
}
sp_YAXIS_NOPARM(YAXIS_NOPARM) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3) + p1 + p2
  |z| <= 4
}
sp_ORIGIN_NOPARM(ORIGIN_NOPARM) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3) + p1 + p2
  |z| <= 4
}
sp_PI_SYM_NOPARM(PI_SYM_NOPARM) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3) + p1 + p2
  |z| <= 4
}
sp_XAXIS(XAXIS) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3) + p1 + p2
  |z| <= 4
}
sp_ORIGIN(ORIGIN) {
  z = pixel:
  z = z*z + pixel + (0.2,0.3) + p1 + p2
  |z| <= 4
}
sp_NONE {
  z = pixel:
  z = z*z + pixel + (0.2,0.3) + p1 + p2
  |z| <= 4
}
n_atanh1 {
  v = atanh(1):
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
n_atanhm1 {
  v = atanh(-1):
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
n_atani {
  v = atan((0,1)):
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
n_atanmi {
  v = atan((0,-1)):
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
n_lrand {
  v = log(rand):
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
n_srand {
  v = srand(pixel*0+1):
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
n_and2 {
  v = (2,0)&&(0,5):
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
n_or2 {
  v = (0,3)||(0,0):
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
n_and3 {
  v = (0.5,0)&&(-2,0):
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
n_pi {
  v = pi:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
n_e {
  v = e:
  z = pixel
  real(pixel) < real(v) && imag(pixel) < imag(v)
}
q_sqrtm1 {
  v = sqrt(-1)
  t = 1 + (real(v) == 0) + 2*(imag(v) == 0) + 4*(real(v) < 0) + 8*(imag(v) < 0) + 16*(real(v) == real(v)) + 32*(imag(v) == imag(v)) + 64*(real(v) < 1e-10 && real(v) > -1e-10) + 128*(imag(v) < 1e-10 && imag(v) > -1e-10)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
q_sqrtm4 {
  v = sqrt(-4)
  t = 1 + (real(v) == 0) + 2*(imag(v) == 0) + 4*(real(v) < 0) + 8*(imag(v) < 0) + 16*(real(v) == real(v)) + 32*(imag(v) == imag(v)) + 64*(real(v) < 1e-10 && real(v) > -1e-10) + 128*(imag(v) < 1e-10 && imag(v) > -1e-10)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
q_powh {
  v = (-1)^0.5
  t = 1 + (real(v) == 0) + 2*(imag(v) == 0) + 4*(real(v) < 0) + 8*(imag(v) < 0) + 16*(real(v) == real(v)) + 32*(imag(v) == imag(v)) + 64*(real(v) < 1e-10 && real(v) > -1e-10) + 128*(imag(v) < 1e-10 && imag(v) > -1e-10)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
q_pm22 {
  v = (-2)^2
  t = 1 + (real(v) == 0) + 2*(imag(v) == 0) + 4*(real(v) < 0) + 8*(imag(v) < 0) + 16*(real(v) == real(v)) + 32*(imag(v) == imag(v)) + 64*(real(v) < 1e-10 && real(v) > -1e-10) + 128*(imag(v) < 1e-10 && imag(v) > -1e-10)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
q_pm23 {
  v = (-2)^3
  t = 1 + (real(v) == 0) + 2*(imag(v) == 0) + 4*(real(v) < 0) + 8*(imag(v) < 0) + 16*(real(v) == real(v)) + 32*(imag(v) == imag(v)) + 64*(real(v) < 1e-10 && real(v) > -1e-10) + 128*(imag(v) < 1e-10 && imag(v) > -1e-10)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
q_pi2 {
  v = (0,1)^2
  t = 1 + (real(v) == 0) + 2*(imag(v) == 0) + 4*(real(v) < 0) + 8*(imag(v) < 0) + 16*(real(v) == real(v)) + 32*(imag(v) == imag(v)) + 64*(real(v) < 1e-10 && real(v) > -1e-10) + 128*(imag(v) < 1e-10 && imag(v) > -1e-10)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
q_ii {
  v = (0,1)*(0,1)
  t = 1 + (real(v) == 0) + 2*(imag(v) == 0) + 4*(real(v) < 0) + 8*(imag(v) < 0) + 16*(real(v) == real(v)) + 32*(imag(v) == imag(v)) + 64*(real(v) < 1e-10 && real(v) > -1e-10) + 128*(imag(v) < 1e-10 && imag(v) > -1e-10)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
q_sqri {
  v = sqr((0,1))
  t = 1 + (real(v) == 0) + 2*(imag(v) == 0) + 4*(real(v) < 0) + 8*(imag(v) < 0) + 16*(real(v) == real(v)) + 32*(imag(v) == imag(v)) + 64*(real(v) < 1e-10 && real(v) > -1e-10) + 128*(imag(v) < 1e-10 && imag(v) > -1e-10)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
q_p2h {
  v = (2,0)^0.5
  t = 1 + (real(v) == 0) + 2*(imag(v) == 0) + 4*(real(v) < 0) + 8*(imag(v) < 0) + 16*(real(v) == real(v)) + 32*(imag(v) == imag(v)) + 64*(real(v) < 1e-10 && real(v) > -1e-10) + 128*(imag(v) < 1e-10 && imag(v) > -1e-10)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
q_p22 {
  v = (2,0)^2
  t = 1 + (real(v) == 0) + 2*(imag(v) == 0) + 4*(real(v) < 0) + 8*(imag(v) < 0) + 16*(real(v) == real(v)) + 32*(imag(v) == imag(v)) + 64*(real(v) < 1e-10 && real(v) > -1e-10) + 128*(imag(v) < 1e-10 && imag(v) > -1e-10)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
q_pm2c {
  v = (-2,0)^(2,0)
  t = 1 + (real(v) == 0) + 2*(imag(v) == 0) + 4*(real(v) < 0) + 8*(imag(v) < 0) + 16*(real(v) == real(v)) + 32*(imag(v) == imag(v)) + 64*(real(v) < 1e-10 && real(v) > -1e-10) + 128*(imag(v) < 1e-10 && imag(v) > -1e-10)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
q_div {
  v = (1,1)/(1,1)
  t = 1 + (real(v) == 0) + 2*(imag(v) == 0) + 4*(real(v) < 0) + 8*(imag(v) < 0) + 16*(real(v) == real(v)) + 32*(imag(v) == imag(v)) + 64*(real(v) < 1e-10 && real(v) > -1e-10) + 128*(imag(v) < 1e-10 && imag(v) > -1e-10)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
q_logm1 {
  v = log(-1)
  t = 1 + (real(v) == 0) + 2*(imag(v) == 0) + 4*(real(v) < 0) + 8*(imag(v) < 0) + 16*(real(v) == real(v)) + 32*(imag(v) == imag(v)) + 64*(real(v) < 1e-10 && real(v) > -1e-10) + 128*(imag(v) < 1e-10 && imag(v) > -1e-10)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
q_expipi {
  v = exp((0,3.141592653589793))
  t = 1 + (real(v) == 0) + 2*(imag(v) == 0) + 4*(real(v) < 0) + 8*(imag(v) < 0) + 16*(real(v) == real(v)) + 32*(imag(v) == imag(v)) + 64*(real(v) < 1e-10 && real(v) > -1e-10) + 128*(imag(v) < 1e-10 && imag(v) > -1e-10)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
c_rotr {
  v = real(rotskew) + 100:
  z = z + 1
  real(z) < real(v)
}
c_roti {
  v = imag(rotskew) + 100:
  z = z + 1
  real(z) < real(v)
}
c_magr {
  v = real(magxmag)*100 + 100:
  z = z + 1
  real(z) < real(v)
}
c_magi {
  v = imag(magxmag)*100 + 100:
  z = z + 1
  real(z) < real(v)
}
c_cenr {
  v = real(center)*100 + 100:
  z = z + 1
  real(z) < real(v)
}
c_ceni {
  v = imag(center)*100 + 100:
  z = z + 1
  real(z) < real(v)
}
c_scrr {
  v = real(scrnmax) + 100:
  z = z + 1
  real(z) < real(v)
}
c_scri {
  v = imag(scrnmax) + 100:
  z = z + 1
  real(z) < real(v)
}
c_maxit {
  v = real(maxit) + 100:
  z = z + 1
  real(z) < real(v)
}
c_pixr {
  v = real(scrnpix) + 100:
  z = z + 1
  real(z) < real(v)
}
nc1 {
  z = pixel:
  z = z*z + pixel
}
nc2 {
  z = pixel:
  |z| <= 4
  z = z*z + pixel
}
nc4 {
  z = pixel:
  z = z*z + pixel
  |z| <= 4
  w = 1
}
nc5 {
  z = pixel:
  z = z*z + pixel
  |z| <= 4
  w = 0
}
lin {
  z = 0:
  z = z + 1
  real(z) < (real(pixel) + 5)*20 + (imag(pixel) + 3)*3
}
linpi(PI_SYM) {
  z = 0:
  z = z + 1
  real(z) < (real(pixel) + 5)*20 + (imag(pixel) + 3)*3
}
linx(XAXIS) {
  z = 0:
  z = z + 1
  real(z) < (real(pixel) + 5)*20 + (imag(pixel) + 3)*3
}
hsh_NONE {
  x = real(pixel)
  y = imag(pixel)
  t = 100 + 60*sin(2.3*x + 0.4)*cos(1.7*y - 0.3) + 40*sin(x*y*1.3 + 0.5)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
hsh_PI_SYM(PI_SYM) {
  x = real(pixel)
  y = imag(pixel)
  t = 100 + 60*sin(2.3*x + 0.4)*cos(1.7*y - 0.3) + 40*sin(x*y*1.3 + 0.5)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
hsh_ORIGIN(ORIGIN) {
  x = real(pixel)
  y = imag(pixel)
  t = 100 + 60*sin(2.3*x + 0.4)*cos(1.7*y - 0.3) + 40*sin(x*y*1.3 + 0.5)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
hsh_XYAXIS(XYAXIS) {
  x = real(pixel)
  y = imag(pixel)
  t = 100 + 60*sin(2.3*x + 0.4)*cos(1.7*y - 0.3) + 40*sin(x*y*1.3 + 0.5)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
er1 {
  z = pixel
  w = (1,1)/0:
  z = z*z + pixel
  |z| <= 4
}
er2 {
  z = pixel:
  w = (1,1)/0
  z = z*z + pixel
  |z| <= 4
}
er3 {
  z = pixel
  w = log(0):
  z = z*z + pixel
  |z| <= 4
}
er4 {
  z = pixel
  w = 0^2:
  z = z*z + pixel
  |z| <= 4
}
er5 {
  v = (1,1)/(0,0)
  v = (2,0)
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
er6 {
  v = 1e300*1e300 - 1e300*1e300
  t = 1 + (real(v) == real(v)) + 2*(real(v) > 1e300) + 4*(real(v) < -1e300) + 8*(real(v) == 0) + 16*(imag(v) == imag(v)) + 32*(imag(v) > 1e300) + 64*(imag(v) < -1e300) + 128*(imag(v) == 0)
  z = 0:
  z = z + 1
  real(z) < real(t)
}
er7 {
  v = 1e300*1e300 - 1e300*1e300:
  z = z
  v
}
er8 {
  z = pixel
  w = 1/real(pixel):
  z = z*z + pixel
  |z| <= 4
}
er9 {
  z = pixel
  k = 0:
  k = k + 1
  w = (1,1)/(real(k) - 3)
  z = z*z + pixel
  |z| <= 1000000
}
