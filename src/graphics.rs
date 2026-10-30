//! Optional modern presentation. Original assets, simulation and hitboxes stay intact.
//! Scale4x = two Scale2x passes; independently written from the published rules:
//! https://www.scale2x.it/algorithm
use crate::core::{Game, Mode};
use macroquad::prelude::*;

pub struct Modern {
    scale: Material,
    finish: Material,
    twice: RenderTarget,
    four: RenderTarget,
    glow: Texture2D,
}
impl Modern {
    pub fn new() -> Self {
        let scale = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: SCALE,
            },
            MaterialParams {
                uniforms: vec![UniformDesc::new("InputSize", UniformType::Float2)],
                ..Default::default()
            },
        )
        .expect("Scale2x shader");
        let finish = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: FINISH,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("Scene", UniformType::Float3),
                    UniformDesc::new("Effects", UniformType::Float1),
                ],
                ..Default::default()
            },
        )
        .expect("modern presentation shader");
        let twice = render_target(512, 448);
        twice.texture.set_filter(FilterMode::Nearest);
        let four = render_target(1024, 896);
        four.texture.set_filter(FilterMode::Linear);
        let mut image = Image::gen_image_color(64, 64, Color::new(0., 0., 0., 0.));
        for y in 0..64 {
            for x in 0..64 {
                let r = ((x as f32 - 31.5).powi(2) + (y as f32 - 31.5).powi(2)).sqrt() / 32.;
                image.set_pixel(x, y, Color::new(1., 1., 1., ((1. - r).max(0.)).powf(3.)));
            }
        }
        let glow = Texture2D::from_image(&image);
        glow.set_filter(FilterMode::Linear);
        Self {
            scale,
            finish,
            twice,
            four,
            glow,
        }
    }
    fn pass(&self, source: &Texture2D, target: &RenderTarget, size: Vec2) {
        set_camera(&Camera2D {
            render_target: Some(target.clone()),
            ..Camera2D::from_display_rect(Rect::new(0., 0., size.x, size.y))
        });
        clear_background(BLACK);
        self.scale
            .set_uniform("InputSize", vec2(source.width(), source.height()));
        gl_use_material(&self.scale);
        draw_texture_ex(
            source,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                flip_y: true,
                ..Default::default()
            },
        );
        gl_use_default_material();
    }
    pub fn upscale(&self, source: &Texture2D) {
        self.pass(source, &self.twice, vec2(512., 448.));
        self.pass(&self.twice.texture, &self.four, vec2(1024., 896.));
        set_default_camera();
    }
    pub fn draw(&self, g: &Game, viewport: Rect, effects: bool, selected: usize) {
        self.finish.set_uniform(
            "Scene",
            vec3(
                g.scroll,
                g.frame as f32 / 60.,
                if g.mode == Mode::Title { 0. } else { 1. },
            ),
        );
        self.finish
            .set_uniform("Effects", if effects { 1f32 } else { 0f32 });
        gl_use_material(&self.finish);
        draw_texture_ex(
            &self.four.texture,
            viewport.x,
            viewport.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(viewport.w, viewport.h)),
                flip_y: true,
                ..Default::default()
            },
        );
        gl_use_default_material();
        if effects {
            self.particles(g, viewport);
        }
        self.hud(g, viewport, selected);
    }
    fn centered(label: &str, x: f32, y: f32, size: f32, color: Color) {
        let m = measure_text(label, None, size as u16, 1.);
        draw_text(label, x - m.width / 2., y, size, color);
    }
    fn panel(r: Rect, color: Color) {
        let round = (r.h * 0.12).min(7.);
        draw_rectangle(r.x + round, r.y, r.w - 2. * round, r.h, color);
        draw_rectangle(r.x, r.y + round, r.w, r.h - 2. * round, color);
        for (x, y) in [
            (r.x + round, r.y + round),
            (r.x + r.w - round, r.y + round),
            (r.x + round, r.y + r.h - round),
            (r.x + r.w - round, r.y + r.h - round),
        ] {
            draw_circle(x, y, round, color);
        }
    }
    fn hud(&self, g: &Game, v: Rect, selected: usize) {
        let s = v.w / 256.;
        let cyan = Color::from_rgba(92, 220, 246, 255);
        let muted = Color::from_rgba(113, 139, 170, 255);
        if g.mode == Mode::Title {
            draw_rectangle(
                v.x,
                v.y + 104. * s,
                v.w,
                120. * s,
                Color::from_rgba(7, 13, 26, 255),
            );
            draw_line(
                v.x + 40. * s,
                v.y + 105. * s,
                v.x + 216. * s,
                v.y + 105. * s,
                0.3 * s,
                Color::from_rgba(30, 72, 90, 255),
            );
            Self::centered("METALION", v.x + 128. * s, v.y + 125. * s, 7. * s, WHITE);
            Self::centered(
                "ENTREE POUR JOUER",
                v.x + 128. * s,
                v.y + 146. * s,
                6. * s,
                cyan,
            );
            Self::centered(
                "Fleches : secteur     Espace : tir     Maj : bonus",
                v.x + 128. * s,
                v.y + 163. * s,
                4.3 * s,
                muted,
            );
            Self::panel(
                Rect::new(v.x + 83. * s, v.y + 177. * s, 90. * s, 20. * s),
                Color::from_rgba(15, 33, 52, 255),
            );
            Self::centered(
                &format!("SECTEUR {:02}", selected),
                v.x + 128. * s,
                v.y + 190. * s,
                6. * s,
                cyan,
            );
            Self::centered(
                "MODERNE   /   F2 : RETRO   /   F3 : LUMIERES",
                v.x + 128. * s,
                v.y + 217. * s,
                4. * s,
                muted,
            );
            return;
        }
        draw_rectangle(v.x, v.y, v.w, 24. * s, Color::from_rgba(7, 13, 26, 255));
        draw_line(
            v.x,
            v.y + 23.5 * s,
            v.x + v.w,
            v.y + 23.5 * s,
            0.3 * s,
            Color::from_rgba(28, 72, 92, 255),
        );
        for (i, (label, value)) in [
            ("SCORE", format!("{:07}", g.score)),
            ("RECORD", format!("{:07}", g.high)),
            ("SECTEUR", format!("{:02}", g.stage)),
            ("VIES", format!("{:02}", g.lives.max(0))),
        ]
        .iter()
        .enumerate()
        {
            let x = v.x + (8. + i as f32 * 64.) * s;
            draw_text(label, x, v.y + 8. * s, 4.2 * s, muted);
            draw_text(
                value,
                x,
                v.y + 18. * s,
                8. * s,
                if i >= 2 { cyan } else { WHITE },
            );
        }
        draw_rectangle(
            v.x,
            v.y + 200. * s,
            v.w,
            24. * s,
            Color::from_rgba(7, 13, 26, 255),
        );
        for (i, name) in [
            "VITESSE", "MISSILE", "DOUBLE", "LASER", "OPTION", "BOUCLIER",
        ]
        .iter()
        .enumerate()
        {
            let active = g.meter == i + 1;
            let owned = match i {
                0 => g.speed > 0,
                1 => g.missile,
                2 => g.double,
                3 => g.laser,
                4 => g.options > 0,
                _ => g.shield > 0,
            };
            let r = Rect::new(
                v.x + (2. + i as f32 * 42.) * s,
                v.y + 204. * s,
                40. * s,
                15. * s,
            );
            Self::panel(
                r,
                if active {
                    Color::from_rgba(71, 51, 19, 255)
                } else if owned {
                    Color::from_rgba(13, 44, 61, 255)
                } else {
                    Color::from_rgba(14, 23, 39, 255)
                },
            );
            let c = if active {
                Color::from_rgba(255, 205, 103, 255)
            } else if owned {
                cyan
            } else {
                muted
            };
            Self::centered(name, r.x + r.w / 2., r.y + 6.2 * s, 4. * s, c);
            let state = match i {
                0 => format!("NIV. {}", g.speed),
                4 => format!("{} / 2", g.options),
                5 => format!("{} / 10", g.shield),
                _ => {
                    if owned {
                        "ACTIF".into()
                    } else {
                        "-".into()
                    }
                }
            };
            Self::centered(&state, r.x + r.w / 2., r.y + 12. * s, 3.2 * s, c);
        }
        let progress = (g.scroll / (g.data.stages[g.stage].end as f32 * 8.)).clamp(0., 1.);
        draw_rectangle(v.x, v.y + 223. * s, v.w * progress, 0.6 * s, cyan);
    }
    fn light(&self, p: Vec2, r: f32, c: Color) {
        draw_texture_ex(
            &self.glow,
            p.x - r,
            p.y - r,
            c,
            DrawTextureParams {
                dest_size: Some(vec2(r * 2., r * 2.)),
                ..Default::default()
            },
        );
    }
    fn particles(&self, g: &Game, v: Rect) {
        let scale = v.w / 256.;
        let point = |x: f32, y: f32| vec2(v.x + x * scale, v.y + (y + 24.) * scale);
        if !matches!(
            g.mode,
            Mode::Title | Mode::Dead | Mode::GameOver | Mode::Victory
        ) {
            let p = point(g.player.x - 1., g.player.y + 8.);
            let length = (6. + (g.frame as f32 * 0.3).sin() * 1.2) * scale;
            self.light(p, 8. * scale, Color::new(0.05, 0.55, 1., 0.42));
            draw_triangle(
                p + vec2(0., -1.3 * scale),
                p - vec2(length, 0.),
                p + vec2(0., 1.3 * scale),
                Color::new(0.1, 0.65, 1., 0.65),
            );
            draw_triangle(
                p + vec2(0., -0.6 * scale),
                p - vec2(length * 0.6, 0.),
                p + vec2(0., 0.6 * scale),
                Color::new(0.8, 0.96, 1., 0.9),
            );
        }
        for shot in &g.shots {
            if !(0. ..176.).contains(&shot.p.y) {
                continue;
            }
            self.light(
                point(shot.p.x, shot.p.y),
                if shot.friendly {
                    3.5 * scale
                } else {
                    4.5 * scale
                },
                if shot.friendly {
                    Color::new(0.15, 0.75, 1., 0.32)
                } else {
                    Color::new(1., 0.25, 0.04, 0.45)
                },
            );
        }
        for fx in &g.fx {
            let life = 1. - fx.age as f32 / 24.;
            let center = point(fx.p.x + 8., fx.p.y + 8.);
            self.light(
                center,
                (10. + fx.age as f32 * 0.4) * scale,
                Color::new(1., 0.35, 0.05, life * 0.55),
            );
            for i in 0..12 {
                let angle = i as f32 * 2.399 + fx.p.x;
                let d = vec2(angle.cos(), angle.sin());
                let p = center + d * fx.age as f32 * 0.65 * scale;
                draw_line(
                    p.x,
                    p.y,
                    p.x - d.x * 2. * scale,
                    p.y - d.y * 2. * scale,
                    0.6 * scale,
                    Color::new(1., 0.7, 0.2, life),
                );
            }
        }
    }
    pub fn export_scale(&self, path: &std::path::Path) {
        self.four
            .texture
            .get_texture_data()
            .export_png(path.to_str().unwrap());
    }
}
const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying highp vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main(){ gl_Position=Projection*Model*vec4(position,1.0);uv=texcoord; }
"#;
const SCALE: &str = r#"#version 100
precision highp float;
varying highp vec2 uv;
uniform sampler2D Texture;
uniform vec2 InputSize;
bool same(vec4 a,vec4 b){return all(equal(a,b));}
vec4 sampleAt(vec2 p){return texture2D(Texture,(clamp(p,vec2(0.0),InputSize-vec2(1.0))+0.5)/InputSize);}
void main(){
 vec2 p=floor(uv*InputSize); vec2 q=fract(uv*InputSize);
 vec4 E=sampleAt(p),B=sampleAt(p+vec2(0.0,-1.0)),H=sampleAt(p+vec2(0.0,1.0));
 vec4 D=sampleAt(p+vec2(-1.0,0.0)),F=sampleAt(p+vec2(1.0,0.0));
 vec4 result=E;
 if(!same(B,H)&&!same(D,F)){
  if(q.y<0.5){ if(q.x<0.5){if(same(D,B))result=D;}else{if(same(B,F))result=F;} }
  else{ if(q.x<0.5){if(same(D,H))result=D;}else{if(same(H,F))result=F;} }
 }
 gl_FragColor=result;
}
"#;
const FINISH: &str = r#"#version 100
precision highp float;
varying highp vec2 uv;
uniform sampler2D Texture;
uniform vec3 Scene;
uniform float Effects;
float hash(vec2 p){return fract(sin(dot(p,vec2(127.1,311.7)))*43758.5453);}
float noise(vec2 p){vec2 i=floor(p),f=fract(p);f=f*f*(3.0-2.0*f);return mix(mix(hash(i),hash(i+vec2(1,0)),f.x),mix(hash(i+vec2(0,1)),hash(i+vec2(1,1)),f.x),f.y);}
vec3 bright(vec2 p){vec3 c=texture2D(Texture,p).rgb;return max(c-vec3(0.48),vec3(0.0));}
void main(){
 vec3 base=texture2D(Texture,uv).rgb;
 if(Effects<0.5){gl_FragColor=vec4(base,1.0);return;}
 vec2 game=vec2(uv.x,1.0-uv.y)*vec2(256.,224.);
 float field=mix(1.0,step(24.,game.y)*step(game.y,200.),Scene.z);
 float empty=1.0-smoothstep(0.0,0.12,max(base.r,max(base.g,base.b)));
 vec2 space=game+vec2(Scene.x*0.16,0.0);
 float mist=noise(space*0.018)*noise(space*0.037+8.0);
 vec3 sky=vec3(0.004,0.008,0.023)+mist*vec3(0.018,0.025,0.065);
 for(int i=0;i<2;i++){
  vec2 p=(game+vec2(Scene.x*(0.08+float(i)*0.07),0.0))/(12.0+float(i)*9.0);
  vec2 cell=floor(p),f=fract(p);float h=hash(cell+float(i)*29.);
  float d=length(f-vec2(h,hash(cell+3.4)));
  sky+=vec3(0.38,0.65,0.92)*smoothstep(0.055,0.0,d)*step(0.56,h)*(0.45+0.15*sin(Scene.y*0.6+h*9.));
 }
 base+=sky*empty*field;
 vec2 t=vec2(1./1024.,1./896.)*2.2;
 vec3 glow=(bright(uv+vec2(t.x,0.))+bright(uv-vec2(t.x,0.))+bright(uv+vec2(0.,t.y))+bright(uv-vec2(0.,t.y)))*0.075;
 glow+=(bright(uv+t)+bright(uv-t)+bright(uv+vec2(t.x,-t.y))+bright(uv+vec2(-t.x,t.y)))*0.045;
 vec3 color=base+glow;
 float l=dot(color,vec3(0.2126,0.7152,0.0722));color=mix(vec3(l),color,1.07);
 vec2 edge=(uv-0.5)*2.;color*=1.0-0.045*dot(edge,edge);
 gl_FragColor=vec4(clamp(color,0.0,1.0),1.0);
}
"#;
