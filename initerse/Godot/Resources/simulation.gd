extends Node2D

var buffer : RID
var rd : RenderingDevice

func to_float(vecs) -> PackedFloat32Array:
	var floats = PackedFloat32Array()
	for vec in vecs:
		floats.append(vec.x)
		floats.append(vec.y)
		floats.append(vec.z)
	return floats

func to_vecs(floats: PackedFloat32Array) -> Array:
	var vecs = Array()
	for i in range(int(ceil(float(floats.size())/3))):
		vecs.append(Vector3(
			floats[i*3+0],
			floats[i*3+1],
			floats[i*3+2],
		))
	return vecs

func _ready():
	rd = RenderingServer.create_local_rendering_device()
	var shader_file := load("res://Resources/compute.glsl")
	var shader_spirv: RDShaderSPIRV = shader_file.get_spirv()
	var shader := rd.shader_create_from_spirv(shader_spirv)

	var particles = [Vector3(2., 10., 2.), Vector3(3., 4., 5.), ]

	var input_bytes := to_float(particles).to_byte_array()
	input_bytes.insert(0, 0)
	input_bytes.insert(0, 0)
	input_bytes.insert(0, 0)
	input_bytes.insert(0, particles.size()&0xFF)
	buffer = rd.storage_buffer_create(input_bytes.size(), input_bytes)

	# Create a uniform to assign the buffer to the rendering device
	var uniform := RDUniform.new()
	uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
	uniform.binding = 0 # this needs to match the "binding" in our shader file
	uniform.add_id(buffer)
	var uniform_set := rd.uniform_set_create([uniform], shader, 0) # the last parameter (the 0) needs to match the "set" in our shader file

	# Create a compute pipeline
	var pipeline := rd.compute_pipeline_create(shader)
	var compute_list := rd.compute_list_begin()
	rd.compute_list_bind_compute_pipeline(compute_list, pipeline)
	rd.compute_list_bind_uniform_set(compute_list, uniform_set, 0)
	rd.compute_list_dispatch(compute_list, 5, 1, 1)
	rd.compute_list_end()

	# Submit to GPU and wait for sync
	rd.submit()
	rd.sync()
	
	# Read back the data from the buffer
	var output_bytes := rd.buffer_get_data(buffer)
	var output := output_bytes.to_float32_array()
	output.remove_at(0)
	var o_parts := to_vecs(output)
	print("Input: ", particles)
	print("Output: ", o_parts)

## Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
	#pass
