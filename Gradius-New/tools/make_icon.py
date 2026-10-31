#!/usr/bin/env python3
"""Original code-drawn ASTRA FRACTURE monogram; no legacy game imagery."""
from pathlib import Path
from PIL import Image,ImageDraw
import math
root=Path(__file__).resolve().parents[1]
size=1024
im=Image.new('RGBA',(size,size))
p=im.load()
for y in range(size):
    for x in range(size):
        d=math.hypot((x-420)/720,(y-390)/720)
        p[x,y]=(int(8+7*max(0,1-d)),int(17+22*max(0,1-d)),int(27+30*max(0,1-d)),255)
d=ImageDraw.Draw(im)
d.rounded_rectangle((18,18,1006,1006),radius=205,outline=(38,78,91,255),width=3)
d.arc((210,210,814,814),start=210,end=520,fill=(55,134,150,255),width=6)
d.line([(310,723),(484,296),(550,296),(714,723)],fill=(218,242,240,255),width=63)
d.line([(382,568),(647,568)],fill=(81,222,227,255),width=42)
d.polygon([(556,222),(493,483),(557,483),(470,805),(626,437),(560,437)],fill=(78,225,233,255))
d.ellipse((741,264,768,291),fill=(244,175,93,255))
iconset=root/'dist/Astra.iconset';iconset.mkdir(parents=True,exist_ok=True)
for n in [16,32,128,256,512]:
    im.resize((n,n),Image.Resampling.LANCZOS).save(iconset/f'icon_{n}x{n}.png')
    im.resize((n*2,n*2),Image.Resampling.LANCZOS).save(iconset/f'icon_{n}x{n}@2x.png')
im.save(root/'assets/app-icon.png')
