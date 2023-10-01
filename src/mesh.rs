use cgmath::EuclideanSpace;
use json::JsonValue;
use std::{fs, collections::HashMap};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3+3]>() as wgpu::BufferAddress, // Note that we offset 3 + 3 here
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ],
        }
    }
}

pub struct Mesh {
    pub verts: Vec<Vertex>,
    pub indices: Vec<u16>,
}

fn f32_from_buffer_slice(offset: usize, slice: &[u8]) -> f32 {
    let mut f32_data: [u8; 4] = [0; 4];
    f32_data.copy_from_slice(&slice[offset..=offset+3]);
    f32::from_le_bytes(f32_data)
}

fn get_attributes_buffer_slice<'a>(buffers: &'a Vec<Vec<u8>>, name: &str, attributes: &JsonValue, accessors: &Vec<&JsonValue>, buffer_views: &Vec<&JsonValue>) -> &'a [u8] {
    let position_attribute_value = &attributes[name];
    assert!(*position_attribute_value != json::Null);
    let position_accessor_index = position_attribute_value.as_usize().unwrap();
    // Only handle positions being floats for now.
    assert!(accessors[position_accessor_index]["componentType"] == 5126);
    let position_buffer_view = buffer_views[accessors[position_accessor_index]["bufferView"].as_usize().unwrap()];
    let position_buffer_offset = position_buffer_view["byteOffset"].as_usize().unwrap();
    let position_buffer_length = position_buffer_view["byteLength"].as_usize().unwrap();
    &buffers[position_buffer_view["buffer"].as_usize().unwrap()][position_buffer_offset..position_buffer_offset+position_buffer_length]
}

impl Mesh {
    pub fn load_gltf (path: &std::path::Path) -> Self {
        // Read and parse json
        let gltf = fs::read_to_string(path).unwrap();
        let gltf_json = json::parse(&gltf).unwrap();

        // Check so it's glTF 2.0
        assert!(gltf_json["asset"]["version"].as_str().unwrap() == "2.0");

        // Load all buffers referenced by this json
        let mut buffers = Vec::<Vec<u8>>::new();
        for i in 0..gltf_json["buffers"].len() {
            println!("{:?}", gltf_json["buffers"][i]["uri"]);
            let buffer_name = gltf_json["buffers"][i]["uri"].as_str().unwrap();
            buffers.push(fs::read(path.parent().unwrap().join(buffer_name)).unwrap())
        }

        // println!("{:?}", buffer_data);

        // TODO: Do i need this? does it even matter?
        // Load accessors
        let mut accessors = Vec::<&json::JsonValue>::new();
        for accessor in  gltf_json["accessors"].members() {
            accessors.push(accessor);
        }

        // Load bufferViews
        let mut buffer_views = Vec::<&json::JsonValue>::new();
        for buffer_view in gltf_json["bufferViews"].members() {
            buffer_views.push(buffer_view);
        }

        let mut verts = Vec::<Vertex>::new();
        let mut indices = Vec::<u16>::new();

        // Read all meshes
        let meshes = &gltf_json["meshes"];
        for i in 0..meshes.len() {
            let mesh = &meshes[i];
            let name = &mesh["name"].as_str().unwrap();

            // Submeshes, or whatever you want to call them
            for primitive in mesh["primitives"].members() {
                // Expect that there is always a position attribute so we use that to figure out the length of our buffer
                let attributes = &primitive["attributes"];

                // Handle positions
                let position_attribute_value = &attributes["POSITION"];
                assert!(*position_attribute_value != json::Null);
                let position_accessor_index = position_attribute_value.as_usize().unwrap();
                let vertex_count = accessors[position_accessor_index]["count"].as_usize().unwrap();
                
                // Handle positions
                let position_buffer = get_attributes_buffer_slice(&buffers, "POSITION", &attributes, &accessors, &buffer_views);
                // Handle uv
                let uv_buffer = get_attributes_buffer_slice(&buffers, "TEXCOORD_0", &attributes, &accessors, &buffer_views);


                // Handle color

                for i in 0..vertex_count {
                    println!("{}", i*12);
                    verts.push(Vertex {
                        position: [f32_from_buffer_slice(i*12, &position_buffer), f32_from_buffer_slice(i*12+4, &position_buffer), f32_from_buffer_slice(i*12+8, &position_buffer)],
                        color: [1.0, 1.0, 1.0],
                        uv: [f32_from_buffer_slice(i*8, &uv_buffer), f32_from_buffer_slice(i*8+4, &uv_buffer)],
                    })
                }

                println!("{:?}", verts);

                // for attribute in primitive["attributes"].entries() {
                //     let accessor_index = attribute.1.as_usize().unwrap();
                //     // println!("Key: {:?} value: {:?}", attribute.0, accessor_index);

                //     let accessor = accessors[accessor_index];
                //     let buffer_view_index = accessor["bufferView"].as_usize().unwrap();

                //     let vertex_count = accessor["count"].as_usize().unwrap();

                //     let buffer_index = buffer_views[buffer_view_index]["buffer"].as_usize().unwrap();
                //     let buffer_length = buffer_views[buffer_view_index]["byteLength"].as_usize().unwrap();
                //     let buffer_offset = buffer_views[buffer_view_index]["byteOffset"].as_usize().unwrap();

                //     println!("buffer: {:?}, length: {:?}, offset: {:?}", buffer_index, buffer_length, buffer_offset);
                // }

                let indices_accessor_index = primitive["indices"].as_usize().unwrap();
                let indices_buffer_index = buffer_views[indices_accessor_index]["buffer"].as_usize().unwrap();
                let indices_buffer_length = buffer_views[indices_accessor_index]["byteLength"].as_usize().unwrap();
                let indices_buffer_offset = buffer_views[indices_accessor_index]["byteOffset"].as_usize().unwrap();
                println!("buffer: {:?}, length: {:?}, offset: {:?}", indices_buffer_index, indices_buffer_length, indices_buffer_offset);
                let index_count = accessors[indices_accessor_index]["count"].as_usize().unwrap();
                let index_buffer = &buffers[indices_buffer_index][indices_buffer_offset..indices_buffer_offset+indices_buffer_length];
                for i in 0..index_count {
                    indices.push(index_buffer[i*2] as u16 + ((index_buffer[i*2+1] as u16) << 8));
                }
                println!("{:?}", indices);
            }

            
            // for attribute in mesh["primitives"]["attributes"].entries() {
            //     println!("Key: {:?} value: {:?}", attribute.0, attribute.1)
            // }
        }
        Self {
            indices,
            verts,
        }
    }
}

pub struct Instance {
    pub position: cgmath::Point3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    pub fn to_data(&self) -> InstanceData {
        InstanceData {
            transform: (cgmath::Matrix4::from_translation(self.position.to_vec()) * cgmath::Matrix4::from(self.rotation)).into()
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    transform: [[f32; 4]; 4],
}

impl InstanceData {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
