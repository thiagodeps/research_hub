#!/usr/bin/env python3
import math
import sys
import os

from scripts.geo_v2 import generate_pad_v2

def create_horizontal_logo(theme="dark"):
    # Size 640 x 140
    # Icon centered at cx=80, cy=70, r_out=58, r_in=16, h=2.5, cr_out=5, cr_in=2.5
    cx, cy = 80, 70
    r_in, r_out = 16, 56
    h = 2.5
    cr_out = 5
    cr_in = 2.5

    p_tr = generate_pad_v2(cx, cy, r_in, r_out, h, cr_out, cr_in, 0)
    p_br = generate_pad_v2(cx, cy, r_in, r_out, h, cr_out, cr_in, 90)
    p_bl = generate_pad_v2(cx, cy, r_in, r_out, h, cr_out, cr_in, 180)
    p_tl = generate_pad_v2(cx, cy, r_in, r_out, h, cr_out, cr_in, 270)

    is_dark = (theme == "dark")
    bg_fill = "#0F172A" if is_dark else "#FFFFFF"
    text_primary = "#F8FAFC" if is_dark else "#0F172A"
    text_secondary = "#94A3B8" if is_dark else "#64748B"
    hub_fill = "#1E293B" if is_dark else "#0F172A"
    white_border = "#CBD5E1" if is_dark else "#94A3B8"
    white_stroke_w = "1.2" if is_dark else "1.5"

    svg = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 140" width="640" height="140">
  <defs>
    <linearGradient id="hRed" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#FF3E3E"/>
      <stop offset="100%" stop-color="#D91E24"/>
    </linearGradient>
    <linearGradient id="hGreen" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#00C853"/>
      <stop offset="100%" stop-color="#008833"/>
    </linearGradient>
    <linearGradient id="hWhite" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#FFFFFF"/>
      <stop offset="100%" stop-color="#E2E8F0"/>
    </linearGradient>
    <linearGradient id="hubAccent" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#38BDF8"/>
      <stop offset="100%" stop-color="#0284C7"/>
    </linearGradient>
    <filter id="subtleShadow" x="-10%" y="-10%" width="120%" height="120%">
      <feDropShadow dx="0" dy="2" stdDeviation="2" flood-color="#000000" flood-opacity="0.25"/>
    </filter>
  </defs>

  <!-- Background (Optional transparent or filled) -->
  <rect width="640" height="140" fill="{bg_fill}" rx="16"/>

  <!-- Logo Icon -->
  <g filter="url(#subtleShadow)">
    <!-- Top-Left: Red -->
    <path d="{p_tl}" fill="url(#hRed)"/>
    <!-- Top-Right: Green -->
    <path d="{p_tr}" fill="url(#hGreen)"/>
    <!-- Bottom-Left: Green -->
    <path d="{p_bl}" fill="url(#hGreen)"/>
    <!-- Bottom-Right: White -->
    <path d="{p_br}" fill="url(#hWhite)" stroke="{white_border}" stroke-width="{white_stroke_w}"/>

    <!-- Center Hub -->
    <circle cx="{cx}" cy="{cy}" r="13" fill="{hub_fill}" stroke="#334155" stroke-width="1.5"/>
    <circle cx="{cx}" cy="{cy}" r="4.5" fill="url(#hubAccent)"/>
  </g>

  <!-- Typography -->
  <text x="160" y="72" font-family="system-ui, -apple-system, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif" font-size="44" font-weight="800" fill="{text_primary}" letter-spacing="-0.5">
    Research<tspan fill="#00C853">Hub</tspan>
  </text>
  
  <text x="162" y="98" font-family="system-ui, -apple-system, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif" font-size="13" font-weight="600" fill="{text_secondary}" letter-spacing="1.8">
    PLATAFORMA DE INTELIGÊNCIA EM PESQUISA
  </text>
</svg>'''
    return svg

if __name__ == '__main__':
    with open('assets/logo/research-hub-horizontal-dark.svg', 'w') as f:
        f.write(create_horizontal_logo(theme="dark"))
    with open('assets/logo/research-hub-horizontal-light.svg', 'w') as f:
        f.write(create_horizontal_logo(theme="light"))
    print("Horizontal logos created")
