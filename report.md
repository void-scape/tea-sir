## Introduction

For the last three years of my life, I have been building games with a variety of tools. First, with the Unity and Godot game engines, among the largest communities with lots of community support. Second, with the Bevy game engine, a free and open source engine with a focus on community driven contributions. The Bevy engine is newer than Unity and Godot, and as such, does not have an editor yet. Building a game without an editor presents a variety of challanges, however, the design of Bevy is better suited towards my style of programming. Third, it naturally follows, I wanted to build everything from the ground up. This means absolutely no dependencies, only those necessary to open a window. So, to this end, earlier this semester I wrote a CPU rasterizer which converts triangle shape data into image pixel data. Traditionally, rasterizers are implemented on GPUs and the CPU must provide the triangle shape data to the GPU. However, some environments, through a variety of reasons, may not provide access to GPU hardware and, ideally, we want a system that is portable to any computer will a CPU. Furthermore, 3D games in the pass have been written entirely on the CPU, for example 1998's _Thief_ and 1996's _Quake_ [1].

This issue with rendering a 3D game on the CPU is performance. The GPUs architecture is specifically designed to handle a large amount of shape and pixel data in parallel for rendering to a high resolution display. Therefore, if we want acceptable performance on the CPU, we must be careful to optimize every step of the rendering process, and ideally, employ parallelization _everywhere_. This report discusses and compares the performance of several implementations of matrix and vector mulitplication in the context of 3D rendering.

## The Problem

In 3D computer graphics, we want to convert 3D shape data into 2D shape data that can be rendered on a screen, a process known as projection. Given a point with x, y, and z coordinates, a simple perspective projection will divide the x and y components by the z component to produce the final x and y coordinates on the 2D screen. However, there are many more kinds of transformations we want to do with 3D points, like translation, rotation, and scaling. If we store the x, y, and z components of a 3D point in a vector, we can use matrix multiplication to apply rotation and scaling, however, we need a fourth dimension, w, in order to apply translation. Given a fourth dimension, we can now additionally construct a projection matrix which produces a w value which, when used to divide the x, y, and z components of a vector, computes the corresponding point on the 2D surface. Due to matrix composition, we can multiply the projection matrix with any number of transformation matrices and produce a single matrix which performs all of the necessary transformations and projections in a single operation.

An object we want to render is represented with a 3D model, which stores information about the triangles composing its surface. For instance, a cube can be modeled with twelve triangles, two for each surface for a total of six surfaces. The number of triangles in a model will depend on the level of detail. A common model used for testing 3D graphics is the Stanford Dragon which contains 871,414 triangles. A triangle is described by a set of three points and the 2D projection of that triangle is created by projecting each of the three points. Therefore, to render an image of the Stanford Dragon, we must project 2,614,242 points. 

According to Amdahl's law, the speedup of the renderer is limited by the fraction of execution time that the projection occupies. Using linux's perf to measure statistics about rendering the Stanford Dragon, I found that 8% of execution time is spent projecting. Therefore, 8% is the largest reduction to execution time we can expect. Later in the report, I will discuss performance gains relative to this 8%, meaning that a doubling of projection performance will reduce the total execution time by 4%.

## Background Information

A discussion about matrix-vector multiplication is necessary before describing how it can be optimized. This will be a practical discussion that does not justify or explain the theory of linear algebra. In memory, a four-by-four matrix is a series of 16, 32-bit floating point values. There are two potential ways to order these values. The first is row-major, where each set of four values represents a horizontal row. The second is column-major, where each set of four values represents a vertical column. To multiply a four element vector by a four-by-four, row-major matrix, each row is multiplied element wise by the vector and summed to produce the corresponding component of the output vector. The following pseudocode describes this operation:

```
fn row_major_mat4x4_mul_vec4(mat, vec) {
    let out = vec4(0, 0, 0, 0)

    out.x = dot(mat.row1, vec)
    out.y = dot(mat.row2, vec)
    out.z = dot(mat.row3, vec)
    out.w = dot(mat.row4, vec)

    return out
}

fn dot(v1, v2) {
    return (
        v1.x * v2.x 
        + v1.y * v2.y
        + v1.z * v2.z
        + v1.w * v2.w
    )
}
```

To multiply a four element vector by a four-by-four, column-major matrix, each column is scaled by the corresponding vector component and addded to the output. The following pseudocode describes this operation:

