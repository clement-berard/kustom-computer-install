// Work-oriented shader: pure black background, bright strong colors, no bloom, selection border.

// Everything darker than this becomes pure black (0.0 to 0.3).
const float BLACK_POINT = 0.06;
// Everything brighter than this becomes full brightness (0.6 to 1.0).
const float WHITE_POINT = 0.80;
// Saturation. 1.0 = unchanged, higher = stronger colors.
const float SATURATION = 1.08;
// Gamma. Below 1.0 brightens midtones.
const float GAMMA = 0.74;

// Must match config.ghostty: selection-background and the theme background (Github Dark).
const vec3 SELECTION_BG = vec3(0.239, 0.267, 0.322); // #3d4452
const vec3 TERMINAL_BG = vec3(0.051, 0.067, 0.090);  // #0d1117
const float COLOR_TOLERANCE = 0.03;

// Selection border width in pixels and its color.
const float BORDER_WIDTH = 1.0;
const vec3 BORDER_COLOR = vec3(0.60, 0.68, 0.80);

bool near(vec3 a, vec3 b) {
  return distance(a, b) < COLOR_TOLERANCE;
}

// True if the pixel at uv is part of the terminal background (not selected, not text).
bool isBackground(vec2 uv) {
  return near(texture(iChannel0, uv).rgb, TERMINAL_BG);
}

void mainImage(out vec4 fragColor, in vec2 fragCoord) {
  vec2 uv = fragCoord.xy / iResolution.xy;
  vec4 color = texture(iChannel0, uv);

  bool onBorder = false;
  if (near(color.rgb, SELECTION_BG)) {
    // A selected pixel touching the plain background is on the selection edge.
    // Text pixels inside the selection never match the background, so they are ignored.
    vec2 px = BORDER_WIDTH / iResolution.xy;
    onBorder = isBackground(uv + vec2(px.x, 0.0)) || isBackground(uv - vec2(px.x, 0.0))
            || isBackground(uv + vec2(0.0, px.y)) || isBackground(uv - vec2(0.0, px.y));
  }

  // Stretch the range: near-blacks to true black, light grays to full white.
  vec3 rgb = (color.rgb - BLACK_POINT) / (WHITE_POINT - BLACK_POINT);
  rgb = clamp(rgb, 0.0, 1.0);

  // Brighten midtones, then boost saturation around the luminance.
  rgb = pow(rgb, vec3(GAMMA));
  float l = dot(rgb, vec3(0.299, 0.587, 0.114));
  rgb = clamp(mix(vec3(l), rgb, SATURATION), 0.0, 1.0);

  // Drawn last so the color grading does not alter the border.
  if (onBorder) {
    rgb = BORDER_COLOR;
  }

  fragColor = vec4(rgb, 1.0);
}
