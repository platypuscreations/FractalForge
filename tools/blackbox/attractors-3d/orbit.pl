#!/usr/bin/perl
# SPDX-License-Identifier: GPL-3.0-or-later
# Prints orbit points of the trig-based orbit maps with the C library's sin/cos
# (run inside the black-box container so that libm matches XFractint's).
# Usage: perl orbit.pl TYPE COUNT X0 Y0 Z0 P1 P2 P3 P4 ...
# Output: one line per iterate (iterate 1 first): "x y z" with %.17g.
use strict; use warnings;
my ($type, $n, $x, $y, $z, @p) = @ARGV;
if ($type eq 'pickover') {
    my ($a, $b, $c, $d) = @p;
    for (1 .. $n) {
        my $nx = sin($a * $y) - $z * cos($b * $x);
        my $ny = $z * sin($c * $x) - cos($d * $y);
        my $nz = sin($x);
        ($x, $y, $z) = ($nx, $ny, $nz);
        printf "%.17g %.17g %.17g\n", $x, $y, $z;
    }
} else {
    die "unknown type $type\n";
}
