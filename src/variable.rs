use crate::BaseIR;
use crate::IString;
use rustc_middle::ty::AdtKind;
use rustc_middle::{
    mir::Mutability,
    ty::{FloatTy, IntTy, Ty, TyCtxt, TyKind, UintTy},
};
use serde::{Deserialize, Serialize};
enum TypePrefix {
    ValueType,
}
impl TypePrefix {
    fn il(&self) -> IString {
        match self {
            Self::ValueType => "valuetype",
        }
        .into()
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) enum VariableType {
    Void,
    I8,
    I16,
    I32,
    I64,
    I128,
    ISize,
    U8,
    U16,
    U32,
    U64,
    U128,
    USize,
    F32,
    F64,
    Bool,
    Ref(Box<Self>),
    RefMut(Box<Self>),
    Array { element: Box<Self>, length: usize },
    Slice(Box<Self>),
    Struct(IString),
    Tuple(Vec<Self>),
    Generic(IString),
}

impl VariableType {
    /// For a type T, returns a BaseIR op which derefreneces T* to T.
    pub(crate) fn deref_op(&self) -> BaseIR {
        match self {
            Self::Ref(_) | Self::RefMut(_) => BaseIR::LDIndI,
            Self::I32 => BaseIR::LDIndIn(std::mem::size_of::<i32>() as u8),
            Self::I64 => BaseIR::LDIndIn(std::mem::size_of::<i64>() as u8),
            Self::F64 => BaseIR::LDIndR8,
            Self::F32 => BaseIR::LDIndR4,
            Self::Struct(name) => BaseIR::LDObj(name.clone()),
            Self::Array { .. } => BaseIR::LDObj(self.il_name()),
            _ => todo!("Can't deference a pointer to type {self:?}"),
        }
    }
    pub(crate) fn sizeof_op(&self) -> BaseIR {
        match self {
            Self::Ref(_) | Self::RefMut(_) => BaseIR::LDIndI,
            Self::I32 => BaseIR::LDConstI8(std::mem::size_of::<i32>() as i8),
            Self::I64 => BaseIR::LDConstI8(std::mem::size_of::<i64>() as i8),
            Self::F64 => BaseIR::LDConstI8(std::mem::size_of::<f64>() as i8),
            Self::F32 => BaseIR::LDConstI8(std::mem::size_of::<f32>() as i8),
            Self::Struct(name) => BaseIR::SizeOf(name.clone()),
            Self::Array { .. } => BaseIR::SizeOf(self.il_name()),
            _ => todo!("Can't deference a pointer to type {self:?}"),
        }
    }
    pub(crate) fn set_pointed_op(&self) -> BaseIR {
        match self {
            Self::Ref(_) | Self::RefMut(_) => BaseIR::STIndI,
            Self::I32 => BaseIR::STIndIn(std::mem::size_of::<i32>() as u8),
            Self::I64 => BaseIR::STIndIn(std::mem::size_of::<i64>() as u8),
            Self::F64 => BaseIR::STIndR8,
            Self::F32 => BaseIR::STIndR4,
            Self::Struct(name) => BaseIR::STObj(name.clone()),
            Self::Array { .. } => BaseIR::STObj(self.il_name()),
            _ => todo!("Can't deference a pointer to type {self:?}"),
        }
    }
    pub(crate) fn is_void(&self) -> bool {
        matches!(self, Self::Void)
    }
    pub(crate) fn get_pointed_type(&self) -> Option<Self> {
        match self {
            Self::Ref(inner) => Some((*inner.as_ref()).clone()),
            Self::RefMut(inner) => Some((*inner.as_ref()).clone()),
            _ => None,
        }
    }
    pub(crate) fn from_ty(ty: Ty, tyctx: TyCtxt) -> Self {
        println!("ty:{ty:?}");
        match ty.kind() {
            TyKind::Int(IntTy::I8) => VariableType::I8,
            TyKind::Int(IntTy::I16) => VariableType::I16,
            TyKind::Int(IntTy::I32) => VariableType::I32,
            TyKind::Int(IntTy::I64) => VariableType::I64,
            TyKind::Int(IntTy::I128) => VariableType::I128,
            TyKind::Int(IntTy::Isize) => VariableType::ISize,
            TyKind::Uint(UintTy::U8) => VariableType::U8,
            TyKind::Uint(UintTy::U16) => VariableType::U16,
            TyKind::Uint(UintTy::U32) => VariableType::U32,
            TyKind::Uint(UintTy::U64) => VariableType::U64,
            TyKind::Uint(UintTy::U128) => VariableType::U128,
            TyKind::Uint(UintTy::Usize) => VariableType::USize,
            TyKind::Float(FloatTy::F32) => VariableType::F32,
            TyKind::Float(FloatTy::F64) => VariableType::F64,
            TyKind::Bool => VariableType::Bool,
            TyKind::Char => todo!("Can't handle chars yet!"),
            TyKind::Foreign(_ftype) => todo!("Can't handle foreign types yet!"),
            TyKind::Str => todo!("Can't handle string slices yet!"),
            TyKind::Array(element_type, length) => Self::Array {
                element: Box::new(Self::from_ty(*element_type, tyctx)),
                length: {
                    let scalar = length
                        .try_to_scalar()
                        .expect("Could not convert the scalar");
                    let value = scalar.to_u64().expect("Could not convert scalar to u64!");
                    value as usize
                },
            },
            TyKind::Slice(element_type) => {
                Self::Slice(Box::new(Self::from_ty(*element_type, tyctx)))
            }
            TyKind::Adt(adt_def, _subst) => {
                let adt = adt_def;
                //let tcxt:&_ = adt.0.0;
                //TODO: Figure out a better way to get this name!
                let name = format!("{adt:?}"); //tcxt.get_diagnostic_name();
                match adt.adt_kind() {
                    AdtKind::Struct => VariableType::Struct(name.into()),
                    AdtKind::Union => todo!("Can't yet handle unions"),
                    AdtKind::Enum => todo!("Can't yet handle enum"),
                }
            }
            TyKind::RawPtr(_target_type) => todo!("Can't handle pointers yet!"),
            TyKind::FnPtr(_sig) => todo!("Can't handle function pointers yet!"),
            TyKind::Ref(region, ref_type, mutability) => {
                // There is no such concept as lifetimes in CLR
                let _ = region;
                match mutability {
                    Mutability::Mut => Self::RefMut(Box::new(Self::from_ty(*ref_type, tyctx))),
                    Mutability::Not => Self::Ref(Box::new(Self::from_ty(*ref_type, tyctx))),
                }
            }
            TyKind::Bound(debrujin_index, bound_ty) => {
                todo!("Bound, debrujin_index:{debrujin_index:?}, bound_ty:{bound_ty:?}");
            }
            TyKind::Tuple(inner_types) => {
                if inner_types.len() == 0 {
                    Self::Void
                } else {
                    Self::Tuple(
                        inner_types
                            .iter()
                            .map(|ty| VariableType::from_ty(ty, tyctx))
                            .collect(),
                    )
                }
            }
            TyKind::FnDef(_, _) => todo!("Can't handle function definition types yet!"),
            TyKind::Dynamic(_, _, _) => todo!("Can't handle dynamic types yet!"),
            TyKind::Closure(_, _) => todo!("Can't handle closure types yet!"),
            TyKind::Generator(_, _, _) => todo!("Can't handle generator types yet!"),
            TyKind::GeneratorWitness(_) => todo!("Can't handle generator types yet!"),
            TyKind::GeneratorWitnessMIR(_, _) => todo!("Can't handle generator types yet!"),
            TyKind::Never => todo!("Can't handle never types yet!"),
            TyKind::Alias(_, _) => todo!("Can't handle type aliases yet!"),
            TyKind::Placeholder(_) => todo!("Can't handle placeholder types yet!"),
            TyKind::Param(_inner) => VariableType::Generic(format!("inner:?").into()), //VariableType::from_ty(inner.to_ty(tyctx),tyctx),
            TyKind::Infer(_) => todo!("Can't handle infered types yet!"),
            TyKind::Error(_) => todo!("Can't handle error types yet!"),
            //_ => todo!("Unhandled type kind {:?}", ty.kind()),
        }
    }
    fn get_prefix(&self) -> Option<TypePrefix> {
        match self {
            Self::Tuple(_) => Some(TypePrefix::ValueType),
            _ => None,
        }
    }
    pub(crate) fn arg_name(&self) -> IString {
        if let Some(prefix) = self.get_prefix() {
            format!(
                "{il_prefix} {il_name}",
                il_prefix = prefix.il(),
                il_name = self.il_name()
            )
            .into()
        } else {
            self.il_name()
        }
    }
    pub(crate) fn il_name(&self) -> IString {
        match self {
            Self::Generic(typename) => typename.clone().into(),
            Self::Void => "void".into(),
            Self::I8 => "int8".into(),
            Self::I16 => "int16".into(),
            Self::I32 => "int32".into(),
            Self::I64 => "int64".into(),
            Self::I128 => "[System.Runtime]System.Int128".into(),
            Self::ISize => "native int".into(),
            Self::U8 => "uint8".into(),
            Self::U16 => "uint16".into(),
            Self::U32 => "uint32".into(),
            Self::U64 => "uint64".into(),
            Self::U128 => "[System.Runtime]System.UInt128".into(),
            Self::USize => "native uint".into(),
            Self::F32 => "float32".into(),
            Self::F64 => "float64".into(),
            Self::Bool => "bool".into(),
            Self::Ref(inner) => format!("{inner}*", inner = inner.il_name()),
            Self::RefMut(inner) => format!("{inner}*", inner = inner.il_name()),
            Self::Struct(name) => (*name).clone().into(),
            Self::Array { element, length } => format!(
                "'RArray_{element_il}_{length}'",
                element_il = element.il_name().replace('\'', "")
            )
            .into(),
            Self::Slice(element) => format!(
                "'RSlice_{element_il}'",
                element_il = element.il_name().replace('\'', "")
            )
            .into(),
            Self::Tuple(elements) => {
                assert!(
                    elements.len() < 8,
                    "Tuples larger than 8 elements are not yet supported!"
                );
                let mut inner = String::new();
                let mut elements_iter = elements.iter();
                if let Some(first_arg) = elements_iter.next() {
                    inner.push_str(&first_arg.il_name());
                }
                for arg in elements_iter {
                    inner.push(',');
                    inner.push_str(&arg.il_name());
                }
                format!(
                    "[System.Runtime]System.ValueTuple`{element_count}<{inner}>",
                    element_count = elements.len()
                )
            }
        }
        .into()
    }
}
