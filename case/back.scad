include<./variables.scad>

bottom_inside_width = hat_width - wall_thickness * 2;
bottom_inside_length = hat_length - wall_thickness * 2;

color("gray")
base();

translate([
    wall_thickness * 2 + wall_buffer,
    wall_thickness * 2 + wall_buffer,
    back_thickness
]) {
    color("blue")
    translate([0, 4, 0])
    pins();
    
    color("red")
    supports();
    
    color("green")
    clips();
}

module base() {
    screw_hole_radius = 2.5;
    screw_hole_inset = 12;
    
    difference() {
        cube([
            case_width,
            case_length,
            back_thickness
        ]);
        
        translate([
            wall_thickness * 2 + wall_buffer + screw_hole_inset,
            hat_length + wall_buffer - screw_hole_inset,
            0
        ])
        cylinder(
            h = 30,
            r = screw_hole_radius,
            center = true
        );
        
        translate([
            hat_width + wall_buffer - screw_hole_inset,
            wall_thickness * 2 + wall_buffer + screw_hole_inset,
            0
        ])
        cylinder(
            h = 30,
            r = screw_hole_radius,
            center = true
        );
    };

    translate([
        wall_thickness + wall_buffer,
        wall_thickness + wall_buffer,
        back_thickness
    ])
    difference() {
        cube([
            hat_width,
            hat_length,
            5
        ]);
        
        translate([wall_thickness, wall_thickness, -2])
        cube([
            bottom_inside_width,
            bottom_inside_length,
            10
        ]);
        
        translate([
            hat_width - 1,
            cable_cutout_offset - wall_thickness - wall_buffer,
            cable_cutout_height - back_thickness
        ])
        rotate([0, 90, 0])
        cylinder(
            h = wall_thickness + 2,
            r = cable_cutout_radius,
            center = true
        );
    };
}

module pins() {
    pi_length = 65;
    pi_width = 30;
    pin_inset = 3.5;
    
    translate([
        pin_inset,
        pin_inset,
        0
    ])
    pin();
    
    translate([
        pi_width - pin_inset,
        pin_inset,
        0
    ])
    pin();
    
    translate([
        pi_width - pin_inset,
        pi_length - pin_inset,
        0
    ])
    pin();
    
    translate([
        pin_inset,
        pi_length - pin_inset,
        0
    ])
    pin();
    
    module pin() {
        pin_height = hat_bottom_padding + 2;
        pin_radius = 0.9;
        
        rotate([0, 0, 90])
        union() {
            cylinder(
                h = pin_height,
                r = pin_radius
            );
            
            translate([
                -pin_radius,
                -pin_radius * 2,
                0
            ])
            cube([
                pin_radius * 2,
                pin_radius * 4,
                hat_bottom_padding
            ]);
            
            translate([
                0,
                -pin_radius * 2,
                0
            ])
            cylinder(
                h = hat_bottom_padding,
                r = pin_radius
            );
            
            translate([
                0,
                pin_radius * 2,
                0
            ])
            cylinder(
                h = hat_bottom_padding,
                r = pin_radius
            );
        }
    }
}

module supports() {
    support_inset = 2.5;
    
    translate([
        hat_width - wall_thickness * 2 - support_inset,
        support_inset,
        0
    ])
    support();
    
    translate([
        hat_width - wall_thickness * 2 - support_inset,
        hat_length - wall_thickness * 2 - support_inset,
        0
    ])
    mirror([0, 1, 0])
    support();
    
    translate([
        support_inset,
        hat_length - wall_thickness * 2 - support_inset,
        0
    ])
    mirror([-1, 1, 0])
    support();
    
    module support() {
        support_height = hat_height + hat_bottom_padding - hat_thickness;
        support_width = 10;
        support_thickness = wall_thickness;
        
        fillet_thickness = support_thickness;
        fillet_inset = 3;
        
        linear_extrude(height = support_height)
        polygon([
            [0, 0],
            [-support_width, 0],
            [-support_width, support_thickness],
            [-support_thickness, support_thickness],
            [-support_thickness, support_width],
            [0, support_width]
        ]);
        
