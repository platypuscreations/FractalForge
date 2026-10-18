; one

; three
err {
  ; comment

  z = 0, k = 1:
  z = z +* 1
  real(z) <= real(k)
}
err2 { z = 0, k = 1: z = z +* 1, real(z) <= real(k) }
err3 {
  z = 0, k = 1 +* 2:
  z = z +* 1
  real(z) <= real(k) +*
}