```
fn column_major_mat4x4_mul_vec4(mat, vec) {
    let out = vec4(0, 0, 0, 0)

    out = out + mat.column1 * vec.x
    out = out + mat.column2 * vec.y
    out = out + mat.column3 * vec.z
    out = out + mat.column4 * vec.w

    return out
}
```

While the previous pseudocode implementations were presented in a sequential form, with the intent of being compiled into a series of ALU instructions, they can be adapted into Single Instruction, Multiple Data (SIMD) form. SIMD is a form of parallel computing that operates on multiple values at once in a single instruction, requiring special SIMD processor hardware [2]. SIMD values are interacted with through vector registers, which contain a specific number of elements [2]. A four element vector is most efficiently stored in an 128-bit wide floating point SIMD vector register. A four-by-four matrix, similary, with four 128-bit wide floating point SIMD vector registers. In this form, the following pseudocode demonstrates a SIMD implementation of the row-major and column-major matrix-vector multiplication operations.

```
fn row_major_mat4x4_mul_vec4(mat, vec) {
    let out = create_simd_vec4(0, 0, 0, 0)

    out.x = simd_sum(mat.row1 * vec)
    out.y = simd_sum(mat.row2 * vec)
    out.z = simd_sum(mat.row3 * vec)
    out.w = simd_sum(mat.row4 * vec)

    return out
}

fn column_major_mat4x4_mul_vec4(mat, vec) {
    let out = create_simd_vec4(0, 0, 0, 0)

    xxxx = simd_splat(vec, 0)
    yyyy = simd_splat(vec, 1)
    zzzz = simd_splat(vec, 2)
    wwww = simd_splat(vec, 3)

    out = out + mat.column1 * xxxx
    out = out + mat.column2 * yyyy
    out = out + mat.column3 * zzzz
    out = out + mat.column4 * wwww

    return out
}
```

where `create_simd_vec4` allocates a new SIMD vector register and `simd_splat` takes a value in the existing `vec` SIMD vector register and creates a new register with the corresponding component copied into all of the vector values, represented as an index into the original register. There are platform specific SIMD instructions to handle both `create_simd_vec4` and `simd_splat` and are therefore represented as functions in the pseudocode. All of the addition and multiplication operators represent parallel SIMD operations.

## Procedure

I will implement the four versions of the matrix-vector multiplication described above in Rust. I will use the criterion library to measure performance statistics and evaluate the optimal implementation. I will compare my SIMD implementations with glam, a linear algebra library that utilizes SIMD with column-major matrices. Finally, I will briefly discuss the generated assembly with respect to the measured performance. The source code required to reproduce my results, as well as the assembly generated for my machine, can be found in the Appendix.

## Results

Each benchmark iterates over the all of the vertices in the Standard Dragon model and computes the projected vertex by multiplying the vertex with the projection matrix. The benchmark code does not exactly replicate the projection code used in the renderer, however, the relative speed between implementations is the only metric that matters. Table I shows the average execution time of the benchmark over ~100,000,000 iterations. 

Table I
| Matrix Representation | glam | Naive | SIMD |
|-----------------------|------|-------|------|
| Column-Major | 45.474 µs | 45.400 µs | 45.421 µs |
| Row-Major | NA | 45.245 µs | 138.06 µs |

The program is compiled with full optimizations in Table I. As such, it is clear that the column-major glam, naive, SIMD, and the row-major naive implementation are all compiled to roughly the same instructions. However, the row-major SIMD implementation is significantly slower than the other four. The generated assembly, shown in the appendix, reinforces this finding. There are only small variations in assembly, observed in Table I with very subtle differences in execution time.

There are two significant conclusions that can be drawn from the results in Table I. The first is that the LLVM, the default compiler backend for Rust, is deriving the intent of the code from the context and generating optimized assembly. The second conclusion is that, ultimately, any implementation, outside of the row-major SIMD matrix, is suitable in production for my use case. 

To circumvent LLVM's optimizations, I compiled with lower optimizations, specifically opt-level one. I chose this optimization level because it inlined all function calls in the benchmark while retaining a relatively one-to-one relationship with the code. To my suprise, the naive implementations were still being placed into the xmm registers and, unfortunately, I spent a large amount of time attempting to prevent this. My initial intution being that the compiler was auto-vectorizing the floating point operations. However, I discovered that the 

