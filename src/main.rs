#![allow(dead_code)]

use std::arch::{asm, naked_asm};
#[cfg(target_arch = "x86_64")]
#[cfg(target_os = "windows")]

const STACK_SIZE: usize = 1024 * 1024 * 2; // 2 MB stack size
const MAX_THREADS: usize = 4; // Maximum number of coroutines
static mut RUNTIME: usize = 0; // Global runtime pointer

struct Runtime {
    coroutines: Vec<Coroutine>,
    current: usize,
}

impl Runtime {
    fn new() -> Self {
        let base_coroutine = Coroutine {
            id: 0,
            stack: vec![0; STACK_SIZE],
            state: CoroutineState::Running,
            ctx: Context::default(),
        };

        let mut coroutines = vec![base_coroutine];
        let mut available_coroutines = (1..MAX_THREADS)
            .map(|id| Coroutine::new(id))
            .collect::<Vec<_>>();
        coroutines.append(&mut available_coroutines);

        Runtime {
            coroutines,
            current: 0,
        }
    }

    pub fn init(&self) {
        unsafe { RUNTIME = self as *const _ as usize }
    }

    pub fn run(&mut self) {
        while self.t_yield() {}
        std::process::exit(0);
    }

    pub fn t_yield(&mut self) -> bool {
        let mut pos = self.current;

        while self.coroutines[pos].state != CoroutineState::Ready {
            pos = (pos + 1) % self.coroutines.len();
            if pos == self.current {
                return false;
            }
        }

        if self.coroutines[self.current].state != CoroutineState::Available {
            self.coroutines[self.current].state = CoroutineState::Ready;
        }

        self.coroutines[pos].state = CoroutineState::Running;
        let old_pos = self.current;
        self.current = pos;

        unsafe {
            let old = &raw mut self.coroutines[old_pos].ctx;
            let next = &raw const self.coroutines[pos].ctx;

            if cfg!(target_os = "windows") {
                asm!("call swap_ctx", in("rcx") old, in("rdx") next, clobber_abi("system"));
            } else {
                // For other platforms, we can use a different context switch mechanism
                // This is a placeholder for non-Windows implementations
                unimplemented!("Context switching not implemented for this platform");
            }
        }

        self.coroutines.len() > 0
    }

    fn t_return(&mut self) {
        if self.current != 0 {
            self.coroutines[self.current].state = CoroutineState::Available;
            self.t_yield();
        }
    }

    #[cfg(target_os = "windows")]
    fn spawn(&mut self, f: fn()) {
        let available = self
            .coroutines
            .iter_mut()
            .find(|c| c.state == CoroutineState::Available)
            .expect("No available coroutine found");

        let size = available.stack.len();

        unsafe {
            let s_ptr = available.stack.as_mut_ptr().offset(size as isize);
            // Align the stack pointer to 16 bytes
            // This is necessary for Windows x86_64 calling conventions
            let s_ptr = (s_ptr as usize & !0xF) as *mut u8;
            std::ptr::write(s_ptr.offset(-16) as *mut u64, guard as u64);
            std::ptr::write(s_ptr.offset(-24) as *mut u64, skip as u64);
            std::ptr::write(s_ptr.offset(-32) as *mut u64, f as u64);
            available.ctx.rsp = s_ptr.offset(-32) as u64;
            available.ctx.stack_start = s_ptr as u64;
            available.ctx.stack_end = available.stack.as_ptr() as u64;
        }

        available.state = CoroutineState::Ready;
    }
}

#[derive(PartialEq, Eq, Debug)]
enum CoroutineState {
    Available,
    Ready,
    Running,
}

#[cfg(target_os = "windows")]
#[cfg(target_arch = "x86_64")]
#[repr(C)]
#[derive(Default, Debug)]
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

impl Coroutine {
    fn new(id: usize) -> Self {
        Coroutine {
            id,
            stack: vec![0; STACK_SIZE],
            state: CoroutineState::Available,
            ctx: Context::default(),
        }
    }
}

#[unsafe(naked)]
unsafe extern "C" fn skip() {
    naked_asm!("ret")
}

fn guard() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        (*rt_ptr).t_return();
    };
}

pub fn yield_thread() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        (*rt_ptr).t_yield();
    };
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
        r#"
        movaps [rcx + 0x00], xmm6
        movaps [rcx + 0x10], xmm7
        movaps [rcx + 0x20], xmm8
        movaps [rcx + 0x30], xmm9
        movaps [rcx + 0x40], xmm10
        movaps [rcx + 0x50], xmm11
        movaps [rcx + 0x60], xmm12
        movaps [rcx + 0x70], xmm13
        movaps [rcx + 0x80], xmm14
        movaps [rcx + 0x90], xmm15

        mov    [rcx + 0xa0], rsp
        mov    [rcx + 0xa8], r15
        mov    [rcx + 0xb0], r14
        mov    [rcx + 0xb8], r13
        mov    [rcx + 0xc0], r12
        mov    [rcx + 0xc8], rbx
        mov    [rcx + 0xd0], rbp
        mov    [rcx + 0xd8], rdi
        mov    [rcx + 0xe0], rsi

        mov    rax         , gs:0x08
        mov    [rcx + 0xe8], rax
        mov    rax         , gs:0x10
        mov    [rcx + 0xf0], rax

        movaps xmm6        , [rdx + 0x00]
        movaps xmm7        , [rdx + 0x10]
        movaps xmm8        , [rdx + 0x20]
        movaps xmm9        , [rdx + 0x30]
        movaps xmm10       , [rdx + 0x40]
        movaps xmm11       , [rdx + 0x50]
        movaps xmm12       , [rdx + 0x60]
        movaps xmm13       , [rdx + 0x70]
        movaps xmm14       , [rdx + 0x80]
        movaps xmm15       , [rdx + 0x90]

        mov    rsp         , [rdx + 0xa0]
        mov    r15         , [rdx + 0xa8]
        mov    r14         , [rdx + 0xb0]
        mov    r13         , [rdx + 0xb8]
        mov    r12         , [rdx + 0xc0]
        mov    rbx         , [rdx + 0xc8]
        mov    rbp         , [rdx + 0xd0]
        mov    rdi         , [rdx + 0xd8]
        mov    rsi         , [rdx + 0xe0]
        mov    rax         , [rdx + 0xe8]
        mov    gs:0x08     , rax
        mov    rax         , [rdx + 0xf0]
        mov    gs:0x10     , rax

        ret
    "#
    );
}

fn main() {
    println!("Hello, world!");
    let mut runtime = Runtime::new();
    runtime.init();
    runtime.spawn(|| {
        println!("Coroutine 1 started");
        yield_thread();
        println!("Coroutine 1 finished");
    });
    runtime.spawn(|| {
        println!("Coroutine 2 started");
        yield_thread();
        println!("Coroutine 2 finished");
    });
    runtime.run();
}
