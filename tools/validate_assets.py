#!/usr/bin/env python3
"""Verify extracted resources independently of the renderer."""
from pathlib import Path
import hashlib,json,struct,wave
from PIL import Image
root=Path(__file__).resolve().parents[1]
manifest=json.loads((root/'assets/manifest.json').read_text())
for name,digest in manifest['files'].items():
    assert hashlib.sha256((root/'assets'/name).read_bytes()).hexdigest()==digest,name
data=json.loads((root/'assets/game.json').read_text())
assert len(data['stages'])==12 and len(data['enemies'])==66
for i,s in enumerate(data['stages']):
    mask=(root/f'assets/collision-{i:02}.bin').read_bytes()
    assert len(mask)==s['width']*22 and set(mask)=={0,1}
    assert s['end']+32<s['width']
    assert len((root/f'assets/vram-{i:02}.bin').read_bytes())==16384
# Regression: character enemies must not punch opaque black rectangles
# through a colored background. Fully transparent texels also keep zero RGB.
for i in range(12):
    im=Image.open(root/f'assets/enemies-{i:02}.png')
    assert im.mode=='RGBA'
    rgba=im.tobytes();visible=0
    for k in range(0,len(rgba),4):
        r,g,b,a=rgba[k:k+4]
        assert (r,g,b,a)!=(0,0,0,255),f'opaque background in stage {i}'
        if a==0:assert (r,g,b)==(0,0,0)
        else:visible+=1
    assert visible>0
tracks=json.loads((root/'assets/audio/tracks.json').read_text())
assert len(tracks)==67
for track in tracks:
    p=root/f"assets/audio/{track['id']:02x}.bin";b=p.read_bytes()
    n,loop=struct.unpack_from('<Ii',b);pos=8;count=0
    assert n==track['frames'] and (loop==-1 or 0<=loop<n)
    for _ in range(n):
        k=struct.unpack_from('<H',b,pos)[0];pos+=2
        for _ in range(k):
            reg,v=struct.unpack_from('<HB',b,pos);pos+=3
            assert reg<16 or 256<=reg<400
        count+=k
    assert pos==len(b) and count==track['writes']
audio=json.loads((root/'extracted/audio-manifest.json').read_text())
for item in audio:
    p=root/'extracted/wav'/item['file']
    assert hashlib.sha256(p.read_bytes()).hexdigest()==item['sha256']
    with wave.open(str(p),'rb') as w:
        assert (w.getnchannels(),w.getsampwidth(),w.getframerate())==(1,2,22050)
        assert abs(w.getnframes()/22050-item['seconds'])<1e-9
print(f"PASS: {len(manifest['files'])} resource hashes, 12 maps, 67 register streams, 67 WAV files")
