include<./variables.scad>

top_inside_width = case_width - wall_thickness * 2;
top_inside_length = case_length - wall_thickness * 2;

clip_point_height_rev = case_height - back_thickness - clip_point_height;

cable_cutout_height_rev = case_height - cable_cutout_height;

color("gray")

union() {
    difference() {
        base();
        
        // Clips inset
        translate([case_width / 2, 0, 0])
        rotate([0, -90, 0])
        linear_extrude(
            height = clip_width + 2,
            center = true
        )
        polygon([
            [clip_point_height_rev + wall_thickness, wall_thickness * 1.5],
            [clip_point_height_rev, wall_thickness / 2],
            [clip_point_height_rev - wall_thickness, wall_thickness * 1.5],
        ]);
        
        translate([case_width / 2, case_length, 0])
        rotate([180, -90, 0])
        linear_extrude(
            height = clip_width + 2,
            center = true
        )
        polygon([
            [clip_point_height_rev + wall_thickness, wall_thickness * 1.5],
            [clip_point_height_rev, wall_thickness / 2],
            [clip_point_height_rev - wall_thickness, wall_thickness * 1.5],
        ]);
    }
    
    // Clips
    translate([case_width / 2, 0, 0])
    rotate([0, -90, 0])
    linear_extrude(
        height = clip_width + 2,
        center = true
    )
    polygon([
        [clip_point_height_rev, wall_thickness / 2],
        [clip_point_height_rev + wall_thickness * 1.5, wall_thickness * 2],
        [clip_point_height_rev + wall_thickness * 3, wall_thickness / 2],
    ]);
    
    translate([case_width / 2, case_length, 0])
    rotate([180, -90, 0])
    linear_extrude(
        height = clip_width + 2,
        center = true
    )
    polygon([
        [clip_point_height_rev, wall_thickness / 2],
        [clip_point_height_rev + wall_thickness * 1.5, wall_thickness * 2],
        [clip_point_height_rev + wall_thickness * 3, wall_thickness / 2],
    ]);
}

module base() {
    // Bottom face
    difference() {
        cube([
            case_width,
            case_length,
            back_thickness
        ]);
        
        translate([
            display_inset + wall_thickness + wall_buffer,
            case_length - (display_inset + wall_thickness + wall_buffer + display_length),
            -1
        ])
        cube([
            display_width,
            display_length,
            back_thickness + 2
        ]);
    }
    
    // Sides
    difference() {
        cube([
            case_width,
            case_length,
            case_height - back_thickness
        ]);
        
        translate([
            wall_thickness,
            wall_thickness,
            -1
        ])
        cube([
            top_inside_width,
            top_inside_length,
            case_height - back_thickness + 2
        ]);
        
        // Cable cutout
        translate([
            -1,
            cable_cutout_offset,
            0
        ])
        rotate([0, 90, 0])
        linear_extrude(
            height = wall_thickness + 2
        )
        union() {
            translate([
                -cable_cutout_height_rev,
                0
            ])
            circle(
                r = cable_cutout_radius
            );
            
            translate([
                -(case_height - back_thickness + 1),
                -cable_cutout_radius
            ])
            square([
                cable_cutout_height,
                cable_cutout_radius * 2
            ]);
        }
    }
}