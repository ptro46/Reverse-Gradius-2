#!/usr/bin/env python3
"""Check actual GPU output produced by --sprite-check, not only PNG metadata."""
from pathlib import Path
import sys,json,re
from PIL import Image
root=Path(__file__).resolve().parents[1]
directory=Path(sys.argv[1]) if len(sys.argv)>1 else root/'extracted/verification/sprite-fix'
atlas=Image.open(root/'assets/enemies-01.png').convert('RGBA')
ship=Image.open(root/'assets/ship.png').convert('RGBA')
terrain=Image.open(root/'assets/map-01.png').convert('RGB')
frames=[Image.open(directory/f'sprites-{i}.png').convert('RGB') for i in range(1,7)]
ship_regions=[]
for frame in frames:
    scale=frame.width//256
    assert frame.size==(256*scale,224*scale)
    # Sample the center of every logical pixel to avoid scaling ambiguities.
    def pixel(x,y):return frame.getpixel((x*scale+scale//2,y*scale+scale//2))
    ship_region=[]
    for y in range(24):
        for x in range(24):
            r,g,b,a=ship.getpixel((x,y))
            expected=(round(r*.65),round(g*.85),b) if a else terrain.getpixel((1280+60+x,100+y-24))
            actual=pixel(60+x,100+y)
            assert max(abs(a-b) for a,b in zip(actual,expected))<=1,('ship',x,y,actual,expected)
            ship_region.append(actual)
    ship_regions.append(ship_region)
    for i,t in enumerate((1,11,41,42,43,44,45)):
        for y in range(24):
            for x in range(24):
                dx,dy=12+i*32+x,156+y
                r,g,b,a=atlas.getpixel((t%16*24+x,t//16*24+y))
                checker=(32,64,112) if (dx//8+(dy-132)//8)%2==0 else (72,112,160)
                expected=(r,g,b) if a else checker
                assert pixel(dx,dy)==expected,('enemy',t,x,y,pixel(dx,dy),expected)
assert all(region==ship_regions[0] for region in ship_regions), 'ship changes between blink phases'
report=dict(version=re.search(r'version = "([^"]+)"',(root/'Cargo.toml').read_text())[1],frames=6,ship='visible and identical in all six original blink phases with invulnerability enabled',enemies='7 character-enemy/collectible cells per frame match exact compositing over a colored checkerboard',checked_pixels=6*(24*24+7*24*24))
(directory/'sprite-regression.json').write_text(json.dumps(report,indent=2)+'\n')
print('PASS:',report['checked_pixels'],'GPU pixels checked; no ship blinking or opaque enemy backgrounds')
