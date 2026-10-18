two {
  z = 0, k = 180:
  k = 181:
  z = z + 1
  real(z) <= real(k)
}
none {
  z = z + 1
  real(z) <= 9
}
none2 {
  k = 7, z = z + 1
  real(z) <= real(k)
}
mid {
  z = 0, k = 183:
  z = z + 1
  real(z) <= real(k)
  z = z
}
mid2 {
  z = 0, k = 184:
  real(z) <= 3
  z = z + 1
  real(z) <= real(k)
}
asg {
  z = 0, k = 185:
  z = z + 1
}
asgk {
  z = 0, k = 186:
  z = z + 1, k = real(z) <= 9
}
asg0 {
  z = 0:
  z = z + 1, w = 0
}
emptyit {
  z = 0, k = 187:
}
emptyin {
  :
  z = z + 1
  real(z) <= 8
}
sameln {
  z = 0, k = 188: z = z + 1
  real(z) <= real(k)
}
space {
  z = 0 k = 189:
  z = z + 1
  real(z) <= real(k)
}
testinit {
  z = 0, k = 190, real(z) < 0:
  z = z + 1
  real(z) <= real(k)
}
colon2 {
  z = 0, k = 191:
  z = z + 1:
  real(z) <= real(k)
}
empty {
}
onlytest {
  real(z) <= 5
}
