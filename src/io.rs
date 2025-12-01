use alloc::string::String;
use alloc::vec::Vec;
use rast::tint::Srgb;

use crate::math::*;

pub fn debug_read_file(path: &str) -> Option<Vec<u8>> {
    extern crate std;
    std::fs::read(path).ok()
}

pub fn debug_read_file_to_string(path: &str) -> Option<String> {
    extern crate std;
    std::fs::read_to_string(path).ok()
}

/// File should contain only interleaved, 2 channel, i16 audio data at 44100kHz.
pub fn debug_audio_file(path: &str) -> Option<Vec<i16>> {
    let bytes = debug_read_file(path)?;
    assert!(bytes.len() % 2 == 0);
    let (ptr, length, capacity) = bytes.into_raw_parts();
    Some(unsafe { Vec::from_raw_parts(ptr as *mut i16, length / 2, capacity / 2) })
}

pub fn debug_image_file(path: &str) -> Option<(usize, usize, Vec<Srgb>)> {
    let bytes = debug_read_file(path)?;
    assert!(bytes.len() % 4 == 0);
    let width = u32::from_le_bytes(bytes[..4].try_into().unwrap());
    let height = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    assert_eq!(width * height, (bytes.len() as u32 - 8) / 4);
    let mut pixels = alloc::vec![Srgb::from_rgb(0, 0, 0); (bytes.len() - 8) / 4];
    for (i, rgba) in bytes[8..].chunks(4).enumerate() {
        pixels[i] = Srgb::new(rgba[0], rgba[1], rgba[2], rgba[3]);
    }
    Some((width as usize, height as usize, pixels))
}

// # List of geometric vertices, with (x, y, z, [w]) coordinates, w is optional and defaults to 1.0.
// v 0.123 0.234 0.345 1.0
// v ...
// ...
// # List of texture coordinates, in (u, [v, w]) coordinates, these will vary between 0 and 1. v, w are optional and default to 0.
// vt 0.500 1 [0]
// vt ...
// ...
// # List of vertex normals in (x,y,z) form; normals might not be unit vectors.
// vn 0.707 0.000 0.707
// vn ...
// ...
// # Parameter space vertices in (u, [v, w]) form; free form geometry statement (see below)
// vp 0.310000 3.210000 2.100000
// vp ...
// ...
// # Polygonal face element (see below)
// f 1 2 3
// f 3/1 4/2 5/3
// f 6/4/1 3/5/3 7/6/5
// f 7//1 8//2 9//3
// f ...
// ...
// # Line element (see below)
// l 5 8 1 2 4 9
pub fn debug_obj_file(
    path: &str,
    materials: Vec<(String, (usize, usize, Vec<Srgb>))>,
) -> Option<crate::model::Model> {
    let obj = debug_read_file_to_string(path)?;

    let (millis, (faces, face_textures, verts, uvs, textures)) = glazer::debug_time_millis(|| {
        fn read<F: core::str::FromStr>(input: &mut &str) -> Option<F> {
            let split_at = input.chars().position(|c| c.is_whitespace());
            match split_at {
                Some(split_at) => {
                    let result = input[..split_at].parse().ok();
                    *input = &input[split_at + 1..];
                    result
                }
                None => input.parse().ok(),
            }
        }

        fn eat_line<'a>(input: &mut &'a str) -> &'a str {
            let to = input
                .char_indices()
                .find_map(|(i, c)| (c == '\n').then_some(i))
                .unwrap_or_else(|| input.len());
            let out = &input[..to];
            *input = &input[(to + 1).min(input.len())..];
            out
        }

        let mut faces = Vec::new();
        let mut face_textures = Vec::new();
        let mut verts = Vec::new();
        let mut uvs = Vec::new();
        let mut textures = Vec::new();

        let mut texture_index = 0;

        let input = &mut obj.as_str();
        while !input.is_empty() {
            let line = eat_line(input);
            if line.starts_with("v ") {
                let input = &mut &line[2..];
                let p1 = read::<f32>(input).unwrap();
                let p2 = read(input).unwrap();
                let p3 = read(input).unwrap();
                verts.push(Vec3::new(p1, p2, p3));
            } else if line.starts_with("vt ") {
                let input = &mut &line[3..];
                let p1 = read::<f32>(input).unwrap();
                let p2 = read(input).unwrap();
                uvs.push(Vec2::new(p1, p2));
            } else if line.starts_with("f ") {
                let input = &mut &line[2..];
                if line.contains("/") {
                    for vset in input.split_whitespace() {
                        for (i, mut v) in vset.split("/").enumerate() {
                            if i == 0 {
                                faces.push(read::<usize>(&mut v).unwrap() - 1);
                            } else if i == 1 {
                                face_textures
                                    .push((read::<usize>(&mut v).unwrap() - 1, texture_index));
                            }
                        }
                    }
                } else {
                    let v1 = read::<usize>(input).unwrap();
                    let v2 = read::<usize>(input).unwrap();
                    let v3 = read::<usize>(input).unwrap();
                    faces.extend([v1 - 1, v2 - 1, v3 - 1]);
                }
            } else if line.starts_with("usemtl") {
                if !textures.is_empty() {
                    texture_index += 1;
                }
                textures.push(
                    materials
                        .iter()
                        .find_map(|(name, texture)| (name == &line[7..]).then_some(texture))
                        .cloned()
                        .unwrap_or_else(|| panic!("failed to find obj mtl: `{}`", &line[7..])),
                );
            }
        }

        (faces, face_textures, verts, uvs, textures)
    });

    glazer::log!(
        "obj `{}`: loaded {} verts, {} faces in {millis:.2}ms",
        path,
        verts.len(),
        faces.len()
    );

    Some(crate::model::Model {
        faces,
        face_textures,
        verts,
        uvs,
        textures,
    })
}
