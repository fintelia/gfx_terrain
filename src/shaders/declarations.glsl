#extension GL_EXT_samplerless_texture_functions: require

struct Globals {
    mat4 view_proj;
	mat4 view_proj_inverse;
	vec4 frustum_planes[5];
	vec3 camera;
	vec3 sun_direction;
};

struct LayerDesc {
	vec3 origin;
	float _step;
	vec3 parent_origin;
	float parent_step;
};
struct NodeState {
    LayerDesc displacements;
	LayerDesc albedo;
	LayerDesc roughness;
	LayerDesc normals;
	vec3 grass_canopy_origin;
	float grass_canopy_step;
	uint resolution;
	uint face;
	uint level;
	uint node_index;
	vec3 relative_position;
	float min_distance;
	vec3 parent_relative_position;
	float padding1;
	vec4 padding2[4];
};

struct Indirect {
    uint vertex_count;
    uint instance_count;
    uint base_index;
    uint vertex_offset;
    uint base_instance;
};

float extract_height(uint encoded) {
	return (float(encoded & 0x7fffff) * (1 / 512.0)) - 1024.0;
}
float extract_height_above_water(uint encoded) {
	float height = extract_height(encoded);
	if ((encoded & 0x800000) != 0) {
		height = max(height, 0);
	}
	return height;
}