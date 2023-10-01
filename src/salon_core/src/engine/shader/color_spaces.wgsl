// https://en.wikipedia.org/wiki/HSL_and_HSV

// hmmc: hue, channel-wise min, Max and chroma (i.e. max-min)
fn rgb_to_hmmc(rgb: vec3<f32>) -> vec4<f32> {
  let M = max(rgb.r, max(rgb.g, rgb.b));
  let m = min(rgb.r, min(rgb.g, rgb.b));
  let chroma = M - m;
  let dc = vec3(rgb.g - rgb.b, rgb.b - rgb.r, rgb.r - rgb.g) / max(chroma, 0.001);
  var hue = dc.z + 4.0;
  hue = mix(hue, dc.y + 2.0, step(M, rgb.g));
  hue = mix(hue, dc.x, step(M, rgb.r));
  hue = (hue / 6.0) % 1.0;
  return vec4(hue, m, M, chroma);
}

fn hue_to_rgb(hue: f32) -> vec3<f32> {
  let r = abs(hue * 6.0 - 3.0) - 1.0;
  let g = -abs(hue * 6.0 - 2.0) + 2.0;
  let b = -abs(hue * 6.0 - 4.0) + 2.0;
  return clamp(vec3(r, g, b), vec3(0.0), vec3(1.0));
}

fn rgb_to_hsv(rgb: vec3<f32>) -> vec3<f32> {
  let hmmc = rgb_to_hmmc(rgb); 
  return vec3(hmmc.x, hmmc.w / max(hmmc.z, 0.001), hmmc.z);
}

fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
  let rgb = hue_to_rgb(hsv.x);
  return hsv.z * mix(vec3(1.0), rgb, hsv.y);
}

fn rgb_to_hsl(rgb: vec3<f32>) -> vec3<f32> {
  let hmmc = rgb_to_hmmc(rgb);
  let sum = hmmc.y + hmmc.z;
  let den = 1.0 - abs(sum - 1.0);
  return vec3(hmmc.x, hmmc.w / max(den, 0.001), sum * 0.5);
}

fn hsl_to_rgb(hsl: vec3<f32>) -> vec3<f32> {
  let rgb = hue_to_rgb(hsl.x);
  let chroma = (1.0 - abs(2.0 * hsl.z - 1.0)) * hsl.y;
  return chroma * (rgb - 0.5) + hsl.z;
}