        translate([-(support_width - fillet_thickness - fillet_inset), support_thickness, 0])
        rotate([0, -90, 0])
        support_fillet();
        
        translate([-support_thickness, support_width - fillet_inset, 0])
        rotate([0, -90, 90])
        support_fillet();
        
        module support_fillet() {
            fillet_size = 3;
            
            linear_extrude(height = fillet_thickness)
            polygon([
                [0, 0],
                [fillet_size, 0],
                [0, fillet_size],
            ]);
        }
    }
}
module clips() {
    clip_inset = 1;
    clip_point_depth = clip_inset + wall_thickness * 1.5 + wall_buffer;
    
    translate([
        bottom_inside_width / 2,
        clip_inset,
        0
    ])
    clip();
    
    translate([
        bottom_inside_width / 2,
        bottom_inside_length - clip_inset,
        0
    ])
    rotate([0, 0, 180])
    clip();
    
    module clip() {
        clip_thickness = wall_thickness;
        
        fillet_thickness = clip_thickness;
        fillet_inset = 2;
        
        rotate([0, -90, 0])
        linear_extrude(height = clip_width, center = true)
        polygon([
            [0, 0],
        
            [clip_point_height - clip_point_depth, 0],
            [clip_point_height, -clip_point_depth],
            [clip_point_height + clip_point_depth + clip_thickness / sqrt(2) / 2, clip_thickness / sqrt(2) / 2],
        
            [clip_point_height + clip_point_depth - clip_thickness / sqrt(2) / 2, clip_thickness],
            [clip_point_height, -(clip_point_depth - (clip_thickness * sqrt(2)))],
            [clip_point_height - clip_point_depth + clip_thickness / sqrt(2) / 2, clip_thickness],
            [0, clip_thickness],
        ]);
        
        translate([clip_width / 2 - fillet_thickness - fillet_inset, 0, 0])
        clip_fillet();
        
        translate([
            -(clip_width / 2 - fillet_thickness - fillet_inset),
            0,
            0
        ])
        clip_fillet();
        
        module clip_fillet() {
            fillet_size = 3;
            
            rotate([0, -90, 0])
            linear_extrude(height = fillet_thickness, center = true)
            union() {
                polygon([
                
                    [clip_point_height + clip_point_depth - clip_thickness / sqrt(2) / 2, clip_thickness],
                    [clip_point_height, -(clip_point_depth - (clip_thickness * sqrt(2)))],
                    [clip_point_height - clip_point_depth + clip_thickness / sqrt(2) / 2, clip_thickness]
                ]);
                
                polygon([
                    [0, clip_thickness],
                    [fillet_size, clip_thickness],
                    [0, fillet_size + clip_thickness],
                ]);
            }
        }
    }
}

module top() {
    top_inside_width = case_width - wall_thickness * 2;
    top_inside_length = case_length - wall_thickness * 2;
    
    clip_point_height_rev = case_height - back_thickness - clip_point_height;
    
    cable_cutout_height_rev = case_height - back_thickness - cable_cutout_height;
    
    color("gray")
    
    union() {
        difference() {
            base();
            
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
        
        translate([case_width / 2, 0, 0])
        rotate([0, -90, 0])
        linear_extrude(
            height = clip_width + 2,
            center = true
        )
        polygon([
            [clip_point_height_rev, wall_thickness / 2],
            [clip_point_height_rev - wall_thickness, wall_thickness * 1.5],
            [clip_point_height_rev - wall_thickness * 2, wall_thickness / 2],
        ]);
        
        translate([case_width / 2, case_length, 0])
        rotate([180, -90, 0])
        linear_extrude(
            height = clip_width + 2,
            center = true
        )
        polygon([
            [clip_point_height_rev, wall_thickness / 2],
            [clip_point_height_rev - wall_thickness, wall_thickness * 1.5],
            [clip_point_height_rev - wall_thickness * 2, wall_thickness / 2],
        ]);
    }
    
    module base() {
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
                cable_cutout_height + 1,
                    cable_cutout_radius * 2
                ]);
            }
        }
    }
}