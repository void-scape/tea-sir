use alloc::string::String;
use alloc::vec::Vec;

use crate::math::*;

pub fn debug_read_file(path: &str) -> Option<Vec<u8>> {
    #[cfg(target_os = "macos")]
    {
        extern crate std;
        std::fs::read(path).ok()
    }
}

pub fn debug_read_file_to_string(path: &str) -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        extern crate std;
        std::fs::read_to_string(path).ok()
    }
}

/// File should contain only interleaved, 2 channel, i16 audio data at 44100kHz.
pub fn debug_audio_file(path: &str) -> Option<Vec<i16>> {
    let bytes = debug_read_file(path)?;
    assert!(bytes.len() % 2 == 0);
    let (ptr, length, capacity) = bytes.into_raw_parts();
    Some(unsafe { Vec::from_raw_parts(ptr as *mut i16, length / 2, capacity / 2) })
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
pub fn debug_obj_file(path: &str) -> Option<crate::model::Model> {
    let obj = debug_read_file_to_string(path)?;

    let (millis, (verts, faces)) = glazer::debug_time_millis(|| {
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

        let mut verts = Vec::new();
        for mut slice in obj.split("v ").filter(|str| !str.is_empty()) {
            let input = &mut slice;
            let p1 = read::<f32>(input).unwrap();
            let p2 = read(input).unwrap();
            let p3 = read(input).unwrap();
            verts.push(Vec3::new(p1, p2, p3));
        }

        let mut faces = Vec::new();
        for mut slice in obj
            .split("f ")
            .filter(|str| !str.is_empty() && !str.starts_with("v "))
        {
            let input = &mut slice;
            let v1 = read::<usize>(input).unwrap();
            let v2 = read::<usize>(input).unwrap();
            let v3 = read::<usize>(input).unwrap();
            faces.extend([v1 - 1, v2 - 1, v3 - 1]);
        }

        (verts, faces)
    });

    glazer::log!(
        "obj: loaded {} verts, {} faces in {millis:.2}ms",
        verts.len(),
        faces.len()
    );

    Some(crate::model::Model { faces, verts })
}