Despite these findings, the results do not reveal whether or not SIMD operations actually decrease exection time, since all of the implementations produce instructions which interfaced with the intel SSE2 extension. To circumvent the compiler from using the vector registers, I tried to pass a variety of compiler flags to LLVM but this failed to make a difference. Ultimately, I had expected the naive implementations to use the x87 FPU hardware on my x86-64 CPU, however it seems that the SSE2 extension is preffered over the x87 FPU when available. As such, I decided it was necessary to implement the naive matrix-vector multiplication with inline assembly that utilized the x87 FPU in order to compare against the SIMD architecture. Table II shows the execution time of the previous implementations in addition to the inline assembly for the naive matrix-vector multiplication.

Table II
| Matrix Representation | glam | Naive | SIMD |
|-----------------------|------|-------|------|
| Column-Major | 45.474 µs | 257.90 µs | 45.421 µs |
| Row-Major | NA | 264.60 µs | 138.06 µs |

The difference in execution time between the column-major and row-major naive implementations is small because the total number of computations does not change, only the representation of the matrix in memory. Both naive implementationscontain 48 x87 instructions, therefore the difference is most likely noise. The column-major SIMD implementation is 5.67 times faster than the colum-major naive implementation, while the row-major SIMD implementation is only 1.91 times faster than the row-major naive implementation. This speedup difference is mostly explained by the number of generated instructions. The column-major and row-major SIMD implementations generate 45 and 17 instructions, respectively. The difference of 45 and 17 is 2.64. The product of 2.64 and the speedup of the row-major SIMD implementation, 1.91, is 5.05, which is very similar to the column-major's 5.67 speedup. This is too be expected because the row-major matrix can only utilize the parallelism of SIMD for the dot product, whereas all axis in a column-major matrix can be efficiently accumulated together.

## Conclusion

In the context of the renderer, I believe the best course of action is to avoid using a hand crafted SIMD implementation due to portability concerns brought up in the introduction. SIMD will not be available in all environements and it is therefore more robust to allow the compiler to automatically vectorize the code when applicable. Furthermore, Table I suggests that the memory representation of the matrix will have no effect on performance during projection.

As to the floating point arithmetic specific to the modern x86-64 architecture, the SSE2 extension utilizing vector registers is reliably faster than the x87 FPU even with suboptimal vectorization, as seen in the row-major SIMD implementation. If this were not the case, I would imagine that producing x87 instructions with LLVM would be simpler, if possible.

## Future Work

To relate this work back to what we have discussed in lecture, I would have liked to use fixed point arithmetic to compare the x86-64 integer data path with the SSE2 SIMD data path. In fact the rasterizer itself accepts integer coordinates, so fixed point arithmetic could remove the overhead of converting floating point coordinates into integer. Matrix-vector multiplication can additionally avoid data dependency, so we wouldn't expect any stalls in the pipeline. While switching to fixed point arithmetic may potentially lead to performance gains in this context, it could lower performance elsewhere and suffer from precision loss if the size of the fractional component is too small or large.

## References

[1] https://nothings.org/gamedev/thief_rendering.html
[2] https://www.sciencedirect.com/topics/computer-science/single-instruction-multiple-data
[3] "Intel® 64 and IA-32 Architectures Software Developer's Manual Volume 1: Basic Architecture". Intel. April 2022. pp. 5-16–5-19. Archived from the original on April 25, 2022. Retrieved May 16, 2022.
[4] https://refspecs.linuxbase.org/elf/x86_64-abi-0.99.pdf

## Appendix

### Github repository containing the benchmarking and rendering code
https://github.com/void-scape/tea-sir/tree/simd-project

### Column-major glam implementation benchmark assembly
model_matrices::glam:
        .cfi_startproc
        test rdx, rdx
        je .LBB45_3
        shl rdx, 4
        movaps xmm0, xmmword ptr [rdi]
        movaps xmm1, xmmword ptr [rdi + 16]
        movaps xmm2, xmmword ptr [rdi + 32]
        movaps xmm3, xmmword ptr [rdi + 48]
        xor eax, eax
        lea rcx, [rsp - 24]
        lea rdi, [rsp - 32]
        .p2align        4
.LBB45_2:
        movaps xmm4, xmmword ptr [rsi + rax]
        movaps xmm5, xmm4
        shufps xmm5, xmm4, 0
        mulps xmm5, xmm0
        movaps xmm6, xmm4
        shufps xmm6, xmm4, 85
        mulps xmm6, xmm1
        addps xmm6, xmm5
        movaps xmm5, xmm4
        shufps xmm5, xmm4, 170
        mulps xmm5, xmm2
        addps xmm5, xmm6
        shufps xmm4, xmm4, 255
        mulps xmm4, xmm3
        addps xmm4, xmm5
        movaps xmmword ptr [rsp - 24], xmm4
        mov qword ptr [rsp - 32], rcx
        #APP
        #NO_APP
        add rax, 16
        cmp rdx, rax
        jne .LBB45_2
