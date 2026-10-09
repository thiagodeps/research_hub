#!/usr/bin/env python3
import math
import sys
import os

from scripts.geo_v2 import generate_pad_v2

def create_full_icon_svg(is_app_icon=True):
    cx, cy = 256, 256
    r_in, r_out = 66, 212
    h = 8
    cr_out = 16
    cr_in = 8

    p_tr = generate_pad_v2(cx, cy, r_in, r_out, h, cr_out, cr_in, 0)
    p_br = generate_pad_v2(cx, cy, r_in, r_out, h, cr_out, cr_in, 90)
    p_bl = generate_pad_v2(cx, cy, r_in, r_out, h, cr_out, cr_in, 180)
    p_tl = generate_pad_v2(cx, cy, r_in, r_out, h, cr_out, cr_in, 270)

    # SVG definitions with gradients and filters
    defs = '''
  <defs>
    <!-- Background Gradient -->
    <linearGradient id="bgGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#0F172A"/>
      <stop offset="100%" stop-color="#020617"/>
    </linearGradient>

    <!-- Outer Bezel Gradient -->
    <linearGradient id="bezelGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#334155"/>
      <stop offset="50%" stop-color="#1E293B"/>
      <stop offset="100%" stop-color="#0F172A"/>
    </linearGradient>

    <!-- Rim Light Gradient -->
    <linearGradient id="rimGrad" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" stop-color="#475569" stop-opacity="0.8"/>
      <stop offset="100%" stop-color="#1E293B" stop-opacity="0.2"/>
    </linearGradient>

    <!-- Red Pad Gradient (Top-Left) -->
    <linearGradient id="redGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#FF4343"/>
      <stop offset="60%" stop-color="#E52427"/>
      <stop offset="100%" stop-color="#B91C1C"/>
    </linearGradient>

    <!-- Green Pad Gradient TR -->
    <linearGradient id="greenGradTR" x1="50%" y1="0%" x2="50%" y2="100%">
      <stop offset="0%" stop-color="#10B981"/>
      <stop offset="60%" stop-color="#00A34A"/>
      <stop offset="100%" stop-color="#047857"/>
    </linearGradient>

    <!-- Green Pad Gradient BL -->
    <linearGradient id="greenGradBL" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#10B981"/>
      <stop offset="60%" stop-color="#00A34A"/>
      <stop offset="100%" stop-color="#047857"/>
    </linearGradient>

    <!-- White Pad Gradient BR -->
    <linearGradient id="whiteGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#FFFFFF"/>
      <stop offset="70%" stop-color="#F8FAFC"/>
      <stop offset="100%" stop-color="#E2E8F0"/>
    </linearGradient>

    <!-- Center Hub Radial Gradient -->
    <radialGradient id="hubGrad" cx="40%" cy="35%" r="65%">
      <stop offset="0%" stop-color="#334155"/>
      <stop offset="50%" stop-color="#1E293B"/>
      <stop offset="100%" stop-color="#0B0F19"/>
    </radialGradient>

    <!-- Center Nucleus Glow -->
    <radialGradient id="nucleusGrad" cx="35%" cy="35%" r="65%">
      <stop offset="0%" stop-color="#38BDF8"/>
      <stop offset="70%" stop-color="#0284C7"/>
      <stop offset="100%" stop-color="#0369A1"/>
    </radialGradient>

    <!-- Drop Shadow Filter for Pads -->
    <filter id="padShadow" x="-10%" y="-10%" width="120%" height="120%">
      <feDropShadow dx="0" dy="4" stdDeviation="4" flood-color="#000000" flood-opacity="0.4"/>
    </filter>

    <!-- Subtle Bevel / Glow for White Pad -->
    <filter id="whiteBevel" x="-10%" y="-10%" width="120%" height="120%">
      <feDropShadow dx="0" dy="3" stdDeviation="3" flood-color="#000000" flood-opacity="0.35"/>
    </filter>
  </defs>'''

    if is_app_icon:
        svg = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
{defs}
  <!-- Squircle App Icon Container -->
  <rect width="512" height="512" rx="112" fill="url(#bgGrad)"/>
  <rect width="508" height="508" x="2" y="2" rx="110" fill="none" stroke="#334155" stroke-width="1.5" stroke-opacity="0.4"/>

  <!-- Outer Console Chassis (Genius Disc) -->
  <circle cx="256" cy="256" r="236" fill="url(#bezelGrad)" filter="url(#padShadow)"/>
  <circle cx="256" cy="256" r="234" fill="none" stroke="url(#rimGrad)" stroke-width="2"/>
  <circle cx="256" cy="256" r="226" fill="#0B0F19"/>

  <!-- Pads with subtle tactile depth -->
  <!-- Top-Left: Red -->
  <path d="{p_tl}" fill="url(#redGrad)" filter="url(#padShadow)"/>
  <path d="{p_tl}" fill="none" stroke="#FFA3A3" stroke-width="1.2" stroke-opacity="0.4"/>

  <!-- Top-Right: Green -->
  <path d="{p_tr}" fill="url(#greenGradTR)" filter="url(#padShadow)"/>
  <path d="{p_tr}" fill="none" stroke="#86EFAC" stroke-width="1.2" stroke-opacity="0.4"/>

  <!-- Bottom-Left: Green -->
  <path d="{p_bl}" fill="url(#greenGradBL)" filter="url(#padShadow)"/>
  <path d="{p_bl}" fill="none" stroke="#86EFAC" stroke-width="1.2" stroke-opacity="0.4"/>

  <!-- Bottom-Right: White -->
  <path d="{p_br}" fill="url(#whiteGrad)" filter="url(#whiteBevel)"/>
  <path d="{p_br}" fill="none" stroke="#94A3B8" stroke-width="1.2" stroke-opacity="0.5"/>

  <!-- Center Hub (Tactile Core) -->
  <circle cx="256" cy="256" r="58" fill="url(#hubGrad)" filter="url(#padShadow)"/>
  <circle cx="256" cy="256" r="57" fill="none" stroke="#475569" stroke-width="1.5"/>
  <circle cx="256" cy="256" r="45" fill="#090D16" stroke="#1E293B" stroke-width="1.5"/>
  
  <!-- Central Scientific Nucleus / Hub Core -->
  <circle cx="256" cy="256" r="18" fill="url(#nucleusGrad)"/>
  <circle cx="256" cy="256" r="18" fill="none" stroke="#BAE6FD" stroke-width="1" stroke-opacity="0.6"/>
  <circle cx="256" cy="256" r="6" fill="#FFFFFF" opacity="0.85"/>
