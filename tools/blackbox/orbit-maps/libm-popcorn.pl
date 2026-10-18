#!/usr/bin/perl
# SPDX-License-Identifier: GPL-3.0-or-later
# Our hypothesised popcorn orbit renderer (docs/specs/orbit-maps.md §6)
# evaluated with the container's glibc libm. Writes W*H colour bytes.
#   perl libm-popcorn.pl W H XMIN XMAX YMIN YMAX H P3 MAXITER F1 F2 F3 F4
# F* are sin, cos, tan, sqr, sinh, cosh or ident. Own code.
use strict; use warnings;
use POSIX qw(tan sinh cosh);
my ($W, $Hh, $xmin, $xmax, $ymin, $ymax, $h, $t3, $M, @fn) = @ARGV;
my %f = (sin => sub { sin($_[0]) }, cos => sub { cos($_[0]) }, tan => sub { tan($_[0]) },
         sqr => sub { $_[0] * $_[0] }, sinh => sub { my $v = sinh($_[0]); $v == 9**9**9 || $v == -9**9**9 ? 1 : $v },
         cosh => sub { my $v = cosh($_[0]); $v == 9**9**9 ? 1 : $v }, ident => sub { $_[0] });
my ($f1, $f2, $f3, $f4) = map { $f{$_} } @fn;
my $sx = ($W - 1) / ($xmax - $xmin); my $sy = ($Hh - 1) / ($ymax - $ymin);
my @img = (0) x ($W * $Hh);
for my $r (0 .. $Hh - 1) {
  my $y0 = $ymax - $r * ($ymax - $ymin) / ($Hh - 1);
  for my $c (0 .. $W - 1) {
    my $x = $xmin + $c * ($xmax - $xmin) / ($W - 1); my $y = $y0;
    for (my $k = 0; $k < $M - 1; $k++) {
      my $nx = $x - $h * $f1->($y + $f2->($t3 * $y));
      my $ny = $y - $h * $f3->($x + $f4->($t3 * $x));
      ($x, $y) = ($nx, $ny);
      my $cx = ($x - $xmin) * $sx; my $cy = ($ymax - $y) * $sy;
      if ($cx > -1 && $cx < $W && $cy > -1 && $cy < $Hh) {
        my $ci = int($cx); my $ri = int($cy);
        $img[$ri * $W + $ci] = ($r + 1) % 256 if $ci < $W && $ri < $Hh;
      }
      last if $x * $x + $y * $y > 4;
    }
  }
}
binmode STDOUT; print pack('C*', @img);