.LBB45_3:
        ret

### Column-major naive implementation benchmark assembly
model_matrices::naive_cm:
        .cfi_startproc
        test rdx, rdx
        je .LBB49_3
        shl rdx, 4
        add rdx, rsi
        movups xmm0, xmmword ptr [rdi]
        movups xmm1, xmmword ptr [rdi + 16]
        movups xmm2, xmmword ptr [rdi + 32]
        movups xmm3, xmmword ptr [rdi + 48]
        lea rax, [rsp - 24]
        lea rcx, [rsp - 32]
        .p2align        4
.LBB49_2:
        movss xmm4, dword ptr [rsi]
        movss xmm5, dword ptr [rsi + 4]
        movss xmm6, dword ptr [rsi + 8]
        movss xmm7, dword ptr [rsi + 12]
        shufps xmm4, xmm4, 0
        mulps xmm4, xmm0
        shufps xmm5, xmm5, 0
        mulps xmm5, xmm1
        addps xmm5, xmm4
        shufps xmm6, xmm6, 0
        mulps xmm6, xmm2
        addps xmm6, xmm5
        shufps xmm7, xmm7, 0
        mulps xmm7, xmm3
        addps xmm7, xmm6
        movaps xmmword ptr [rsp - 24], xmm7
        mov qword ptr [rsp - 32], rax
        #APP
        #NO_APP
        add rsi, 16
        cmp rsi, rdx
        jne .LBB49_2
.LBB49_3:
        ret

### Row-major naive implementation benchmark assembly
model_matrices::naive_rm:
        .cfi_startproc
        test rdx, rdx
        je .LBB50_3
        shl rdx, 4
        movups xmm4, xmmword ptr [rdi]
        movups xmm0, xmmword ptr [rdi + 16]
        movups xmm5, xmmword ptr [rdi + 32]
        movups xmm6, xmmword ptr [rdi + 48]
        movaps xmm2, xmm6
        shufps xmm2, xmm5, 16
        movaps xmm1, xmm0
        shufps xmm1, xmm4, 16
        shufps xmm1, xmm2, 34
        movaps xmm3, xmm6
        shufps xmm3, xmm5, 1
        movaps xmm2, xmm0
        shufps xmm2, xmm4, 1
        shufps xmm2, xmm3, 34
        movaps xmm7, xmm6
        unpckhpd xmm7, xmm5
        movaps xmm3, xmm4
        unpckhps xmm3, xmm0
        shufps xmm3, xmm7, 36
        unpckhps xmm5, xmm6
        shufps xmm0, xmm4, 51
        shufps xmm0, xmm5, 226
        xor eax, eax
        lea rcx, [rsp - 24]
        lea rdi, [rsp - 32]
        .p2align        4
.LBB50_2:
        movsd xmm4, qword ptr [rsi + rax]
        movaps xmm5, xmm4
        shufps xmm5, xmm4, 17
        mulps xmm5, xmm1
        movlhps xmm4, xmm4
        mulps xmm4, xmm2
        addps xmm4, xmm5
        movss xmm5, dword ptr [rsi + rax + 8]
        shufps xmm5, xmm5, 0
        mulps xmm5, xmm3
        addps xmm5, xmm4
        movss xmm4, dword ptr [rsi + rax + 12]
        shufps xmm4, xmm4, 0
        mulps xmm4, xmm0
        addps xmm4, xmm5
        movaps xmmword ptr [rsp - 24], xmm4
        mov qword ptr [rsp - 32], rcx
        #APP
        #NO_APP
        add rax, 16
        cmp rdx, rax
        jne .LBB50_2
.LBB50_3:
        ret

### Column-major SIMD implementation benchmark assembly
model_matrices::simd_cm:
        .cfi_startproc
        test rdx, rdx
        je .LBB47_3
        shl rdx, 4
        movaps xmm0, xmmword ptr [rdi]
        movaps xmm1, xmmword ptr [rdi + 16]
        movaps xmm2, xmmword ptr [rdi + 32]
        movaps xmm3, xmmword ptr [rdi + 48]
        xor eax, eax
        lea rcx, [rsp - 24]
        lea rdi, [rsp - 32]
        .p2align        4
