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

fn main() {
    println!("Hello, world!");
}
