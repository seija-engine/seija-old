/*
#[test]
fn build_shader() {
    use std::io::{Read};
    use std::ops::{Add};

    fn wirte_one(file:&str,shader_type:glsl_to_spirv::ShaderType) {
        let path = String::from("src/render/shaders/");
        let byte_path = String::from("src/render/shaders/compiled/");
        let fs_vert = std::fs::read_to_string(path.add(file)).unwrap();
        let mut fs_code = glsl_to_spirv::compile(fs_vert.as_str(), shader_type).unwrap();
        let mut code_byte:Vec<u8> = Vec::new();
        fs_code.read_to_end(&mut code_byte).unwrap();
        std::fs::write(byte_path.add(file).add(".spv"), code_byte).unwrap();
    }

    wirte_one("sprite.vert", glsl_to_spirv::ShaderType::Vertex);
    wirte_one("sprite.frag", glsl_to_spirv::ShaderType::Fragment);
}

#[test]
fn test_event() {
    use shrev::{EventChannel,ReaderId,EventIterator};
    let mut ec:EventChannel<i32> = EventChannel::new();
    let mut reader: ReaderId<i32> = ec.register_reader();
    ec.single_write(1);
    ec.single_write(2);
    let eviter:EventIterator<i32> = ec.read(&mut reader);
    for ev in eviter {
        println!("{:?}",ev);
    }
    ec.single_write(3);
}*/