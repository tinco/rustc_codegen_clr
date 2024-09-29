use crate::assembly::MethodCompileCtx;
use crate::r#type::get_type;

use cilly::cil_node::CILNode;

use cilly::v2::cilnode::MethodKind;
use cilly::v2::{ClassRef, FieldDesc, FnSig, Int, MethodRef};
use cilly::{call, ld_field, Type};

use rustc_middle::mir::{Operand, UnOp};
use rustc_middle::ty::{IntTy, TyKind, UintTy};

/// Implements an unary operation, such as negation.
pub fn unop<'tcx>(
    unnop: UnOp,
    operand: &Operand<'tcx>,
    ctx: &mut MethodCompileCtx<'tcx, '_>,
) -> CILNode {
    let parrent_node = crate::operand::handle_operand(operand, ctx);
    let ty = operand.ty(&ctx.body().local_decls, ctx.tcx());
    match unnop {
        UnOp::Neg => match ty.kind() {
            TyKind::Int(IntTy::I128) => {
                let mref = MethodRef::new(
                    ClassRef::int_128(ctx).into(),
                    ctx.alloc_string("op_UnaryNegation"),
                    ctx.sig(([Type::Int(Int::I128)]), Type::Int(Int::I128)),
                    MethodKind::Static,
                    vec![].into(),
                );
                call!(ctx.alloc_methodref(mref), [parrent_node])
            }
            TyKind::Int(IntTy::I8) => CILNode::Neg(CILNode::ConvI8(parrent_node.into()).into()),
            TyKind::Int(IntTy::I16) => CILNode::Neg(CILNode::ConvI16(parrent_node.into()).into()),
            TyKind::Uint(UintTy::U128) => {
                let mref = MethodRef::new(
                    ClassRef::uint_128(ctx).into(),
                    ctx.alloc_string("op_UnaryNegation"),
                    ctx.sig(([Type::Int(Int::U128)]), Type::Int(Int::U128)),
                    MethodKind::Static,
                    vec![].into(),
                );
                call!(ctx.alloc_methodref(mref), [parrent_node])
            }
            _ => CILNode::Neg(parrent_node.into()),
        },
        UnOp::Not => match ty.kind() {
            TyKind::Bool => CILNode::Eq(CILNode::LdFalse.into(), parrent_node.into()),
            TyKind::Uint(UintTy::U128) => {
                let mref = MethodRef::new(
                    ClassRef::uint_128(ctx).into(),
                    ctx.alloc_string("op_OnesComplement"),
                    ctx.sig([Type::Int(Int::U128)], Type::Int(Int::U128)),
                    MethodKind::Static,
                    vec![].into(),
                );
                call!(ctx.alloc_methodref(mref), [parrent_node])
            }
            TyKind::Int(IntTy::I128) => {
                let mref = MethodRef::new(
                    ClassRef::int_128(ctx).into(),
                    ctx.alloc_string("op_OnesComplement"),
                    ctx.sig([Type::Int(Int::I128)], Type::Int(Int::I128)),
                    MethodKind::Static,
                    vec![].into(),
                );
                call!(ctx.alloc_methodref(mref), [parrent_node])
            }
            _ => CILNode::Not(parrent_node.into()),
        },
        rustc_middle::mir::UnOp::PtrMetadata => {
            let tpe = get_type(ty, ctx)
                .as_class_ref()
                .expect("Invalid pointer type");
            let metadata = ctx.alloc_string(crate::METADATA);
            ld_field!(
                parrent_node,
                ctx.alloc_field(FieldDesc::new(
                    tpe,
                    metadata,
                    cilly::v2::Type::Int(cilly::v2::Int::USize),
                ))
            )
        }
    }
}
