#!/usr/bin/env python3
"""Extract this RC-751 variant, without substituting resources from another ROM.
Reference decoders: antxiko/Nemesis2-disassembly, MIT (tools/reference/LICENSE).
The Z80 interpreter is used OFFLINE for audio extraction only.
"""
from pathlib import Path
import sys, types, hashlib, json, struct, re
from PIL import Image
ROOT = Path(__file__).resolve().parents[1]
path = Path(sys.argv[1]) if len(sys.argv)>1 else next(ROOT.glob('*.rom'))
rom = path.read_bytes()
sha = hashlib.sha256(rom).hexdigest()
assert sha == '1a0f09df467036758c3cc8c9382e8fb79319278575e7a86b9461ed835f7d7608', 'Unknown ROM variant: re-audit offsets'
assert len(rom)==131072 and rom[:4]==b'AB\x90@'
out=ROOT/'assets';out.mkdir(exist_ok=True)
sys.path.insert(0,str(ROOT/'tools/reference'))
origins={0:0x4000,**{b:(0x6000,0x8000,0xa000)[(b-1)%3] for b in range(1,16)},10:0xa000,11:0xa000}
class Ventana:
    def __init__(self,s): self.s=s
    def __call__(self,a):
        assert 0x4000<=a<0xc000,hex(a)
        b=0 if a<0x6000 else self.s[(a-0x6000)//8192]
        return rom[b*8192+a%8192]
    def w(self,a):return self(a)+256*self(a+1)
def ventana_de(b):
    s=[1,2,3]
    if b:s[(origins[b]-0x6000)//8192]=b
    return Ventana(s)
B=types.ModuleType('bloques');B.Ventana=Ventana;B.ventana_de=ventana_de
sys.modules['bloques']=B
import vram as V, dibuja as D, nave as N, enemigos as E, objetos as O, titulo as T
v0,v1,v14=ventana_de(0),ventana_de(1),Ventana((13,14,15))
# Variant: the generator table and following data in bank 14 are three
# bytes earlier; bank 15's shape pointer table is likewise at B65E.
# The ROM's own pointers already contain the relocated destinations.
def shape(n,w,h):
    a=v14.w(0xb65e+2*n)
    return [[v14(a+w*y+x) for x in range(w)] for y in range(h)]
O.forma=shape
O.CARGAS[5]=(0xb3ac,0xb3b9)
def columns(stage):
    starts=[v14.w(0x7900+2*k) for k in range(12)]
    a=starts[stage];end=(sorted(starts)+[0x91a8])[sorted(starts).index(a)+1]
    assert (end-a)%6==0
    return [[v14(x+i) for i in range(6)] for x in range(a,end,6)]
D.columnas=columns
stages=[]
for f in range(12):
    rows=D.filas_del_mapa(f);v=V.sprites_de_la_fase(f)
    original_width=len(rows[0]);loop_start=v1.w(0x6986+6*f)
    stop=v1.w(0x6986+6*f+4)
    for row in rows:
        original=row.copy()
        while len(row)<stop+33:row.append(original[loop_start+(len(row)-loop_start)%(original_width-loop_start)])
    # Static initial forms of map objects. Their animation/generators are native.
    for cls,x,y,flags,second in O.lista_de_la_fase(f):
        fam=next((fam for fam in O.FAMILIAS if fam[0]==cls),None)
        if fam is None or second:continue
        _,w,h,forms,_=fam
        if isinstance(forms,dict):forms=forms.get(f,forms['*'])
        forms=list(forms);index=min(len(forms)-1,flags*3 if cls==9 else flags*7 if cls in (1,7,16) else 0)
        tile_rows=shape(forms[index],w,h)
        for yy,rr in enumerate(tile_rows):
            for xx,c in enumerate(rr):
                if 0<=y//8+yy<22 and x+xx<len(rows[0]):rows[y//8+yy][x+xx]=c

    D.pinta(rows,v).save(out/f'map-{f:02}.png')
    (out/f'vram-{f:02}.bin').write_bytes(v)
    # Exact character classification used by p02:8465, including the special
    # non-solid characters 0..8 in the fortress and boss interior.
    threshold=v1(0x697a+f)
    collision=bytes(int((9<=c<threshold) if f in (0,7,8) else (0<c<threshold)) for row in rows for c in row)
    (out/f'collision-{f:02}.bin').write_bytes(collision)
    atlas=Image.new('RGBA',(256,256))
    for n in range(64):E.sprite(atlas.load(),v,n*4,15,n%16*16,n//16*16)
    atlas.save(out/f'sprites-{f:02}.png')
    enemies=Image.new('RGBA',(16*24,5*24))
    for t in range(1,66):
        x,y=t%16*24,t//16*24
        rec=E.ficha(t)
        if t==1:
            n=Ventana((1,2,3))(0x66d0+f)
            E.bloque(enemies.load(),v,E.ficha_de_caracteres(n,2,2),2,2,x+4,y+4)
        elif t==0xb:E.bloque(enemies.load(),v,E.ficha_de_caracteres(rec[6],2,2),2,2,x+4,y+4)
        elif 0x29<=t<=0x2d:
            c=v0(0x5265+t-0x29);E.bloque(enemies.load(),v,[c,c+1,c+2,c+3],2,2,x+4,y+4)
        elif not rec[0]&0x40:E.sprite(enemies.load(),v,*E.dibujo_de_salida(t,f),x+4,y+4)
    # Character-based enemies were painted by the RGB-oriented reference
    # decoder: black screen background became opaque in the RGBA atlas.
    # Only floating enemy/collectible atlases use this transparency key;
    # terrain and opaque map objects keep their original black pixels.
    rgba=bytearray(enemies.tobytes())
    for k in range(0,len(rgba),4):
        if rgba[k:k+3]==b'\x00\x00\x00':rgba[k:k+4]=b'\x00\x00\x00\x00'
    enemies.frombytes(bytes(rgba))
    enemies.save(out/f'enemies-{f:02}.png')
    events=[];gen=[];turrets=[]
    end=v1.w(0x6986+6*f+4)
    if f<9:
        a=v14.w(0x9a64+2*f)
        for _ in range(500):
            if v14.w(a)==65535:break
            events.append([v14.w(a),v14(a+2),v14(a+3)]);a+=4
        else:raise ValueError('unterminated events')
        a=v14.w(0x99c4+2*f)
        gen=[v14(a+k) for k in range((end>>5)+1)]
    a=v14.w(0x9554+2*f)
    for _ in range(600):
        if v14.w(a)==65535:break
        turrets.append([v14.w(a),v14(a+2)]);a+=3
    music=[];a=v1.w(0x7ce6+2*f)
    for _ in range(20):
        pos=v1.w(a);track=v1(a+2);music.append([pos,track]);a+=3
        if pos==65535:break
    stages.append(dict(width=len(rows[0]),threshold=threshold,end=end,checkpoint=v1.w(0x694a+2*f),events=events,generators=gen,turrets=turrets,music=music,objects=O.lista_de_la_fase(f),wave=v1(0x6ef2+f) if f<8 else 4))
    print('stage',f,'width',len(rows[0]),'events',len(events),'turrets',len(turrets),flush=True)
for t,_,fs,*_ in O.JEFES:
    im=O.jefe(t,fs[0],1).convert('RGBA')
    im.putdata([(r,g,b,0 if (r,g,b) in ((0,0,0),(40,40,40)) else 255) for r,g,b,a in im.getdata()])
    im.save(out/f'boss-{t}.png')
ship=Image.new('RGBA',(3*24,24));sv=V.sprites_de_la_fase(1)
for i,a in enumerate((0xa889,0xa88d,0xa891)):N.nave(ship.load(),sv,N.ficha(a),i*24+4,4)
ship.save(out/'ship.png')
rows,v=T.pantalla(True,False);D.pinta(rows,v).save(out/'title.png')
# ROM font, not a replacement pixel typeface.
font=Image.new('RGBA',(128,24))
for c in range(48):
    base=0x80+c*8
    for y in range(8):
        for x in range(8):
            if v[0x2000+base+y]&(128>>x):font.putpixel((c%16*8+x,c//16*8+y),(255,255,255,255))
font.save(out/'font.png')
(out/'game.json').write_text(json.dumps(dict(stages=stages,enemies=[E.ficha(t) for t in range(66)])))
manifest=dict(source=path.name,sha256=sha,bytes=len(rom),reference='https://github.com/antxiko/Nemesis2-disassembly',reference_rom_sha256='227113f94d9900cb48d2f9e88627224dd8cc3d9504614a01e94cf6b23ba508dc',variant_notes=['banks 4-9,11-13 match reference data byte for byte','bank14 generator table 99C4, enemy table 9A64; bank15 shapes B65E; map end 91A8','all output uses supplied ROM bytes'],gameplay='New Rust implementation; not a Z80 emulator; behaviors not yet cycle-exact',audio='Original driver executed offline; PSG/SCC writes extracted; synthesis is approximate')
if '--graphics-only' not in sys.argv:
    import musica as M, motor_musica as A
    audio=out/'audio';audio.mkdir(exist_ok=True);tracks=[]
    for num in list(range(1,0x43))+[0x5b]:
        frames,loop=A.ejecuta(M.CFG,rom,num,4500 if num>=0x28 else 300)
        data=bytearray(struct.pack('<Ii',len(frames),-1 if loop is None else loop))
        for frame in frames:
            data+=struct.pack('<H',len(frame))
            for reg,val in frame:data+=struct.pack('<HB',reg,val)
        (audio/f'{num:02x}.bin').write_bytes(data)
        tracks.append(dict(id=num,frames=len(frames),loop=loop,writes=sum(map(len,frames))))
        print('audio',hex(num),len(frames),'loop',loop,flush=True)
    (audio/'tracks.json').write_text(json.dumps(tracks,indent=2))
    manifest['audio_tracks']=len(tracks)
if (out/'audio/tracks.json').exists():
    manifest['audio_tracks']=len(json.loads((out/'audio/tracks.json').read_text()))
    manifest['audio_wav_directory']='extracted/wav'
manifest['files']={str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in out.rglob('*') if p.is_file() and p.name!='manifest.json'}
(out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
