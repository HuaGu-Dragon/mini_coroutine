use std::arch::naked_asm;

struct Runtime {
    coroutines: Vec<Coroutine>,
    current: usize,
}

enum CoroutineState {
    Available,
    Ready,
    Running,
}

#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86_64")]
#[repr(C)]
struct Context {
    xmm6: [u64; 2],
    xmm7: [u64; 2],
    xmm8: [u64; 2],
    xmm9: [u64; 2],
    xmm10: [u64; 2],
    xmm11: [u64; 2],
    xmm12: [u64; 2],
    xmm13: [u64; 2],
    xmm14: [u64; 2],
    xmm15: [u64; 2],
    rsp: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
    rdi: u64,
    rsi: u64,
    stack_start: u64,
    stack_end: u64,
}

struct Coroutine {
    id: usize,
    stack: Vec<u8>,
    state: CoroutineState,
    ctx: Context,
}

/***
 * This function is used to swap the context of the current coroutine with the next one.
 * It is marked as naked to avoid prologue/epilogue code generation by the compiler
 * and uses inline assembly to perform the context switch.
 *
 * RCX is expected to contain the address of the current coroutine's context.
 * RDX is expected to contain the address of the next coroutine's context.
 */
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86_64")]
unsafe extern "C" fn swap_ctx() {
    naked_asm!(
        "movaps [rcx + 0x00], xmm6",
        "movaps [rcx + 0x10], xmm7",
        "movaps [rcx + 0x20], xmm8",
        "movaps [rcx + 0x30], xmm9",
        "movaps [rcx + 0x40], xmm10",
        "movaps [rcx + 0x50], xmm11",
        "movaps [rcx + 0x60], xmm12",
        "movaps [rcx + 0x70], xmm13",
        "movaps [rcx + 0x80], xmm14",
        "movaps [rcx + 0x90], xmm15",
        "mov    [rcx + 0xa0], rsp  ",
        "mov    [rcx + 0xa8], r15  ",
        "mov    [rcx + 0xb0], r14  ",
        "mov    [rcx + 0xb8], r13  ",
        "mov    [rcx + 0xc0], r12  ",
        "mov    [rcx + 0xc8], rbx  ",
        "mov    [rcx + 0xd0], rbp  ",
        "mov    [rcx + 0xd8], rdi  ",
        "mov    [rcx + 0xe0], rsi  ",
        "movaps  xmm6, [rdx + 0x00]",
        "movaps  xmm7, [rdx + 0x10]",
        "movaps  xmm8, [rdx + 0x20]",
        "movaps  xmm9, [rdx + 0x30]",
        "movaps  xmm10, [rdx + 0x40]",
        "movaps  xmm11, [rdx + 0x50]",
        "movaps  xmm12, [rdx + 0x60]",
        "movaps  xmm13, [rdx + 0x70]",
        "movaps  xmm14, [rdx + 0x80]",
        "movaps  xmm15, [rdx + 0x90]",
        "mov     rsp,   [rdx + 0xa0]",
        "mov     r15,  [rdx + 0xa8]",
        "mov     r14,  [rdx + 0xb0]",
        "mov     r13,  [rdx + 0xb8]",
        "mov     r12,  [rdx + 0xc0]",
        "mov     rbx,  [rdx + 0xc8]",
        "mov     rbp,  [rdx + 0xd0]",
        "mov     rdi,  [rdx + 0xd8]",
        "mov     rsi,  [rdx + 0xe0]",
        "mov     rax,  [rdx + 0xe8]",
        "mov     gs:0x08,       rax",
        "mov     rax,  [rdx + 0xf0]",
        "mov     gs:0x10,       rax",
        "ret",
    );
}

fn main() {
    println!("Hello, world!");
}
