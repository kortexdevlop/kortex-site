"""Read-only pixel analysis. Never edits either image or fabricates a reference."""
import json
import math
import hashlib
from pathlib import Path
import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]

def lab(rgb):
    values = [v / 255 for v in rgb]
    r, g, b = [v / 12.92 if v <= .04045 else ((v + .055) / 1.055) ** 2.4 for v in values]
    xyz = [(r * .4124564 + g * .3575761 + b * .1804375) / .95047,
           r * .2126729 + g * .7151522 + b * .0721750,
           (r * .0193339 + g * .1191920 + b * .9503041) / 1.08883]
    x, y, z = [v ** (1 / 3) if v > 216 / 24389 else (24389 / 27 * v + 16) / 116 for v in xyz]
    return (116 * y - 16, 500 * (x - y), 200 * (y - z))

def delta_e(first, second):
    l1, a1, b1 = first
    l2, a2, b2 = second
    c1, c2 = math.hypot(a1, b1), math.hypot(a2, b2)
    mean_c = (c1 + c2) / 2
    g = (1 - math.sqrt(mean_c ** 7 / (mean_c ** 7 + 25 ** 7))) / 2
    ap1, ap2 = (1 + g) * a1, (1 + g) * a2
    cp1, cp2 = math.hypot(ap1, b1), math.hypot(ap2, b2)
    h1, h2 = math.degrees(math.atan2(b1, ap1)) % 360, math.degrees(math.atan2(b2, ap2)) % 360
    dl, dc = l2 - l1, cp2 - cp1
    dh = 0 if cp1 * cp2 == 0 else h2 - h1
    if dh > 180:
        dh -= 360
    elif dh < -180:
        dh += 360
    dh = 2 * math.sqrt(cp1 * cp2) * math.sin(math.radians(dh / 2))
    lm, cm = (l1 + l2) / 2, (cp1 + cp2) / 2
    if cp1 * cp2 == 0:
        hm = h1 + h2
    elif abs(h1 - h2) <= 180:
        hm = (h1 + h2) / 2
    elif h1 + h2 < 360:
        hm = (h1 + h2 + 360) / 2
    else:
        hm = (h1 + h2 - 360) / 2
    cos = lambda angle: math.cos(math.radians(angle))
    t = 1 - .17 * cos(hm - 30) + .24 * cos(2 * hm) + .32 * cos(3 * hm + 6) - .20 * cos(4 * hm - 63)
    sl = 1 + .015 * (lm - 50) ** 2 / math.sqrt(20 + (lm - 50) ** 2)
    sc, sh = 1 + .045 * cm, 1 + .015 * cm * t
    rt = -2 * math.sqrt(cm ** 7 / (cm ** 7 + 25 ** 7)) * math.sin(math.radians(60 * math.exp(-((hm - 275) / 25) ** 2)))
    return math.sqrt((dl / sl) ** 2 + (dc / sc) ** 2 + (dh / sh) ** 2 + rt * dc / sc * dh / sh)

def compare():
    # Published Sharma/Wu/Dalal pairs, plus symmetry. doi:10.1002/col.20070.
    for first, expected in [((50, 2.6772, -79.7751), 2.0425), ((50, 3.1571, -77.2803), 2.8615), ((50, 2.8361, -74.0200), 3.4412)]:
        second = (50, 0, -82.7485)
        assert abs(delta_e(first, second) - expected) < .0001
        assert abs(delta_e(first, second) - delta_e(second, first)) < 1e-10
    source_path = ROOT / 'docs/reference/workspace-selection.png'
    render_path = ROOT / 'docs/evidence/workspace-chromium-reference.png'
    source = np.asarray(Image.open(source_path).convert('RGB'), dtype=float)
    render = np.asarray(Image.open(render_path).convert('RGB'), dtype=float)
    assert source.shape == render.shape == (1764, 2706, 3)
    points = {'outer': (0, 0), 'sidebar': (200, 1100), 'selected_navigation': (200, 280),
              'cyan': (1600, 200), 'upper_left': (600, 200), 'composer': (1000, 600),
              'neutral': (600, 900), 'lilac': (2550, 1650),
              'image_icon': (630, 1385), 'document_icon': (2160, 1385), 'developer_icon': (1150, 1690)}
    samples = []
    for name, (x, y) in points.items():
        a, b = source[y, x], render[y, x]
        samples.append({'name': name, 'position': [x, y], 'reference_rgb': a.tolist(), 'render_rgb': b.tolist(), 'ciede2000': round(delta_e(lab(a), lab(b)), 5)})
    difference = abs(source - render)
    report = {'reference_sha256': hashlib.sha256(source_path.read_bytes()).hexdigest(),
              'render_sha256': hashlib.sha256(render_path.read_bytes()).hexdigest(),
              'size': [2706, 1764], 'mean_absolute_rgb_error_0_255': round(float(difference.mean()), 6),
              'root_mean_square_rgb_error': round(float(np.sqrt(((source-render)**2).mean())), 6),
              'samples': samples, 'maximum_sample_ciede2000': max(s['ciede2000'] for s in samples),
              'limits': 'RGB error is a descriptive metric, not a perceptual similarity percentage. Sample Delta E only applies to listed flat zones. Original font is unidentified; no perfect pixel identity claimed.'}
    (ROOT / 'docs/evidence/workspace-comparison.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf8')
    assert report['maximum_sample_ciede2000'] <= 12
    print(json.dumps({'rgb_mae': report['mean_absolute_rgb_error_0_255'], 'max_sample_ciede2000': report['maximum_sample_ciede2000'], 'samples': len(samples)}))

if __name__ == '__main__':
    compare()
