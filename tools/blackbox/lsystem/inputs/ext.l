prot1 {
  angle 4
  axiom FC12F
  1=F
}
prot2 {
  angle 4
  axiom F<12F
  2=F
}
prot3 {
  angle 4
  axiom D/90D
  9=+D
}
prot4 {
  angle 4
  axiom F@2F+F
  2=FF
}
prot5 {
  angle 4
  axiom F@Q4F+F
  Q=FF
  4=FF
}
prot6 {
  angle 4
  axiom FC1.5F+F
  5=FF
}
ruled {
  angle 4
  axiom D
  D=D+D
}
rulem {
  angle 4
  axiom DMD
  M=G+G
}
plant {
  angle 16
  axiom F
  F=C2FF-[C3-F+F+F]+[C4+F-F-F]
}
arrow {
  angle 6
  axiom A
  A=B-A-B
  B=A+B+A
  A=
  B=
}
arrowf {
  angle 6
  axiom F
  F=G-F-G
  G=F+G+F
}
dragon {
  angle 4
  axiom FX
  X=X+YF+
  Y=-FX-Y
}
levy {
  angle 8
  axiom F
  F=+F--F+
}
bush {
  angle 18
  axiom ++++F
  F=FF-[-F+F+F]+[+F-F-F]
}
degmix {
  angle 4
  axiom F
  F=D\30F/60F\30D+F
}
colcyc {
  angle 6
  axiom F
  F=<1F+F--F+F
}
