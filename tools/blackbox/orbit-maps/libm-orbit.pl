#!/usr/bin/perl
# SPDX-License-Identifier: GPL-3.0-or-later
# Iterates one of our hypothesised orbit maps with the container's C library
# maths (glibc libm via Perl's sin/cos/atan2/log/sqrt, IEEE doubles) and
# writes the orbit as raw little-endian doubles x0 y0 x1 y1 ... to stdout
# (the start point first, then N iterates).
# Own code, used only to see whether XFractint's orbits use the same libm.
#   docker run --rm -v DIR:/w --entrypoint perl fractalforge-blackbox \
#     /w/libm-orbit.pl TYPE N X0 Y0 P1 P2 P3 ... > orbit.bin
# TYPE may also be 'expr:NX:NY' with Perl expressions in $x, $y, $p[0..9].
use strict; use warnings;
my ($t, $n, $x, $y, @p) = @ARGV;
my ($ex, $ey);
if ($t =~ /^expr:(.*):(.*)$/) { ($ex, $ey) = ($1, $2); $t = 'expr'; }
push @p, 0 while @p < 10;
sub sgn { $_[0] > 0 ? 1 : ($_[0] < 0 ? -1 : 0) }
sub atan { atan2($_[0], 1) }
sub sqr { $_[0] * $_[0] }   # not **2: Perl's pow() is not always x*x
binmode STDOUT;
my $buf = pack('d<d<', $x, $y);
for (my $k = 0; $k < $n; $k++) {
  my ($nx, $ny);
  if ($t eq 'expr') { $nx = eval $ex; $ny = eval $ey; die $@ if $@; }
  elsif ($t eq 'martin') { $nx = $y - sin($x); $ny = $p[0] - $x; }
  elsif ($t eq 'henon') { $nx = 1 + $y - $p[0] * $x * $x; $ny = $p[1] * $x; }
  elsif ($t eq 'chip') {
    $nx = $y - sgn($x) * cos(sqr(log(abs($p[1] * $x - $p[2])))) * atan(sqr(log(abs($p[2] * $x - $p[1]))));
    $ny = $p[0] - $x; }
  elsif ($t eq 'quadruptwo') {
    $nx = $y - sgn($x) * sin(log(abs($p[1] * $x - $p[2]))) * atan(sqr(log(abs($p[2] * $x - $p[1]))));
    $ny = $p[0] - $x; }
  elsif ($t eq 'threeply') {
    $nx = $y - sgn($x) * abs(sin($x) * cos($p[1]) + $p[2] - $x * sin($p[0] + $p[1] + $p[2]));
    $ny = $p[0] - $x; }
  elsif ($t eq 'latoocarfian') {
    $nx = sin($y * $p[1]) + $p[2] * sin($x * $p[1]);
    $ny = sin($x * $p[0]) + $p[3] * sin($y * $p[0]); }
  else { die "unknown type $t\n"; }
  ($x, $y) = ($nx, $ny);
  $buf .= pack('d<d<', $x, $y);
  if (length($buf) > 65536) { print $buf; $buf = ''; }
}
print $buf;
