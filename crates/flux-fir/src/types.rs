use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FirType {
    Void,
    Int(u8),       // Int(8), Int(16), Int(32), Int(64)
    Float(u8),     // Float(32), Float(64)
    Bool,
    String,
    Bytes,
    Agent,
    Channel,
    Capability,
    Region,
    Array(Box<FirType>),
    Map(Box<FirType>, Box<FirType>),
    Tuple(Vec<FirType>),
    Function(Vec<FirType>, Box<FirType>),  // (params, return)
    Opaque(String),
}

impl FirType {
    /// Returns a short display string for the type.
    pub fn display(&self) -> String {
        match self {
            FirType::Void => "void".to_string(),
            FirType::Int(bits) => format!("i{bits}"),
            FirType::Float(bits) => format!("f{bits}"),
            FirType::Bool => "bool".to_string(),
            FirType::String => "string".to_string(),
            FirType::Bytes => "bytes".to_string(),
            FirType::Agent => "agent".to_string(),
            FirType::Channel => "channel".to_string(),
            FirType::Capability => "capability".to_string(),
            FirType::Region => "region".to_string(),
            FirType::Array(elem) => format!("[{}]", elem.display()),
            FirType::Map(key, val) => format!("{{{}:{}}}", key.display(), val.display()),
            FirType::Tuple(elems) => {
                let inner: Vec<_> = elems.iter().map(|e| e.display()).collect();
                format!("({})", inner.join(", "))
            }
            FirType::Function(params, ret) => {
                let p: Vec<_> = params.iter().map(|e| e.display()).collect();
                format!("fn({}) -> {}", p.join(", "), ret.display())
            }
            FirType::Opaque(name) => format!("opaque({name})"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeContext {
    _cache: HashMap<String, FirType>,
}

impl TypeContext {
    pub fn new() -> Self {
        Self {
            _cache: HashMap::new(),
        }
    }

    pub fn void(&self) -> FirType {
        FirType::Void
    }

    pub fn i8(&self) -> FirType {
        FirType::Int(8)
    }

    pub fn i16(&self) -> FirType {
        FirType::Int(16)
    }

    pub fn i32(&self) -> FirType {
        FirType::Int(32)
    }

    pub fn i64(&self) -> FirType {
        FirType::Int(64)
    }

    pub fn f32(&self) -> FirType {
        FirType::Float(32)
    }

    pub fn f64(&self) -> FirType {
        FirType::Float(64)
    }

    pub fn boolean(&self) -> FirType {
        FirType::Bool
    }

    pub fn string(&self) -> FirType {
        FirType::String
    }

    pub fn bytes(&self) -> FirType {
        FirType::Bytes
    }

    pub fn agent(&self) -> FirType {
        FirType::Agent
    }

    pub fn channel(&self) -> FirType {
        FirType::Channel
    }

    pub fn capability(&self) -> FirType {
        FirType::Capability
    }

    pub fn region(&self) -> FirType {
        FirType::Region
    }

    pub fn array(&self, elem: FirType) -> FirType {
        FirType::Array(Box::new(elem))
    }

    pub fn map(&self, key: FirType, val: FirType) -> FirType {
        FirType::Map(Box::new(key), Box::new(val))
    }

    pub fn tuple(&self, elems: Vec<FirType>) -> FirType {
        FirType::Tuple(elems)
    }

    pub fn function(&self, params: Vec<FirType>, ret: FirType) -> FirType {
        FirType::Function(params, Box::new(ret))
    }

    /// Returns true if the type is a numeric type (integer or float).
    pub fn is_numeric(&self, ty: &FirType) -> bool {
        matches!(ty, FirType::Int(_) | FirType::Float(_))
    }

    /// Returns true if the type is an integer type.
    pub fn is_integer(&self, ty: &FirType) -> bool {
        matches!(ty, FirType::Int(_))
    }

    /// Returns true if the type is a float type.
    pub fn is_float(&self, ty: &FirType) -> bool {
        matches!(ty, FirType::Float(_))
    }

    /// Returns the conceptual size in bytes for the type.
    /// For opaque/complex types, returns 0 (pointer-sized is not assumed here).
    pub fn size_of(&self, ty: &FirType) -> usize {
        match ty {
            FirType::Void => 0,
            FirType::Int(bits) => (*bits as usize) / 8,
            FirType::Float(bits) => (*bits as usize) / 8,
            FirType::Bool => 1,
            FirType::String => 0, // variable-length
            FirType::Bytes => 0,
            FirType::Agent => 8,  // opaque pointer
            FirType::Channel => 8,
            FirType::Capability => 8,
            FirType::Region => 8,
            FirType::Array(_) => 8, // pointer
            FirType::Map(_, _) => 8,
            FirType::Tuple(elems) => elems.iter().map(|e| self.size_of(e)).sum(),
            FirType::Function(_, _) => 8,
            FirType::Opaque(_) => 8,
        }
    }

    /// Simple type unification: returns a unified type if a and b are compatible.
    /// For integer types, returns the wider integer. For floats, the wider float.
    /// Same types unify to themselves. Otherwise None.
    pub fn unify(&self, a: &FirType, b: &FirType) -> Option<FirType> {
        if a == b {
            return Some(a.clone());
        }
        match (a, b) {
            (FirType::Int(b1), FirType::Int(b2)) => Some(FirType::Int((*b1).max(*b2))),
            (FirType::Float(b1), FirType::Float(b2)) => Some(FirType::Float((*b1).max(*b2))),
            _ => None,
        }
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_types() {
        let ctx = TypeContext::new();
        assert_eq!(ctx.i32(), FirType::Int(32));
        assert_eq!(ctx.f64(), FirType::Float(64));
        assert_eq!(ctx.boolean(), FirType::Bool);
        assert_eq!(ctx.void(), FirType::Void);
        assert_eq!(ctx.string(), FirType::String);
    }

    #[test]
    fn test_composite_types() {
        let ctx = TypeContext::new();
        let arr = ctx.array(ctx.i32());
        assert_eq!(arr, FirType::Array(Box::new(FirType::Int(32))));

        let tup = ctx.tuple(vec![ctx.i32(), ctx.f64()]);
        assert_eq!(
            tup,
            FirType::Tuple(vec![FirType::Int(32), FirType::Float(64)])
        );

        let fn_ty = ctx.function(vec![ctx.i32(), ctx.i32()], ctx.i32());
        assert_eq!(
            fn_ty,
            FirType::Function(
                vec![FirType::Int(32), FirType::Int(32)],
                Box::new(FirType::Int(32))
            )
        );
    }

    #[test]
    fn test_is_numeric() {
        let ctx = TypeContext::new();
        assert!(ctx.is_numeric(&ctx.i32()));
        assert!(ctx.is_numeric(&ctx.f64()));
        assert!(!ctx.is_numeric(&ctx.boolean()));
        assert!(!ctx.is_numeric(&ctx.string()));
    }

    #[test]
    fn test_size_of() {
        let ctx = TypeContext::new();
        assert_eq!(ctx.size_of(&ctx.i32()), 4);
        assert_eq!(ctx.size_of(&ctx.i64()), 8);
        assert_eq!(ctx.size_of(&ctx.f32()), 4);
        assert_eq!(ctx.size_of(&ctx.void()), 0);
        assert_eq!(ctx.size_of(&ctx.boolean()), 1);
    }

    #[test]
    fn test_unify_same_types() {
        let ctx = TypeContext::new();
        let result = ctx.unify(&ctx.i32(), &ctx.i32());
        assert_eq!(result, Some(ctx.i32()));
    }

    #[test]
    fn test_unify_different_ints() {
        let ctx = TypeContext::new();
        let result = ctx.unify(&ctx.i8(), &ctx.i32());
        assert_eq!(result, Some(ctx.i32()));
    }

    #[test]
    fn test_unify_incompatible() {
        let ctx = TypeContext::new();
        let result = ctx.unify(&ctx.i32(), &ctx.f64());
        assert!(result.is_none());
    }

    #[test]
    fn test_map_type() {
        let ctx = TypeContext::new();
        let m = ctx.map(ctx.string(), ctx.i32());
        assert_eq!(
            m,
            FirType::Map(Box::new(FirType::String), Box::new(FirType::Int(32)))
        );
    }

    #[test]
    fn test_function_type() {
        let ctx = TypeContext::new();
        let fn_ty = ctx.function(vec![ctx.f64(), ctx.f64()], ctx.f64());
        assert_eq!(
            fn_ty,
            FirType::Function(
                vec![FirType::Float(64), FirType::Float(64)],
                Box::new(FirType::Float(64))
            )
        );
    }

    #[test]
    fn test_display() {
        let ctx = TypeContext::new();
        assert_eq!(ctx.i32().display(), "i32");
        assert_eq!(ctx.f64().display(), "f64");
        assert_eq!(ctx.void().display(), "void");
        assert_eq!(ctx.boolean().display(), "bool");
        assert_eq!(ctx.string().display(), "string");
    }
}
