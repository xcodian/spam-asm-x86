extern crate proc_macro;
use proc_macro::TokenStream;
use rand::{seq::SliceRandom, Rng};
use syn::{Expr, Item, Stmt};

const SAFE_REGS_32: &[&str] = &["eax", "ebx", "ecx", "edx", "edi", "ebp"]; // esi isn't good
const SAFE_REGS_8: &[&str] = &["ah", "bh", "ch", "dh", "al", "bl", "cl", "dl"];
const SAFE_REGS_MM: &[&str] = &[
    "mm0", "mm1", "mm2", "mm3", "mm4", "mm5", "mm6", "mm7", "xmm0", "xmm1", "xmm2", "xmm3", "xmm4",
    "xmm5", "xmm6", "xmm7",
];

const OPS_ALL: &[&str] = &[
    "mov $reg$, $reg$",
    "add $reg$, 0x00000000",
    "sub $reg$, 0x00000000",
    "xor $reg$, 0x00000000",
    "and $reg$, 0xffffffff",
    "and $reg$, $reg$",
    "or $reg$, 0x00000000",
    "or $reg$, $reg$",
    "xchg $reg$, $reg$",
];

const OPS_32_ONLY: &[&str] = &["lea $reg$, [$reg$]", "ror $reg$, 0x20", "rol $reg$, 0x20"];
const OPS_8_ONLY: &[&str] = &["ror $reg$, 0x08", "rol $reg$, 0x08"];
const OPS_MM_ONLY: &[&str] = &["pand $reg$, $reg$", "por $reg$, $reg$"];

fn get_useless_asm() -> String {
    let mut rng = rand::thread_rng();

    let regs;
    let ops;

    match rng.gen_range(0..3) {
        0u8 => {
            regs = SAFE_REGS_32;
            ops = [OPS_ALL, OPS_32_ONLY].concat();
        }
        1u8 => {
            regs = SAFE_REGS_8;
            ops = [OPS_ALL, OPS_8_ONLY].concat();
        }
        2u8 => {
            regs = SAFE_REGS_MM;
            ops = OPS_MM_ONLY.to_vec();
        }
        _ => {
            return "NOP".to_owned();
        }
    }
    let reg = regs.choose(&mut rng).unwrap();
    ops.choose(&mut rng).unwrap().replace("$reg$", reg)
}

fn get_useless_block() -> TokenStream {
    let mut asm_calls = String::with_capacity(128);

    for _ in 0..rand::thread_rng().gen_range(5..20) {
        let op = get_useless_asm();
        asm_calls += format!("\"{}\",", op).as_str();
    }

    let block = format!("unsafe {{core::arch::asm!({})}}", asm_calls);

    println!("{}", block);

    return block.parse().unwrap();
}

// recursively adds a line of code before every line of
// code that does something, no matter how deeply it's nested
fn patch_expr(expr: &mut Expr) {
    match expr {
        Expr::ForLoop(x) => patch_inner(&mut x.body.stmts),
        Expr::Block(x) => patch_inner(&mut x.block.stmts),
        Expr::Group(x) => patch_expr(&mut x.expr),
        Expr::Loop(x) => patch_inner(&mut x.body.stmts),
        Expr::TryBlock(x) => patch_inner(&mut x.block.stmts),
        Expr::Unsafe(x) => patch_inner(&mut x.block.stmts),
        Expr::While(x) => patch_inner(&mut x.body.stmts),
        Expr::Closure(_) => {
            /*
                the closure's body needs to have stuff added
                like: `|x| x+1` needs to become `|x| { injected; x+1 }`
                will implement it later
            */
        }
        Expr::If(x) => {
            patch_inner(&mut x.then_branch.stmts);

            if let Some(else_branch) = &mut x.else_branch {
                patch_expr(&mut else_branch.1);
            }
        }
        Expr::Match(x) => {
            for arm in &mut x.arms {
                patch_expr(&mut arm.body);
            }
        }
        _ => {}
    }
}

fn patch_inner(stmts: &mut Vec<Stmt>) {
    // for every statement
    for stmt in stmts.as_mut_slice() {
        // only if it's an expression
        if let Stmt::Expr(expr_ptr) = stmt {
            // find the body of statements & patch it
            patch_expr(expr_ptr);
        }
    }

    *stmts = stmts
        .iter()
        .flat_map(|original| {
            let injected: Stmt = syn::parse(get_useless_block()).unwrap();
            [injected, original.clone()]
        })
        .collect();
}

#[proc_macro_attribute]
pub fn spam_asm(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut item: Item = syn::parse(input).unwrap();

    let fn_item = match &mut item {
        Item::Fn(fn_item) => fn_item,
        _ => panic!("put the macro on a fn"),
    };

    // go down the rabbit hole
    patch_inner(&mut fn_item.block.stmts);

    use quote::ToTokens;
    item.into_token_stream().into()
}
