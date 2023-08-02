use std::cmp::min;
use std::mem::size_of;
use std::mem::transmute;
use std::path::PathBuf;
use std::ptr;

use blazesym::symbolize::Process;
use blazesym::symbolize::Source;
use blazesym::symbolize::Sym;
use blazesym::symbolize::Symbolizer;
use blazesym::Addr;
use blazesym::Pid;


fn symbolize_current_bt() {
    assert_eq!(size_of::<*mut libc::c_void>(), size_of::<Addr>());
    // Retrieve up to 64 stack frames of the calling thread.
    const MAX_CNT: usize = 64;

    let mut bt_buf = [ptr::null_mut::<libc::c_void>(); MAX_CNT];
    let bt_cnt = unsafe { libc::backtrace(bt_buf.as_mut_ptr(), MAX_CNT as _) } as usize;
    let bt = &bt_buf[0..min(bt_cnt, MAX_CNT)];
    let bt = unsafe { transmute::<&[*mut libc::c_void], &[Addr]>(bt) };

    // Symbolize the addresses for the current process, as that's where
    // they were captured.
    let src = Source::Process(Process::new(Pid::Slf));
    let symbolizer = Symbolizer::new();

    let syms = symbolizer.symbolize(&src, bt).unwrap();
    for (addr, syms) in bt.iter().zip(syms) {
        let mut addr_fmt = format!("0x{addr:016x}:");
        if syms.is_empty() {
            println!("{addr_fmt} <no-symbol>")
        } else {
            for (i, sym) in syms.into_iter().enumerate() {
                if i == 1 {
                    addr_fmt = addr_fmt.replace(|_c| true, " ");
                }

                let Sym {
                    name, addr, offset, ..
                } = sym;

                let path = match (sym.dir, sym.file) {
                    (Some(dir), Some(file)) => Some(dir.join(file)),
                    (dir, file) => dir.or_else(|| file.map(PathBuf::from)),
                };

                let src_loc = if let (Some(path), Some(line)) = (path, sym.line) {
                    if let Some(col) = sym.column {
                        format!(" {}:{line}:{col}", path.display())
                    } else {
                        format!(" {}:{line}", path.display())
                    }
                } else {
                    String::new()
                };

                println!("{addr_fmt} {name} @ 0x{addr:x}+0x{offset:x}{src_loc}");
            }
        }
    }
}


#[inline(never)]
fn f() {
    g()
}

#[inline(never)]
fn g() {
    h()
}

#[inline(never)]
fn h() {
    symbolize_current_bt()
}

fn main() {
    f();
}
