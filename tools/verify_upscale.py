#!/usr/bin/env python3
"""Verify the two GPU Scale2x passes against an independent CPU reference."""
from pathlib import Path
import json,sys,hashlib
from PIL import Image
directory=Path(sys.argv[1]) if len(sys.argv)>1 else Path(__file__).resolve().parents[1]/'extracted/verification/modern'
def scale2x(im):
    w,h=im.size;src=im.load();out=Image.new('RGBA',(w*2,h*2));dst=out.load()
    for y in range(h):
        for x in range(w):
            b=src[x,max(0,y-1)];down=src[x,min(h-1,y+1)]
            left=src[max(0,x-1),y];right=src[min(w-1,x+1),y];center=src[x,y]
            q=[center]*4
            if b!=down and left!=right:
                q=[left if left==b else center,right if b==right else center,
                   left if left==down else center,right if down==right else center]
            for i,color in enumerate(q):dst[x*2+i%2,y*2+i//2]=color
    return out
base=Image.open(directory/'base.png').convert('RGBA')
gpu=Image.open(directory/'scale4x.png').convert('RGBA')
reference=scale2x(scale2x(base))
assert base.size==(256,224) and gpu.size==(1024,896)
assert reference.tobytes()==gpu.tobytes(),'GPU upscaler differs from CPU Scale4x reference'
report=dict(version='0.2.0',algorithm='Two Scale2x passes, published rules at https://www.scale2x.it/algorithm',input=list(base.size),output=list(gpu.size),pixels=gpu.width*gpu.height,comparison='Exact RGBA equality for all output pixels',image_sha256=hashlib.sha256(gpu.tobytes()).hexdigest())
(directory/'upscale-regression.json').write_text(json.dumps(report,indent=2)+'\n')
print('PASS: all 917504 GPU output pixels match CPU Scale4x exactly')
