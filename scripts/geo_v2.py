#!/usr/bin/env python3
import math

def generate_pad_v2(cx, cy, r_in, r_out, h, cr_out, cr_in, angle_offset_deg):
    """
    Constructs an SVG path for one quadrant pad.
    cx, cy: center
    r_in: inner radius (facing hub)
    r_out: outer radius
    h: half-gap between pads (straight channels)
    cr_out: corner radius on outer corners
    cr_in: corner radius on inner corners
    """
    # In Cartesian (x > 0, y > 0 for standard first quadrant)
    # y = h is horizontal divider line
    # x = h is vertical divider line
    
    # 1. Vertical edge at x = h:
    # Runs from y_in to y_out
    y_in_raw = math.sqrt(max(0, r_in**2 - h**2))
    y_out_raw = math.sqrt(max(0, r_out**2 - h**2))
    
    # Inner-vertical fillet:
    # Corner at (h, y_in_raw). Fillet radius cr_in.
    p_vert_start = (h, y_in_raw + cr_in)
    
    # Outer-vertical fillet:
    # Corner at (h, y_out_raw). Fillet radius cr_out.
    p_vert_end = (h, y_out_raw - cr_out)
    c_out_top = (h, y_out_raw)
    
    # Outer arc starts after fillet:
    x_arc_start = h + cr_out
    y_arc_start = math.sqrt(max(0, r_out**2 - x_arc_start**2))
    p_arc_start = (x_arc_start, y_arc_start)
    
    # Outer arc ends before right-outer fillet:
    y_arc_end = h + cr_out
    x_arc_end = math.sqrt(max(0, r_out**2 - y_arc_end**2))
    p_arc_end = (x_arc_end, y_arc_end)
    
    # Right-outer fillet:
    c_out_right = (y_out_raw, h)
    p_horiz_start = (y_out_raw - cr_out, h)
    
    # Horizontal edge at y = h runs down to inner-right fillet:
    p_horiz_end = (y_in_raw + cr_in, h)
    c_in_right = (y_in_raw, h)
    
    # Inner arc starts after inner-right fillet:
    y_in_arc_start = h + cr_in
    x_in_arc_start = math.sqrt(max(0, r_in**2 - y_in_arc_start**2))
    p_in_arc_start = (x_in_arc_start, y_in_arc_start)
    
    # Inner arc ends before inner-vertical fillet:
    x_in_arc_end = h + cr_in
    y_in_arc_end = math.sqrt(max(0, r_in**2 - x_in_arc_end**2))
    p_in_arc_end = (x_in_arc_end, y_in_arc_end)
    c_in_top = (h, y_in_raw)
    
    # In SVG screen coordinates, Top-Right has x > 0, y < 0:
    # So pt_svg = (x, -y)
    raw_points = [
        ("M", p_vert_start),
        ("L", p_vert_end),
        ("Q", c_out_top, p_arc_start),
        ("A_OUT", p_arc_end),
        ("Q", c_out_right, p_horiz_start),
        ("L", p_horiz_end),
        ("Q", c_in_right, p_in_arc_start),
        ("A_IN", p_in_arc_end),
        ("Q", c_in_top, p_vert_start)
    ]
    
    # Rotate each point by angle_offset_deg in screen space
    # (Top-Right = 0, Bottom-Right = 90, Bottom-Left = 180, Top-Left = 270)
    rad = math.radians(angle_offset_deg)
    cos_a = math.cos(rad)
    sin_a = math.sin(rad)
    
    def transform(p):
        px, py = p[0], -p[1] # convert first quadrant to screen TR (x > 0, y < 0)
        rx = px * cos_a - py * sin_a
        ry = px * sin_a + py * cos_a
        return (cx + rx, cy + ry)
        
    cmds = []
    for item in raw_points:
        cmd = item[0]
        if cmd == "M":
            pt = transform(item[1])
            cmds.append(f"M {pt[0]:.2f} {pt[1]:.2f}")
        elif cmd == "L":
            pt = transform(item[1])
            cmds.append(f"L {pt[0]:.2f} {pt[1]:.2f}")
        elif cmd == "Q":
            c = transform(item[1])
            p = transform(item[2])
            cmds.append(f"Q {c[0]:.2f} {c[1]:.2f} {p[0]:.2f} {p[1]:.2f}")
        elif cmd == "A_OUT":
            p = transform(item[1])
            cmds.append(f"A {r_out:.2f} {r_out:.2f} 0 0 1 {p[0]:.2f} {p[1]:.2f}")
        elif cmd == "A_IN":
            p = transform(item[1])
            cmds.append(f"A {r_in:.2f} {r_in:.2f} 0 0 0 {p[0]:.2f} {p[1]:.2f}")
            
    cmds.append("Z")
    return " ".join(cmds)

print("generate_pad_v2 compiled")
