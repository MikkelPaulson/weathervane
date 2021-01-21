include<./back.scad>

translate([100, 0, 0])
front();

$vpt = [(100 + hat_width) / 2, hat_length / 2, 0];

module front() {
    include<./front.scad>
}