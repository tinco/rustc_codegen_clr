#![feature(lang_items,adt_const_params,associated_type_defaults,core_intrinsics,start)]
#![allow(internal_features,incomplete_features,unused_variables,dead_code)]
#![no_std]

include!("../common.rs");
fn main(){
    let  tst = black_box(Some((0_usize,85_u32)));
    black_box(tst);
     let  tst = black_box(Some((0_usize,())));
    black_box(tst);
}
