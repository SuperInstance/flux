use crate::types::FirType;

/// An SSA value — defined exactly once in the program.
#[derive(Debug, Clone)]
pub struct Value {
    pub id: u32,
    pub name: String,
    pub ty: FirType,
}

impl Value {
    pub fn new(id: u32, name: impl Into<String>, ty: FirType) -> Self {
        Self {
            id,
            name: name.into(),
            ty,
        }
    }

    pub fn with_id(id: u32, ty: FirType) -> Self {
        Self {
            id,
            name: format!("v{}", id),
            ty,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{} ({} : {})", self.id, self.name, self.ty.display())
    }
}

/// A compile-time constant value.
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Bytes(Vec<u8>),
    Unit,
}

impl Constant {
    /// Returns the type of this constant.
    pub fn ty(&self) -> FirType {
        match self {
            Constant::Int(_) => FirType::Int(64),
            Constant::Float(_) => FirType::Float(64),
            Constant::Bool(_) => FirType::Bool,
            Constant::String(_) => FirType::String,
            Constant::Bytes(_) => FirType::Bytes,
            Constant::Unit => FirType::Void,
        }
    }
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::Int(v) => write!(f, "{v}"),
            Constant::Float(v) => write!(f, "{v}"),
            Constant::Bool(v) => write!(f, "{v}"),
            Constant::String(v) => write!(f, "\"{v}\""),
            Constant::Bytes(v) => write!(f, "b\"{:?}\"", v),
            Constant::Unit => write!(f, "()"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_new() {
        let v = Value::new(0, "x", FirType::Int(32));
        assert_eq!(v.id, 0);
        assert_eq!(v.name, "x");
        assert_eq!(v.ty, FirType::Int(32));
    }

    #[test]
    fn test_value_with_id() {
        let v = Value::with_id(5, FirType::Float(64));
        assert_eq!(v.id, 5);
        assert_eq!(v.name, "v5");
        assert_eq!(v.ty, FirType::Float(64));
    }

    #[test]
    fn test_value_display() {
        let v = Value::new(0, "x", FirType::Int(32));
        let s = format!("{v}");
        assert!(s.contains("%0"));
        assert!(s.contains("x"));
    }

    #[test]
    fn test_constant_ty() {
        assert_eq!(Constant::Int(42).ty(), FirType::Int(64));
        assert_eq!(Constant::Float(3.14).ty(), FirType::Float(64));
        assert_eq!(Constant::Bool(true).ty(), FirType::Bool);
        assert_eq!(Constant::String("hi".into()).ty(), FirType::String);
        assert_eq!(Constant::Unit.ty(), FirType::Void);
    }

    #[test]
    fn test_constant_equality() {
        assert_eq!(Constant::Int(42), Constant::Int(42));
        assert_ne!(Constant::Int(42), Constant::Int(43));
        assert_eq!(Constant::Bool(true), Constant::Bool(true));
    }

    #[test]
    fn test_constant_display() {
        assert_eq!(format!("{}", Constant::Int(42)), "42");
        assert_eq!(format!("{}", Constant::Bool(true)), "true");
        assert_eq!(format!("{}", Constant::Unit), "()");
    }
}
