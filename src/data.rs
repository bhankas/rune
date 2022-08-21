use std::cell::RefCell;
use std::sync::Mutex;

use crate::core::{
    cons::Cons,
    env::{Env, Symbol, INTERNED_SYMBOLS},
    error::{Type, TypeError},
    gc::{Context, Root},
    object::{nil, Gc, GcObj, Object},
};
use crate::hashmap::HashSet;
use anyhow::{anyhow, Result};
use fn_macros::defun;
use lazy_static::lazy_static;

lazy_static! {
    pub(crate) static ref FEATURES: Mutex<HashSet<Symbol>> = Mutex::new({
        HashSet::with_capacity_and_hasher(0, std::hash::BuildHasherDefault::default())
    });
}

#[defun]
pub(crate) fn fset(symbol: Symbol, definition: GcObj) -> Result<Symbol> {
    if definition == nil() {
        symbol.unbind_func();
    } else {
        let func = definition.try_into()?;
        let map = INTERNED_SYMBOLS.lock().unwrap();
        map.set_func(symbol, func)?;
    }
    Ok(symbol)
}

#[defun]
pub(crate) fn defalias(
    symbol: Symbol,
    definition: GcObj,
    _docstring: Option<&String>,
) -> Result<Symbol> {
    fset(symbol, definition)
}

#[defun]
pub(crate) fn set<'ob>(
    place: Symbol,
    newlet: GcObj<'ob>,
    env: &mut Root<Env>,
    cx: &Context,
) -> Result<GcObj<'ob>> {
    env.as_mut(cx).set_var(place, newlet)?;
    Ok(newlet)
}

#[defun]
pub(crate) fn put<'ob>(
    symbol: Symbol,
    propname: Symbol,
    value: GcObj<'ob>,
    env: &mut Root<Env>,
    cx: &Context,
) -> GcObj<'ob> {
    env.as_mut(cx).set_prop(symbol, propname, value);
    value
}

#[defun]
pub(crate) fn get<'ob>(
    symbol: Symbol,
    propname: Symbol,
    env: &Root<Env>,
    cx: &'ob Context,
) -> GcObj<'ob> {
    match env.props.get(&symbol) {
        Some(plist) => match plist.iter().find(|x| x.0 == propname) {
            Some(element) => cx.bind(element.1.bind(cx)),
            None => nil(),
        },
        None => nil(),
    }
}

#[defun]
pub(crate) fn eq(obj1: GcObj, obj2: GcObj) -> bool {
    obj1.ptr_eq(obj2)
}

#[defun]
pub(crate) fn equal<'ob>(obj1: GcObj<'ob>, obj2: GcObj<'ob>) -> bool {
    obj1 == obj2
}

#[defun]
pub(crate) fn eql<'ob>(obj1: GcObj<'ob>, obj2: GcObj<'ob>) -> bool {
    match (obj1.get(), obj2.get()) {
        (Object::Float(f1), Object::Float(f2)) => f1.to_bits() == f2.to_bits(),
        _ => obj1.ptr_eq(obj2),
    }
}

#[defun]
fn logand(int_or_markers: &[Gc<i64>]) -> i64 {
    int_or_markers.iter().fold(0, |accum, x| accum & x.get())
}

#[defun]
pub(crate) fn symbol_function<'ob>(symbol: Symbol, cx: &'ob Context) -> GcObj<'ob> {
    match symbol.func(cx) {
        Some(f) => f.into(),
        None => nil(),
    }
}

#[defun]
pub(crate) fn symbol_value<'ob>(
    symbol: Symbol,
    env: &Root<Env>,
    cx: &'ob Context,
) -> Option<GcObj<'ob>> {
    env.vars.get(&symbol).map(|x| x.bind(cx))
}

#[defun]
pub(crate) fn symbol_name(symbol: Symbol) -> &'static str {
    symbol.name
}

#[defun]
pub(crate) fn null(obj: GcObj) -> bool {
    obj.nil()
}

#[defun]
pub(crate) fn fboundp(symbol: Symbol) -> bool {
    symbol.has_func()
}

#[defun]
pub(crate) fn fmakunbound(symbol: Symbol) -> Symbol {
    symbol.unbind_func();
    symbol
}

#[defun]
pub(crate) fn boundp(symbol: Symbol, env: &Root<Env>) -> bool {
    env.vars.get(&symbol).is_some()
}

#[defun]
pub(crate) fn makunbound(symbol: Symbol, env: &mut Root<Env>, cx: &Context) -> Symbol {
    env.as_mut(cx).vars.remove(symbol);
    symbol
}

#[defun]
pub(crate) fn default_boundp(symbol: Symbol, env: &Root<Env>) -> bool {
    env.vars.get(&symbol).is_some()
}

#[defun]
pub(crate) fn listp(object: GcObj) -> bool {
    match object.get() {
        Object::Symbol(s) if s.nil() => true,
        Object::Cons(_) => true,
        _ => false,
    }
}