.LBB47_2:
        movaps xmm4, xmmword ptr [rsi + rax]
        movaps xmm5, xmm4
        shufps xmm5, xmm4, 0
        mulps xmm5, xmm0
        movaps xmm6, xmm4
        shufps xmm6, xmm4, 85
        mulps xmm6, xmm1
        movaps xmm7, xmm4
        shufps xmm7, xmm4, 170
        mulps xmm7, xmm2
        shufps xmm4, xmm4, 255
        mulps xmm4, xmm3
        addps xmm4, xmm7
        addps xmm4, xmm6
        addps xmm4, xmm5
        movaps xmmword ptr [rsp - 24], xmm4
        mov qword ptr [rsp - 32], rcx
        #APP
        #NO_APP
        add rax, 16
        cmp rdx, rax
        jne .LBB47_2
.LBB47_3:
        ret

### Row-major SIMD implementation benchmark assembly
model_matrices::simd_rm:
        .cfi_startproc
        test rdx, rdx
        je .LBB48_3
        shl rdx, 4
        movaps xmm0, xmmword ptr [rdi]
        movaps xmm1, xmmword ptr [rdi + 16]
        movaps xmm2, xmmword ptr [rdi + 32]
        movaps xmm3, xmmword ptr [rdi + 48]
        xor eax, eax
        lea rcx, [rsp - 24]
        lea rdi, [rsp - 32]
        .p2align        4
.LBB48_2:
        movaps xmm5, xmmword ptr [rsi + rax]
        movaps xmm4, xmm5
        mulps xmm4, xmm0
        movaps xmm7, xmm5
        mulps xmm7, xmm1
        movaps xmm6, xmm5
        mulps xmm6, xmm2
        mulps xmm5, xmm3
        movaps xmm8, xmm4
        shufps xmm8, xmm4, 85
        addss xmm8, xmm4
        movaps xmm9, xmm4
        unpckhpd xmm9, xmm4
        addss xmm9, xmm8
        shufps xmm4, xmm4, 255
        addss xmm4, xmm9
        movaps xmm8, xmm7
        shufps xmm8, xmm7, 85
        addss xmm8, xmm7
        movaps xmm9, xmm7
        unpckhpd xmm9, xmm7
        addss xmm9, xmm8
        shufps xmm7, xmm7, 255
        addss xmm7, xmm9
        unpcklps xmm4, xmm7
        movaps xmm7, xmm6
        shufps xmm7, xmm6, 85
        addss xmm7, xmm6
        movaps xmm8, xmm6
        unpckhpd xmm8, xmm6
        addss xmm8, xmm7
        shufps xmm6, xmm6, 255
        addss xmm6, xmm8
        movaps xmm7, xmm5
        shufps xmm7, xmm5, 85
        addss xmm7, xmm5
        movaps xmm8, xmm5
        unpckhpd xmm8, xmm5
        addss xmm8, xmm7
        shufps xmm5, xmm5, 255
        addss xmm5, xmm8
        unpcklps xmm6, xmm5
        movlhps xmm4, xmm6
        movaps xmmword ptr [rsp - 24], xmm4
        mov qword ptr [rsp - 32], rcx
        #APP
        #NO_APP
        add rax, 16
        cmp rdx, rax
        jne .LBB48_2
.LBB48_3:
        ret

### Column-major naive x87 FPU implementation benchmark assembly
model_matrices::naive_cm:
        .cfi_startproc
        test rdx, rdx
        je .LBB49_4
        sub rsp, 56
        .cfi_def_cfa_offset 64
        shl rdx, 4
        xor eax, eax
        xorps xmm0, xmm0
        lea rcx, [rsp + 32]
        mov r8, rsp
        lea r9, [rsp + 16]
        .p2align        4
