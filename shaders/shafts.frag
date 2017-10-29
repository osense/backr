#ifdef GL_ES
precision mediump float;
#endif

#define STEPS 16
#define LIGHTPASSES 1
#define SPEED 0.002
uniform float time;
uniform vec2 mouse;
uniform vec2 resolution;

vec2 c = vec2(0.5,0.5*resolution.y/resolution.x);
vec3 wallColor = vec3(1.,1.,1.);

//Modified by Robobo1221

/*by Olivier de Schaetzen (citiral)
haven't really seen anything like this before, so either I don't check enough shaders or I might have made something original once ;)
*/

//random function not by me, found it somewhere on the internet!

float bayer2(vec2 a){
    a = floor(a);
    return fract(dot(a,vec2(.5, a.y*.75)));
}

float bayer4(vec2 a)   {return bayer2( .5*a)   * .25     + bayer2(a); }
float bayer8(vec2 a)   {return bayer4( .5*a)   * .25     + bayer2(a); }
float bayer16(vec2 a)  {return bayer4( .25*a)  * .0625   + bayer4(a); }
float bayer32(vec2 a)  {return bayer8( .25*a)  * .0625   + bayer4(a); }
float bayer64(vec2 a)  {return bayer8( .125*a) * .015625 + bayer8(a); }

vec3 getColor(vec2 p)
{	
	
	vec3 actualColor = vec3(smoothstep(0.5, 0.8, length(sin(p * 50.0))));
	
	vec3 color = vec3(1.0 - actualColor);
	color = mix(color, vec3(1.0), smoothstep(0.1, 0.11, vec3(length(p-c))));	
	
	return color;
}

vec3 getLighting(vec2 p, vec2 lp)
{
	vec2 sp = p;
	vec2 v = (lp-p)/float(STEPS);
	vec3 resultWeight = vec3(0.0);
	vec3 totalResult = vec3(0.0);
	vec3 result = vec3(0.0);
	
	float dither = bayer16(p * resolution / vec2(1.0, resolution.y/resolution.x));
	
	for (int i = 0 ; i < STEPS ; i++) {
		result += getColor(sp + dither * v);
		sp += v;
		
	}
	
	return result / float(STEPS);
}

void main( void ) {

	vec2 p = gl_FragCoord.xy/resolution.xy;
	float t = time * SPEED + 2.;
	vec2 lp = vec2(cos(t), sin(t * 1.14)) * 0.15 + 0.5;
	p.y *= resolution.y/resolution.x;
	lp.y *= resolution.y/resolution.x;
	
	float mie = 0.2 / distance(p, lp) - 0.2;
	vec3 color = getLighting(p,lp) * mie;
	color /= color + 1.0;
	
	
	gl_FragColor = vec4(color,1.);
}

