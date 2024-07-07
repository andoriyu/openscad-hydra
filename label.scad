  include <BOSL2/std.scad>
//include <screw_head.scad>
//include <screw_profiles.scad>

$fa=1;
$fs=0.4;


z_origin = 0.68;
normalization = 0.7;
screw_shaft = 0.5;

MIN_LENGTH = 4;
MAX_LENGTH = 20;


R=2.1;


H = 1;

// Calculate top line length
// Screw head located at the origin and has a diameter of 4.2mm, so it has 2.1 mm offset in either direction
// there is 5mm buffer between screw head and screw shaft
// screw shaft is 2mm head + length (range between 4 and 25)
// therefore total length is 4.2 + 5 + 2 + normalized length
// ^^ those numbers are wrong, I don't care enough to figure out which one, below are eyeballed numbers that work well enough

function calcTopLineLength(length) = (R * 2) + 3.8 + length;

// calculate how much top line needs to move to move left to be centered
// safe label area is around 30mm
// "(30 - top line length)/2" gives us margin on left and right
// label is 2.1mm to the left off center
// 15 - 2.1 - margin

function calcMargin(length) = (30 - length)/2;

function calcMove(length) = 15 - 2.1 - calcMargin(calcTopLineLength(length));

module topLine(length, head_type, cs) {
  nlength = min(max(length, MIN_LENGTH), MAX_LENGTH);
  
  tz_h = cs ? 2.1 : 2.1;
  tz_w1 = cs ? 2 : 4.2;
  tz_ang = cs ? 120 : 90;
  up(0.2) linear_extrude(H) left(calcMove(nlength)) back(2.7) diff() {
    circle(r=R) {
      if (head_type == "philips") {
        tag("remove") rect([0.75, 3]);
        tag("remove") rect([0.5, 3], spin = 90);
      } else if (head_type == "hex") {
        tag("remove") hexagon(R * 0.7);
      }
    }
    
    tag("add") right (5) trapezoid(h=tz_h, w1=tz_w1, ang=tz_ang, rounding=0.2, spin = 90) {
      tag("add") attach(BOT, TOP) fwd(0.2) rect([2.1, nlength - 2], anchor=TOP) {
        if (nlength < length) {
          tag("remove") position(CENTER) text("//", anchor=CENTER, spin = 90);
        }
      }
    } 
  }
}

module bottomLine(width, length) {
    txt = str("M",width," X ", length, "mm");
    up(0.2) fwd(2.2) text3d(txt, size=4.2 ,h=H, anchor=CENTER+BOT, atype="ycenter");
}

module blank() {
  xscale(0.99) import("./label.3mf");
}


module drawScrewLabel(width, length, head_type, cs) {
  union() {
    blank();
    #topLine(length, head_type,cs);
    bottomLine(width, length);
  }
}

module washerBottomLine(width) {
    left(6) up(0.2) fwd(2.2) text3d("Washers", size=4.2 ,h=H, anchor=LEFT+BOT, atype="ycenter");
}

module washerTopLine(width) {
    txt = str("M",width);
    left(6) up(0.2) back(2.2) text3d(txt, size=4.2 ,h=H, anchor=LEFT+BOT, atype="ycenter");
}

module drawWasherLabel(width) {
  union() {
    color("green") left(11) tube(h=H, or=4.5, wall=3, anchor=BOT);
    washerTopLine(width);
    washerBottomLine(width);
    blank();
  }
}

//drawScrewLabel(2, 12, "hex", false);

//drawWasherLabel(3);