.LBB49_2:
        movups xmm1, xmmword ptr [rsi + rax]
        movaps xmmword ptr [rsp + 32], xmm1
        movaps xmmword ptr [rsp], xmm0
        #APP

        fld dword ptr [rdi]
        fmul dword ptr [rcx]
        fld dword ptr [rdi + 16]
        fmul dword ptr [rcx + 4]
        faddp st(1), st
        fld dword ptr [rdi + 32]
        fmul dword ptr [rcx + 8]
        faddp st(1), st
        fld dword ptr [rdi + 48]
        fmul dword ptr [rcx + 12]
        faddp st(1), st
        fstp dword ptr [r8]
        fld dword ptr [rdi + 4]
        fmul dword ptr [rcx]
        fld dword ptr [rdi + 20]
        fmul dword ptr [rcx + 4]
        faddp st(1), st
        fld dword ptr [rdi + 36]
        fmul dword ptr [rcx + 8]
        faddp st(1), st
        fld dword ptr [rdi + 52]
        fmul dword ptr [rcx + 12]
        faddp st(1), st
        fstp dword ptr [r8 + 4]
        fld dword ptr [rdi + 8]
        fmul dword ptr [rcx]
        fld dword ptr [rdi + 24]
        fmul dword ptr [rcx + 4]
        faddp st(1), st
        fld dword ptr [rdi + 40]
        fmul dword ptr [rcx + 8]
        faddp st(1), st
        fld dword ptr [rdi + 56]
        fmul dword ptr [rcx + 12]
        faddp st(1), st
        fstp dword ptr [r8 + 8]
        fld dword ptr [rdi + 12]
        fmul dword ptr [rcx]
        fld dword ptr [rdi + 28]
        fmul dword ptr [rcx + 4]
        faddp st(1), st
        fld dword ptr [rdi + 44]
        fmul dword ptr [rcx + 8]
        faddp st(1), st
        fld dword ptr [rdi + 60]
        fmul dword ptr [rcx + 12]
        faddp st(1), st
        fstp dword ptr [r8 + 12]

        #NO_APP
        mov r10, qword ptr [rsp]
        mov qword ptr [rsp + 16], r10
        mov r10, qword ptr [rsp + 8]
        mov qword ptr [rsp + 24], r10
        mov qword ptr [rsp], r9
        #APP
        #NO_APP
        add rax, 16
        cmp rdx, rax
        jne .LBB49_2
        add rsp, 56
        .cfi_def_cfa_offset 8
.LBB49_4:
        ret

### Row-major naive x87 FPU implementation benchmark assembly
model_matrices::naive_rm:
        .cfi_startproc
        test rdx, rdx
        je .LBB50_4
        sub rsp, 56
        .cfi_def_cfa_offset 64
        shl rdx, 4
        xor eax, eax
        xorps xmm0, xmm0
        lea rcx, [rsp + 32]
        mov r8, rsp
        lea r9, [rsp + 16]
        .p2align        4
.LBB50_2:
        movups xmm1, xmmword ptr [rsi + rax]
        movaps xmmword ptr [rsp + 32], xmm1
        movaps xmmword ptr [rsp], xmm0
        #APP

        fld dword ptr [rdi]
        fmul dword ptr [rcx]
        fld dword ptr [rdi + 4]
        fmul dword ptr [rcx + 4]
        faddp st(1), st
        fld dword ptr [rdi + 8]
        fmul dword ptr [rcx + 8]
        faddp st(1), st
        fld dword ptr [rdi + 12]
        fmul dword ptr [rcx + 12]
        faddp st(1), st
        fstp dword ptr [r8]
        fld dword ptr [rdi + 16]
        fmul dword ptr [rcx]
        fld dword ptr [rdi + 20]
        fmul dword ptr [rcx + 4]
        faddp st(1), st
        fld dword ptr [rdi + 24]
        fmul dword ptr [rcx + 8]
        faddp st(1), st
        fld dword ptr [rdi + 28]
        fmul dword ptr [rcx + 12]
        faddp st(1), st
        fstp dword ptr [r8 + 4]
        fld dword ptr [rdi + 32]
        fmul dword ptr [rcx]
        fld dword ptr [rdi + 36]
        fmul dword ptr [rcx + 4]
        faddp st(1), st
        fld dword ptr [rdi + 40]
        fmul dword ptr [rcx + 8]
        faddp st(1), st
        fld dword ptr [rdi + 44]
        fmul dword ptr [rcx + 12]
        faddp st(1), st
        fstp dword ptr [r8 + 8]
        fld dword ptr [rdi + 48]
        fmul dword ptr [rcx]
        fld dword ptr [rdi + 52]
        fmul dword ptr [rcx + 4]
        faddp st(1), st
        fld dword ptr [rdi + 56]
        fmul dword ptr [rcx + 8]
        faddp st(1), st
        fld dword ptr [rdi + 60]
        fmul dword ptr [rcx + 12]
        faddp st(1), st
        fstp dword ptr [r8 + 12]

        #NO_APP
        mov r10, qword ptr [rsp]
        mov qword ptr [rsp + 16], r10
        mov r10, qword ptr [rsp + 8]
        mov qword ptr [rsp + 24], r10
        mov qword ptr [rsp], r9
        #APP
        #NO_APP
        add rax, 16
        cmp rdx, rax
        jne .LBB50_2
        add rsp, 56
        .cfi_def_cfa_offset 8
.LBB50_4:
        ret