#[defun]
pub(crate) fn nlistp(object: GcObj) -> bool {
    !listp(object)
}

#[defun]
pub(crate) fn symbolp(object: GcObj) -> bool {
    matches!(object.get(), Object::Symbol(_))
}

#[defun]
pub(crate) fn functionp(object: GcObj) -> bool {
    matches!(object.get(), Object::LispFn(_) | Object::SubrFn(_))
}

#[defun]
pub(crate) fn subrp(object: GcObj) -> bool {
    matches!(object.get(), Object::SubrFn(_))
}

#[defun]
pub(crate) fn stringp(object: GcObj) -> bool {
    matches!(object.get(), Object::String(_))
}

#[defun]
pub(crate) fn numberp(object: GcObj) -> bool {
    matches!(object.get(), Object::Int(_) | Object::Float(_))
}

#[defun]
pub(crate) fn markerp(_: GcObj) -> bool {
    // TODO: implement
    false
}

#[defun]
pub(crate) fn vectorp(object: GcObj) -> bool {
    matches!(object.get(), Object::Vec(_))
}

#[defun]
pub(crate) fn consp(object: GcObj) -> bool {
    matches!(object.get(), Object::Cons(_))
}

#[defun]
pub(crate) fn keywordp(object: GcObj) -> bool {
    match object.get() {
        Object::Symbol(s) => s.name.starts_with(':'),
        _ => false,
    }
}

#[defun]
pub(crate) fn integerp(object: GcObj) -> bool {
    matches!(object.get(), Object::Int(_))
}

#[defun]
pub(crate) fn atom(object: GcObj) -> bool {
    !consp(object)
}

#[defun]
fn byte_code_function_p(_object: GcObj) -> bool {
    // TODO: implement once byte compiling is added
    false
}

#[defun]
fn bufferp(_object: GcObj) -> bool {
    // TODO: Implement once buffers are added
    false
}

#[defun]
pub(crate) fn defvar<'ob>(
    symbol: Symbol,
    initvalue: Option<GcObj<'ob>>,
    _docstring: Option<&String>,
    env: &mut Root<Env>,
    cx: &Context,
) -> Result<GcObj<'ob>> {
    let value = initvalue.unwrap_or_default();
    set(symbol, value, env, cx)
}

#[defun]
pub(crate) fn make_variable_buffer_local(variable: Symbol) -> Symbol {
    // TODO: Implement
    variable
}

#[defun]
pub(crate) fn aset<'ob>(
    array: &RefCell<Vec<GcObj<'ob>>>,
    idx: usize,
    newlet: GcObj<'ob>,
) -> Result<GcObj<'ob>> {
    let mut vec = array.try_borrow_mut()?;
    if idx < vec.len() {
        vec[idx] = newlet;
        Ok(newlet)
    } else {
        let len = vec.len();
        Err(anyhow!("index {idx} is out of bounds. Length was {len}"))
    }
}

#[defun]
pub(crate) fn aref(array: GcObj, idx: usize) -> Result<GcObj> {
    match array.get() {
        Object::Vec(vec) => {
            let vec = vec.borrow();
            match vec.get(idx) {
                Some(x) => Ok(*x),
                None => {
                    let len = vec.len();
                    Err(anyhow!("index {idx} is out of bounds. Length was {len}"))
                }
            }
        }
        Object::String(string) => match string.chars().nth(idx) {
            Some(x) => Ok((x as i64).into()),
            None => {
                let len = string.len();
                Err(anyhow!("index {idx} is out of bounds. Length was {len}"))
            }
        },
        x => Err(TypeError::new(Type::Vec, x).into()),
    }
}

#[defun]
pub(crate) fn indirect_function<'ob>(object: GcObj<'ob>, cx: &'ob Context) -> GcObj<'ob> {
    match object.get() {
        Object::Symbol(sym) => match sym.follow_indirect(cx) {
            Some(func) => func.into(),
            None => nil(),
        },
        _ => object,
    }
}

#[defun]
pub(crate) fn provide(feature: Symbol, _subfeatures: Option<&Cons>) -> Symbol {
    FEATURES.lock().unwrap().insert(feature);
    feature
}

define_symbols!(
    FUNCS => {
        eq,
        equal,
        eql,
        logand,
        set,
        put,
        get,
        defvar,
        make_variable_buffer_local,
        fset,
        aset,
        aref,
        defalias,
        provide,
        symbol_function,
        symbol_value,
        symbol_name,
        null,
        fmakunbound,
        fboundp,
        boundp,
        makunbound,
        default_boundp,
        listp,
        nlistp,
        stringp,
        symbolp,
        functionp,
        subrp,
        vectorp,
        numberp,
        markerp,
        consp,
        keywordp,
        integerp,
        atom,
        byte_code_function_p,
        bufferp,
        indirect_function,
    }
);
