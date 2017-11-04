#line 2
#extension GL_ARB_texture_query_lod : enable
#extension GL_ARB_gpu_shader_fp64 : enable

uniform mat4 modelViewProjection;

uniform sampler2DArray colors;
uniform sampler2DArray normals;
uniform sampler2DArray water;
uniform sampler2DArray materials;
uniform samplerCube sky;

uniform sampler2DArray oceanSurface;
uniform sampler2D noise;
uniform float noiseWavelength;
uniform vec3 cameraPosition;

in vec3 fPosition;
in vec2 fTexcoord;
in float fColorsLayer;
in float fNormalsLayer;
in float fWaterLayer;

out vec4 OutColor;

vec4 fractal(vec2 pos) {
	vec4 value = vec4(0.0);
	float scale = 0.5;
	float wavelength = 2.0;
	for(int i = 0; i < 5; i++) {
		vec4 v = texture(noise, pos * noiseWavelength / wavelength) * 6 - 3;
		value += v * scale;
		scale *= 0.5;
		wavelength *= 0.5;
	}
	return value;
}

float fractal2(vec2 pos) {
	float value = 0.0;
	float scale = 1.0 / 10;
	float wavelength = 64.0;
	for(int i = 0; i < 10; i++) {
		vec3 v = texture(noise, pos * noiseWavelength / wavelength + vec2(0.123 * i)).rgb;
		value += v.x * scale;
		// scale *= 0.5;
		wavelength *= 0.5;
	}
	return value;
}

float compute_fog(vec3 position) {
	// NOTE: When `-b*height` gets close to -88, `exp(-b*height)` would round to
	// zero. This means that when the camera is above around 17000 meters
	// elevation, the fog would vanish. To prevent this, we clamp the height to
	// 15000 meters.
	float MAX_HEIGHT = 15000;

	float b = 0.005;
	float distance = distance(position, cameraPosition);
	vec3 rayDir = normalize(position - cameraPosition);
	float height = cameraPosition.y;
	if(height > MAX_HEIGHT) {
		distance *= MAX_HEIGHT / height;
		height = MAX_HEIGHT;
	}
	return clamp(0.05 * (exp(-b*height) * (1.0 - exp(-b*rayDir.y*distance))) / rayDir.y, 0, 1.0);
}

vec3 material(vec3 pos, uint mat) {
	return texture(materials, vec3(pos.xz * 0.5, mat)).rgb;// * (1.0 + fractal2(pos.xz) * 0.2);
}

vec3 compute_splatting(vec3 pos, vec2 t) {
	//	t += 0.0001 * fractal(pos.xz).xy * 10;

	vec2 weights = fract(t.xy * textureSize(normals, 0).xy - 0.5);
	uvec4 m = uvec4(ceil(textureGather(normals, vec3(t, fNormalsLayer), 3) * 255));
	vec4 w = mix(mix(vec4(0,0,0,1), vec4(1,0,0,0), weights.y),
				 mix(vec4(0,0,1,0), vec4(0,1,0,0), weights.y), weights.x);

	return material(pos, m.x) * w.x +
		material(pos, m.y) * w.y +
		material(pos, m.z) * w.z +
		material(pos, m.w) * w.w;
}
void main() {
	if(fNormalsLayer >= 0) {
		vec3 normal = normalize(texture(normals, vec3(fTexcoord, fNormalsLayer)).xyz * 2.0 - 1.0);

		OutColor.rgb = compute_splatting(fPosition, fTexcoord);
		OutColor.rgb *= dot(normal, normalize(vec3(0,1,1)));
	} else {
		vec4 color = texture(colors, vec3(fTexcoord, fColorsLayer));

		OutColor = vec4(color.rgb, 1);
		OutColor.rgb *= color.a;
	}

	float waterAmount = texture(water, vec3(fTexcoord, fWaterLayer)).x;
	if(waterAmount > 0) {
		vec3 ray = normalize(fPosition - cameraPosition);
		vec3 normal = texture(oceanSurface, vec3(fPosition.xz * 0.0001, 0)).xzy * 2 - 1;
		vec3 reflected = reflect(ray, normalize(normal));

		vec3 reflectedColor = texture(sky, normalize(reflected)).rgb;
		vec3 refractedColor = vec3(0,0.1,0.2);

		float R0 = pow(0.333 / 2.333, 2);
		float R = R0 + (1 - R0) * pow(1 - reflected.y, 5);
		vec3 waterColor = mix(refractedColor, reflectedColor, R);
		OutColor.rgb = mix(OutColor.rgb, waterColor, waterAmount);
	}

	OutColor.rgb = mix(OutColor.rgb, vec3(0.6), compute_fog(fPosition));
}
