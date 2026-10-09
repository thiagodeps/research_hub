#!/usr/bin/env python3
import math
import sys
import os

from scripts.geo_v2 import generate_pad_v2

def create_symbol_flat_svg():
    cx, cy = 256, 256
    r_in, r_out = 60, 230
    h = 9
    cr_out = 20
    cr_in = 10

    p_tr = generate_pad_v2(cx, cy, r_in, r_out, h, cr_out, cr_in, 0)
    p_br = generate_pad_v2(cx, cy, r_in, r_out, h, cr_out, cr_in, 90)
    p_bl = generate_pad_v2(cx, cy, r_in, r_out, h, cr_out, cr_in, 180)
    p_tl = generate_pad_v2(cx, cy, r_in, r_out, h, cr_out, cr_in, 270)

    svg = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
  <defs>
    <!-- Red Pad Gradient -->
    <linearGradient id="flatRed" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#FF3838"/>
      <stop offset="100%" stop-color="#D91E24"/>
    </linearGradient>

    <!-- Green Pad Gradient TR -->
    <linearGradient id="flatGreenTR" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#00C853"/>
      <stop offset="100%" stop-color="#008833"/>
    </linearGradient>

    <!-- Green Pad Gradient BL -->
    <linearGradient id="flatGreenBL" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#00C853"/>
      <stop offset="100%" stop-color="#008833"/>
    </linearGradient>

    <!-- White Pad Gradient -->
    <linearGradient id="flatWhite" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#FFFFFF"/>
      <stop offset="100%" stop-color="#E2E8F0"/>
    </linearGradient>

    <!-- Center Hub Gradient -->
    <linearGradient id="flatHub" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#1E293B"/>
      <stop offset="100%" stop-color="#0F172A"/>
    </linearGradient>
  </defs>

  <!-- Pads -->
  <!-- Top-Left: Vermelho IFES -->
  <path d="{p_tl}" fill="url(#flatRed)"/>

  <!-- Top-Right: Verde IFES -->
  <path d="{p_tr}" fill="url(#flatGreenTR)"/>

  <!-- Bottom-Left: Verde IFES -->
  <path d="{p_bl}" fill="url(#flatGreenBL)"/>

  <!-- Bottom-Right: Branco (com suave borda para contraste em fundo claro) -->
  <path d="{p_br}" fill="url(#flatWhite)" stroke="#CBD5E1" stroke-width="2"/>

  <!-- Center Hub Disc -->
  <circle cx="256" cy="256" r="48" fill="url(#flatHub)" stroke="#334155" stroke-width="3"/>
  <circle cx="256" cy="256" r="16" fill="#38BDF8"/>
  <circle cx="256" cy="256" r="6" fill="#FFFFFF"/>
</svg>'''
    return svg

if __name__ == '__main__':
    with open('assets/logo/research-hub-symbol-flat.svg', 'w') as f:
        f.write(create_symbol_flat_svg())
    print("Flat symbol created")
