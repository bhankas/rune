use fn_macros::defun;

use crate::core::{
    arena::{Arena, IntoRoot, Root, RootOwner},
    object::{Function, Gc, GcObj},
};
use crate::root_struct;

use crate::core::env::Environment;

use anyhow::Result;

#[defun]
pub(crate) fn apply<'ob, 'id>(
    function: Gc<Function<'ob>>,
    arguments: &[GcObj<'ob>],
    env: &Root<'id, Environment>,
    owner: &mut RootOwner<'id>,
    arena: &'ob mut Arena,
) -> Result<GcObj<'ob>> {
    let args = match arguments.len() {
        0 => Vec::new(),
        len => {
            let end = len - 1;
            let last = arguments[end];
            let mut args: Vec<_> = arguments[..end].iter().map(|x| arena.bind(*x)).collect();
            for element in last.as_list(arena)? {
                let e = arena.bind(element?);
                args.push(e);
            }
            args
        }
    };
    root_struct!(args, args.into_root(), arena);
    function.call(args, env, arena, owner)
}

#[defun]
pub(crate) fn funcall<'ob, 'id>(
    function: Gc<Function<'ob>>,
    arguments: &[GcObj<'ob>],
    env: &Root<'id, Environment>,
    owner: &mut RootOwner<'id>,
    arena: &'ob mut Arena,
) -> Result<GcObj<'ob>> {
    root_struct!(arg_list, arguments.to_vec().into_root(), arena);
    function.call(arg_list, env, arena, owner)
}

defsubr!(apply, funcall);
