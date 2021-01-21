$fa = 1;
$fs = 0.4;

wall_thickness = 1;
wall_buffer = 0.25;
back_thickness = 1.5;

hat_width = 58;
hat_length = 96.5;
hat_thickness = 3;
hat_height = 21;
hat_bottom_padding = 2;

display_width = 47.32;
display_length = 81.12;
display_inset = (hat_width - display_width) / 2;

case_width = hat_width + (wall_thickness + wall_buffer) * 2;
case_length = hat_length + (wall_thickness + wall_buffer) * 2;
case_height = hat_height + hat_bottom_padding + back_thickness * 2;

cable_cutout_radius = 2;
cable_cutout_offset = 61;
cable_cutout_height = 6.5;

clip_width = 10;
clip_point_height = 10;

$vpt = [hat_width / 2, hat_length / 2, 0];
$vpd = 500;