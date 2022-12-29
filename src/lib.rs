extern crate image;
extern crate pollster;
extern crate wgpu;

pub mod object;
pub mod state;
pub mod texture;
pub mod vertex;

use image::io::Reader as ImageReader;
use object::Object;
use state::State;
use vertex::Vertex;

pub async fn run() {
    let mut st = State::new(2048, 2048).await;
    let test_imgs = read_test_imgs();
    let mut index = 0usize;
    let vers: Vec<Vec<Vertex>> = vec![
        vec![
            Vertex {
                position: [-1.0, 1.0, 0.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [-1.0, 0.0, 0.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [0.0, 0.0, 0.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [0.0, 1.0, 0.0],
                tex_coords: [1.0, 0.0],
            },
        ],
        vec![
            Vertex {
                position: [0.0, 1.0, 0.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [0.0, 0.0, 0.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, 0.0, 0.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0, 0.0],
                tex_coords: [1.0, 0.0],
            },
        ],
        vec![
            Vertex {
                position: [-1.0, 0.0, 0.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [-1.0, -1.0, 0.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [0.0, -1.0, 0.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [0.0, 0.0, 0.0],
                tex_coords: [1.0, 0.0],
            },
        ],
        vec![
            Vertex {
                position: [0.0, 0.0, 0.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [0.0, -1.0, 0.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0, 0.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [1.0, 0.0, 0.0],
                tex_coords: [1.0, 0.0],
            },
        ],
    ];
    let indices = vec![0, 1, 3, 1, 2, 3];
    let mut objs: Vec<Object> = vec![];
    while index < 4 {
        let img_path = &test_imgs[index];
        let img = ImageReader::open(img_path).unwrap().decode().unwrap();
        let tex = texture::Texture::from_image(&st.device, &st.queue, &img, Some("")).unwrap();
        let vertices = vers[index].clone();
        let obj = Object::new()
            .set_vertex_buffer(&st.device, vertices)
            .set_index_buffer(&st.device, indices.clone())
            .set_texture(Some(tex))
            .create_bind_group(&st.device)
            .create_render_pipeline(&st.device, wgpu::TextureFormat::Rgba8UnormSrgb);
        objs.push(obj);
        index += 1;
    }
    st.set_objects(objs);
    st.render().await
}

pub fn read_test_imgs() -> Vec<String> {
    let current = std::env::current_dir().unwrap();
    let mut img_dir = current.clone();
    img_dir.push("test_imgs");
    let mut result: Vec<String> = vec![];
    for entry in img_dir.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            let img_path = entry.path();
            result.push(String::from(img_path.to_string_lossy()));
        }
    }
    result
}
