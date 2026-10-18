c15 {
  ; } closing brace in a comment
  z = 0, k = 150:
  z = z + 1
  real(z) <= real(k)
}
c15b {
  z = 0, k = 151: ; {
  z = z + 1
  real(z) <= real(k)
}
c15c {
  z = 0, k = 152:
  { z = z + 1 }
  real(z) <= real(k)
}