</svg>'''
    else:
        # Standalone vector icon with transparent background
        svg = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
{defs}
  <!-- Outer Ring Base -->
  <circle cx="256" cy="256" r="236" fill="url(#bezelGrad)"/>
  <circle cx="256" cy="256" r="234" fill="none" stroke="#475569" stroke-width="2" stroke-opacity="0.4"/>
  <circle cx="256" cy="256" r="226" fill="#0B0F19"/>

  <!-- Pads -->
  <!-- Top-Left: Red -->
  <path d="{p_tl}" fill="url(#redGrad)"/>
  <path d="{p_tl}" fill="none" stroke="#FFA3A3" stroke-width="1.2" stroke-opacity="0.4"/>

  <!-- Top-Right: Green -->
  <path d="{p_tr}" fill="url(#greenGradTR)"/>
  <path d="{p_tr}" fill="none" stroke="#86EFAC" stroke-width="1.2" stroke-opacity="0.4"/>

  <!-- Bottom-Left: Green -->
  <path d="{p_bl}" fill="url(#greenGradBL)"/>
  <path d="{p_bl}" fill="none" stroke="#86EFAC" stroke-width="1.2" stroke-opacity="0.4"/>

  <!-- Bottom-Right: White -->
  <path d="{p_br}" fill="url(#whiteGrad)"/>
  <path d="{p_br}" fill="none" stroke="#94A3B8" stroke-width="1.2" stroke-opacity="0.6"/>

  <!-- Center Hub -->
  <circle cx="256" cy="256" r="58" fill="url(#hubGrad)"/>
  <circle cx="256" cy="256" r="57" fill="none" stroke="#475569" stroke-width="1.5"/>
  <circle cx="256" cy="256" r="45" fill="#090D16" stroke="#1E293B" stroke-width="1.5"/>
  <circle cx="256" cy="256" r="18" fill="url(#nucleusGrad)"/>
  <circle cx="256" cy="256" r="18" fill="none" stroke="#BAE6FD" stroke-width="1" stroke-opacity="0.6"/>
  <circle cx="256" cy="256" r="6" fill="#FFFFFF" opacity="0.85"/>
</svg>'''
    return svg

if __name__ == '__main__':
    with open('assets/logo/research-hub-app-icon.svg', 'w') as f:
        f.write(create_full_icon_svg(is_app_icon=True))
    with open('assets/logo/research-hub-icon.svg', 'w') as f:
        f.write(create_full_icon_svg(is_app_icon=False))
    print("SVGs created successfully")
